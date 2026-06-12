use serenity::all::{EventHandler, GatewayIntents};
use serenity::Client;

/// Start a Discord bot with the given EventHandler and block until the process
/// receives SIGINT / SIGTERM.
pub async fn run_bot(
    bot_name: &str,
    intents: GatewayIntents,
    handler: impl EventHandler + 'static,
) -> anyhow::Result<()> {
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

/// Standard intents used by most reply bots.
pub fn default_intents() -> GatewayIntents {
    GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT
}
