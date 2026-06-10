use serenity::all::CreateCommand;

pub fn history_command() -> CreateCommand {
    CreateCommand::new("history").description("Show session playback history")
}
