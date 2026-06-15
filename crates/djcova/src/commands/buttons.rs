use crate::manager::{spawn_idle_timer, GuildAudioManager};
use serenity::all::{ComponentInteraction, Context, EditInteractionResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle(
    ctx: &Context,
    comp: &ComponentInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    let guild_id = comp
        .guild_id
        .ok_or_else(|| anyhow::anyhow!("no guild_id on button interaction"))?;

    match comp.data.custom_id.as_str() {
        "djcova_stop" => {
            tracing::info!(bot = "djcova", guild = %guild_id, user = %comp.user.name, "Button 'Stop' clicked");
            if let Err(e) = mgr.lock().await.stop().await {
                tracing::error!(bot = "djcova", guild = %guild_id, err = %e, "Failed to stop playback via button");
                crate::record_error("button_stop_failed");
            }
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
            tracing::info!(bot = "djcova", guild = %guild_id, user = %comp.user.name, "Button 'Skip' clicked");
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
                        edit = edit.components(vec![]);
                        spawn_idle_timer(mgr.clone());
                    }
                    let _ = comp.edit_response(&ctx.http, edit).await;
                }
                Err(e) => {
                    tracing::error!(
                        bot = "djcova",
                        guild = %guild_id,
                        err = %e,
                        "Failed to skip track via button"
                    );
                    crate::record_error("button_skip_failed");
                    let _ = comp
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new().content(format!("Error: {}", e)),
                        )
                        .await;
                }
            }
        }
        "djcova_restart" => {
            tracing::info!(bot = "djcova", guild = %guild_id, user = %comp.user.name, "Button 'Restart' clicked");
            let result = mgr.lock().await.restart().await;
            match result {
                Ok(msg) => {
                    let _ = comp
                        .edit_response(&ctx.http, EditInteractionResponse::new().content(msg))
                        .await;
                }
                Err(e) => {
                    tracing::error!(
                        bot = "djcova",
                        guild = %guild_id,
                        err = %e,
                        "Failed to restart track via button"
                    );
                    crate::record_error("button_restart_failed");
                    let _ = comp
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new().content(format!("Error: {}", e)),
                        )
                        .await;
                }
            }
        }
        "djcova_requeue" => {
            tracing::info!(bot = "djcova", guild = %guild_id, user = %comp.user.name, "Button 'Re-queue' clicked");
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
                        let title = track.title.clone();
                        let mut m = mgr.lock().await;
                        let play_res = m
                            .play(
                                Some(ctx.http.clone()),
                                comp.channel_id,
                                vc,
                                track.url,
                                track.requester,
                                track.requester_id,
                            )
                            .await;
                        drop(m);

                        match play_res {
                            Ok(_) => {
                                let _ = comp
                                    .edit_response(
                                        &ctx.http,
                                        EditInteractionResponse::new().content(format!(
                                            "Re-queued: {} (Queue size: {})",
                                            title,
                                            queue_len + 1
                                        )),
                                    )
                                    .await;
                            }
                            Err(e) => {
                                tracing::error!(
                                    bot = "djcova",
                                    guild = %guild_id,
                                    err = %e,
                                    "Failed to re-queue track via button"
                                );
                                crate::record_error("button_requeue_failed");
                                let _ = comp
                                    .edit_response(
                                        &ctx.http,
                                        EditInteractionResponse::new()
                                            .content(format!("Error: {}", e)),
                                    )
                                    .await;
                            }
                        }
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
