mod strategy;

use async_trait::async_trait;
use serenity::all::{Context, EventHandler, Message, Ready};
use starbunk::discord::{DiscordMessageService, WebhookService};
use starbunk::middleware::{all_of, GUILD_ONLY, HAS_CONTENT, NOT_BOT, NOT_SELF};
use starbunk::replybot::ReplyBot;
use std::sync::Arc;
use tokio::sync::OnceCell;

struct Handler {
    filter: Arc<dyn starbunk::middleware::MessageFilter>,
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
                let state = strategy::state::SharedState::new();
                ReplyBot::new(
                    Arc::new(DiscordMessageService::new(ctx.http.clone(), ws)),
                    vec![Box::new(strategy::BlueStrategy::new(state.clone()))],
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
    starbunk::utils::run_bot(
        "BlueBot",
        starbunk::utils::default_intents(),
        Handler::new(),
    )
    .await
}
