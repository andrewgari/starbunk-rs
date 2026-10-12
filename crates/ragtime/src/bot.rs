//! Discord event handler. Foundation only: logs `ready`, no commands or game logic.

use crate::repository::QuestionRepository;
use async_trait::async_trait;
use serenity::all::{Context, EventHandler, Ready};
use std::sync::Arc;

pub struct RagtimeHandler {
    #[allow(dead_code)] // consumed by game logic in a later task
    questions: Arc<dyn QuestionRepository>,
}

impl std::fmt::Debug for RagtimeHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RagtimeHandler").finish_non_exhaustive()
    }
}

impl RagtimeHandler {
    pub fn new(questions: Arc<dyn QuestionRepository>) -> Self {
        Self { questions }
    }
}

#[async_trait]
impl EventHandler for RagtimeHandler {
    #[tracing::instrument(skip_all, fields(bot = "ragtime"))]
    async fn ready(&self, _ctx: Context, ready: Ready) {
        tracing::info!(
            bot = "ragtime",
            user = %ready.user.name,
            guilds = ready.guilds.len(),
            "Ragtime connected and ready"
        );
    }
}
