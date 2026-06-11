use async_trait::async_trait;
use serenity::all::{Context, EventHandler, GuildId, Message, Ready, WebhookId};

/// A wrapper EventHandler used in E2E/debugging mode.
/// It filters out all events from non-whitelisted guilds and intercepts
/// E2E webhook messages to simulate both human and bot authors.
pub struct E2eDebugHandler<H: EventHandler> {
    inner: H,
    debug_guild_id: GuildId,
    e2e_webhook_id: Option<WebhookId>,
}

impl<H: EventHandler> E2eDebugHandler<H> {
    pub fn new(inner: H, debug_guild_id: GuildId, e2e_webhook_id: Option<WebhookId>) -> Self {
        Self {
            inner,
            debug_guild_id,
            e2e_webhook_id,
        }
    }
}

#[async_trait]
impl<H: EventHandler> EventHandler for E2eDebugHandler<H> {
    async fn ready(&self, ctx: Context, data: Ready) {
        tracing::info!(
            "E2E: E2eDebugHandler wrapping bot readiness check. Connected as {}",
            data.user.name
        );
        self.inner.ready(ctx, data).await;
    }

    async fn message(&self, ctx: Context, mut msg: Message) {
        // 1. Whitelist filter: ignore any message from other guilds
        if let Some(guild_id) = msg.guild_id {
            if guild_id != self.debug_guild_id {
                return;
            }
        } else {
            // Drop DM messages in E2E mode
            return;
        }

        // 2. Webhook simulation check
        if let Some(webhook_id) = msg.webhook_id {
            let matches_webhook = match self.e2e_webhook_id {
                Some(wh_id) => webhook_id == wh_id,
                None => true, // Match any webhook in the debug guild if not specified
            };

            if matches_webhook {
                if msg.content.starts_with("[E2E_HUMAN]") {
                    msg.content = msg.content["[E2E_HUMAN]".len()..].trim().to_string();
                    msg.author.bot = false;
                    tracing::debug!(
                        "E2E: Intercepted webhook message, simulated human author. Content: {:?}",
                        msg.content
                    );
                } else if msg.content.starts_with("[E2E_BOT]") {
                    msg.content = msg.content["[E2E_BOT]".len()..].trim().to_string();
                    msg.author.bot = true;
                    tracing::debug!(
                        "E2E: Intercepted webhook message, simulated bot author. Content: {:?}",
                        msg.content
                    );
                }
            }
        }

        self.inner.message(ctx, msg).await;
    }
}
