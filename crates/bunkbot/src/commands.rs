pub mod bot;
pub mod clearwebhooks;
pub mod ping;

use serenity::all::{
    Context, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
    Interaction,
};

pub fn all_commands() -> Vec<CreateCommand> {
    vec![
        bot::bot_command(),
        clearwebhooks::clearwebhooks_command(),
        ping::ping_command(),
    ]
}

pub async fn handle_interaction(
    ctx: &Context,
    interaction: &Interaction,
    engine: &crate::engine::BunkBotEngine,
) -> anyhow::Result<()> {
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
    Ok(())
}
