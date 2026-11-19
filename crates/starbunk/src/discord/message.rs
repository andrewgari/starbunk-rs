use super::sender::SenderCategory;
use serenity::all::Message;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct StarbunkMessage {
    pub inner: Message,
    pub sender: SenderCategory,
}

impl StarbunkMessage {
    pub fn new(inner: Message, sender: SenderCategory) -> Self {
        Self { inner, sender }
    }

    pub fn from_serenity(inner: Message) -> Self {
        // Spoofing is only possible when E2E_MODE is explicitly "true"/"1" AND the
        // message came from the runner's own webhook (E2E_WEBHOOK_ID).
        let e2e_webhook = if e2e_mode_enabled(std::env::var("E2E_MODE").ok().as_deref()) {
            std::env::var("E2E_WEBHOOK_ID")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
        } else {
            None
        };
        Self::from_serenity_with(inner, e2e_webhook)
    }

    /// `e2e_webhook_id` is `Some` only when E2E test mode is explicitly enabled.
    pub fn from_serenity_with(mut inner: Message, e2e_webhook_id: Option<u64>) -> Self {
        let mut sender = if inner.author.bot {
            SenderCategory::Bot
        } else {
            SenderCategory::User
        };

        {
            if let Some(webhook_id) = inner.webhook_id {
                let is_e2e_webhook = e2e_webhook_id
                    .map(|id| webhook_id.get() == id)
                    .unwrap_or(false);

                if is_e2e_webhook {
                    if inner.content.starts_with("[E2E_HUMAN]") {
                        inner.content = inner.content["[E2E_HUMAN]".len()..].trim().to_string();
                        sender = SenderCategory::E2eUser;
                    } else if inner.content.starts_with("[E2E_BOT]") {
                        inner.content = inner.content["[E2E_BOT]".len()..].trim().to_string();
                        sender = SenderCategory::E2eBot;
                    } else if inner.content.starts_with("[E2E_MOCK_USER:") {
                        if let Some(end_idx) = inner.content.find(']') {
                            let id_str = &inner.content["[E2E_MOCK_USER:".len()..end_idx];
                            match id_str.parse::<u64>() {
                                Ok(id) => {
                                    inner.author.id = serenity::all::UserId::new(id);
                                    inner.content = inner.content[end_idx + 1..].trim().to_string();
                                    sender = SenderCategory::E2eUser;
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        id_str = %id_str,
                                        err = %e,
                                        "E2E_MOCK_USER prefix has non-numeric id; stripping prefix and treating as E2eUser"
                                    );
                                    inner.content = inner.content[end_idx + 1..].trim().to_string();
                                    sender = SenderCategory::E2eUser;
                                }
                            }
                        }
                    }
                }
            }
        }

        Self { inner, sender }
    }
}

impl Deref for StarbunkMessage {
    type Target = Message;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Returns true only for an explicit opt-in value of `E2E_MODE`.
pub(crate) fn e2e_mode_enabled(value: Option<&str>) -> bool {
    matches!(value, Some("true" | "1"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(content: &str, webhook: Option<u64>) -> Message {
        let mut v = serde_json::json!({
            "id": "1", "channel_id": "1",
            "author": {"id": "1", "username": "u", "bot": false, "discriminator": "0", "public_flags": 0},
            "content": content, "timestamp": "2024-01-01T12:00:00+00:00",
            "edited_timestamp": null, "tts": false, "mention_everyone": false,
            "mentions": [], "mention_roles": [], "attachments": [], "embeds": [],
            "pinned": false, "type": 0
        });
        if let Some(w) = webhook {
            v["webhook_id"] = serde_json::json!(w.to_string());
        }
        serde_json::from_value(v).expect("test message")
    }

    const SPOOF: &str = "[E2E_MOCK_USER:999] hello";

    #[test]
    fn mock_user_prefix_ignored_when_test_mode_off() {
        let m = StarbunkMessage::from_serenity_with(msg(SPOOF, Some(42)), None);
        assert_eq!(m.author.id.get(), 1);
        assert_eq!(m.content, SPOOF);
        assert_eq!(m.sender, SenderCategory::User);
    }

    #[test]
    fn mock_user_prefix_ignored_from_non_webhook_author() {
        let m = StarbunkMessage::from_serenity_with(msg(SPOOF, None), Some(42));
        assert_eq!(m.author.id.get(), 1);
        assert_eq!(m.content, SPOOF);
    }

    #[test]
    fn mock_user_prefix_ignored_from_other_webhook() {
        let m = StarbunkMessage::from_serenity_with(msg(SPOOF, Some(7)), Some(42));
        assert_eq!(m.author.id.get(), 1);
    }

    #[test]
    fn mock_user_prefix_honoured_in_test_mode_from_e2e_webhook() {
        let m = StarbunkMessage::from_serenity_with(msg(SPOOF, Some(42)), Some(42));
        assert_eq!(m.author.id.get(), 999);
        assert_eq!(m.content, "hello");
        assert_eq!(m.sender, SenderCategory::E2eUser);
    }

    #[test]
    fn e2e_mode_requires_explicit_truthy_value() {
        assert!(!e2e_mode_enabled(None));
        assert!(!e2e_mode_enabled(Some("")));
        assert!(!e2e_mode_enabled(Some("false")));
        assert!(!e2e_mode_enabled(Some("0")));
        assert!(e2e_mode_enabled(Some("true")));
        assert!(e2e_mode_enabled(Some("1")));
    }
}
