use serenity::all::CreateCommand;

pub fn nowplaying_command() -> CreateCommand {
    CreateCommand::new("nowplaying").description("Show the currently playing song")
}
