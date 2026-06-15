use crate::manager::GuildAudioManager;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    EditInteractionResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn play_command() -> CreateCommand {
    CreateCommand::new("play")
        .description("Play a YouTube URL or query, or upload an audio file")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "input",
                "YouTube URL or search query",
            )
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Attachment,
                "file",
                "Audio file attachment (MP3, FLAC, OGG, WAV)",
            )
            .required(false),
        )
}

pub async fn handle(
    ctx: &Context,
    cmd: &CommandInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    let guild_id = cmd.guild_id.ok_or_else(|| anyhow::anyhow!("no guild_id"))?;

    let _ = cmd.defer(&ctx.http).await;

    let voice_channel_id = guild_id.to_guild_cached(&ctx.cache).and_then(|g| {
        g.voice_states
            .get(&cmd.user.id)
            .and_then(|vs| vs.channel_id)
    });

    let Some(voice_channel) = voice_channel_id else {
        let _ = cmd
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new()
                    .content("You need to be in a voice channel to use this command."),
            )
            .await;
        return Ok(());
    };

    let mut input: Option<String> = None;
    let mut file_url: Option<String> = None;

    for opt in &cmd.data.options {
        if opt.name == "input" {
            input = opt.value.as_str().map(|s| s.to_string());
        } else if opt.name == "file" {
            if let serenity::all::CommandDataOptionValue::Attachment(id) = opt.value {
                if let Some(att) = cmd.data.resolved.attachments.get(&id) {
                    let name = att.filename.to_lowercase();
                    if name.ends_with(".mp3")
                        || name.ends_with(".flac")
                        || name.ends_with(".ogg")
                        || name.ends_with(".wav")
                    {
                        file_url = Some(att.url.clone());
                    } else {
                        let _ = cmd
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new().content(
                                    "Unsupported file type. Please upload an MP3, FLAC, OGG, or WAV file.",
                                ),
                            )
                            .await;
                        return Ok(());
                    }
                }
            }
        }
    }

    let input_str = match (input, file_url) {
        (Some(inp), _) => inp,
        (_, Some(url)) => url,
        _ => {
            let _ = cmd
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content("Please provide a song name, URL, or upload a file."),
                )
                .await;
            return Ok(());
        }
    };

    let play_res = mgr
        .lock()
        .await
        .play(
            Some(ctx.http.clone()),
            cmd.channel_id,
            voice_channel,
            input_str.clone(),
            cmd.user.name.clone(),
            cmd.user.id,
        )
        .await;

    let mut track_id = None;
    let mut needs_resolution = false;
    if let Ok(msg) = &play_res {
        let m = mgr.lock().await;
        if msg.contains("Now playing") {
            if let Some(track) = m.get_current_track() {
                track_id = Some(track.id);
                needs_resolution = track.title == "Loading...";
            }
        } else {
            if let Some(track) = m.get_queue().last() {
                track_id = Some(track.id);
                needs_resolution = track.title == "Loading...";
            }
        }
    }

    match &play_res {
        Ok(msg) => {
            let mut edit = EditInteractionResponse::new().content(msg);
            if msg.contains("Now playing") {
                if let Some(track) = mgr.lock().await.get_current_track() {
                    edit = edit
                        .embed(super::shared::create_now_playing_embed(&track))
                        .components(vec![super::shared::create_buttons()]);
                }
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

    if let (Ok(msg), Some(id)) = (play_res, track_id) {
        if needs_resolution {
            let http = ctx.http.clone();
            let cmd = cmd.clone();
            let mgr = mgr.clone();

            tokio::spawn(async move {
                let voice = {
                    let m = mgr.lock().await;
                    m.get_voice_service()
                };

                let resolved = voice.resolve_metadata(&input_str).await;

                let edit_res = {
                    let mut m = mgr.lock().await;
                    match resolved {
                        Ok(info) => {
                            m.update_track_metadata(
                                id,
                                info.title.clone(),
                                info.duration,
                                info.thumbnail_url.clone(),
                            );

                            let mut edit = EditInteractionResponse::new();
                            if msg.contains("Now playing") {
                                if let Some(track) = m.get_current_track() {
                                    if track.id == id {
                                        let embed = super::shared::create_now_playing_embed(&track);
                                        edit = edit
                                            .content(format!(
                                                "Now playing: {} requested by {}",
                                                track.title, track.requester
                                            ))
                                            .embed(embed)
                                            .components(vec![super::shared::create_buttons()]);
                                        Some(edit)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                let title = info.title;
                                let requester = m
                                    .get_queue()
                                    .iter()
                                    .find(|t| t.id == id)
                                    .map(|t| t.requester.clone())
                                    .or_else(|| {
                                        m.get_current_track()
                                            .filter(|t| t.id == id)
                                            .map(|t| t.requester.clone())
                                    })
                                    .unwrap_or_else(|| cmd.user.name.clone());

                                edit = edit.content(format!(
                                    "Queued: {} requested by {}",
                                    title, requester
                                ));
                                Some(edit)
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to resolve metadata in background: {:?}", e);
                            m.update_track_metadata(
                                id,
                                format!("Unknown: {}", input_str),
                                None,
                                None,
                            );
                            let mut edit = EditInteractionResponse::new();
                            if msg.contains("Now playing") {
                                if let Some(track) = m.get_current_track() {
                                    if track.id == id {
                                        let embed = super::shared::create_now_playing_embed(&track);
                                        edit = edit
                                            .content(format!(
                                                "Now playing: {} requested by {}",
                                                track.title, track.requester
                                            ))
                                            .embed(embed)
                                            .components(vec![super::shared::create_buttons()]);
                                        Some(edit)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                edit = edit.content(format!(
                                    "Queued: Unknown: {} requested by {}",
                                    input_str, cmd.user.name
                                ));
                                Some(edit)
                            }
                        }
                    }
                }; // Mutex lock is dropped here

                if let Some(edit) = edit_res {
                    let _ = cmd.edit_response(&http, edit).await;
                }
            });
        }
    }

    Ok(())
}
