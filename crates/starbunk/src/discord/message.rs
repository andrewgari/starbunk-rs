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

    pub fn from_serenity(mut inner: Message) -> Self {
        let mut sender = if inner.author.bot {
            SenderCategory::Bot
        } else {
            SenderCategory::User
        };

        if std::env::var("E2E_MODE").is_ok() {
            if let Some(webhook_id) = inner.webhook_id {
                let is_e2e_webhook = std::env::var("E2E_WEBHOOK_ID")
                    .ok()
                    .and_then(|id_str| id_str.parse::<u64>().ok())
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
