pub mod bot;
pub mod clearwebhooks;
pub mod ping;

use crate::state::BotStateService;
use serenity::all::{Context, CreateCommand, Interaction};
use std::sync::Arc;

pub fn all_commands() -> Vec<CreateCommand> {
    vec![
        bot::bot_command(),
        clearwebhooks::clearwebhooks_command(),
        ping::ping_command(),
    ]
}

pub async fn handle_interaction(
    _ctx: &Context,
    _interaction: &Interaction,
    _state_service: Arc<dyn BotStateService>,
) -> anyhow::Result<()> {
    // Stub for TDD PR 1
    Ok(())
}
