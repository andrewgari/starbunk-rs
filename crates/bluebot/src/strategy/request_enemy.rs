use crate::strategy::request_confirm::RequestConfirmStrategy;
use async_trait::async_trait;
use serenity::all::{Context, Message};
use starbunk::replybot::Strategy;
use std::env;

pub struct RequestEnemyStrategy {
    enemy_name: String,
}

impl RequestEnemyStrategy {
    pub fn new() -> Self {
        let enemy_name = env::var("BLUEBOT_ENEMY_NAME").unwrap_or_else(|_| "theenemy".to_string());
        Self { enemy_name }
    }

    pub fn check_trigger(content: &str, is_friend_enemy: bool) -> bool {
        is_friend_enemy && RequestConfirmStrategy::check_trigger(content)
    }
}

#[async_trait]
impl Strategy for RequestEnemyStrategy {
    fn name(&self) -> &str {
        "RequestEnemyStrategy"
    }

    async fn should_trigger(&self, _ctx: &Context, msg: &Message) -> bool {
        let extracted = RequestConfirmStrategy::extract_friend_name(&msg.content);
        let is_enemy = extracted
            .map(|name| name.eq_ignore_ascii_case(&self.enemy_name))
            .unwrap_or(false);

        Self::check_trigger(&msg.content, is_enemy)
    }

    async fn response(&self, _ctx: &Context, _msg: &Message) -> String {
        "No way, they can suck my blue cane :unamused:".to_string()
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn triggers_when_friend_is_enemy() {
        assert!(RequestEnemyStrategy::check_trigger(
            "bluebot, say something nice about theenemy",
            true
        ));
    }

    #[test]
    fn does_not_trigger_for_normal_friend() {
        assert!(!RequestEnemyStrategy::check_trigger(
            "bluebot, say something nice about covabot",
            false
        ));
    }
}
