use super::MessageFilter;
use chrono::{Datelike, Weekday};
use serenity::all::{ChannelId, Context, Message};
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};

/// Drops direct messages (messages with no guild_id).
pub static GUILD_ONLY: LazyLock<Arc<dyn MessageFilter>> =
    LazyLock::new(|| Arc::new(GuildOnlyFilter));

/// Drops guild messages, passing only direct messages.
pub static DM_ONLY: LazyLock<Arc<dyn MessageFilter>> = LazyLock::new(|| Arc::new(DmOnlyFilter));

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

#[cfg(test)]
mod tests {
    use super::*;

    fn build_msg_with_opts(timestamp: &str, guild_id: Option<&str>, channel_id: &str) -> Message {
        let mut val = serde_json::json!({
            "id": "1",
            "channel_id": channel_id,
            "author": {
                "id": "1",
                "username": "testuser",
                "bot": false,
                "discriminator": "0",
                "public_flags": 0
            },
            "content": "hello",
            "timestamp": timestamp,
            "edited_timestamp": null,
            "tts": false,
            "mention_everyone": false,
            "mentions": [],
            "mention_roles": [],
            "attachments": [],
            "embeds": [],
            "pinned": false,
            "type": 0
        });
        if let Some(gid) = guild_id {
            val["guild_id"] = serde_json::json!(gid);
        }
        serde_json::from_value(val).expect("test message")
    }

    fn check_filter(filter: &dyn MessageFilter, msg: &Message) -> bool {
        // SAFETY: these filters declare `_ctx` and never dereference ctx.
        // A dangling pointer is used only to satisfy the type signature.
        let ctx_ptr = std::ptr::NonNull::<Context>::dangling();
        filter.check(unsafe { ctx_ptr.as_ref() }, msg)
    }

    #[test]
    fn guild_only_passes_guild_message() {
        let msg = build_msg_with_opts("2024-01-01T12:00:00+00:00", Some("42"), "1");
        assert!(check_filter(&**GUILD_ONLY, &msg));
    }

    #[test]
    fn guild_only_drops_dm() {
        let msg = build_msg_with_opts("2024-01-01T12:00:00+00:00", None, "1");
        assert!(!check_filter(&**GUILD_ONLY, &msg));
    }

    #[test]
    fn dm_only_passes_dm() {
        let msg = build_msg_with_opts("2024-01-01T12:00:00+00:00", None, "1");
        assert!(check_filter(&**DM_ONLY, &msg));
    }

    #[test]
    fn dm_only_drops_guild_message() {
        let msg = build_msg_with_opts("2024-01-01T12:00:00+00:00", Some("42"), "1");
        assert!(!check_filter(&**DM_ONLY, &msg));
    }

    #[test]
    fn in_channel_passes_matching_channel() {
        let msg = build_msg_with_opts("2024-01-01T12:00:00+00:00", None, "99");
        let filter = in_channel(ChannelId::new(99));
        assert!(check_filter(&*filter, &msg));
    }

    #[test]
    fn in_channel_drops_different_channel() {
        let msg = build_msg_with_opts("2024-01-01T12:00:00+00:00", None, "99");
        let filter = in_channel(ChannelId::new(1));
        assert!(!check_filter(&*filter, &msg));
    }

    // 2024-01-01 is Monday, 2024-01-02 Tuesday, 2024-01-05 Friday, 2024-01-07 Sunday.
    #[test]
    fn on_weekdays_passes_matching_day() {
        // Monday
        let msg = build_msg_with_opts("2024-01-01T12:00:00+00:00", None, "1");
        let filter = on_weekdays([Weekday::Mon]);
        assert!(check_filter(&*filter, &msg));
    }

    #[test]
    fn on_weekdays_drops_non_matching_day() {
        // Monday message, asking for Tuesday
        let msg = build_msg_with_opts("2024-01-01T12:00:00+00:00", None, "1");
        let filter = on_weekdays([Weekday::Tue]);
        assert!(!check_filter(&*filter, &msg));
    }

    #[test]
    fn on_weekdays_passes_friday() {
        // 2024-01-05 is Friday
        let msg = build_msg_with_opts("2024-01-05T12:00:00+00:00", None, "1");
        let filter = on_weekdays([Weekday::Fri]);
        assert!(check_filter(&*filter, &msg));
    }

    #[test]
    fn on_weekdays_drops_weekend_when_only_weekdays_allowed() {
        // 2024-01-07 is Sunday
        let msg = build_msg_with_opts("2024-01-07T12:00:00+00:00", None, "1");
        let filter = on_weekdays([
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
        ]);
        assert!(!check_filter(&*filter, &msg));
    }
}
