pub mod commands;
pub mod gif_client;
pub mod manager;
pub mod voice;

use async_trait::async_trait;
use serenity::all::{
    Context, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler,
    GatewayIntents, GuildId, Interaction, Ready, VoiceState,
};
use songbird::SerenityInit;

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use tokio::sync::Mutex;

#[derive(Debug)]
struct Handler {
    managers: Arc<Mutex<HashMap<GuildId, Arc<Mutex<manager::GuildAudioManager>>>>>,
    voice_service: OnceLock<Arc<dyn voice::VoiceService>>,
    gif_service: OnceLock<Arc<dyn gif_client::GifService>>,
}

impl Handler {
    fn new() -> Self {
        Self {
            managers: Arc::new(Mutex::new(HashMap::new())),
            voice_service: OnceLock::new(),
            gif_service: OnceLock::new(),
        }
    }

    async fn get_or_create_manager(
        &self,
        guild_id: GuildId,
    ) -> anyhow::Result<Arc<Mutex<manager::GuildAudioManager>>> {
        let mut managers = self.managers.lock().await;
        if let Some(mgr) = managers.get(&guild_id) {
            Ok(mgr.clone())
        } else {
            let voice = self
                .voice_service
                .get()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("VoiceService not initialized"))?;
            let gif = self
                .gif_service
                .get()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("GifService not initialized"))?;
            let mgr = Arc::new(Mutex::new(manager::GuildAudioManager::new(
                guild_id, voice, gif,
            )));
            managers.insert(guild_id, mgr.clone());
            Ok(mgr)
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("DJCova connected as {}", ready.user.name);

        let songbird = songbird::get(&ctx)
            .await
            .expect("Songbird not registered")
            .clone();
        let _ = self
            .voice_service
            .set(Arc::new(voice::DiscordVoiceService::new(songbird)));
        let _ = self
            .gif_service
            .set(Arc::new(gif_client::TenorGifClient::new()));

        let commands = commands::all_commands();

        if let Ok(guild_id_str) = std::env::var("DEV_GUILD_ID") {
            if let Ok(guild_id_num) = guild_id_str.parse::<u64>() {
                let guild_id = GuildId::new(guild_id_num);
                if let Err(e) = guild_id.set_commands(&ctx.http, commands).await {
                    tracing::error!("Failed to register guild commands: {:?}", e);
                } else {
                    tracing::info!("Registered guild commands for guild {}", guild_id);
                }
                return;
            }
        }

        if let Err(e) = serenity::all::Command::set_global_commands(&ctx.http, commands).await {
            tracing::error!("Failed to register global commands: {:?}", e);
        } else {
            tracing::info!("Registered global commands");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(cmd) => {
                let guild_id = match cmd.guild_id {
                    Some(id) => id,
                    None => {
                        let _ = cmd
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content("This command can only be used in a server.")
                                        .ephemeral(true),
                                ),
                            )
                            .await;
                        return;
                    }
                };

                let mgr = match self.get_or_create_manager(guild_id).await {
                    Ok(m) => m,
                    Err(_) => {
                        let _ = cmd
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content(
                                            "Bot is still starting up, please try again in a moment.",
                                        )
                                        .ephemeral(true),
                                ),
                            )
                            .await;
                        return;
                    }
                };

                let _ = match cmd.data.name.as_str() {
                    "play" => commands::handle_play(&ctx, &cmd, mgr).await,
                    "skip" => commands::handle_skip(&ctx, &cmd, mgr).await,
                    "skipnext" => commands::handle_skipnext(&ctx, &cmd, mgr).await,
                    "skiplast" => commands::handle_skiplast(&ctx, &cmd, mgr).await,
                    "stop" => commands::handle_stop(&ctx, &cmd, mgr).await,
                    "pause" => commands::handle_pause(&ctx, &cmd, mgr).await,
                    "clear" => commands::handle_clear(&ctx, &cmd, mgr).await,
                    "shuffle" => commands::handle_shuffle(&ctx, &cmd, mgr).await,
                    "volume" => commands::handle_volume(&ctx, &cmd, mgr).await,
                    "repeat" => commands::handle_repeat(&ctx, &cmd, mgr).await,
                    "queue" => commands::handle_queue(&ctx, &cmd, mgr).await,
                    "history" => commands::handle_history(&ctx, &cmd, mgr).await,
                    "nowplaying" => commands::handle_nowplaying(&ctx, &cmd, mgr).await,
                    "help" => commands::handle_help(&ctx, &cmd).await,
                    _ => Ok(()),
                };
            }
            Interaction::Component(comp) => {
                let _ = comp.defer(&ctx.http).await;
                let guild_id = match comp.guild_id {
                    Some(id) => id,
                    None => return,
                };
                let mgr = match self.get_or_create_manager(guild_id).await {
                    Ok(m) => m,
                    Err(_) => return,
                };
                let _ = commands::buttons::handle(&ctx, &comp, mgr).await;
            }
            _ => {}
        }
    }

    async fn voice_state_update(&self, ctx: Context, _old: Option<VoiceState>, new: VoiceState) {
        let guild_id = match new.guild_id {
            Some(id) => id,
            None => return,
        };

        let bot_user_id = ctx.cache.current_user().id;
        let Ok(mgr) = self.get_or_create_manager(guild_id).await else {
            return;
        };

        let start_leave_timer = {
            let mut m = mgr.lock().await;
            if let Some(voice_channel) = m.voice_channel_id {
                if let Some(guild) = guild_id.to_guild_cached(&ctx.cache) {
                    let non_bot_count = guild
                        .voice_states
                        .values()
                        .filter(|vs| vs.channel_id == Some(voice_channel))
                        .filter(|vs| vs.user_id != bot_user_id)
                        .filter(|vs| {
                            // Exclude other bots; treat unknown users as non-bots (conservative).
                            ctx.cache.user(vs.user_id).map(|u| !u.bot).unwrap_or(true)
                        })
                        .count();
                    if non_bot_count == 0 {
                        m.user_left_voice_channel();
                        true
                    } else {
                        m.user_returned_to_voice_channel();
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };

        if start_leave_timer {
            let mgr_clone = mgr.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(60)).await;
                let mut locked = mgr_clone.lock().await;
                if locked.leave_timer_active {
                    let _ = locked.stop().await;
                }
            });
        }
    }
}

pub async fn run() -> anyhow::Result<()> {
    let token = std::env::var("DISCORD_TOKEN")
        .map_err(|_| anyhow::anyhow!("DJCova: DISCORD_TOKEN not set"))?;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILDS;

    let mut client = serenity::Client::builder(&token, intents)
        .event_handler(Handler::new())
        .register_songbird()
        .await
        .map_err(|e| anyhow::anyhow!("error creating client: {}", e))?;

    tracing::info!(bot = "DJCova", "starting");
    client
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("client error: {}", e))?;
    Ok(())
}
