//! Ragtime — Discord trivia bot.
//!
//! Foundation only: question bank persistence and a connected Discord client.
//! Game logic, grading and the question editor are future work.

pub mod bot;
pub mod question;
pub mod repository;

use std::sync::Arc;

/// Connect to Postgres, run migrations and start the Discord client.
pub async fn run(conn_str: &str) -> anyhow::Result<()> {
    let repo = repository::PgQuestionRepository::connect(conn_str).await?;
    tracing::info!(bot = "ragtime", "question bank migrations applied");
    let handler = bot::RagtimeHandler::new(Arc::new(repo));
    starbunk::utils::run_bot("Ragtime", starbunk::utils::default_intents(), handler).await
}
