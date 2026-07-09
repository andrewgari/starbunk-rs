use crate::assignment::{generate_assignments, AssignmentError};
use crate::store::{EventStatus, Store};
use serenity::all::{
    Command, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
};
use std::sync::Arc;

pub async fn register_commands(ctx: &Context) -> anyhow::Result<()> {
    Command::create_global_command(
        ctx,
        CreateCommand::new("ratmas")
            .description("Manage SecretRat events (Admin only)")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "init",
                    "Initialize a new SecretRat event",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Role,
                        "role",
                        "The role containing participants",
                    )
                    .required(true),
                ),
            )
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "assign",
                "Generate assignments and start Ratmas",
            ))
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "status",
                "Check the status of the Ratmas event",
            ))
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "cancel",
                "Cancel the current Ratmas event",
            )),
    )
    .await?;
    Ok(())
}

pub async fn handle_ratmas_command(
    ctx: &Context,
    command: &CommandInteraction,
    store: Arc<dyn Store>,
) -> anyhow::Result<String> {
    // Check permissions
    let member = match command.member.as_ref() {
        Some(m) => m,
        None => return Ok("Command must be run in a guild.".to_string()),
    };
    let permissions = member.permissions.unwrap_or_default();
    if !permissions.administrator() {
        return Ok("You must be an administrator to use this command.".to_string());
    }

    let guild_id = match command.guild_id {
        Some(id) => id,
        None => return Ok("Command must be run in a guild.".to_string()),
    };
    let subcommand = command.data.options.first();

    if let Some(sub) = subcommand {
        match sub.name.as_str() {
            "init" => {
                if let serenity::all::CommandDataOptionValue::SubCommand(ref options) = sub.value {
                    if let Some(role_opt) = options.first() {
                        if let Some(role_id) = role_opt.value.as_role_id() {
                            store.init_event(guild_id, role_id).await?;
                            return Ok(format!(
                                "Ratmas initialized! Participants will be drawn from <@&{}>.",
                                role_id
                            ));
                        }
                    }
                }
                return Ok("Missing role option for init command.".to_string());
            }
            "assign" => {
                let event = store.get_event(guild_id).await?;
                if let Some(ev) = event {
                    if ev.status == EventStatus::Assigned {
                        return Ok("Assignments have already been made!".to_string());
                    }

                    // Fetch members
                    let guild = guild_id.to_partial_guild(&ctx.http).await?;
                    let mut members = Vec::new();
                    let mut after: Option<serenity::all::UserId> = None;
                    loop {
                        let chunk = guild.members(&ctx.http, Some(1000), after).await?;
                        if chunk.is_empty() {
                            break;
                        }
                        after = Some(chunk.last().unwrap().user.id);
                        for m in chunk {
                            if m.roles.contains(&ev.participant_role_id) {
                                members.push(m.user.id);
                            }
                        }
                    }

                    match generate_assignments(&members) {
                        Ok(assignments) => {
                            store.save_assignments(guild_id, &assignments).await?;
                            store
                                .update_event_status(guild_id, EventStatus::Assigned)
                                .await?;

                            // Send DMs to participants
                            let mut success_count = 0;
                            for a in &assignments {
                                if let Ok(user) = a.gifter.to_user(&ctx.http).await {
                                    if let Ok(giftee) = a.recipient.to_user(&ctx.http).await {
                                        let msg = format!(
                                            "🐀 **Ratmas has begun!** 🎁\n\n\
                                            You are gifting: **{}**\n\n\
                                            Reply to this DM to send them an anonymous message, or to message your SecretRat!",
                                            giftee.name
                                        );
                                        if let Ok(dm) = user.create_dm_channel(&ctx.http).await {
                                            if dm
                                                .send_message(
                                                    &ctx.http,
                                                    serenity::all::CreateMessage::new()
                                                        .content(msg),
                                                )
                                                .await
                                                .is_ok()
                                            {
                                                success_count += 1;
                                            }
                                        }
                                    }
                                }
                            }

                            return Ok(format!(
                                "Ratmas started! Generated {} assignments. Sent {} notification DMs.",
                                assignments.len(), success_count
                            ));
                        }
                        Err(AssignmentError::NotEnoughParticipants) => {
                            return Ok(
                                "Not enough participants. Need at least 3 members with the role."
                                    .to_string(),
                            );
                        }
                        Err(AssignmentError::DuplicateParticipants) => {
                            return Ok("Error: Duplicate participants detected.".to_string());
                        }
                    }
                } else {
                    return Ok("No Ratmas event initialized. Run `/ratmas init` first.".to_string());
                }
            }
            "status" => {
                let event = store.get_event(guild_id).await?;
                if let Some(ev) = event {
                    let status_str = match ev.status {
                        EventStatus::Initializing => "Initializing (Waiting for /ratmas assign)",
                        EventStatus::Assigned => "Assigned (Ratmas is active)",
                    };
                    return Ok(format!(
                        "Ratmas Status: {}\nRole: <@&{}>",
                        status_str, ev.participant_role_id
                    ));
                } else {
                    return Ok("No active Ratmas event for this server.".to_string());
                }
            }
            "cancel" => {
                store.cancel_event(guild_id).await?;
                return Ok("Ratmas event canceled.".to_string());
            }
            _ => return Ok("Unknown subcommand".to_string()),
        }
    }

    Ok("Invalid command usage.".to_string())
}
