//! The `/ragtime` slash command (matches ratbot's `/ratmas` convention:
//! one global command with subcommands).

use serenity::all::{
    Command, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
};

/// Top-level slash command name.
pub const COMMAND_NAME: &str = "ragtime";

/// A parsed `/ragtime` subcommand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RagtimeCommand {
    /// `/ragtime start` — host only.
    Start,
    /// `/ragtime cancel` — host only.
    Cancel,
}

impl RagtimeCommand {
    /// Parse a subcommand name.
    pub fn parse(subcommand: &str) -> Option<Self> {
        match subcommand {
            "start" => Some(Self::Start),
            "cancel" => Some(Self::Cancel),
            _ => None,
        }
    }

    /// Parse a received command interaction.
    pub fn from_interaction(command: &CommandInteraction) -> Option<Self> {
        if command.data.name != COMMAND_NAME {
            return None;
        }
        command
            .data
            .options
            .first()
            .and_then(|o| Self::parse(&o.name))
    }
}

/// The command definition registered with Discord.
pub fn command_definition() -> CreateCommand {
    CreateCommand::new(COMMAND_NAME)
        .description("Ragtime trivia contest")
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "start",
            "Start the round you're hosting in this channel",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "cancel",
            "Cancel the round you're hosting in this channel",
        ))
}

/// Register `/ragtime` globally.
pub async fn register_commands(ctx: &Context) -> anyhow::Result<()> {
    Command::create_global_command(ctx, command_definition()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_subcommands_only() {
        assert_eq!(RagtimeCommand::parse("start"), Some(RagtimeCommand::Start));
        assert_eq!(
            RagtimeCommand::parse("cancel"),
            Some(RagtimeCommand::Cancel)
        );
        assert_eq!(RagtimeCommand::parse("stop"), None);
    }

    #[test]
    fn definition_has_start_and_cancel() {
        let json = serde_json::to_value(command_definition()).unwrap();
        assert_eq!(json["name"], COMMAND_NAME);
        let names: Vec<&str> = json["options"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["start", "cancel"]);
    }
}
