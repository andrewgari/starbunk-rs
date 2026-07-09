pub mod assignment;
pub mod bot;
pub mod commands;
pub mod interaction;
pub mod routing;
pub mod store;

pub async fn run(conn_str: &str) -> anyhow::Result<()> {
    let store = std::sync::Arc::new(store::PgStore::new(conn_str).await?);
    let handler = bot::RatBotHandler::new(store);
    let intents =
        starbunk::utils::default_intents() | serenity::all::GatewayIntents::DIRECT_MESSAGES;
    starbunk::utils::run_bot("RatBot", intents, handler).await
}
