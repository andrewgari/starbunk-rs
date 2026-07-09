use crate::routing::{route_message, RouteError, RouteTarget};
use crate::store::Store;
use serenity::all::{
    ComponentInteraction, Context, CreateActionRow, CreateButton, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage, Message,
};
use std::sync::Arc;

pub async fn handle_dm_message(
    ctx: &Context,
    msg: &Message,
    _store: Arc<dyn Store>,
) -> anyhow::Result<()> {
    if msg.author.bot {
        return Ok(());
    }

    // Since users might be in multiple guilds (unlikely for Ratmas but possible),
    // we would theoretically need to prompt for which guild if multiple are active.
    // For simplicity, we assume one active Ratmas event for now or pick the first.
    // Since RatBot is mostly scoped to a single primary guild logic, we will check
    // if the user is in any active assignment list.

    // Note: To implement properly across multiple guilds without a guild context in a DM,
    // we would need to load all assignments across all active events where this user is present.
    // As a simplification, we will just present the prompt if they are in at least one active ring.
    // We can fetch assignments by searching all guilds, but it's simpler to store
    // a reverse lookup or just let the button interaction handle the specific guild.
    // For now, we will just present the buttons to ANY DM message.

    let buttons = vec![CreateActionRow::Buttons(vec![
        CreateButton::new("send_giftee")
            .label("🎁 Send to your Giftee")
            .style(serenity::all::ButtonStyle::Primary),
        CreateButton::new("send_rat")
            .label("🐀 Send to your SecretRat")
            .style(serenity::all::ButtonStyle::Secondary),
        CreateButton::new("cancel_dm")
            .label("❌ Cancel")
            .style(serenity::all::ButtonStyle::Danger),
    ])];

    msg.channel_id
        .send_message(
            &ctx.http,
            CreateMessage::new()
                .reference_message(msg)
                .content("Where would you like to send this message?")
                .components(buttons),
        )
        .await?;

    Ok(())
}

pub async fn handle_component_interaction(
    ctx: &Context,
    interaction: &ComponentInteraction,
    store: Arc<dyn Store>,
) -> anyhow::Result<()> {
    let custom_id = interaction.data.custom_id.as_str();

    if custom_id == "cancel_dm" {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content("Canceled.")
                        .components(vec![]),
                ),
            )
            .await?;
        return Ok(());
    }

    let target_type = match custom_id {
        "send_giftee" => RouteTarget::Giftee,
        "send_rat" => RouteTarget::SecretRat,
        _ => return Ok(()),
    };

    // We need the original message text.
    // The interaction is on the bot's prompt message.
    // The prompt message replied to the user's message.
    let prompt_msg = &interaction.message;
    let original_msg_id = match &prompt_msg.message_reference {
        Some(rf) => rf.message_id.unwrap(),
        None => {
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .content("Error: Could not find original message.")
                            .components(vec![]),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    let original_msg = ctx
        .http
        .get_message(interaction.channel_id, original_msg_id)
        .await?;

    // We need to find which guild's Ratmas the user is participating in.
    // For this, we'll need to do a DB query to find the assignment for this user.
    // To avoid complex DB schema updates right now, we can just fetch all assignments
    // and find the first one that matches the user.
    // Wait, we don't have a `get_all_assignments` method in Store yet.
    // Let's assume a single guild for the bot's deployment or we can add
    // a method to store to find the user's active assignments.

    // For now, let's add `get_assignments_for_user` to the store.
    // We will do a generic query via pool here if needed, or update Store trait.
    // To keep it clean, let's assume we update the Store trait to include `find_user_guilds`.
    let guilds = store
        .get_active_guilds_for_user(original_msg.author.id)
        .await?;
    if guilds.is_empty() {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content("You are not participating in any active Ratmas events.")
                        .components(vec![]),
                ),
            )
            .await?;
        return Ok(());
    }

    let guild_id = guilds[0]; // Just pick the first active one for now.
    let assignments = store.get_assignments(guild_id).await?;

    match route_message(original_msg.author.id, target_type, &assignments) {
        Ok(target_user_id) => {
            if let Ok(target_user) = target_user_id.to_user(&ctx.http).await {
                if let Ok(dm) = target_user.create_dm_channel(&ctx.http).await {
                    let prefix = if custom_id == "send_giftee" {
                        "[Anonymous Message from your SecretRat] 🐀:"
                    } else {
                        "[Anonymous Message from your Giftee] 🎁:"
                    };

                    let sent_msg = format!("{}\n\n{}", prefix, original_msg.content);
                    if dm
                        .send_message(&ctx.http, CreateMessage::new().content(sent_msg))
                        .await
                        .is_ok()
                    {
                        interaction
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::UpdateMessage(
                                    CreateInteractionResponseMessage::new()
                                        .content("Message sent successfully!")
                                        .components(vec![]),
                                ),
                            )
                            .await?;
                        return Ok(());
                    }
                }
            }
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .content("Failed to send message to the target user.")
                            .components(vec![]),
                    ),
                )
                .await?;
        }
        Err(RouteError::UserNotParticipating) => {
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .content("You are not participating in this Ratmas event.")
                            .components(vec![]),
                    ),
                )
                .await?;
        }
        Err(RouteError::AssignmentNotFound) => {
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .content("Could not find your assignment route.")
                            .components(vec![]),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}
