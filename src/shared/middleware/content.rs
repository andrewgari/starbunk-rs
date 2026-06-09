use super::MessageFilter;
use regex::Regex;
use serenity::all::{Context, Message};
use std::sync::{Arc, LazyLock};

/// Drops messages with empty or whitespace-only content.
pub static HAS_CONTENT: LazyLock<Arc<dyn MessageFilter>> =
    LazyLock::new(|| Arc::new(HasContentFilter));

/// Passes only messages that include at least one file attachment.
pub static HAS_ATTACHMENT: LazyLock<Arc<dyn MessageFilter>> =
    LazyLock::new(|| Arc::new(HasAttachmentFilter));

/// Passes only messages whose content includes `substr`.
pub fn content_contains(substr: impl Into<String>) -> Arc<dyn MessageFilter> {
    Arc::new(ContentContainsFilter(substr.into()))
}

/// Passes only messages whose content matches the given compiled regex.
pub fn content_matches(re: Regex) -> Arc<dyn MessageFilter> {
    Arc::new(ContentMatchesFilter(re))
}

// --- implementations ---

struct HasContentFilter;
impl MessageFilter for HasContentFilter {
    fn check(&self, _ctx: &Context, msg: &Message) -> bool {
        !msg.content.trim().is_empty()
    }
}

struct HasAttachmentFilter;
impl MessageFilter for HasAttachmentFilter {
    fn check(&self, _ctx: &Context, msg: &Message) -> bool {
        !msg.attachments.is_empty()
    }
}

struct ContentContainsFilter(String);
impl MessageFilter for ContentContainsFilter {
    fn check(&self, _ctx: &Context, msg: &Message) -> bool {
        msg.content.contains(&self.0)
    }
}

struct ContentMatchesFilter(Regex);
impl MessageFilter for ContentMatchesFilter {
    fn check(&self, _ctx: &Context, msg: &Message) -> bool {
        self.0.is_match(&msg.content)
    }
}
