use serenity::all::{CreateCommand, Permissions};

pub fn clearwebhooks_command() -> CreateCommand {
    CreateCommand::new("clearwebhooks")
        .description("Clear all webhooks made by the bot")
        // JS reference (clear-webhooks.ts) uses ManageWebhooks, not ADMINISTRATOR.
        .default_member_permissions(Permissions::MANAGE_WEBHOOKS)
}

pub async fn execute_clearwebhooks<F, Fut>(
    has_manage_webhooks: bool,
    find_webhooks_and_delete: F,
) -> Result<String, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<usize>>,
{
    if !has_manage_webhooks {
        return Err("You need Manage Webhooks permission to use this command.".to_string());
    }

    match find_webhooks_and_delete().await {
        Ok(count) => Ok(format!("Deleted {} webhook(s).", count)),
        Err(_) => Err("Failed to clear webhooks.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clearwebhooks_with_permission_success() {
        let res = execute_clearwebhooks(true, || async { Ok(3) }).await;
        assert_eq!(res, Ok("Deleted 3 webhook(s).".to_string()));
    }

    #[tokio::test]
    async fn test_clearwebhooks_without_permission_failure() {
        let res = execute_clearwebhooks(false, || async { Ok(3) }).await;
        assert_eq!(
            res,
            Err("You need Manage Webhooks permission to use this command.".to_string())
        );
    }

    #[tokio::test]
    async fn test_clearwebhooks_service_error() {
        let res =
            execute_clearwebhooks(true, || async { Err(anyhow::anyhow!("mock error")) }).await;
        assert_eq!(res, Err("Failed to clear webhooks.".to_string()));
    }
}
