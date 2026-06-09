pub mod covabot;
pub mod discord;
pub mod llm;
pub mod memory;
pub mod middleware;
pub mod replybot;

use serenity::all::{EventHandler, GatewayIntents};
use serenity::Client;

/// Generic bot runner. Each binary creates an EventHandler and passes it here.
pub async fn run_bot(
    bot_name: &str,
    intents: GatewayIntents,
    handler: impl EventHandler + 'static,
) -> anyhow::Result<()> {
    let token = std::env::var("DISCORD_TOKEN")
        .map_err(|_| anyhow::anyhow!("{}: DISCORD_TOKEN environment variable not set", bot_name))?;

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .map_err(|e| anyhow::anyhow!("error creating client: {}", e))?;

    tracing::info!(bot = bot_name, "bot starting");
    client
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("client error: {}", e))?;
    Ok(())
}

/// Standard intents used by most reply bots (guild messages + message content).
pub fn default_intents() -> GatewayIntents {
    GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT
}
