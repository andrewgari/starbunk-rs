use async_trait::async_trait;
use serenity::all::{Context, EventHandler, Message, Ready};
use starbunk::discord::{DiscordMessageService, MessageService, WebhookService};
use starbunk::middleware::{all_of, HAS_CONTENT, NOT_SELF};
use std::sync::{Arc, OnceLock};

struct Handler {
    filter: Arc<dyn starbunk::middleware::MessageFilter>,
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
        tracing::info!(bot = "djcova", user = %ready.user.name, "connected");
        let _ = self
            .webhooks
            .set(Arc::new(WebhookService::new(ctx.http.clone())));
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if !self.filter.check(&ctx, &msg) {
            return;
        }
        if msg.content == "ping djcova" {
            let ws = self
                .webhooks
                .get()
                .cloned()
                .unwrap_or_else(|| Arc::new(WebhookService::new(ctx.http.clone())));
            let sender = DiscordMessageService::new(ctx.http.clone(), ws);
            if let Err(e) = sender.send(msg.channel_id, "Pong from djcova!").await {
                tracing::error!(bot = "djcova", channel = %msg.channel_id, err = %e, "send failed");
            }
        }
    }
}

pub async fn run() -> anyhow::Result<()> {
    starbunk::utils::run_bot("DJCova", starbunk::utils::default_intents(), Handler::new()).await
}
