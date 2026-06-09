mod strategy;

use crate::shared::discord::{DiscordMessageService, WebhookService};
use crate::shared::middleware::{all_of, GUILD_ONLY, HAS_CONTENT, NOT_BOT, NOT_SELF};
use crate::shared::replybot::ReplyBot;
use async_trait::async_trait;
use serenity::all::{Context, EventHandler, Message, Ready};
use std::sync::Arc;
use tokio::sync::OnceCell;

struct Handler {
    filter: Arc<dyn crate::shared::middleware::MessageFilter>,
    bot: OnceCell<ReplyBot>,
    webhooks: OnceCell<Arc<WebhookService>>,
}

impl Handler {
    fn new() -> Self {
        Self {
            filter: all_of(vec![
                NOT_SELF.clone(),
                NOT_BOT.clone(),
                GUILD_ONLY.clone(),
                HAS_CONTENT.clone(),
            ]),
            bot: OnceCell::new(),
            webhooks: OnceCell::new(),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("BlueBot connected as {}", ready.user.name);

        let ws = Arc::new(WebhookService::new(ctx.http.clone()));
        let _ = self.webhooks.set(ws.clone());

        let _ = self
            .bot
            .get_or_init(|| async {
                ReplyBot::new(
                    Arc::new(DiscordMessageService::new(ctx.http.clone(), ws)),
                    vec![Box::new(strategy::BlueStrategy)],
                )
            })
            .await;
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if !self.filter.check(&ctx, &msg) {
            return;
        }
        if let Some(bot) = self.bot.get() {
            bot.handle(&ctx, &msg).await;
        }
    }
}

pub async fn run() -> anyhow::Result<()> {
    crate::run_bot("BlueBot", crate::default_intents(), Handler::new()).await
}
