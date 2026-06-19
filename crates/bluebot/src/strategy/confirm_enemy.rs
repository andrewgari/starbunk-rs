use crate::strategy::state::SharedState;
use async_trait::async_trait;
use regex::Regex;
use serenity::all::{Context, Message};
use starbunk::replybot::Strategy;
use std::sync::{Arc, LazyLock};
use tokio::sync::RwLock;

static MEAN_WORDS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(fuck(ing)?|hate|die|kill|worst|mom|shit|murder|bots?)\b")
        .expect("mean words regex")
});

pub struct ConfirmEnemyStrategy {
    pub state: Arc<RwLock<SharedState>>,
}

impl ConfirmEnemyStrategy {
    pub fn new(state: Arc<RwLock<SharedState>>) -> Self {
        Self { state }
    }

    pub fn check_trigger(
        is_enemy: bool,
        content: &str,
        is_within_reply_window: bool,
        is_within_murder_cooldown: bool,
    ) -> bool {
        if !is_enemy || !is_within_reply_window || is_within_murder_cooldown {
            return false;
        }
        MEAN_WORDS.is_match(content)
    }
}

#[async_trait]
impl Strategy for ConfirmEnemyStrategy {
    fn name(&self) -> &str {
        "ConfirmEnemyStrategy"
    }

    async fn should_trigger(&self, _ctx: &Context, msg: &Message) -> bool {
        let now = chrono::Utc::now();
        let (is_reply, is_murder, enemy_id) = {
            let state = self.state.read().await;
            (
                state.is_within_reply_window(now),
                state.is_within_murder_window(now),
                state.enemy_user_id,
            )
        };

        let is_enemy = enemy_id != 0 && msg.author.id.get() == enemy_id;

        if Self::check_trigger(is_enemy, &msg.content, is_reply, is_murder) {
            let mut state = self.state.write().await;
            state.clear_reply_window();
            state.open_murder_window(now);
            true
        } else {
            false
        }
    }

    async fn response(&self, _ctx: &Context, _msg: &Message) -> String {
        "I will fucking murder you".to_string()
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
