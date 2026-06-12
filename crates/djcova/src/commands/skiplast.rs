use crate::manager::GuildAudioManager;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn skiplast_command() -> CreateCommand {
    CreateCommand::new("skiplast").description("Skip the last song in the queue that you requested")
}

pub async fn handle(
    ctx: &Context,
    cmd: &CommandInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    let requester = cmd.user.name.as_str();
    let result = mgr.lock().await.skip_last_by(requester);

    let content = match result {
        Some(title) => format!("Removed your last queued song: **{}**", title),
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
