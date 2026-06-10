use serenity::all::CreateCommand;

pub fn queue_command() -> CreateCommand {
    CreateCommand::new("queue").description("Show the current playback queue")
}
