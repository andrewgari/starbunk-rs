use super::strategy::Strategy;
use crate::shared::discord::MessageService;
use serenity::all::{Context, Message};
use std::sync::Arc;

/// Dispatches incoming Discord messages through an ordered list of strategies.
/// The first strategy whose `should_trigger` returns true wins; the rest are
/// skipped. Optionally pre-condition guards run before `should_trigger`.
pub struct ReplyBot {
    strategies: Vec<Box<dyn Strategy>>,
    sender: Arc<dyn MessageService>,
}

impl ReplyBot {
    pub fn new(sender: Arc<dyn MessageService>, strategies: Vec<Box<dyn Strategy>>) -> Self {
        Self { strategies, sender }
    }

    pub async fn handle(&self, ctx: &Context, msg: &Message) {
        for strategy in &self.strategies {
            // Check optional per-strategy condition.
            if let Some(cond) = strategy.condition() {
                if !cond.check(ctx, msg) {
                    continue;
                }
            }

            if !strategy.should_trigger(ctx, msg).await {
                continue;
            }

            let resp = strategy.response(ctx, msg).await;

            if let Some(identity) = strategy.identity(ctx, msg).await {
                if let Err(e) = self
                    .sender
                    .send_with_identity(msg.channel_id, &resp, identity)
                    .await
                {
                    tracing::error!(
                        strategy = strategy.name(),
                        channel = %msg.channel_id,
                        "failed to send identified response: {}",
                        e
                    );
                }
            } else if let Err(e) = self.sender.send(msg.channel_id, &resp).await {
                tracing::error!(
                    strategy = strategy.name(),
                    channel = %msg.channel_id,
                    "failed to send response: {}",
                    e
                );
            }
            return;
        }
    }
}
