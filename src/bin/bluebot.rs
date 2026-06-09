use async_trait::async_trait;
use regex::Regex;
use serenity::all::{Context, EventHandler, Message, Ready};
use starbunk::discord::{DiscordMessageService, WebhookService};
use starbunk::middleware::{all_of, NOT_BOT, NOT_SELF, GUILD_ONLY, HAS_CONTENT};
use starbunk::replybot::{ReplyBot, Strategy};
use std::sync::{Arc, OnceLock};
use tokio::sync::OnceCell;

// ─── BlueStrategy ────────────────────────────────────────────────────────────

/// Pattern matches any plausible reference to "blue": the colour, the job,
/// common homophones, and foreign-language spellings. Word boundaries prevent
/// false positives on "bluetooth", "blueprint", etc.
static BLUE_PATTERN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?i)\b(bluebot|bloo+|bleu|blew|azul|blau|blu+|blue?)\b")
        .expect("blue regex")
});

struct BlueStrategy;

#[async_trait]
impl Strategy for BlueStrategy {
    fn name(&self) -> &str {
        "BlueStrategy"
    }

    async fn should_trigger(&self, _ctx: &Context, msg: &Message) -> bool {
        BLUE_PATTERN.is_match(&msg.content)
    }

    async fn response(&self, _ctx: &Context, _msg: &Message) -> String {
        "Did somebody say Blu?".to_string()
    }
}

// ─── Event handler ───────────────────────────────────────────────────────────

struct BlueBotHandler {
    filter: Arc<dyn starbunk::middleware::MessageFilter>,
    bot: OnceCell<ReplyBot>,
    webhooks: OnceLock<Arc<WebhookService>>,
}

impl BlueBotHandler {
    fn new() -> Self {
        Self {
            filter: all_of(vec![
                NOT_SELF.clone(),
                NOT_BOT.clone(),
                GUILD_ONLY.clone(),
                HAS_CONTENT.clone(),
            ]),
            bot: OnceCell::new(),
            webhooks: OnceLock::new(),
        }
    }
}

#[async_trait]
impl EventHandler for BlueBotHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("BlueBot connected as {}", ready.user.name);
        let ws = Arc::new(WebhookService::new(ctx.http.clone()));
        let _ = self.webhooks.set(ws.clone());
        let _ = self.bot.get_or_init(|| async {
            ReplyBot::new(
                Arc::new(DiscordMessageService::new(ctx.http.clone(), ws)),
                vec![Box::new(BlueStrategy)],
            )
        }).await;
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

// ─── Entry point ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    starbunk::run_bot(
        "BlueBot",
        starbunk::default_intents(),
        BlueBotHandler::new(),
    )
    .await
}
