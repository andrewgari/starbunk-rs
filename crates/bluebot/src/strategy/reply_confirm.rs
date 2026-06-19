#![allow(dead_code)]
use crate::strategy::state::SharedState;
use async_trait::async_trait;
use serenity::all::{Context, Message};
use starbunk::replybot::Strategy;
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(dead_code)]
pub struct ReplyConfirmStrategy {
    pub state: Arc<RwLock<SharedState>>,
}

impl ReplyConfirmStrategy {
    pub fn new(state: Arc<RwLock<SharedState>>) -> Self {
        Self { state }
    }

    pub fn check_trigger(
        _content: &str,
        _is_reply_to_bot: bool,
        _is_within_reply_window: bool,
    ) -> bool {
        false
    }
}

#[async_trait]
impl Strategy for ReplyConfirmStrategy {
    fn name(&self) -> &str {
        "ReplyConfirmStrategy"
    }

    async fn should_trigger(&self, _ctx: &Context, _msg: &Message) -> bool {
        // TODO: Implement reply window and confirm phrase check
        false
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
