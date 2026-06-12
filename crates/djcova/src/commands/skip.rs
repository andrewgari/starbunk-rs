use crate::manager::{spawn_idle_timer, GuildAudioManager};
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, EditInteractionResponse, Permissions,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn skip_command() -> CreateCommand {
    CreateCommand::new("skip")
        .description("Skip the current song (own songs only; admins skip any)")
}

pub async fn handle(
    ctx: &Context,
    cmd: &CommandInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    let is_admin = cmd
        .member
        .as_ref()
        .and_then(|m| m.permissions)
        .map(|p| p.contains(Permissions::MANAGE_MESSAGES))
        .unwrap_or(false);

    let caller_id = cmd.user.id;

    // Read ownership state under the lock, then release before any HTTP await.
    let should_deny = {
        let manager = mgr.lock().await;
        manager
            .get_current_track()
            .is_some_and(|track| !is_admin && track.requester_id != caller_id)
    };

    if should_deny {
        let _ = cmd
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("You can only skip songs you requested.")
                        .ephemeral(true),
                ),
            )
            .await;
        return Ok(());
    }

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
