use async_trait::async_trait;
use serenity::all::{Context, EventHandler, Message, Ready};
use starbunk::discord::{DiscordMessageService, MessageService, WebhookService};
use starbunk::middleware::{all_of, NOT_SELF, HAS_CONTENT};
use std::sync::{Arc, OnceLock};

struct DjCovaHandler {
    filter: Arc<dyn starbunk::middleware::MessageFilter>,
    webhooks: OnceLock<Arc<WebhookService>>,
}

impl DjCovaHandler {
    fn new() -> Self {
        Self {
            filter: all_of(vec![NOT_SELF.clone(), HAS_CONTENT.clone()]),
            webhooks: OnceLock::new(),
        }
    }
}

#[async_trait]
impl EventHandler for DjCovaHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("DJCova connected as {}", ready.user.name);
        let _ = self.webhooks.set(Arc::new(WebhookService::new(ctx.http.clone())));
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if !self.filter.check(&ctx, &msg) {
            return;
        }

        if msg.content == "ping djcova" {
            let sender = DiscordMessageService::new(
                ctx.http.clone(),
                self.webhooks
                    .get()
                    .cloned()
                    .unwrap_or_else(|| Arc::new(WebhookService::new(ctx.http.clone()))),
            );
            if let Err(e) = sender.send(msg.channel_id, "Pong from djcova!").await {
                tracing::error!("djcova: failed to send message: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    starbunk::run_bot(
        "DJCova",
        starbunk::default_intents(),
        DjCovaHandler::new(),
    )
    .await
}
