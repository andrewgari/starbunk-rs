use crate::manager::{spawn_idle_timer, GuildAudioManager};
use serenity::all::{ComponentInteraction, Context, EditInteractionResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle(
    ctx: &Context,
    comp: &ComponentInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    match comp.data.custom_id.as_str() {
        "djcova_stop" => {
            let _ = mgr.lock().await.stop().await;
            let _ = comp
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content("Playback stopped and disconnected.")
                        .components(vec![]),
                )
                .await;
        }
        "djcova_skip" => {
            let skip_res = mgr.lock().await.skip(Some(ctx.http.clone())).await;
            if let Ok(msg) = skip_res {
                let mut edit = EditInteractionResponse::new().content(&msg);
                if msg.contains("Skipped to") {
                    if let Some(track) = mgr.lock().await.get_current_track() {
                        edit = edit
                            .embed(super::shared::create_now_playing_embed(&track))
                            .components(vec![super::shared::create_buttons()]);
                    }
                } else if msg.contains("stopped") {
                    edit = edit.components(vec![]);
                    spawn_idle_timer(mgr.clone());
                }
                let _ = comp.edit_response(&ctx.http, edit).await;
            }
        }
        "djcova_restart" => {
            let result = mgr.lock().await.restart().await;
            match result {
                Ok(msg) => {
                    let _ = comp
                        .edit_response(&ctx.http, EditInteractionResponse::new().content(msg))
                        .await;
                }
                Err(_) => {
                    let _ = comp
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new().content("Nothing is currently playing."),
                        )
                        .await;
                }
            }
        }
        "djcova_requeue" => {
            let (track, queue_len, voice_channel) = {
                let m = mgr.lock().await;
                (
                    m.get_current_track(),
                    m.get_queue().len(),
                    m.voice_channel_id,
                )
            };

            if let Some(track) = track {
                match voice_channel {
                    Some(vc) => {
                        let mut m = mgr.lock().await;
                        let _ = m
                            .play(
                                Some(ctx.http.clone()),
                                comp.channel_id,
                                vc,
                                track.url,
                                track.requester,
                            )
                            .await;
                        drop(m);
                        let _ = comp
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new().content(format!(
                                    "Re-queued: {} (Queue size: {})",
                                    track.title,
                                    queue_len + 1
                                )),
                            )
                            .await;
                    }
                    None => {
                        let _ = comp
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new()
                                    .content("Cannot re-queue: bot is not in a voice channel."),
                            )
                            .await;
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}
