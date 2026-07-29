use async_trait::async_trait;
use serenity::all::{Context, EventHandler, GuildId, Message, Ready};

/// A wrapper EventHandler used in E2E/debugging mode.
/// It filters out all events from non-whitelisted guilds and intercepts
/// E2E webhook messages to simulate both human and bot authors.
pub struct E2eDebugHandler<H: EventHandler> {
    inner: H,
    debug_guild_id: GuildId,
}

impl<H: EventHandler> E2eDebugHandler<H> {
    pub fn new(inner: H, debug_guild_id: GuildId) -> Self {
        Self {
            inner,
            debug_guild_id,
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

    async fn message(&self, ctx: Context, msg: Message) {
        // 1. Whitelist filter: ignore any message from other guilds
        if let Some(guild_id) = msg.guild_id {
            if guild_id != self.debug_guild_id {
                return;
            }
        } else {
            // Drop DM messages in E2E mode
            return;
        }

        // 2. We no longer intercept and mutate the webhook messages here.
        // That logic has been moved to StarbunkMessage::from_serenity where it translates
        // to a SenderCategory enum without mutating the Serenity Message's author field.

        self.inner.message(ctx, msg).await;
    }
}
