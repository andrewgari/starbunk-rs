use crate::shared::replybot::Strategy;
use async_trait::async_trait;
use regex::Regex;
use serenity::all::{Context, Message};
use std::sync::LazyLock;

/// Pattern matches any plausible reference to "blue" — the colour, the job,
/// common homophones, and other-language spellings. Word boundaries (`\b`)
/// prevent false positives on "bluetooth", "blueprint", "blueberry", etc.
static BLUE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(bluebot|bloo+|bleu|blew|azul|blau|blu+|blue?)\b").expect("blue regex")
});

pub struct BlueStrategy;

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
