use super::MessageFilter;
use serenity::all::{Context, Message, RoleId};
use std::sync::{Arc, LazyLock};

/// Drops messages sent by the bot itself.
pub static NOT_SELF: LazyLock<Arc<dyn MessageFilter>> =
    LazyLock::new(|| Arc::new(NotSelfFilter));

/// Drops messages where the author is any bot account.
pub static NOT_BOT: LazyLock<Arc<dyn MessageFilter>> =
    LazyLock::new(|| Arc::new(NotBotFilter));

/// Passes only messages where the author is a bot account.
pub static IS_BOT: LazyLock<Arc<dyn MessageFilter>> =
    LazyLock::new(|| super::not(NOT_BOT.clone()));

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
            .and_then(|g| g.members.get(&msg.author.id).map(|m| m.roles.contains(&self.0)))
            .unwrap_or(false)
    }
}
