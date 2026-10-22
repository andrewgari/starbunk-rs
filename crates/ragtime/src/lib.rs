//! Ragtime — Discord trivia contest bot.
//!
//! * [`question`] / [`repository`] — the question bank.
//! * [`round`] — the Discord-free round state machine.
//! * [`presentation`] — pure embed/component/modal builders.
//! * [`api`] — token-protected admin HTTP API over the question bank.
//! * [`bot`] — Discord wiring: event handler, timers, shared state.

pub mod api;
pub mod bot;
pub mod presentation;
pub mod question;
pub mod repository;
pub mod round;

use std::sync::Arc;

/// Connect to Postgres, run migrations and start the Discord client.
pub async fn run(conn_str: &str) -> anyhow::Result<()> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(conn_str)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!(bot = "ragtime", "migrations applied");

    let repo = Arc::new(repository::PgQuestionRepository::from_pool(pool));

    // Token-protected admin HTTP API (fails closed with 401 when no token is set).
    let admin_token = api::admin_token_from_env();
    if admin_token.is_none() {
        tracing::warn!(
            bot = "ragtime",
            "RAGTIME_ADMIN_TOKEN unset; admin API rejects all requests"
        );
    }
    let app = api::router(api::ApiState {
        repo: repo.clone(),
        admin_token,
    });
    let addr = std::env::var("RAGTIME_API_ADDR").unwrap_or_else(|_| "0.0.0.0:9085".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(bot = "ragtime", %addr, "admin API listening");
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!(bot = "ragtime", err = %e, "admin API stopped");
        }
    });

    let handler = bot::RagtimeHandler::new(repo, bot::interim_grader());
    let intents = starbunk::utils::default_intents()
        | serenity::all::GatewayIntents::GUILDS
        | serenity::all::GatewayIntents::GUILD_MESSAGE_REACTIONS;
    starbunk::utils::run_bot("Ragtime", intents, handler).await
}
