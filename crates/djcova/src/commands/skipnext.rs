use crate::manager::GuildAudioManager;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn skipnext_command() -> CreateCommand {
    CreateCommand::new("skipnext").description("Skip the next song in the queue that you requested")
}

pub async fn handle(
    ctx: &Context,
    cmd: &CommandInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    let requester_id = cmd.user.id;
    let result = mgr.lock().await.skip_next_by(requester_id);

    let content = match result {
        Some(title) => format!("Removed your next queued song: **{}**", title),
        None => "You have no songs in the queue.".to_string(),
    };

    let _ = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content(content),
            ),
        )
        .await;

    Ok(())
}
