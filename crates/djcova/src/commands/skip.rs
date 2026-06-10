use serenity::all::CreateCommand;

pub fn skip_command() -> CreateCommand {
    CreateCommand::new("skip").description("Skip the current song")
}
