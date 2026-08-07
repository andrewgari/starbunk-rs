pub mod api;
pub mod commands;
pub mod config;
pub mod engine;
pub mod startup_dm;
pub mod state;
pub mod template;

use async_trait::async_trait;
use engine::BunkBotEngine;
use serenity::all::{Context, EventHandler, Interaction, Message, Ready};
use starbunk::config_store::ConfigStore;
use starbunk::discord::{
    DiscordIdentityProvider, DiscordMessageService, MessageService, WebhookService,
};
use starbunk::middleware::{MessageFilter, HAS_CONTENT};
use starbunk::tracking::{BotTrackingHandler, PgBotTrackingStore};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<tokio::sync::RwLock<Option<Arc<BunkBotEngine>>>>,
    pub state_service: Arc<dyn state::BotStateService>,
}

struct Handler {
    filter: Arc<dyn MessageFilter>,
    engine: Arc<tokio::sync::RwLock<Option<Arc<BunkBotEngine>>>>,
    state_service: Arc<dyn state::BotStateService>,
    audit: Arc<starbunk::audit::AuditStore>,
    tracking_handler: Arc<BotTrackingHandler>,
}

impl Handler {
    fn new(
        engine: Arc<tokio::sync::RwLock<Option<Arc<BunkBotEngine>>>>,
        state_service: Arc<dyn state::BotStateService>,
        audit: Arc<starbunk::audit::AuditStore>,
        tracking_handler: Arc<BotTrackingHandler>,
    ) -> Self {
        Self {
            filter: HAS_CONTENT.clone(),
            engine,
            state_service,
            audit,
            tracking_handler,
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        self.tracking_handler
            .ready(ctx.clone(), ready.clone())
            .await;
        tracing::info!("BunkBot connected as {}", ready.user.name);

        // Spawn a non-blocking DM notification if the deployment version changed.
        let http_for_dm = ctx.http.clone();
        tokio::spawn(async move {
            if let Err(e) = startup_dm::check_and_notify(&http_for_dm, "BunkBot").await {
                tracing::warn!(err = %e, "startup DM check failed");
            }
        });

        let ws = Arc::new(WebhookService::new(ctx.http.clone()));
        let sender: Arc<dyn MessageService> =
            Arc::new(DiscordMessageService::new(ctx.http.clone(), ws));
        let identity_provider = Arc::new(DiscordIdentityProvider::new(ctx.http.clone()));

        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://starbunk:starbunk@localhost/starbunk_memory".to_string()
        });

        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect(&db_url)
            .await
            .expect("Failed to connect to DB");

        let config_store = Arc::new(
            starbunk::config_store::PgConfigStore::new(pool)
                .await
                .expect("Failed to init config store"),
        );

        let mut bots = Vec::new();
        match config_store.get_all_bots().await {
            Ok(db_bots) => {
                for record in db_bots {
                    if let Ok(parsed) = serde_json::from_value::<config::BotConfig>(record.config) {
                        bots.push(parsed);
                    }
                }
            }
            Err(e) => tracing::error!("Failed to read bots from DB: {}", e),
        }

        // If database is empty, seed from the backup YAML
        if bots.is_empty() {
            tracing::info!("Database empty. Seeding from config/bunkbot/bots.yml");
            let config_dir = std::env::var("BUNKBOT_CONFIG_DIR")
                .unwrap_or_else(|_| "config/bunkbot".to_string());
            let path = format!("{}/bots.yml", config_dir);
            if let Ok(yaml) = tokio::fs::read_to_string(&path).await {
                if let Ok(mut parsed_bots) = config::parse_bots(&yaml) {
                    for bot in &parsed_bots {
                        if let Ok(val) = serde_json::to_value(bot) {
                            let _ = config_store.upsert_bot(&bot.name, val).await;
                        }
                    }
                    bots.append(&mut parsed_bots);
                }
            }
        }

        tracing::info!(count = bots.len(), "loaded reply bots from database");

        let new_engine = BunkBotEngine::new(
            bots,
            sender,
            identity_provider,
            self.state_service.clone(),
            Some(self.audit.clone()),
        );

