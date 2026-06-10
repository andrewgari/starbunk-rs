use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption};

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
