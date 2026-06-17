use crate::state::BotStateService;
use serenity::all::{CreateCommand, Permissions};

pub fn bot_command() -> CreateCommand {
    CreateCommand::new("bot")
        .description("Manage bot settings")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

#[allow(clippy::too_many_arguments)]
pub fn execute_bot_command(
    _subcommand: &str,
    _bot_name: Option<&str>,
    _setting: Option<&str>,
    _percent: Option<u8>,
    _user_id: &str,
    _is_admin: bool,
    _state_service: &dyn BotStateService,
    _available_bots: &[String],
) -> Result<String, String> {
    // Stub for TDD phase 1
    Err("Not implemented".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::InMemoryBotStateManager;

    fn test_bots() -> Vec<String> {
        vec!["bluebot".to_string(), "bunkbot".to_string()]
    }

    #[test]
    fn test_bot_enable_disable() {
        let state = InMemoryBotStateManager::new();
        let bots = test_bots();

        // Try to disable as non-admin -> should fail
        let res = execute_bot_command(
            "disable",
            Some("bluebot"),
            None,
            None,
            "123",
            false,
            &state,
            &bots,
        );
        assert_eq!(
            res,
            Err("You need administrator permissions to use this command.".to_string())
        );

        // Disable as admin -> success
        let res = execute_bot_command(
            "disable",
            Some("bluebot"),
            None,
            None,
            "123",
            true,
            &state,
            &bots,
        );
        assert_eq!(res, Ok("Bot bluebot has been disabled.".to_string()));
        assert!(!state.is_bot_enabled("bluebot"));

        // Enable as admin -> success
        let res = execute_bot_command(
            "enable",
            Some("bluebot"),
            None,
            None,
            "123",
            true,
            &state,
            &bots,
        );
        assert_eq!(res, Ok("Bot bluebot has been enabled.".to_string()));
        assert!(state.is_bot_enabled("bluebot"));
    }

    #[test]
    fn test_bot_unknown_name() {
        let state = InMemoryBotStateManager::new();
        let bots = test_bots();
        let res = execute_bot_command(
            "disable",
            Some("unknown"),
            None,
            None,
            "123",
            true,
            &state,
            &bots,
        );
        assert_eq!(res, Err("Unknown bot: unknown".to_string()));
    }

    #[test]
    fn test_bot_frequency_override_and_reset() {
        let state = InMemoryBotStateManager::new();
        let bots = test_bots();

        // Override frequency
        let res = execute_bot_command(
            "override",
            Some("bluebot"),
            Some("frequency"),
            Some(30),
            "admin_user",
            true,
            &state,
            &bots,
        );
        assert_eq!(
            res,
            Ok("✅ bluebot frequency set to 30% (was 100%)".to_string())
        );
        assert_eq!(state.get_frequency("bluebot"), Some(30));

        // Reset frequency (has override)
        let res = execute_bot_command(
            "reset",
            Some("bluebot"),
            Some("frequency"),
            None,
            "admin_user",
            true,
            &state,
            &bots,
        );
        assert_eq!(res, Ok("✅ bluebot frequency reset to 100%".to_string()));
        assert_eq!(state.get_frequency("bluebot"), None);

        // Reset frequency again (no override)
        let res = execute_bot_command(
            "reset",
            Some("bluebot"),
            Some("frequency"),
            None,
            "admin_user",
            true,
            &state,
            &bots,
        );
        assert_eq!(
            res,
            Ok("ℹ️ bluebot has no frequency override to reset".to_string())
        );
    }

    #[test]
    fn test_bot_list() {
        let state = InMemoryBotStateManager::new();
        let bots = test_bots();

        // Set an override on one bot, and disable another
        state.disable_bot("bluebot");
        state.set_frequency("bunkbot", 40, "admin", 100);

        let res = execute_bot_command("list", None, None, None, "123", true, &state, &bots);
        let expected = "📊 Bot Status (2 total)\n\n\
                        ❌ bluebot              [DISABLED]\n\
                        ✅ bunkbot              [ENABLED] [FREQ: 40% ← 100%]\n";
        assert_eq!(res, Ok(expected.to_string()));
    }
}
