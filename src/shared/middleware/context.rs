use super::MessageFilter;
use chrono::{Datelike, Weekday};
use serenity::all::{ChannelId, Context, Message};
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};

/// Drops direct messages (messages with no guild_id).
pub static GUILD_ONLY: LazyLock<Arc<dyn MessageFilter>> =
    LazyLock::new(|| Arc::new(GuildOnlyFilter));

/// Drops guild messages, passing only direct messages.
pub static DM_ONLY: LazyLock<Arc<dyn MessageFilter>> =
    LazyLock::new(|| Arc::new(DmOnlyFilter));

/// Passes only messages sent in the given channel.
pub fn in_channel(channel_id: ChannelId) -> Arc<dyn MessageFilter> {
    Arc::new(InChannelFilter(channel_id))
}

/// Passes only messages sent on one of the given weekdays (UTC).
pub fn on_weekdays(days: impl IntoIterator<Item = Weekday>) -> Arc<dyn MessageFilter> {
    Arc::new(OnWeekdaysFilter(days.into_iter().collect()))
}

// --- implementations ---

struct GuildOnlyFilter;
impl MessageFilter for GuildOnlyFilter {
    fn check(&self, _ctx: &Context, msg: &Message) -> bool {
        msg.guild_id.is_some()
    }
}

struct DmOnlyFilter;
impl MessageFilter for DmOnlyFilter {
    fn check(&self, _ctx: &Context, msg: &Message) -> bool {
        msg.guild_id.is_none()
    }
}

struct InChannelFilter(ChannelId);
impl MessageFilter for InChannelFilter {
    fn check(&self, _ctx: &Context, msg: &Message) -> bool {
        msg.channel_id == self.0
    }
}

struct OnWeekdaysFilter(HashSet<Weekday>);
impl MessageFilter for OnWeekdaysFilter {
    fn check(&self, _ctx: &Context, msg: &Message) -> bool {
        let secs = msg.timestamp.unix_timestamp();
        if let Some(dt) = chrono::DateTime::from_timestamp(secs, 0) {
            let dt_utc: chrono::DateTime<chrono::Utc> = dt;
            self.0.contains(&dt_utc.weekday())
        } else {
            false
        }
    }
}
