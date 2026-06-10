use serenity::all::CreateCommand;

pub fn clear_command() -> CreateCommand {
    CreateCommand::new("clear").description("Clear the current queue")
}
