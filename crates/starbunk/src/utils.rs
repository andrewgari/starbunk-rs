use std::sync::Arc;

use serenity::all::{EventHandler, GatewayIntents};
use serenity::Client;

use crate::health::HealthMonitor;

/// Start a Discord bot with the given EventHandler and block until the process
/// receives SIGINT / SIGTERM.
pub async fn run_bot(
    bot_name: &str,
    intents: GatewayIntents,
    handler: impl EventHandler + 'static,
) -> anyhow::Result<()> {
    run_bot_inner(bot_name, intents, handler, None).await
}

/// Standard intents used by most reply bots.
pub fn default_intents() -> GatewayIntents {
    GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT
}

/// Like [`run_bot`], but also installs a startup watchdog that logs a
/// structured `health.status = "startup_timeout"` error if the Discord
/// `ready` event has not fired within 30 seconds.
///
/// Bots that opt in pass their [`HealthMonitor`] here and call
/// [`HealthMonitor::on_connected`] from their `EventHandler::ready` impl.
pub async fn run_bot_with_health(
    bot_name: &'static str,
    intents: GatewayIntents,
    health: Arc<HealthMonitor>,
    handler: impl EventHandler + 'static,
) -> anyhow::Result<()> {
    let watchdog = Arc::clone(&health);
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        if !watchdog.is_connected() {
            tracing::error!(
                bot = bot_name,
                health.status = "startup_timeout",
                "health: startup timeout — bot did not connect within 30s; \
                 check DISCORD_TOKEN and network connectivity"
            );
        }
    });

    run_bot_inner(bot_name, intents, handler, Some(health)).await
}

async fn run_bot_inner(
    bot_name: &str,
    intents: GatewayIntents,
    handler: impl EventHandler + 'static,
    health: Option<Arc<HealthMonitor>>,
) -> anyhow::Result<()> {
    crate::health::start_health_server(bot_name, health);

    let token = std::env::var("DISCORD_TOKEN")
        .map_err(|_| anyhow::anyhow!("{}: DISCORD_TOKEN not set", bot_name))?;

    let client_builder = Client::builder(&token, intents);
    let mut client = if std::env::var("E2E_MODE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
    {
        let guild_id_str = std::env::var("E2E_GUILD_ID")
            .map_err(|_| anyhow::anyhow!("E2E_MODE is active but E2E_GUILD_ID is not set"))?;
        let guild_id = guild_id_str
            .parse::<u64>()
            .map_err(|e| anyhow::anyhow!("invalid E2E_GUILD_ID: {}", e))?;

        let webhook_id = if let Ok(wh_str) = std::env::var("E2E_WEBHOOK_ID") {
            let wh_val = wh_str
                .parse::<u64>()
                .map_err(|e| anyhow::anyhow!("invalid E2E_WEBHOOK_ID: {}", e))?;
            Some(serenity::all::WebhookId::new(wh_val))
        } else {
            None
        };

        let e2e_handler = crate::discord::e2e::E2eDebugHandler::new(
            handler,
            serenity::all::GuildId::new(guild_id),
            webhook_id,
        );
        client_builder.event_handler(e2e_handler).await
    } else {
        client_builder.event_handler(handler).await
    }
    .map_err(|e| anyhow::anyhow!("error creating client: {}", e))?;

    tracing::info!(bot = bot_name, "starting");
    client
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("client error: {}", e))?;
    Ok(())
}
