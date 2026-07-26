pub mod api;
pub mod commands;
pub mod config;
pub mod engine;
pub mod state;
pub mod template;

use async_trait::async_trait;
use engine::BunkBotEngine;

/// Returns `true` if `path` is a YAML bot-config file (`.yml` or `.yaml` extension).
///
/// Used by `start_config_watcher` to ignore editor temp files, swap files, and other
/// filesystem noise that must not trigger a bot-config reload (issue #147).
pub(crate) fn is_yml_config_path(path: &std::path::Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("yml") | Some("yaml")
    )
}

/// Returns `true` when `status` represents a failed reload that should be logged.
///
/// A reload is considered failed when the HTTP status is not a 2xx success code.
/// Used by `start_config_watcher` to surface reload errors instead of silently
/// discarding the `StatusCode` returned by `reload_all_bots` (issue #149).
pub(crate) fn is_reload_failure(status: axum::http::StatusCode) -> bool {
    !status.is_success()
}
use serenity::all::{Context, EventHandler, Interaction, Message, Ready};
use starbunk::discord::{
    DiscordIdentityProvider, DiscordMessageService, MessageService, WebhookService,
};
use starbunk::middleware::{MessageFilter, HAS_CONTENT};
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
}

impl Handler {
    fn new(
        engine: Arc<tokio::sync::RwLock<Option<Arc<BunkBotEngine>>>>,
        state_service: Arc<dyn state::BotStateService>,
        audit: Arc<starbunk::audit::AuditStore>,
    ) -> Self {
        Self {
            filter: HAS_CONTENT.clone(),
            engine,
            state_service,
            audit,
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

        // Read all .yml files in config/bunkbot/
        let mut bots = Vec::new();
        let config_dir =
            std::env::var("BUNKBOT_CONFIG_DIR").unwrap_or_else(|_| "config/bunkbot".to_string());

        let mut read_dir = match tokio::fs::read_dir(&config_dir).await {
            Ok(dir) => dir,
            Err(e) => {
                tracing::warn!(dir = %config_dir, "Failed to read bunkbot config directory: {}", e);
                // Return empty dir iterator equivalent or panic depending on preference. Here we just log.
                return;
            }
        };

        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let path = entry.path();
            if path.is_file()
                && (path.extension().unwrap_or_default() == "yml"
                    || path.extension().unwrap_or_default() == "yaml")
            {
                match tokio::fs::read_to_string(&path).await {
                    Ok(yaml) => {
                        let mut parsed_bots = config::parse_bots(&yaml).unwrap_or_else(|e| {
                            tracing::error!(
                                "failed to parse bots config from {}: {}",
                                path.display(),
                                e
                            );
                            vec![]
                        });
                        bots.append(&mut parsed_bots);
                    }
                    Err(e) => {
                        tracing::error!("failed to read file {}: {}", path.display(), e);
                    }
                }
            }
        }

        tracing::info!(count = bots.len(), "loaded reply bots from filesystem");

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
        if !self.filter.check(&ctx, &msg) {
            return;
        }
        let engine_opt = { self.engine.read().await.clone() };
        if let Some(engine) = engine_opt.as_ref() {
            let self_id = ctx.cache.current_user().id;
            engine.handle(&ctx, &msg, self_id).await;
        }
    }
}

pub async fn run() -> anyhow::Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/starbunk_memory".to_string());

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
    let api_state = api::ApiState {
        engine: engine_ref.clone(),
        config_dir,
    };

    crate::start_config_watcher(api_state.clone());

    let app = api::router(api_state);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!(err = %e, "api server error");
        }
    });

    starbunk::utils::run_bot(
        "BunkBot",
        starbunk::utils::default_intents(),
        Handler::new(engine_ref, state_service, audit),
    )
    .await
}

