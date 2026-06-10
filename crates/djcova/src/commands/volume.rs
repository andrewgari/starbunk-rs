use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption};

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
