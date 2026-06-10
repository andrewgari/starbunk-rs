pub mod commands;
pub mod gif_client;
pub mod manager;
pub mod voice;

use async_trait::async_trait;
use serenity::all::{
    ButtonStyle, Context, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, EditInteractionResponse, EventHandler, GatewayIntents,
    GuildId, Interaction, Ready, VoiceState,
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
    ) -> Arc<Mutex<manager::GuildAudioManager>> {
        let mut managers = self.managers.lock().await;
        if let Some(mgr) = managers.get(&guild_id) {
            mgr.clone()
        } else {
            let voice = self
                .voice_service
                .get()
                .cloned()
                .expect("VoiceService not initialized");
            let gif = self
                .gif_service
                .get()
                .cloned()
                .expect("GifService not initialized");
            let mgr = Arc::new(Mutex::new(manager::GuildAudioManager::new(
                guild_id, voice, gif,
            )));
            managers.insert(guild_id, mgr.clone());
            mgr
        }
    }

    fn create_now_playing_embed(&self, track: &manager::QueueItem) -> CreateEmbed {
        let mut embed = CreateEmbed::new()
            .title("Now Playing")
            .description(format!("**{}**", track.title))
            .field("Requested By", &track.requester, true);

        if let Some(dur) = track.duration {
            embed = embed.field(
                "Duration",
                format!("{}:{:02}", dur.as_secs() / 60, dur.as_secs() % 60),
                true,
            );
        }
        if let Some(ref thumb) = track.thumbnail_url {
            embed = embed.thumbnail(thumb);
        } else {
            embed = embed.thumbnail("https://cdn.discordapp.com/embed/avatars/0.png");
        }
        embed
    }

    fn create_buttons(&self) -> CreateActionRow {
        CreateActionRow::Buttons(vec![
            CreateButton::new("djcova_stop")
                .label("Stop")
                .style(ButtonStyle::Danger),
            CreateButton::new("djcova_skip")
                .label("Skip")
                .style(ButtonStyle::Primary),
            CreateButton::new("djcova_restart")
                .label("Restart")
                .style(ButtonStyle::Secondary),
            CreateButton::new("djcova_requeue")
                .label("Re-queue")
                .style(ButtonStyle::Secondary),
        ])
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

                let mgr = self.get_or_create_manager(guild_id).await;

                match cmd.data.name.as_str() {
                    "play" => {
                        let _ = cmd.defer(&ctx.http).await;

                        let voice_channel_id = match guild_id.to_guild_cached(&ctx.cache) {
                            Some(guild) => guild
                                .voice_states
                                .get(&cmd.user.id)
                                .and_then(|vs| vs.channel_id),
                            None => None,
                        };

                        let Some(voice_channel) = voice_channel_id else {
                            let _ = cmd
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new().content(
                                        "You need to be in a voice channel to use this command.",
                                    ),
                                )
                                .await;
                            return;
                        };

                        let mut input = None;
                        let mut file_url = None;

                        for opt in &cmd.data.options {
                            if opt.name == "input" {
                                input = opt.value.as_str().map(|s| s.to_string());
                            } else if opt.name == "file" {
                                if let serenity::all::CommandDataOptionValue::Attachment(id) =
                                    opt.value
                                {
                                    if let Some(att) = cmd.data.resolved.attachments.get(&id) {
                                        let name = att.filename.to_lowercase();
                                        if name.ends_with(".mp3")
                                            || name.ends_with(".flac")
                                            || name.ends_with(".ogg")
                                            || name.ends_with(".wav")
                                        {
                                            file_url = Some(att.url.clone());
                                        } else {
                                            let _ = cmd.edit_response(&ctx.http, EditInteractionResponse::new().content("Unsupported file type. Please upload an MP3, FLAC, OGG, or WAV file.")).await;
                                            return;
                                        }
                                    }
                                }
                            }
                        }

                        let input_str = match (input, file_url) {
                            (Some(inp), _) => inp,
                            (_, Some(url)) => url,
                            _ => {
                                let _ = cmd
                                    .edit_response(
                                        &ctx.http,
                                        EditInteractionResponse::new().content(
                                            "Please provide a song name, URL, or upload a file.",
                                        ),
                                    )
                                    .await;
                                return;
                            }
                        };

                        let requester = cmd.user.name.clone();
                        let play_res = mgr
                            .lock()
                            .await
                            .play(
                                Some(ctx.http.clone()),
                                cmd.channel_id,
                                voice_channel,
                                input_str,
                                requester,
                            )
                            .await;

                        match play_res {
                            Ok(msg) => {
                                let mut edit = EditInteractionResponse::new().content(&msg);
                                if msg.contains("Now playing") {
                                    if let Some(track) = mgr.lock().await.get_current_track() {
                                        edit = edit
                                            .embed(self.create_now_playing_embed(&track))
                                            .components(vec![self.create_buttons()]);
                                    }
                                }
                                let _ = cmd.edit_response(&ctx.http, edit).await;
                            }
                            Err(e) => {
                                let _ = cmd
                                    .edit_response(
                                        &ctx.http,
                                        EditInteractionResponse::new()
                                            .content(format!("Error: {}", e)),
                                    )
                                    .await;
                            }
                        }
                    }
                    "skip" => {
                        let _ = cmd.defer(&ctx.http).await;
                        let skip_res = mgr.lock().await.skip(Some(ctx.http.clone())).await;
                        match skip_res {
                            Ok(msg) => {
                                let mut edit = EditInteractionResponse::new().content(&msg);
                                if msg.contains("Skipped to") {
                                    if let Some(track) = mgr.lock().await.get_current_track() {
                                        edit = edit
                                            .embed(self.create_now_playing_embed(&track))
                                            .components(vec![self.create_buttons()]);
                                    }
                                } else if msg.contains("stopped") {
                                    spawn_idle_timer(mgr.clone());
                                }
                                let _ = cmd.edit_response(&ctx.http, edit).await;
                            }
                            Err(e) => {
                                let _ = cmd
                                    .edit_response(
                                        &ctx.http,
                                        EditInteractionResponse::new()
                                            .content(format!("Error: {}", e)),
                                    )
                                    .await;
                            }
                        }
                    }
                    "stop" => {
                        let _ = mgr.lock().await.stop().await;
                        let _ = cmd
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content("Playback stopped and disconnected."),
                                ),
                            )
                            .await;
                    }
                    "clear" => {
                        mgr.lock().await.clear();
                        let _ = cmd
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content("Queue cleared."),
                                ),
                            )
                            .await;
                    }
                    "shuffle" => {
                        mgr.lock().await.shuffle();
                        let _ = cmd
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content("Queue shuffled."),
                                ),
                            )
                            .await;
                    }
                    "volume" => {
                        let mut level = 50;
                        if let Some(opt) = cmd.data.options.first() {
                            if let serenity::all::CommandDataOptionValue::Integer(val) = opt.value {
                                level = val;
                            }
                        }
                        mgr.lock().await.set_volume(level.clamp(0, 100) as u8);
                        let _ = cmd
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content(format!("Volume set to {}%.", level)),
                                ),
                            )
                            .await;
                    }
                    "repeat" => {
                        let mut mode_str = "off";
                        if let Some(opt) = cmd.data.options.first() {
                            if let serenity::all::CommandDataOptionValue::String(ref val) =
                                opt.value
                            {
                                mode_str = val;
                            }
                        }
                        let mode = match mode_str {
                            "song" => manager::RepeatMode::Song,
                            "queue" => manager::RepeatMode::Queue,
                            _ => manager::RepeatMode::Off,
                        };
                        mgr.lock().await.set_repeat_mode(mode);
                        let _ = cmd
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content(format!("Repeat mode set to {}.", mode_str)),
                                ),
                            )
                            .await;
                    }
                    "queue" => {
                        let m = mgr.lock().await;
                        let queue = m.get_queue();
                        if queue.is_empty() {
                            let _ = cmd
                                .create_response(
                                    &ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("The queue is empty."),
                                    ),
                                )
                                .await;
                        } else {
                            let mut list = String::new();
                            for (i, item) in queue.iter().enumerate() {
                                list.push_str(&format!(
                                    "{}. {} (requested by {})\n",
                                    i + 1,
                                    item.title,
                                    item.requester
                                ));
                            }
                            let embed = CreateEmbed::new().title("Queue").description(list);
                            let _ = cmd
                                .create_response(
                                    &ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new().embed(embed),
                                    ),
                                )
                                .await;
                        }
                    }
                    "history" => {
                        let m = mgr.lock().await;
                        let history = m.get_history();
                        if history.is_empty() {
                            let _ = cmd
                                .create_response(
                                    &ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("No history yet this session."),
                                    ),
                                )
                                .await;
                        } else {
                            let mut list = String::new();
                            for (i, item) in history.iter().enumerate() {
                                list.push_str(&format!(
                                    "{}. {} (requested by {})\n",
                                    i + 1,
                                    item.title,
                                    item.requester
                                ));
                            }
                            let embed = CreateEmbed::new().title("History").description(list);
                            let _ = cmd
                                .create_response(
                                    &ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new().embed(embed),
                                    ),
                                )
                                .await;
                        }
                    }
                    "nowplaying" => {
                        let m = mgr.lock().await;
                        if let Some(track) = m.get_current_track() {
                            let embed = self.create_now_playing_embed(&track);
                            let _ = cmd
                                .create_response(
                                    &ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .embed(embed)
                                            .components(vec![self.create_buttons()]),
                                    ),
                                )
                                .await;
                        } else {
                            let _ = cmd
                                .create_response(
                                    &ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("Nothing is currently playing."),
                                    ),
                                )
                                .await;
                        }
                    }
                    "help" => {
                        let embed = CreateEmbed::new()
                            .title("DJCova Commands")
                            .field("/play", "Play a song (accepts URL, query, or file)", false)
                            .field("/skip", "Skip current song", false)
                            .field("/stop", "Stop playback and leave channel", false)
                            .field("/queue", "Show upcoming queue", false)
                            .field("/history", "Show session history", false)
                            .field("/volume", "Set volume (0-100)", false)
                            .field("/repeat", "Set repeat mode (off/song/queue)", false)
                            .field("/shuffle", "Shuffle queue", false)
                            .field("/clear", "Clear queue", false);
                        let _ = cmd
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new().embed(embed),
                                ),
                            )
                            .await;
                    }
                    _ => {}
                }
            }
            Interaction::Component(comp) => {
                let _ = comp.defer(&ctx.http).await;
                let guild_id = match comp.guild_id {
                    Some(id) => id,
                    None => return,
                };
                let mgr = self.get_or_create_manager(guild_id).await;

                match comp.data.custom_id.as_str() {
                    "djcova_stop" => {
                        let _ = mgr.lock().await.stop().await;
                        let _ = comp
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new()
                                    .content("Playback stopped and disconnected.")
                                    .components(vec![]),
                            )
                            .await;
                    }
                    "djcova_skip" => {
                        let skip_res = mgr.lock().await.skip(Some(ctx.http.clone())).await;
                        if let Ok(msg) = skip_res {
                            let mut edit = EditInteractionResponse::new().content(&msg);
                            if msg.contains("Skipped to") {
                                if let Some(track) = mgr.lock().await.get_current_track() {
                                    edit = edit
                                        .embed(self.create_now_playing_embed(&track))
                                        .components(vec![self.create_buttons()]);
                                }
                            } else if msg.contains("stopped") {
                                edit = edit.components(vec![]);
                                spawn_idle_timer(mgr.clone());
                            }
                            let _ = comp.edit_response(&ctx.http, edit).await;
                        }
                    }
                    "djcova_restart" => {
                        let (track, volume, voice) = {
                            let m = mgr.lock().await;
                            let track = m.get_current_track();
                            let volume = m.get_volume();
                            let voice = self
                                .voice_service
                                .get()
                                .cloned()
                                .expect("VoiceService not initialized");
                            (track, volume, voice)
                        };
                        if let Some(track) = track {
                            let _ = voice.play(guild_id, &track.url).await;
                            let _ = voice.set_volume(guild_id, volume as f32 / 100.0).await;
                            let _ = comp
                                .edit_response(
                                    &ctx.http,
                                    EditInteractionResponse::new()
                                        .content(format!("Restarted: {}", track.title)),
                                )
                                .await;
                        }
                    }
                    "djcova_requeue" => {
                        let (track, queue_len, voice_channel) = {
                            let m = mgr.lock().await;
                            let track = m.get_current_track();
                            let queue_len = m.get_queue().len();
                            let voice_channel = m.voice_channel_id;
                            (track, queue_len, voice_channel)
                        };
                        if let Some(track) = track {
                            let text_channel = comp.channel_id;
                            match voice_channel {
                                Some(vc) => {
                                    let mut m = mgr.lock().await;
                                    let _ = m
                                        .play(
                                            Some(ctx.http.clone()),
                                            text_channel,
                                            vc,
                                            track.url,
                                            track.requester,
                                        )
                                        .await;
                                    drop(m);
                                    let _ = comp
                                        .edit_response(
                                            &ctx.http,
                                            EditInteractionResponse::new().content(format!(
                                                "Re-queued: {} (Queue size: {})",
                                                track.title,
                                                queue_len + 1
                                            )),
                                        )
                                        .await;
                                }
                                None => {
                                    let _ = comp
                                        .edit_response(
                                            &ctx.http,
                                            EditInteractionResponse::new().content(
                                                "Cannot re-queue: bot is not in a voice channel.",
                                            ),
                                        )
                                        .await;
                                }
                            }
                        }
                    }
                    _ => {}
                }
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
        let mgr = self.get_or_create_manager(guild_id).await;

        let start_leave_timer = {
            let mut m = mgr.lock().await;
            if let Some(voice_channel) = m.voice_channel_id {
                if let Some(guild) = guild_id.to_guild_cached(&ctx.cache) {
                    let non_bot_count = guild
                        .voice_states
                        .values()
                        .filter(|vs| vs.channel_id == Some(voice_channel))
                        .filter(|vs| vs.user_id != bot_user_id)
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

fn spawn_idle_timer(mgr: Arc<Mutex<manager::GuildAudioManager>>) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(120)).await;
        let mut m = mgr.lock().await;
        if m.idle_timer_active {
            let _ = m.stop().await;
        }
    });
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
