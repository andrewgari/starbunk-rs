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

pub fn skip_command() -> CreateCommand {
    CreateCommand::new("skip").description("Skip the current song")
}

pub fn stop_command() -> CreateCommand {
    CreateCommand::new("stop").description("Stop playback and disconnect the bot")
}

pub fn queue_command() -> CreateCommand {
    CreateCommand::new("queue").description("Show the current playback queue")
}

pub fn nowplaying_command() -> CreateCommand {
    CreateCommand::new("nowplaying").description("Show the currently playing song")
}

pub fn history_command() -> CreateCommand {
    CreateCommand::new("history").description("Show session playback history")
}

pub fn shuffle_command() -> CreateCommand {
    CreateCommand::new("shuffle").description("Shuffle the current queue")
}

pub fn help_command() -> CreateCommand {
    CreateCommand::new("help").description("Show all available commands")
}

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

pub fn clear_command() -> CreateCommand {
    CreateCommand::new("clear").description("Clear the current queue")
}

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

pub fn all_commands() -> Vec<CreateCommand> {
    vec![
        play_command(),
        skip_command(),
        stop_command(),
        queue_command(),
        nowplaying_command(),
        history_command(),
        shuffle_command(),
        help_command(),
        volume_command(),
        clear_command(),
        repeat_command(),
    ]
}
