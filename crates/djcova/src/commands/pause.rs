use crate::manager::GuildAudioManager;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, Permissions,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn pause_command() -> CreateCommand {
    CreateCommand::new("pause")
        .description("Pause or resume playback (admin only)")
        .default_member_permissions(Permissions::MANAGE_MESSAGES)
}

pub async fn handle(
    ctx: &Context,
    cmd: &CommandInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    let mut manager = mgr.lock().await;

    let content = if manager.is_paused() {
        match manager.resume().await {
            Ok(()) => "▶️ Resumed.".to_string(),
            Err(e) => format!("Error: {}", e),
        }
    } else {
        match manager.pause().await {
            Ok(()) => "⏸️ Paused.".to_string(),
            Err(e) => format!("Error: {}", e),
        }
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
