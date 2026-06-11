use serenity::all::Http;
use std::collections::HashMap;

/// The persona a bot or poster assumes when sending messages.
#[derive(Debug, Clone, Default)]
pub struct Identity {
    pub nickname: String,
    pub username: String,
    pub avatar_url: String,
    pub metadata: HashMap<String, String>,
}

impl Identity {
    /// Returns true if the Identity carries the minimum fields required to
    /// execute a webhook: a display name and avatar URL.
    pub fn is_valid(&self) -> bool {
        !self.username.is_empty() && !self.avatar_url.is_empty()
    }

    /// If any required webhook field is missing, fills it in from the bot's own
    /// Discord profile. Returns self (consumed) to allow chaining.
    pub async fn resolve(mut self, http: &Http) -> Self {
        if !self.is_valid() {
            if let Ok(user) = http.get_current_user().await {
                if self.username.is_empty() {
                    self.username = user.name.clone();
                }
                if self.avatar_url.is_empty() {
                    self.avatar_url = user.face();
                }
            }
        }
        self
    }
}
