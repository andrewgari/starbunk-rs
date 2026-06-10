use crate::manager::{spawn_idle_timer, GuildAudioManager};
use serenity::all::{CommandInteraction, Context, CreateCommand, EditInteractionResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn skip_command() -> CreateCommand {
    CreateCommand::new("skip").description("Skip the current song")
}

pub async fn handle(
    ctx: &Context,
    cmd: &CommandInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    let _ = cmd.defer(&ctx.http).await;

    let skip_res = mgr.lock().await.skip(Some(ctx.http.clone())).await;

    match skip_res {
        Ok(msg) => {
            let mut edit = EditInteractionResponse::new().content(&msg);
            if msg.contains("Skipped to") {
                if let Some(track) = mgr.lock().await.get_current_track() {
                    edit = edit
                        .embed(super::shared::create_now_playing_embed(&track))
                        .components(vec![super::shared::create_buttons()]);
                }
            } else if msg.contains("stopped") {
                spawn_idle_timer(mgr.clone());
            }
            let _ = cmd.edit_response(&ctx.http, edit).await;
        }
        Err(e) => {
            let _ = cmd
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().content(format!("Error: {}", e)),
                )
                .await;
        }
    }

    Ok(())
}
