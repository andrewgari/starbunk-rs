use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

pub fn help_command() -> CreateCommand {
    CreateCommand::new("help").description("Show all available commands")
}

pub async fn handle(ctx: &Context, cmd: &CommandInteraction) -> anyhow::Result<()> {
    let embed = CreateEmbed::new()
        .title("DJCova Commands")
        .field("/play", "Play a song (accepts URL, query, or file)", false)
        .field("/skip", "Skip current song", false)
        .field("/stop", "Stop playback and leave channel", false)
        .field("/queue", "Show upcoming queue", false)
        .field("/history", "Show session history", false)
        .field("/volume", "Set volume (0-100)", false)
        .field("/repeat", "Set repeat mode (off/song/queue)", false)
        .field("/shuffle", "Shuffle queue", false)
        .field("/clear", "Clear queue", false)
        .field("/nowplaying", "Show current track with controls", false);

    let _ = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(embed),
            ),
        )
        .await;

    Ok(())
}
