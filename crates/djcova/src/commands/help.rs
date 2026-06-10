use serenity::all::CreateCommand;

pub fn help_command() -> CreateCommand {
    CreateCommand::new("help").description("Show all available commands")
}
