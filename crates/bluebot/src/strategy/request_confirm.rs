#![allow(dead_code)]
use async_trait::async_trait;
use serenity::all::{Context, Message};
use starbunk::replybot::Strategy;

pub struct RequestConfirmStrategy;

impl RequestConfirmStrategy {
    pub fn new() -> Self {
        Self {}
    }

    pub fn check_trigger(_content: &str) -> bool {
        false
    }

    pub fn extract_friend_name(_content: &str) -> Option<&str> {
        None
    }
}

#[async_trait]
impl Strategy for RequestConfirmStrategy {
    fn name(&self) -> &str {
        "RequestConfirmStrategy"
    }

    async fn should_trigger(&self, _ctx: &Context, _msg: &Message) -> bool {
        // TODO: Implement regex check for "say something nice about"
        false
    }

    async fn response(&self, _ctx: &Context, _msg: &Message) -> String {
        // TODO: extract name and return
        "".to_string()
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn triggers_on_nice_request() {
        assert!(RequestConfirmStrategy::check_trigger(
            "bluebot, say something nice about covabot"
        ));
    }

    #[test]
    fn does_not_trigger_on_normal_message() {
        assert!(!RequestConfirmStrategy::check_trigger("bluebot is cool"));
    }

    #[test]
    fn extracts_friend_name() {
        assert_eq!(
            RequestConfirmStrategy::extract_friend_name(
                "bluebot, say something nice about covabot"
            ),
            None
        );
    }
}
