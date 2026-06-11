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

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
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
