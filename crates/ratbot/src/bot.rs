use crate::commands::{handle_ratmas_command, register_commands};
use crate::interaction::{handle_component_interaction, handle_dm_message};
use crate::store::Store;
use async_trait::async_trait;
use serenity::all::{Context, EventHandler, Interaction, Message, Ready};
use std::sync::Arc;

pub struct RatBotHandler {
    store: Arc<dyn Store>,
}

impl RatBotHandler {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl EventHandler for RatBotHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("RatBot connected as {}", ready.user.name);
        if let Err(e) = register_commands(&ctx).await {
            tracing::error!("Failed to register RatBot commands: {}", e);
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if msg.guild_id.is_none() {
            if let Err(e) = handle_dm_message(&ctx, &msg, self.store.clone()).await {
                tracing::error!("Error handling DM message: {}", e);
            }
        } else if msg.content == "ping ratbot" {
            if let Err(e) = msg
                .channel_id
                .send_message(
                    &ctx.http,
                    serenity::all::CreateMessage::new().content("Pong from ratbot!"),
                )
                .await
            {
                tracing::error!("Failed to send ping response: {}", e);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                if command.data.name == "ratmas" {
                    let response =
                        match handle_ratmas_command(&ctx, &command, self.store.clone()).await {
                            Ok(msg) => msg,
                            Err(e) => {
                                tracing::error!("Ratmas command error: {}", e);
                                "An error occurred executing the command.".to_string()
                            }
                        };

                    let _ = command
                        .create_response(
                            &ctx.http,
                            serenity::all::CreateInteractionResponse::Message(
                                serenity::all::CreateInteractionResponseMessage::new()
                                    .content(response),
                            ),
                        )
                        .await;
                }
            }
            Interaction::Component(component) => {
                if let Err(e) =
                    handle_component_interaction(&ctx, &component, self.store.clone()).await
                {
                    tracing::error!("Error handling component interaction: {}", e);
                }
            }
            _ => {}
        }
    }
}
