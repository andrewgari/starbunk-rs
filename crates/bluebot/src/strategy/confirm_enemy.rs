#![allow(dead_code)]
use crate::strategy::state::SharedState;
use async_trait::async_trait;
use serenity::all::{Context, Message};
use starbunk::replybot::Strategy;
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(dead_code)]
pub struct ConfirmEnemyStrategy {
    pub state: Arc<RwLock<SharedState>>,
}

impl ConfirmEnemyStrategy {
    pub fn new(state: Arc<RwLock<SharedState>>) -> Self {
        Self { state }
    }

    pub fn check_trigger(
        _is_enemy: bool,
        _content: &str,
        _is_within_reply_window: bool,
        _is_within_murder_cooldown: bool,
    ) -> bool {
        false
    }
}

#[async_trait]
impl Strategy for ConfirmEnemyStrategy {
    fn name(&self) -> &str {
        "ConfirmEnemyStrategy"
    }

    async fn should_trigger(&self, _ctx: &Context, _msg: &Message) -> bool {
        // TODO: Implement enemy check and murder window
        false
    }

    async fn response(&self, _ctx: &Context, _msg: &Message) -> String {
        "MURDER_RESPONSE".to_string()
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn triggers_on_enemy_with_mean_word_within_window() {
        assert!(ConfirmEnemyStrategy::check_trigger(
            true, "fuck", true, false
        ));
    }

    #[test]
    fn does_not_trigger_outside_reply_window() {
        assert!(!ConfirmEnemyStrategy::check_trigger(
            true, "fuck", false, false
        ));
    }

    #[test]
    fn does_not_trigger_within_murder_cooldown() {
        assert!(!ConfirmEnemyStrategy::check_trigger(
            true, "fuck", true, true
        ));
    }

    #[test]
    fn does_not_trigger_on_friend() {
        assert!(!ConfirmEnemyStrategy::check_trigger(
            false, "fuck", true, false
        ));
    }
}
