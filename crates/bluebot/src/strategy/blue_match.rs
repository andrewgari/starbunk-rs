use async_trait::async_trait;
use regex::Regex;
use serenity::all::{Context, Message};
use starbunk::replybot::Strategy;
use std::sync::LazyLock;

/// Pattern matches any plausible reference to "blue" — the colour, the job,
/// common homophones, and other-language spellings. Word boundaries (`\b`)
/// prevent false positives on "bluetooth", "blueprint", "blueberry", etc.
static BLUE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(bluebot|bloo+|bleu|blew|azul|blau|blu+|blue?)\b").expect("blue regex")
});

use crate::strategy::state::SharedState;
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(dead_code)]
pub struct BlueStrategy {
    pub state: Arc<RwLock<SharedState>>,
}

impl BlueStrategy {
    pub fn new(state: Arc<RwLock<SharedState>>) -> Self {
        Self { state }
    }

    pub fn response_text() -> &'static str {
        "Did somebody say Blu?"
    }
}

#[async_trait]
impl Strategy for BlueStrategy {
    fn name(&self) -> &str {
        "BlueStrategy"
    }

    async fn should_trigger(&self, _ctx: &Context, msg: &Message) -> bool {
        BLUE_PATTERN.is_match(&msg.content)
    }

    async fn response(&self, _ctx: &Context, _msg: &Message) -> String {
        Self::response_text().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(content: &str) -> bool {
        BLUE_PATTERN.is_match(content)
    }

    #[test]
    fn triggers_on_blue_variants() {
        assert!(matches("i like blue"));
        assert!(matches("Blue is my favourite colour"));
        assert!(matches("BLUE"));
        assert!(matches("I play Blue Mage"));
        assert!(matches("the sky is blue today"));
        assert!(matches("blu"));
        assert!(matches("bluu"));
        assert!(matches("bloo"));
        assert!(matches("blooo"));
        assert!(matches("blew the whistle"));
        assert!(matches("cordon bleu"));
        assert!(matches("azul"));
        assert!(matches("blau"));
        assert!(matches("hey bluebot"));
        assert!(matches("blue!"));
        assert!(matches("say: blue"));
    }

    #[test]
    fn does_not_trigger_on_compound_words() {
        assert!(!matches("connect via bluetooth"));
        assert!(!matches("read the blueprint"));
        assert!(!matches("eat a blueberry"));
        assert!(!matches("hello world"));
        assert!(!matches(""));
        assert!(!matches("I like red"));
        assert!(!matches("12345"));
    }

    #[tokio::test]
    async fn test_name() {
        let s = BlueStrategy::new(SharedState::new());
        assert_eq!(s.name(), "BlueStrategy");
    }

    #[tokio::test]
    async fn response_returns_catchphrase() {
        assert_eq!(BlueStrategy::response_text(), "Did somebody say Blu?");
    }
}
