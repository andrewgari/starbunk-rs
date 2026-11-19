//! Ragtime — Discord trivia bot.
//!
//! TDD PR 1: crate skeleton, question model, repository trait and tests.
//! The bot runtime and the Postgres repository are implemented in PR 2.

pub mod question;
pub mod repository;

/// Connect to Postgres, run migrations and start the Discord client.
pub async fn run(_conn_str: &str) -> anyhow::Result<()> {
    todo!("ragtime: bot runtime (PR 2)")
}
