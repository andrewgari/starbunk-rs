use async_trait::async_trait;
use regex::Regex;
use serenity::all::{Context, Message};
use starbunk::replybot::Strategy;
use std::sync::LazyLock;

static NICE_REQUEST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)bluebot,\s*say\s+something\s+nice\s+about\s+(.+)").expect("nice request regex")
});

pub struct RequestConfirmStrategy;

impl RequestConfirmStrategy {
    pub fn new() -> Self {
        Self {}
    }

    pub fn check_trigger(content: &str) -> bool {
        NICE_REQUEST.is_match(content)
    }

    pub fn extract_friend_name(content: &str) -> Option<&str> {
        NICE_REQUEST
            .captures(content)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
    }
}

#[async_trait]
impl Strategy for RequestConfirmStrategy {
    fn name(&self) -> &str {
        "RequestConfirmStrategy"
    }

    async fn should_trigger(&self, _ctx: &Context, msg: &Message) -> bool {
        Self::check_trigger(&msg.content)
    }

    async fn response(&self, _ctx: &Context, msg: &Message) -> String {
        if let Some(friend) = Self::extract_friend_name(&msg.content) {
            let author = if friend.to_lowercase() == "me" {
                format!("<@{}>", msg.author.id.get())
            } else {
                friend.to_string()
            };
            format!("{}, I think you're pretty blue! :wink:", author)
        } else {
            "".to_string()
        }
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
            Some("covabot")
        );
    }
}
