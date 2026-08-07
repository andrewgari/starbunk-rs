pub mod bot;
pub mod clearwebhooks;
pub mod comments;
pub mod ping;

use serenity::all::{
    AutocompleteChoice, Context, CreateAutocompleteResponse, CreateCommand,
    CreateInteractionResponse, CreateInteractionResponseMessage, Interaction,
};

pub fn all_commands() -> Vec<CreateCommand> {
    vec![
        bot::bot_command(),
        clearwebhooks::clearwebhooks_command(),
        comments::comments_command(),
        ping::ping_command(),
    ]
}

pub async fn handle_interaction(
    ctx: &Context,
    interaction: &Interaction,
    engine: &crate::engine::BunkBotEngine,
) -> anyhow::Result<()> {
    // Handle autocomplete interactions for the "comments" command.
    if let Interaction::Autocomplete(ac) = interaction {
        if ac.data.name == "comments" {
            // Find the focused bot_name option anywhere in the subcommand options.
            let focused_value = ac
                .data
                .options
                .iter()
                .find_map(|sub| {
                    if let serenity::all::CommandDataOptionValue::SubCommand(ref opts) = sub.value {
                        opts.iter().find_map(|o| {
                            if o.name == "bot_name" {
                                if let serenity::all::CommandDataOptionValue::Autocomplete {
                                    ref value,
                                    ..
                                } = o.value
                                {
                                    return Some(value.clone());
                                }
                            }
                            None
                        })
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let choices: Vec<AutocompleteChoice> = engine
                .bot_configs()
                .into_iter()
                .filter(|(name, _)| name.to_lowercase().contains(&focused_value.to_lowercase()))
                .take(25)
                .map(|(name, _)| AutocompleteChoice::new(name.clone(), name))
                .collect();

            let _ = ac
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Autocomplete(
                        CreateAutocompleteResponse::new().set_choices(choices),
                    ),
                )
                .await;
        }
        return Ok(());
    }

    if let Interaction::Command(cmd) = interaction {
        let content = match cmd.data.name.as_str() {
            "ping" => ping::execute_ping(),
            "clearwebhooks" => {
                let has_manage_webhooks = cmd
                    .member
                    .as_ref()
                    .map(|m| {
                        m.permissions
                            .unwrap_or_else(serenity::all::Permissions::empty)
                            .manage_webhooks()
                    })
                    .unwrap_or(false);

                match clearwebhooks::execute_clearwebhooks(has_manage_webhooks, || async {
                    let mut count = 0;
                    let webhooks = ctx
                        .http
                        .get_channel_webhooks(cmd.channel_id)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch webhooks: {}", e))?;
                    for webhook in webhooks {
                        // Intentional deviation from JS (which filters by name "Starbunk Bot"):
                        // matching by bot user ID is more robust against name changes.
                        if webhook
                            .user
                            .is_some_and(|u| u.id == ctx.cache.current_user().id)
                            && ctx.http.delete_webhook(webhook.id, None).await.is_ok()
                        {
                            count += 1;
                        }
                    }
                    Ok(count)
                })
                .await
                {
                    Ok(msg) => msg,
                    Err(e) => format!("Error clearing webhooks: {}", e),
                }
            }
            "bot" => {
                let mut result_msg = "Invalid bot command format".to_string();
                if let Some(sub_opt) = cmd.data.options.first() {
                    if let serenity::all::CommandDataOptionValue::SubCommand(ref sub_options) =
                        sub_opt.value
                    {
                        let subcommand = sub_opt.name.as_str();

                        let mut bot_name = None;
                        let mut setting = None;
                        let mut percent = None;

                        for opt in sub_options {
                            match opt.name.as_str() {
                                "bot_name" => {
                                    if let serenity::all::CommandDataOptionValue::String(ref s) =
                                        opt.value
                                    {
                                        bot_name = Some(s.as_str());
                                    }
                                }
                                "setting" => {
                                    if let serenity::all::CommandDataOptionValue::String(ref s) =
                                        opt.value
                                    {
                                        setting = Some(s.as_str());
                                    }
                                }
                                "percent" => {
                                    if let serenity::all::CommandDataOptionValue::Integer(i) =
                                        opt.value
                                    {
                                        percent = Some(i as u8);
                                    }
                                }
                                _ => {}
                            }
                        }

                        let is_admin = cmd
                            .member
                            .as_ref()
                            .map(|m| {
                                m.permissions
                                    .unwrap_or_else(serenity::all::Permissions::empty)
                                    .administrator()
                            })
                            .unwrap_or(false);

                        let state_service = engine.state_service();
                        match bot::execute_bot_command(
                            subcommand,
                            bot_name,
                            setting,
                            percent,
                            &cmd.user.id.to_string(),
                            is_admin,
                            &*state_service,
                            &engine.bot_configs(),
                        ) {
                            Ok(msg) => result_msg = msg,
                            Err(msg) => result_msg = msg,
                        }
                    }
                }
                result_msg
            }
            "comments" => {
                let mut result_msg = "Invalid comments command format".to_string();
                if let Some(sub_opt) = cmd.data.options.first() {
                    if let serenity::all::CommandDataOptionValue::SubCommand(ref sub_options) =
                        sub_opt.value
                    {
                        let subcommand = sub_opt.name.as_str();

                        let mut bot_name: Option<&str> = None;
                        let mut text: Option<&str> = None;

                        for opt in sub_options {
                            match opt.name.as_str() {
                                "bot_name" => {
                                    if let serenity::all::CommandDataOptionValue::String(ref s) =
                                        opt.value
                                    {
                                        bot_name = Some(s.as_str());
                                    }
                                }
                                "text" => {
                                    if let serenity::all::CommandDataOptionValue::String(ref s) =
                                        opt.value
                                    {
                                        text = Some(s.as_str());
                                    }
                                }
                                _ => {}
                            }
                        }

                        let is_admin = cmd
                            .member
                            .as_ref()
                            .map(|m| {
                                m.permissions
                                    .unwrap_or_else(serenity::all::Permissions::empty)
                                    .administrator()
                            })
                            .unwrap_or(false);

                        match comments::execute_comments_command(
                            subcommand,
                            bot_name,
                            text,
                            is_admin,
                            &engine.comment_config_service(),
                            &engine.bot_configs(),
                        ) {
                            Ok(msg) => result_msg = msg,
                            Err(msg) => result_msg = msg,
                        }
                    }
                }
                result_msg
            }
            _ => "Unknown command".to_string(),
        };

        let _ = cmd
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(content)
                        .ephemeral(true),
                ),
            )
            .await;
    }

    if let Interaction::Autocomplete(cmd) = interaction {
        if cmd.data.name == "bot" {
            if let Some(focused) = cmd.data.autocomplete() {
                let bot_configs = engine.bot_configs();
                let matches = bot::filter_bot_names(&bot_configs, focused.value);
                let response = matches
                    .into_iter()
                    .take(25)
                    .fold(CreateAutocompleteResponse::new(), |acc, name| {
                        acc.add_string_choice(name, name)
                    });
                if let Err(e) = cmd
                    .create_response(&ctx.http, CreateInteractionResponse::Autocomplete(response))
                    .await
                {
                    tracing::error!(
                        command = "bot",
                        focused_value = focused.value,
                        err = %e,
                        "failed to send autocomplete response"
                    );
                }
            }
        }
    }

    Ok(())
}
