use serenity::all::CreateCommand;

pub fn shuffle_command() -> CreateCommand {
    CreateCommand::new("shuffle").description("Shuffle the current queue")
}
