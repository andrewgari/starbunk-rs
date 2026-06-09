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
}
