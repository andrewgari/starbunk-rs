#![allow(dead_code)]
use async_trait::async_trait;
use serenity::all::{Context, Message};
use starbunk::replybot::Strategy;

pub struct RequestEnemyStrategy;

impl RequestEnemyStrategy {
    pub fn new() -> Self {
        Self {}
    }

    pub fn check_trigger(_content: &str, _is_friend_enemy: bool) -> bool {
        false
    }
}

#[async_trait]
impl Strategy for RequestEnemyStrategy {
    fn name(&self) -> &str {
        "RequestEnemyStrategy"
    }

    async fn should_trigger(&self, _ctx: &Context, _msg: &Message) -> bool {
        // TODO: Implement regex check and enemy check
        false
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
