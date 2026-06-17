use crate::state::BotStateService;
use serenity::all::{CreateCommand, Permissions};

pub fn bot_command() -> CreateCommand {
    CreateCommand::new("bot")
        .description("Manage bot settings")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

#[allow(clippy::too_many_arguments)]
pub fn execute_bot_command(
    subcommand: &str,
    bot_name: Option<&str>,
    setting: Option<&str>,
    percent: Option<u8>,
    user_id: &str,
    is_admin: bool,
    state_service: &dyn BotStateService,
    available_bots: &[String],
) -> Result<String, String> {
    if !is_admin && subcommand != "list" {
        return Err("You need administrator permissions to use this command.".to_string());
    }

    if subcommand == "list" {
        let mut response = format!("📊 Bot Status ({} total)\n\n", available_bots.len());
        for bot in available_bots {
            let enabled = state_service.is_bot_enabled(bot);
            let status = if enabled { "✅" } else { "❌" };
            let state_str = if enabled { "[ENABLED]" } else { "[DISABLED]" };

            let mut freq_str = String::new();
            if enabled {
                if let Some(current) = state_service.get_frequency(bot) {
                    let orig = state_service.get_original_frequency(bot).unwrap_or(100);
                    freq_str = format!(" [FREQ: {}% ← {}%]", current, orig);
                }
            }

            response.push_str(&format!(
                "{} {:<21}{}{}\n",
                status, bot, state_str, freq_str
            ));
        }
        return Ok(response);
    }

    let bot_name = bot_name.ok_or_else(|| "Bot name is required".to_string())?;

    if !available_bots.contains(&bot_name.to_string()) {
        return Err(format!("Unknown bot: {}", bot_name));
    }

    match subcommand {
        "enable" => {
            state_service.enable_bot(bot_name);
            Ok(format!("Bot {} has been enabled.", bot_name))
        }
        "disable" => {
            state_service.disable_bot(bot_name);
            Ok(format!("Bot {} has been disabled.", bot_name))
        }
        "override" => {
            let setting = setting.ok_or_else(|| "Setting is required".to_string())?;
            if setting != "frequency" {
                return Err(format!("Unknown setting: {}", setting));
            }
            let percent = percent.ok_or_else(|| "Percent is required".to_string())?;
            let orig = state_service
                .get_original_frequency(bot_name)
                .unwrap_or(100);
            state_service.set_frequency(bot_name, percent, user_id, orig);
            Ok(format!(
                "✅ {} frequency set to {}% (was {}%)",
                bot_name, percent, orig
            ))
        }
        "reset" => {
            let setting = setting.ok_or_else(|| "Setting is required".to_string())?;
            if setting != "frequency" {
                return Err(format!("Unknown setting: {}", setting));
            }
            match state_service.reset_frequency(bot_name) {
                Some(orig) => Ok(format!("✅ {} frequency reset to {}%", bot_name, orig)),
                None => Ok(format!(
                    "ℹ️ {} has no frequency override to reset",
                    bot_name
                )),
            }
        }
        _ => Err(format!("Unknown subcommand: {}", subcommand)),
    }
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
