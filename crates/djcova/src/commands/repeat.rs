use crate::manager::{GuildAudioManager, RepeatMode};
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn repeat_command() -> CreateCommand {
    CreateCommand::new("repeat")
        .description("Set repeat mode")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "mode", "off, song, or queue")
                .required(true)
                .add_string_choice("off", "off")
                .add_string_choice("song", "song")
                .add_string_choice("queue", "queue"),
        )
}

pub async fn handle(
    ctx: &Context,
    cmd: &CommandInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    let mode_str = cmd
        .data
        .options
        .first()
        .and_then(|opt| {
            if let serenity::all::CommandDataOptionValue::String(ref val) = opt.value {
                Some(val.as_str())
            } else {
                None
            }
        })
        .unwrap_or("off");

    let mode = match mode_str {
        "song" => RepeatMode::Song,
        "queue" => RepeatMode::Queue,
        _ => RepeatMode::Off,
    };

    mgr.lock().await.set_repeat_mode(mode);

    let _ = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("Repeat mode set to {}.", mode_str)),
            ),
        )
        .await;

    Ok(())
}
