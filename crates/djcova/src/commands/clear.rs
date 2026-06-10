use crate::manager::GuildAudioManager;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn clear_command() -> CreateCommand {
    CreateCommand::new("clear").description("Clear the current queue")
}

pub async fn handle(
    ctx: &Context,
    cmd: &CommandInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    mgr.lock().await.clear();
    let _ = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("Queue cleared."),
            ),
        )
        .await;
    Ok(())
}
