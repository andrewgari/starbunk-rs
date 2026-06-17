use serenity::all::{CreateCommand, Permissions};

pub fn clearwebhooks_command() -> CreateCommand {
    CreateCommand::new("clearwebhooks")
        .description("Clear all webhooks made by the bot")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

pub fn execute_clearwebhooks<F>(
    _is_admin: bool,
    _find_webhooks_and_delete: F,
) -> Result<String, String>
where
    F: FnOnce() -> anyhow::Result<usize>,
{
    // Stub for TDD phase 1
    Err("Not implemented".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_clearwebhooks_admin_success() {
        let res = execute_clearwebhooks(true, || Ok(3));
        assert_eq!(res, Ok("Deleted 3 webhook(s).".to_string()));
    }

    #[test]
    #[ignore]
    fn test_clearwebhooks_non_admin_failure() {
        let res = execute_clearwebhooks(false, || Ok(3));
        assert_eq!(
            res,
            Err("You need administrator permissions to use this command.".to_string())
        );
    }

    #[test]
    #[ignore]
    fn test_clearwebhooks_service_error() {
        let res = execute_clearwebhooks(true, || Err(anyhow::anyhow!("mock error")));
        assert_eq!(res, Err("Failed to clear webhooks.".to_string()));
    }
}