        let mut engine_lock = self.engine.write().await;
        *engine_lock = Some(Arc::new(new_engine));
        drop(engine_lock);

        // Register slash commands
        let commands = commands::all_commands();
        let mut is_dev = false;
        if let Ok(guild_id_str) = std::env::var("DEV_GUILD_ID") {
            if let Ok(guild_id_num) = guild_id_str.parse::<u64>() {
                is_dev = true;
                let guild_id = serenity::all::GuildId::new(guild_id_num);
                if let Err(e) = guild_id.set_commands(&ctx.http, commands.clone()).await {
                    tracing::error!(err = %e, "Failed to register guild commands");
                } else {
                    tracing::info!("registered guild commands");
                }
            }
        }
        if !is_dev {
            if let Err(e) = serenity::all::Command::set_global_commands(&ctx.http, commands).await {
                tracing::error!(err = %e, "Failed to register global commands");
            } else {
                tracing::info!("registered global commands");
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let engine_opt = { self.engine.read().await.clone() };
        if let Some(engine) = engine_opt.as_ref() {
            if let Err(e) = commands::handle_interaction(&ctx, &interaction, engine).await {
                tracing::error!("error handling interaction: {}", e);
            }
        } else {
            tracing::warn!("received interaction before engine was initialized");
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let msg = starbunk::discord::StarbunkMessage::from_serenity(msg);
        if !self.filter.check(&ctx, &msg) {
            return;
        }
        let engine_opt = { self.engine.read().await.clone() };
        if let Some(engine) = engine_opt.as_ref() {
            let self_id = ctx.cache.current_user().id;
            engine.handle(&ctx, &msg, self_id).await;
        }
    }

    async fn guild_create(&self, ctx: Context, guild: serenity::all::Guild, is_new: Option<bool>) {
        self.tracking_handler.guild_create(ctx, guild, is_new).await;
    }

    async fn guild_delete(
        &self,
        ctx: Context,
        incomplete: serenity::all::UnavailableGuild,
        full: Option<serenity::all::Guild>,
    ) {
        self.tracking_handler
            .guild_delete(ctx, incomplete, full)
            .await;
    }

    async fn channel_create(&self, ctx: Context, channel: serenity::all::GuildChannel) {
        self.tracking_handler.channel_create(ctx, channel).await;
    }

    async fn channel_delete(
        &self,
        ctx: Context,
        channel: serenity::all::GuildChannel,
        messages: Option<Vec<serenity::all::Message>>,
    ) {
        self.tracking_handler
            .channel_delete(ctx, channel, messages)
            .await;
    }
}

pub async fn run() -> anyhow::Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://starbunk:starbunk@localhost/starbunk_memory".to_string());

    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Failed to connect to DB");

    let engine_ref = Arc::new(tokio::sync::RwLock::new(None));
    let audit = Arc::new(starbunk::audit::AuditStore::new(pool.clone()).await?);

    let state_service = Arc::new(state::InMemoryBotStateManager::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9082").await?;
    let config_dir =
        std::env::var("BUNKBOT_CONFIG_DIR").unwrap_or_else(|_| "config/bunkbot".to_string());
    let config_store = Arc::new(
        starbunk::config_store::PgConfigStore::new(pool.clone())
            .await
            .expect("Failed to init config store"),
    );
    let tracking_store = Arc::new(PgBotTrackingStore::new(pool.clone()).await?);
    let tracking_handler = Arc::new(BotTrackingHandler {
        bot_name: "BunkBot".to_string(),
        store: tracking_store.clone(),
    });

    let api_state = api::ApiState {
        engine: engine_ref.clone(),
        config_dir,
        config_store,
        tracking_store,
    };

    let app = api::router(api_state);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!(err = %e, "api server error");
        }
    });

    starbunk::utils::run_bot(
        "BunkBot",
        starbunk::utils::default_intents() | serenity::all::GatewayIntents::GUILDS,
        Handler::new(engine_ref, state_service, audit, tracking_handler),
    )
    .await
}

// Removed start_config_watcher since we no longer hot reload from local filesystem.
