use crate::manager::GuildAudioManager;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn nowplaying_command() -> CreateCommand {
    CreateCommand::new("nowplaying").description("Show the currently playing song")
}

pub async fn handle(
    ctx: &Context,
    cmd: &CommandInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    let m = mgr.lock().await;

    if let Some(track) = m.get_current_track() {
        let embed = super::shared::create_now_playing_embed(&track);
        let _ = cmd
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .components(vec![super::shared::create_buttons()]),
                ),
            )
            .await;
    } else {
        let _ = cmd
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Nothing is currently playing."),
                ),
            )
            .await;
    }

    Ok(())
}
