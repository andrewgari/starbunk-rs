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
            input_str,
            cmd.user.name.clone(),
        )
        .await;

    match play_res {
        Ok(msg) => {
            let mut edit = EditInteractionResponse::new().content(&msg);
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

    Ok(())
}
