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

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    fn build_msg(content: &str) -> Message {
        serde_json::from_value(serde_json::json!({
            "id": "1",
            "channel_id": "1",
            "author": {
                "id": "1",
                "username": "testuser",
                "bot": false,
                "discriminator": "0",
                "public_flags": 0
            },
            "content": content,
            "timestamp": "2024-01-01T12:00:00+00:00",
            "edited_timestamp": null,
            "tts": false,
            "mention_everyone": false,
            "mentions": [],
            "mention_roles": [],
            "attachments": [],
            "embeds": [],
            "pinned": false,
            "type": 0
        }))
        .expect("test message")
    }

    fn check_filter(filter: &dyn MessageFilter, msg: &Message) -> bool {
        // SAFETY: these filters declare `_ctx` and never dereference ctx.
        // A dangling pointer is used only to satisfy the type signature.
        let ctx_ptr = std::ptr::NonNull::<Context>::dangling();
        filter.check(unsafe { ctx_ptr.as_ref() }, msg)
    }

    #[test]
    fn has_content_passes_non_empty() {
        let msg = build_msg("hello");
        assert!(check_filter(&**HAS_CONTENT, &msg));
    }

    #[test]
    fn has_content_drops_empty_string() {
        let msg = build_msg("");
        assert!(!check_filter(&**HAS_CONTENT, &msg));
    }

    #[test]
    fn has_content_drops_whitespace_only() {
        let msg = build_msg("   ");
        assert!(!check_filter(&**HAS_CONTENT, &msg));
    }

    #[test]
    fn has_attachment_drops_message_without_attachments() {
        let msg = build_msg("no attach");
        assert!(!check_filter(&**HAS_ATTACHMENT, &msg));
    }

    #[test]
    fn content_contains_passes_when_substr_present() {
        let msg = build_msg("hello world");
        let filter = content_contains("world");
        assert!(check_filter(&*filter, &msg));
    }

    #[test]
    fn content_contains_drops_when_substr_absent() {
        let msg = build_msg("hello world");
        let filter = content_contains("foo");
        assert!(!check_filter(&*filter, &msg));
    }

    #[test]
    fn content_matches_passes_on_regex_match() {
        let msg = build_msg("I like blue");
        let filter = content_matches(Regex::new(r"(?i)blue").unwrap());
        assert!(check_filter(&*filter, &msg));
    }

    #[test]
    fn content_matches_drops_on_no_match() {
        let msg = build_msg("I like red");
        let filter = content_matches(Regex::new(r"(?i)blue").unwrap());
        assert!(!check_filter(&*filter, &msg));
    }
}