pub fn start_config_watcher(state: crate::api::ApiState) {
    let config_dir = state.config_dir.clone();

    tokio::spawn(async move {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let mut debouncer = match notify_debouncer_mini::new_debouncer(
            std::time::Duration::from_millis(500),
            move |res: notify_debouncer_mini::DebounceEventResult| {
                if let Err(e) = tx.try_send(res) {
                    tracing::warn!(err = %e, "Dropped config reload event due to full channel");
                }
            },
        ) {
            Ok(d) => d,
            Err(e) => {
                tracing::error!(err = %e, "Failed to create config watcher");
                return;
            }
        };

        if let Err(e) = debouncer.watcher().watch(
            std::path::Path::new(&config_dir),
            notify::RecursiveMode::Recursive,
        ) {
            tracing::error!(err = %e, "Failed to watch config dir");
            return;
        }

        // Keep the debouncer alive
        let _debouncer = debouncer;

        while let Some(res) = rx.recv().await {
            match res {
                Ok(events) => {
                    // Only reload when at least one changed path is a *.yml / *.yaml file.
                    // Editor temp files, swap files, and other filesystem noise are ignored.
                    let has_yml = events.iter().any(|e| is_yml_config_path(e.path.as_path()));
                    if !has_yml {
                        continue;
                    }
                    tracing::info!(
                        event_count = events.len(),
                        "Config change detected, reloading bots."
                    );
                    let status = crate::api::reload_all_bots(&state).await;
                    if is_reload_failure(status) {
                        tracing::warn!(status = %status, "Config reload failed");
                    }
                }
                Err(e) => {
                    tracing::error!(err = %e, "Config watcher error");
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    struct DummySender;
    #[async_trait::async_trait]
    impl starbunk::discord::MessageService for DummySender {
        async fn send(&self, _: serenity::all::ChannelId, _: &str) -> anyhow::Result<Message> {
            unimplemented!()
        }
        async fn send_with_identity(
            &self,
            _: serenity::all::ChannelId,
            _: &str,
            _: starbunk::discord::Identity,
        ) -> anyhow::Result<Message> {
            unimplemented!()
        }
        async fn reply(
            &self,
            _: serenity::all::ChannelId,
            _: serenity::all::MessageId,
            _: &str,
        ) -> anyhow::Result<Message> {
            unimplemented!()
        }
        async fn edit(
            &self,
            _: serenity::all::ChannelId,
            _: serenity::all::MessageId,
            _: &str,
        ) -> anyhow::Result<Message> {
            unimplemented!()
        }
        async fn delete(
            &self,
            _: serenity::all::ChannelId,
            _: serenity::all::MessageId,
        ) -> anyhow::Result<()> {
            unimplemented!()
        }
        async fn close(&self) {}
    }
    struct DummyProvider;
    #[async_trait::async_trait]
    impl starbunk::discord::IdentityProvider for DummyProvider {
        async fn get_identity(
            &self,
            _: serenity::all::UserId,
            _: Option<serenity::all::GuildId>,
        ) -> anyhow::Result<starbunk::discord::Identity> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_hot_reload_config_file() {
        let dir = std::env::temp_dir().join(format!(
            "bunkbot_test_watch_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        tokio::fs::create_dir_all(&dir).await.unwrap();

        let state_service = Arc::new(crate::state::InMemoryBotStateManager::new());
        let engine = Arc::new(BunkBotEngine::new(
            vec![],
            Arc::new(DummySender),
            Arc::new(DummyProvider),
            state_service,
            None,
        ));
        let engine_ref = Arc::new(RwLock::new(Some(engine)));

        let api_state = crate::api::ApiState {
            engine: engine_ref.clone(),
            config_dir: dir.to_string_lossy().to_string(),
        };

        // Start watcher (the function to be implemented by The Artist)
        crate::start_config_watcher(api_state.clone());

        // Wait a bit for the watcher to initialize
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Write a new config file
        let path = format!("{}/bots.yml", api_state.config_dir);
        let dummy_yaml = "reply-bots:\n  - name: test_bot_hot_reload\n    triggers: []\n    identity:\n      type: random";
        tokio::fs::write(&path, dummy_yaml).await.unwrap();

        // Wait a bit for the watcher to detect and reload by polling
        let timeout_duration = std::time::Duration::from_millis(5000);
        let start = tokio::time::Instant::now();
        let mut found = false;

        while start.elapsed() < timeout_duration {
            let engine_lock = engine_ref.read().await;
            if let Some(loaded_engine) = engine_lock.as_ref() {
                if loaded_engine.bot_configs().len() == 1 {
                    found = true;
                    break;
                }
            }
            drop(engine_lock);
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        assert!(found, "Engine should have reloaded 1 bot within timeout");

        let engine_lock = engine_ref.read().await;
        let loaded_engine = engine_lock.as_ref().unwrap();
        let bots = loaded_engine.bot_configs();

        assert_eq!(
            bots[0].0, "test_bot_hot_reload",
            "Reloaded bot should match config"
        );
    }

    // ── issue #147: watcher should only fire for *.yml / *.yaml paths ────────

    #[test]
    fn yml_path_is_accepted() {
        assert!(
            is_yml_config_path(std::path::Path::new("/config/bots.yml")),
            ".yml files must be accepted"
        );
    }

    #[test]
    fn yaml_path_is_accepted() {
        assert!(
            is_yml_config_path(std::path::Path::new("/config/bots.yaml")),
            ".yaml files must be accepted"
        );
    }

    #[test]
    fn non_yml_paths_are_rejected() {
        for name in &[
            "bots.yml.tmp",
            "bots.swp",
            ".bots.yml.swx",
            "bots.json",
            "bots",
        ] {
            assert!(
                !is_yml_config_path(std::path::Path::new(name)),
                "non-yml path '{name}' must be rejected by is_yml_config_path"
            );
        }
    }

    // ── issue #149: failed reload status must be detectable ──────────────────

    #[test]
    fn success_status_is_not_a_failure() {
        assert!(
            !is_reload_failure(axum::http::StatusCode::OK),
            "200 OK must not be reported as a reload failure"
        );
    }

    #[test]
    fn error_status_is_a_failure() {
        assert!(
            is_reload_failure(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
            "500 INTERNAL_SERVER_ERROR must be reported as a reload failure"
        );
    }

    #[test]
    fn other_non_2xx_statuses_are_failures() {
        for status in &[
            axum::http::StatusCode::BAD_REQUEST,
            axum::http::StatusCode::UNAUTHORIZED,
            axum::http::StatusCode::FOUND,
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
        ] {
            assert!(
                is_reload_failure(*status),
                "{status} must be reported as a reload failure"
            );
        }
    }
}
