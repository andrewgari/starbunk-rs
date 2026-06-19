use crate::strategy::state::SharedState;
use async_trait::async_trait;
use regex::Regex;
use serenity::all::{Context, Message};
use starbunk::replybot::Strategy;
use std::sync::{Arc, LazyLock};
use tokio::sync::RwLock;

static CONFIRM_WORDS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(yes|yeah|yep|yup|true|definitely|absolutely|blue|blu|bluebot|correct|right)\b",
    )
    .expect("confirm words regex")
});

pub struct ReplyConfirmStrategy {
    pub state: Arc<RwLock<SharedState>>,
}

impl ReplyConfirmStrategy {
    pub fn new(state: Arc<RwLock<SharedState>>) -> Self {
        Self { state }
    }

    pub fn check_trigger(
        content: &str,
        is_reply_to_bot: bool,
        is_within_reply_window: bool,
    ) -> bool {
        if !is_reply_to_bot && !is_within_reply_window {
            return false;
        }

        let words = content.split_whitespace().count();
        if words > 5 {
            return false;
        }

        CONFIRM_WORDS.is_match(content)
    }
}

#[async_trait]
impl Strategy for ReplyConfirmStrategy {
    fn name(&self) -> &str {
        "ReplyConfirmStrategy"
    }

    async fn should_trigger(&self, ctx: &Context, msg: &Message) -> bool {
        let is_within_reply_window = self
            .state
            .read()
            .await
            .is_within_reply_window(chrono::Utc::now());

        let is_reply_to_bot = msg
            .referenced_message
            .as_ref()
            .map(|m| m.author.id == ctx.cache.current_user().id)
            .unwrap_or(false);

        if Self::check_trigger(&msg.content, is_reply_to_bot, is_within_reply_window) {
            self.state.write().await.clear_reply_window();
            true
        } else {
            false
        }
    }

    async fn response(&self, _ctx: &Context, _msg: &Message) -> String {
        "Somebody definitely said Blu!".to_string()
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn triggers_within_window_with_confirm_phrase() {
        assert!(ReplyConfirmStrategy::check_trigger("yes", false, true));
    }

    #[test]
    fn does_not_trigger_outside_window() {
        assert!(!ReplyConfirmStrategy::check_trigger("yes", false, false));
    }

    #[test]
    fn does_not_trigger_for_long_messages() {
        assert!(!ReplyConfirmStrategy::check_trigger(
            "yes i am very long and not short",
            false,
            true
        ));
    }

    #[test]
    fn triggers_if_direct_reply() {
        assert!(ReplyConfirmStrategy::check_trigger("yes", true, false));
    }
}
