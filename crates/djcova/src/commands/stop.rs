use serenity::all::CreateCommand;

pub fn stop_command() -> CreateCommand {
    CreateCommand::new("stop").description("Stop playback and disconnect the bot")
}
