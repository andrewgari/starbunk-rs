use crate::comment_config::{parse_comment_input, CommentConfigService};
use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, Permissions};
use std::sync::Arc;

pub fn comments_command() -> CreateCommand {
    CreateCommand::new("comments")
        .description("Manage runtime response-pool overrides for reply bots")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "set",
                "Replace a bot's response pool (pipe- or newline-separated)",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "bot_name",
                    "The bot whose pool to replace",
                )
                .required(true)
                .set_autocomplete(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "text",
                    "New responses (pipe- or newline-separated)",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "append",
                "Append to a bot's response pool",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "bot_name",
                    "The bot to append responses for",
                )
                .required(true)
                .set_autocomplete(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "text",
                    "Responses to append (pipe- or newline-separated)",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "get",
                "List the current response-pool overrides for a bot",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "bot_name",
                    "The bot to inspect",
                )
                .required(true)
                .set_autocomplete(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "clear",
                "Remove the response-pool override and restore YAML responses",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "bot_name", "The bot to clear")
                    .required(true)
                    .set_autocomplete(true),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "list",
            "Show all bots with active response-pool overrides",
        ))
}

/// Pure business-logic handler for /comments subcommands.
///
/// All Discord plumbing lives in `commands.rs`; this function is free of
/// Serenity types so it can be unit-tested without a Discord context.
#[allow(clippy::too_many_arguments)]
pub fn execute_comments_command(
    subcommand: &str,
    bot_name: Option<&str>,
    text: Option<&str>,
    is_admin: bool,
    comment_svc: &Arc<dyn CommentConfigService>,
    available_bots: &[(String, u8)],
) -> Result<String, String> {
    // list is read-only and allowed for everyone; all other subcommands require admin.
    if !is_admin && subcommand != "list" {
        return Err("You need administrator permissions to use this command.".to_string());
    }

    match subcommand {
        "list" => {
            let entries = comment_svc.list_all();
            if entries.is_empty() {
                return Ok("No active response-pool overrides.".to_string());
            }
            let mut out = format!("Active overrides ({} bot(s)):\n", entries.len());
            for (name, count) in entries {
                out.push_str(&format!("  {} — {} response(s)\n", name, count));
            }
            Ok(out)
        }

        "set" | "append" | "get" | "clear" => {
            let name = bot_name.ok_or_else(|| "Bot name is required.".to_string())?;

            if !available_bots.iter().any(|(n, _)| n == name) {
                return Err(format!("Unknown bot: {}", name));
            }

            match subcommand {
                "set" => {
                    let raw = text.ok_or_else(|| "Text is required.".to_string())?;
                    let entries = parse_comment_input(raw);
                    if entries.is_empty() {
                        return Err("No valid responses found in text.".to_string());
                    }
                    let count = comment_svc.set_comments(name, entries);
                    Ok(format!(
                        "Response pool for **{}** replaced ({} response(s)).",
                        name, count
                    ))
                }
                "append" => {
                    let raw = text.ok_or_else(|| "Text is required.".to_string())?;
                    let entries = parse_comment_input(raw);
                    if entries.is_empty() {
                        return Err("No valid responses found in text.".to_string());
                    }
                    let count = comment_svc.append_comments(name, entries);
                    Ok(format!(
                        "Appended to **{}** response pool ({} total response(s)).",
                        name, count
                    ))
                }
                "get" => match comment_svc.get_comments(name) {
                    None => Ok(format!("No active override for **{}**.", name)),
                    Some(pool) => {
                        let mut out =
                            format!("Override pool for **{}** ({} item(s)):\n", name, pool.len());
                        for (i, r) in pool.iter().enumerate() {
                            out.push_str(&format!("  {}. {}\n", i + 1, r));
                        }
                        Ok(out)
                    }
                },
                "clear" => {
                    comment_svc.clear_comments(name);
                    Ok(format!(
                        "Override cleared for **{}**. YAML responses restored.",
                        name
                    ))
                }
                _ => unreachable!(),
            }
        }

        _ => Err(format!("Unknown subcommand: {}", subcommand)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comment_config::InMemoryCommentConfigService;

    fn make_svc() -> Arc<dyn CommentConfigService> {
        Arc::new(InMemoryCommentConfigService::new())
    }

    fn bots() -> Vec<(String, u8)> {
        vec![("bluebot".to_string(), 100), ("bunkbot".to_string(), 100)]
    }

    // --- permission checks ---

    #[test]
    fn non_admin_rejects_set() {
        let svc = make_svc();
        let res =
            execute_comments_command("set", Some("bluebot"), Some("hi"), false, &svc, &bots());
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("administrator"));
    }

    #[test]
    fn non_admin_rejects_append() {
        let svc = make_svc();
        let res =
            execute_comments_command("append", Some("bluebot"), Some("hi"), false, &svc, &bots());
        assert!(res.is_err());
    }

    #[test]
    fn non_admin_rejects_get() {
        let svc = make_svc();
        let res = execute_comments_command("get", Some("bluebot"), None, false, &svc, &bots());
        assert!(res.is_err());
    }

    #[test]
    fn non_admin_rejects_clear() {
        let svc = make_svc();
        let res = execute_comments_command("clear", Some("bluebot"), None, false, &svc, &bots());
        assert!(res.is_err());
    }

    #[test]
    fn non_admin_allowed_for_list() {
        let svc = make_svc();
        let res = execute_comments_command("list", None, None, false, &svc, &bots());
        assert!(res.is_ok());
    }

    // --- unknown bot ---

    #[test]
    fn unknown_bot_returns_error_for_set() {
        let svc = make_svc();
        let res = execute_comments_command("set", Some("ghost"), Some("hi"), true, &svc, &bots());
        assert_eq!(res, Err("Unknown bot: ghost".to_string()));
    }

    #[test]
    fn unknown_bot_returns_error_for_append() {
        let svc = make_svc();
        let res =
            execute_comments_command("append", Some("ghost"), Some("hi"), true, &svc, &bots());
        assert_eq!(res, Err("Unknown bot: ghost".to_string()));
    }

    #[test]
    fn unknown_bot_returns_error_for_get() {
        let svc = make_svc();
        let res = execute_comments_command("get", Some("ghost"), None, true, &svc, &bots());
        assert_eq!(res, Err("Unknown bot: ghost".to_string()));
    }

    #[test]
    fn unknown_bot_returns_error_for_clear() {
        let svc = make_svc();
        let res = execute_comments_command("clear", Some("ghost"), None, true, &svc, &bots());
        assert_eq!(res, Err("Unknown bot: ghost".to_string()));
    }

    // --- set ---

    #[test]
    fn set_command_stores_and_reports_count() {
        let svc = make_svc();
        let res = execute_comments_command(
            "set",
            Some("bluebot"),
            Some("hello|world"),
            true,
            &svc,
            &bots(),
        );
        assert!(res.is_ok());
        let msg = res.unwrap();
        assert!(msg.contains("2 response"));
        let pool = svc.get_comments("bluebot").unwrap();
        assert_eq!(pool, vec!["hello", "world"]);
    }

    // --- append ---

    #[test]
    fn append_command_adds_to_pool() {
        let svc = make_svc();
        execute_comments_command("set", Some("bluebot"), Some("a"), true, &svc, &bots()).unwrap();
        let res =
            execute_comments_command("append", Some("bluebot"), Some("b|c"), true, &svc, &bots());
        assert!(res.is_ok());
        let msg = res.unwrap();
        assert!(msg.contains("3 total"));
    }

    // --- get ---

    #[test]
    fn get_command_shows_list_when_present() {
        let svc = make_svc();
        execute_comments_command(
            "set",
            Some("bluebot"),
            Some("alpha|beta"),
            true,
            &svc,
            &bots(),
        )
        .unwrap();
        let res = execute_comments_command("get", Some("bluebot"), None, true, &svc, &bots());
        let msg = res.unwrap();
        assert!(msg.contains("alpha"));
        assert!(msg.contains("beta"));
    }

    #[test]
    fn get_command_reports_none_when_empty() {
        let svc = make_svc();
        let res = execute_comments_command("get", Some("bluebot"), None, true, &svc, &bots());
        let msg = res.unwrap();
        assert!(msg.contains("No active override"));
    }

    // --- clear ---

    #[test]
    fn clear_command_removes_override() {
        let svc = make_svc();
        execute_comments_command("set", Some("bluebot"), Some("a"), true, &svc, &bots()).unwrap();
        let res = execute_comments_command("clear", Some("bluebot"), None, true, &svc, &bots());
        assert!(res.is_ok());
        assert!(svc.get_comments("bluebot").is_none());
    }

    // --- list ---

    #[test]
    fn list_command_shows_all_bots_with_counts() {
        let svc = make_svc();
        execute_comments_command("set", Some("bluebot"), Some("a|b"), true, &svc, &bots()).unwrap();
        execute_comments_command("set", Some("bunkbot"), Some("x"), true, &svc, &bots()).unwrap();
        let res = execute_comments_command("list", None, None, true, &svc, &bots());
        let msg = res.unwrap();
        assert!(msg.contains("bluebot"));
        assert!(msg.contains("bunkbot"));
        assert!(msg.contains("2 response"));
        assert!(msg.contains("1 response"));
    }

    #[test]
    fn list_command_empty_message_when_no_overrides() {
        let svc = make_svc();
        let res = execute_comments_command("list", None, None, true, &svc, &bots());
        let msg = res.unwrap();
        assert!(msg.contains("No active"));
    }
}
