//! Ragtime — Discord trivia contest bot.
//!
//! * [`question`] / [`repository`] — the question bank.
//! * [`round`] — the Discord-free round state machine.
//! * [`presentation`] — pure embed/component/modal builders.
//! * [`bot`] — Discord wiring: event handler, timers, shared state.

pub mod bot;
pub mod fuzzy;
pub mod presentation;
pub mod question;
pub mod repository;
pub mod round;

pub use fuzzy::{grade, grade_with_threshold, DEFAULT_GRADE_THRESHOLD};

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
    let handler = bot::RagtimeHandler::new(repo, bot::interim_grader());
    let intents = starbunk::utils::default_intents()
        | serenity::all::GatewayIntents::GUILDS
        | serenity::all::GatewayIntents::GUILD_MESSAGE_REACTIONS;
    starbunk::utils::run_bot("Ragtime", intents, handler).await
}
