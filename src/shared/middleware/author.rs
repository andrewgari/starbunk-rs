use super::MessageFilter;
use serenity::all::{Context, Message, RoleId, UserId};
use std::sync::{Arc, LazyLock};

/// Drops messages sent by the bot itself.
pub static NOT_SELF: LazyLock<Arc<dyn MessageFilter>> = LazyLock::new(|| Arc::new(NotSelfFilter));

/// Drops messages where the author is any bot account.
pub static NOT_BOT: LazyLock<Arc<dyn MessageFilter>> = LazyLock::new(|| Arc::new(NotBotFilter));

/// Passes only messages where the author is a bot account.
pub static IS_BOT: LazyLock<Arc<dyn MessageFilter>> = LazyLock::new(|| super::not(NOT_BOT.clone()));

/// Passes only messages from the given Discord user ID (string snowflake).
pub fn author_id(id: impl Into<String>) -> Arc<dyn MessageFilter> {
    Arc::new(AuthorIdFilter(id.into()))
}

/// Drops messages from the given user ID.
pub fn not_author_id(id: impl Into<String>) -> Arc<dyn MessageFilter> {
    super::not(author_id(id))
}

/// Passes only messages whose author username equals `name` (case-sensitive).
pub fn author_named(name: impl Into<String>) -> Arc<dyn MessageFilter> {
    Arc::new(AuthorNamedFilter(name.into()))
}

/// Passes only messages where the author holds `role_id` in the guild.
/// Drops if guild member data is unavailable.
pub fn author_has_role(role_id: RoleId) -> Arc<dyn MessageFilter> {
    Arc::new(AuthorHasRoleFilter(role_id))
}

/// Drops messages from `bot_id`. Intended for tests where a serenity [`Context`]
/// is unavailable and the caller knows the bot's ID directly.
pub fn not_self_with_bot_id(bot_id: UserId) -> Arc<dyn MessageFilter> {
    Arc::new(NotSelfBotIdFilter(bot_id))
}

// --- implementations ---

struct NotSelfFilter;
impl MessageFilter for NotSelfFilter {
    fn check(&self, ctx: &Context, msg: &Message) -> bool {
        ctx.cache.current_user().id != msg.author.id
    }
}

struct NotBotFilter;
impl MessageFilter for NotBotFilter {
    fn check(&self, _ctx: &Context, msg: &Message) -> bool {
        !msg.author.bot
    }
}

struct AuthorIdFilter(String);
impl MessageFilter for AuthorIdFilter {
    fn check(&self, _ctx: &Context, msg: &Message) -> bool {
        msg.author.id.to_string() == self.0
    }
}

struct AuthorNamedFilter(String);
impl MessageFilter for AuthorNamedFilter {
    fn check(&self, _ctx: &Context, msg: &Message) -> bool {
        msg.author.name == self.0
    }
}

struct AuthorHasRoleFilter(RoleId);
impl MessageFilter for AuthorHasRoleFilter {
    fn check(&self, ctx: &Context, msg: &Message) -> bool {
        let Some(guild_id) = msg.guild_id else {
            return false;
        };
        ctx.cache
            .guild(guild_id)
            .and_then(|g| {
                g.members
                    .get(&msg.author.id)
                    .map(|m| m.roles.contains(&self.0))
            })
            .unwrap_or(false)
    }
}

struct NotSelfBotIdFilter(UserId);
impl MessageFilter for NotSelfBotIdFilter {
    fn check(&self, _ctx: &Context, msg: &Message) -> bool {
        msg.author.id != self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal valid [`Message`] from a JSON template. The author
    /// has id="1", username="testuser", bot=false.
    fn build_msg() -> Message {
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
            "content": "hello",
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

    fn build_bot_msg() -> Message {
        serde_json::from_value(serde_json::json!({
            "id": "2",
            "channel_id": "1",
            "author": {
                "id": "2",
                "username": "somebot",
                "bot": true,
                "discriminator": "0",
                "public_flags": 0
            },
            "content": "bot message",
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
        .expect("test bot message")
    }

    /// Creates a zeroed fake context. Only safe to pass to filters that never
    /// actually read from `ctx` (i.e., those with `_ctx` parameters).
    /// Uses `ManuallyDrop` to prevent calling `Context::drop` on null Arcs.
    fn check_filter(filter: &dyn MessageFilter, msg: &Message) -> bool {
        // SAFETY: these filters declare `_ctx` and never dereference ctx.
        // A dangling pointer is used only to satisfy the type signature.
        let ctx_ptr = std::ptr::NonNull::<Context>::dangling();
        filter.check(unsafe { ctx_ptr.as_ref() }, msg)
    }

    #[test]
    fn not_bot_passes_human_messages() {
        let msg = build_msg();
        assert!(check_filter(&**NOT_BOT, &msg));
    }

    #[test]
    fn not_bot_drops_bot_messages() {
        let msg = build_bot_msg();
        assert!(!check_filter(&**NOT_BOT, &msg));
    }

    #[test]
    fn is_bot_passes_bot_messages() {
        let msg = build_bot_msg();
        assert!(check_filter(&**IS_BOT, &msg));
    }

    #[test]
    fn is_bot_drops_human_messages() {
        let msg = build_msg();
        assert!(!check_filter(&**IS_BOT, &msg));
    }

    #[test]
    fn author_id_passes_matching_id() {
        let msg = build_msg();
        let filter = author_id("1");
        assert!(check_filter(&*filter, &msg));
    }

    #[test]
    fn author_id_drops_non_matching_id() {
        let msg = build_msg();
        let filter = author_id("999");
        assert!(!check_filter(&*filter, &msg));
    }

    #[test]
    fn not_author_id_drops_matching_id() {
        let msg = build_msg();
        let filter = not_author_id("1");
        assert!(!check_filter(&*filter, &msg));
    }

    #[test]
    fn not_author_id_passes_non_matching_id() {
        let msg = build_msg();
        let filter = not_author_id("999");
        assert!(check_filter(&*filter, &msg));
    }

    #[test]
    fn author_named_passes_matching_name() {
        let msg = build_msg();
        let filter = author_named("testuser");
        assert!(check_filter(&*filter, &msg));
    }

    #[test]
    fn author_named_drops_wrong_name() {
        let msg = build_msg();
        let filter = author_named("other");
        assert!(!check_filter(&*filter, &msg));
    }

    #[test]
    fn not_self_with_bot_id_drops_message_from_bot_id() {
        let msg = build_msg(); // author id = 1
        let bot_id = UserId::new(1);
        let filter = not_self_with_bot_id(bot_id);
        assert!(!check_filter(&*filter, &msg));
    }

    #[test]
    fn not_self_with_bot_id_passes_message_from_other_id() {
        let msg = build_msg(); // author id = 1
        let bot_id = UserId::new(999);
        let filter = not_self_with_bot_id(bot_id);
        assert!(check_filter(&*filter, &msg));
    }
}
