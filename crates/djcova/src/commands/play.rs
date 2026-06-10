use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption};

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
