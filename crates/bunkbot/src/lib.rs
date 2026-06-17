pub mod commands;
pub mod config;
pub mod engine;
pub mod state;
pub mod template;

use async_trait::async_trait;
use engine::BunkBotEngine;
use serenity::all::{Context, EventHandler, Interaction, Message, Ready};
use starbunk::discord::{
    DiscordIdentityProvider, DiscordMessageService, MessageService, WebhookService,
};
use starbunk::middleware::{MessageFilter, HAS_CONTENT};
use std::sync::{Arc, OnceLock};

struct Handler {
    filter: Arc<dyn MessageFilter>,
    engine: OnceLock<BunkBotEngine>,
    state_service: Arc<state::InMemoryBotStateManager>,
    bots_config_path: String,
}

impl Handler {
    fn new() -> Self {
        let path = std::env::var("BOTS_CONFIG").unwrap_or_else(|_| "config/bots.yml".to_string());
        Self {
            filter: HAS_CONTENT.clone(),
            engine: OnceLock::new(),
            state_service: Arc::new(state::InMemoryBotStateManager::new()),
            bots_config_path: path,
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("BunkBot connected as {}", ready.user.name);

        let ws = Arc::new(WebhookService::new(ctx.http.clone()));
        let sender: Arc<dyn MessageService> =
            Arc::new(DiscordMessageService::new(ctx.http.clone(), ws));
        let identity_provider = Arc::new(DiscordIdentityProvider::new(ctx.http.clone()));

        let bots = match tokio::fs::read_to_string(&self.bots_config_path).await {
            Ok(yaml) => config::parse_bots(&yaml).unwrap_or_else(|e| {
                tracing::error!(path = %self.bots_config_path, "failed to parse bots config: {}", e);
                vec![]
            }),
            Err(e) => {
                tracing::warn!(path = %self.bots_config_path, "bots config not found: {}", e);
                vec![]
            }
        };

        tracing::info!(count = bots.len(), "loaded reply bots from YAML");

        let new_engine =
            BunkBotEngine::new(bots, sender, identity_provider, self.state_service.clone());
        let _ = self.engine.set(new_engine);

        // Register slash commands
        let commands = commands::all_commands();
        if let Ok(guild_id_str) = std::env::var("DEV_GUILD_ID") {
            if let Ok(guild_id_num) = guild_id_str.parse::<u64>() {
                let guild_id = serenity::all::GuildId::new(guild_id_num);
                if let Err(e) = guild_id.set_commands(&ctx.http, commands).await {
                    tracing::error!("failed to register guild commands: {}", e);
                } else {
                    tracing::info!("registered guild-specific slash commands");
                }
            }
        } else {
            if let Err(e) = serenity::all::Command::set_global_commands(&ctx.http, commands).await {
                tracing::error!("failed to register global slash commands: {}", e);
            } else {
                tracing::info!("registered global slash commands");
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Err(e) =
            commands::handle_interaction(&ctx, &interaction, self.state_service.clone()).await
        {
            tracing::error!("error handling interaction: {}", e);
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if !self.filter.check(&ctx, &msg) {
            return;
        }
        if let Some(engine) = self.engine.get() {
            let self_id = ctx.cache.current_user().id;
            engine.handle(&ctx, &msg, self_id).await;
        }
    }
}

pub async fn run() -> anyhow::Result<()> {
    starbunk::utils::run_bot(
        "BunkBot",
        starbunk::utils::default_intents(),
        Handler::new(),
    )
    .await
}
