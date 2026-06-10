use crate::manager::GuildAudioManager;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn volume_command() -> CreateCommand {
    CreateCommand::new("volume")
        .description("Set playback volume (0-100)")
        .add_option(
            CreateCommandOption::new(CommandOptionType::Integer, "level", "Volume level (0-100)")
                .required(true)
                .min_int_value(0)
                .max_int_value(100),
        )
}

pub async fn handle(
    ctx: &Context,
    cmd: &CommandInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    let level = cmd
        .data
        .options
        .first()
        .and_then(|opt| {
            if let serenity::all::CommandDataOptionValue::Integer(val) = opt.value {
                Some(val)
            } else {
                None
            }
        })
        .unwrap_or(50);

    mgr.lock().await.set_volume(level.clamp(0, 100) as u8);

    let _ = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("Volume set to {}%.", level)),
            ),
        )
        .await;

    Ok(())
}
