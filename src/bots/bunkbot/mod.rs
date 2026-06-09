use crate::shared::discord::{DiscordMessageService, MessageService, WebhookService};
use crate::shared::middleware::{all_of, HAS_CONTENT, NOT_SELF};
use async_trait::async_trait;
use serenity::all::{Context, EventHandler, Message, Ready};
use std::sync::{Arc, OnceLock};

struct Handler {
    filter: Arc<dyn crate::shared::middleware::MessageFilter>,
    webhooks: OnceLock<Arc<WebhookService>>,
}

impl Handler {
    fn new() -> Self {
        Self {
            filter: all_of(vec![NOT_SELF.clone(), HAS_CONTENT.clone()]),
            webhooks: OnceLock::new(),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("BunkBot connected as {}", ready.user.name);
        let _ = self.webhooks.set(Arc::new(WebhookService::new(ctx.http.clone())));
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if !self.filter.check(&ctx, &msg) {
            return;
        }
        if msg.content == "ping bunkbot" {
            let ws = self
                .webhooks
                .get()
                .cloned()
                .unwrap_or_else(|| Arc::new(WebhookService::new(ctx.http.clone())));
            let sender = DiscordMessageService::new(ctx.http.clone(), ws);
            if let Err(e) = sender.send(msg.channel_id, "Pong from bunkbot!").await {
                tracing::error!("bunkbot: send failed: {}", e);
            }
        }
    }
}

pub async fn run() -> anyhow::Result<()> {
    crate::run_bot("BunkBot", crate::default_intents(), Handler::new()).await
}
