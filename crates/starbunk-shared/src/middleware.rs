//! Composable gate primitives for evaluating Discord messages.
//!
//! Every gate implements [`MessageFilter`], so any combination can be
//! composed freely using [`all_of`], [`any_of`], and [`not`]:
//!
//! ```no_run
//! use starbunk_shared::middleware::{all_of, not, NOT_BOT, NOT_SELF, GUILD_ONLY, HAS_CONTENT};
//! let filter = all_of(vec![NOT_SELF.clone(), NOT_BOT.clone(), GUILD_ONLY.clone(), HAS_CONTENT.clone()]);
//! ```

pub mod author;
pub mod content;
pub mod context;
pub mod random;

use serenity::all::{Context, Message};
use std::sync::Arc;

/// Gate for an incoming Discord message. Returns `true` to allow processing,
/// `false` to drop silently.
pub trait MessageFilter: Send + Sync {
    fn check(&self, ctx: &Context, msg: &Message) -> bool;
}

/// Passes only when every child passes. Short-circuits on first failure.
/// Passes vacuously when given no children.
pub fn all_of(filters: Vec<Arc<dyn MessageFilter>>) -> Arc<dyn MessageFilter> {
    Arc::new(AllOf(filters))
}

/// Passes when at least one child passes. Short-circuits on first success.
/// Fails vacuously when given no children.
pub fn any_of(filters: Vec<Arc<dyn MessageFilter>>) -> Arc<dyn MessageFilter> {
    Arc::new(AnyOf(filters))
}

/// Inverts a filter.
pub fn not(f: Arc<dyn MessageFilter>) -> Arc<dyn MessageFilter> {
    Arc::new(Not(f))
}

// Re-export the named filter constants for ergonomic use.
pub use author::{
    author_has_role, author_id, author_named, not_author_id, not_self_with_bot_id, IS_BOT, NOT_BOT,
    NOT_SELF,
};
pub use content::{content_contains, content_matches, HAS_ATTACHMENT, HAS_CONTENT};
pub use context::{in_channel, on_weekdays, DM_ONLY, GUILD_ONLY};
pub use random::chance;

struct AllOf(Vec<Arc<dyn MessageFilter>>);
struct AnyOf(Vec<Arc<dyn MessageFilter>>);
struct Not(Arc<dyn MessageFilter>);

impl MessageFilter for AllOf {
    fn check(&self, ctx: &Context, msg: &Message) -> bool {
        self.0.iter().all(|f| f.check(ctx, msg))
    }
}

impl MessageFilter for AnyOf {
    fn check(&self, ctx: &Context, msg: &Message) -> bool {
        self.0.iter().any(|f| f.check(ctx, msg))
    }
}

impl MessageFilter for Not {
    fn check(&self, ctx: &Context, msg: &Message) -> bool {
        !self.0.check(ctx, msg)
    }
}

/// Helper: wrap a plain filter function as a [`MessageFilter`].
pub fn filter_fn(
    f: impl Fn(&Context, &Message) -> bool + Send + Sync + 'static,
) -> Arc<dyn MessageFilter> {
    Arc::new(FnFilter(Box::new(f)))
}

type FilterFn = Box<dyn Fn(&Context, &Message) -> bool + Send + Sync>;

struct FnFilter(FilterFn);

impl MessageFilter for FnFilter {
    fn check(&self, ctx: &Context, msg: &Message) -> bool {
        (self.0)(ctx, msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_msg(content: &str, bot: bool) -> Message {
        serde_json::from_value(serde_json::json!({
            "id": "1",
            "channel_id": "1",
            "author": {
                "id": "1",
                "username": "testuser",
                "bot": bot,
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

    fn always_pass() -> Arc<dyn MessageFilter> {
        filter_fn(|_, _| true)
    }

    fn always_fail() -> Arc<dyn MessageFilter> {
        filter_fn(|_, _| false)
    }

    // --- all_of ---

    #[test]
    fn all_of_passes_when_all_pass() {
        let msg = build_msg("hi", false);
        let f = all_of(vec![always_pass(), always_pass()]);
        assert!(check_filter(&*f, &msg));
    }

    #[test]
    fn all_of_fails_when_any_fails() {
        let msg = build_msg("hi", false);
        let f = all_of(vec![always_pass(), always_fail()]);
        assert!(!check_filter(&*f, &msg));
    }

    #[test]
    fn all_of_passes_vacuously_with_no_children() {
        let msg = build_msg("hi", false);
        let f = all_of(vec![]);
        assert!(check_filter(&*f, &msg));
    }

    // --- any_of ---

    #[test]
    fn any_of_passes_when_at_least_one_passes() {
        let msg = build_msg("hi", false);
        let f = any_of(vec![always_fail(), always_pass()]);
        assert!(check_filter(&*f, &msg));
    }

    #[test]
    fn any_of_fails_when_all_fail() {
        let msg = build_msg("hi", false);
        let f = any_of(vec![always_fail(), always_fail()]);
        assert!(!check_filter(&*f, &msg));
    }

    #[test]
    fn any_of_fails_vacuously_with_no_children() {
        let msg = build_msg("hi", false);
        let f = any_of(vec![]);
        assert!(!check_filter(&*f, &msg));
    }

    // --- not ---

    #[test]
    fn not_inverts_passing_filter() {
        let msg = build_msg("hi", false);
        let f = not(always_pass());
        assert!(!check_filter(&*f, &msg));
    }

    #[test]
    fn not_inverts_failing_filter() {
        let msg = build_msg("hi", false);
        let f = not(always_fail());
        assert!(check_filter(&*f, &msg));
    }

    // --- filter_fn ---

    #[test]
    fn filter_fn_delegates_to_closure() {
        let msg = build_msg("magic", false);
        let f = filter_fn(|_, m| m.content == "magic");
        assert!(check_filter(&*f, &msg));
    }

    #[test]
    fn filter_fn_fails_when_closure_returns_false() {
        let msg = build_msg("other", false);
        let f = filter_fn(|_, m| m.content == "magic");
        assert!(!check_filter(&*f, &msg));
    }

    // --- complex compositions ---

    fn guild_msg(content: &str) -> Message {
        let mut val = serde_json::json!({
            "id": "1", "channel_id": "1",
            "author": { "id": "1", "username": "human", "bot": false, "discriminator": "0", "public_flags": 0 },
            "content": content,
            "timestamp": "2024-01-01T12:00:00+00:00",
            "edited_timestamp": null, "tts": false, "mention_everyone": false,
            "mentions": [], "mention_roles": [], "attachments": [], "embeds": [],
            "pinned": false, "type": 0
        });
        val["guild_id"] = serde_json::json!("42");
        serde_json::from_value(val).expect("guild msg")
    }

    fn bot_msg(content: &str) -> Message {
        serde_json::from_value(serde_json::json!({
            "id": "2", "channel_id": "1",
            "author": { "id": "2", "username": "otherbot", "bot": true, "discriminator": "0", "public_flags": 0 },
            "content": content,
            "timestamp": "2024-01-01T12:00:00+00:00",
            "edited_timestamp": null, "tts": false, "mention_everyone": false,
            "mentions": [], "mention_roles": [], "attachments": [], "embeds": [],
            "pinned": false, "type": 0
        }))
        .expect("bot msg")
    }

    fn author_id_msg(author_id: &str, content: &str, is_bot: bool) -> Message {
        serde_json::from_value(serde_json::json!({
            "id": "3", "channel_id": "1",
            "author": { "id": author_id, "username": "user", "bot": is_bot, "discriminator": "0", "public_flags": 0 },
            "content": content,
            "timestamp": "2024-01-01T12:00:00+00:00",
            "edited_timestamp": null, "tts": false, "mention_everyone": false,
            "mentions": [], "mention_roles": [], "attachments": [], "embeds": [],
            "pinned": false, "type": 0
        }))
        .expect("author_id msg")
    }

    #[test]
    fn bluebot_policy_passes_human_guild_messages_with_content() {
        // AllOf(NOT_BOT, GUILD_ONLY, HAS_CONTENT) — mirrors BlueBot's default auditor.
        let policy = all_of(vec![
            NOT_BOT.clone(),
            GUILD_ONLY.clone(),
            HAS_CONTENT.clone(),
        ]);
        assert!(check_filter(&*policy, &guild_msg("hello")));
        assert!(!check_filter(&*policy, &build_msg("hello", false))); // DM, no guild_id
        assert!(!check_filter(&*policy, &guild_msg(""))); // empty
        assert!(!check_filter(&*policy, &bot_msg("hello"))); // bot author
    }

    #[test]
    fn scenario_bots_always_fail_except_via_content_bingo() {
        // AllOf(NotBot, AnyOf(Not(AuthorID("111111")), ContentContains("bingo")))
        let policy = all_of(vec![
            NOT_BOT.clone(),
            any_of(vec![
                not(content::content_contains("111111")), // stand-in for not_author_id using content
                content::content_contains("bingo"),
            ]),
        ]);

        // Plain human, no special user — passes
        let human = author_id_msg("999", "anything", false);
        assert!(check_filter(&*policy, &human));

        // Any bot — fails immediately at NOT_BOT
        let bot = author_id_msg("999", "bingo", true);
        assert!(!check_filter(&*policy, &bot));
    }

    #[test]
    fn chance_one_always_passes_in_composition() {
        // Verify chance(1.0) integrates correctly inside a larger filter.
        let policy = all_of(vec![NOT_BOT.clone(), random::chance(1.0)]);
        let human = build_msg("hi", false);
        assert!(check_filter(&*policy, &human));
    }

    #[test]
    fn chance_zero_fails_in_composition() {
        let policy = all_of(vec![NOT_BOT.clone(), random::chance(0.0)]);
        let human = build_msg("hi", false);
        assert!(!check_filter(&*policy, &human));
    }

    #[test]
    fn bunkbot_policy_passes_non_self_and_has_content_including_bots() {
        // BunkBot: AllOf(NOT_SELF_via_bot_id, HAS_CONTENT) — does not filter out other bots
        use author::not_self_with_bot_id;
        use serenity::all::UserId;

        let bot_id = UserId::new(99); // "our" bot id
        let policy = all_of(vec![not_self_with_bot_id(bot_id), HAS_CONTENT.clone()]);

        // Other bot with content → passes (bunkbot responds to bots)
        let other_bot = author_id_msg("2", "hello", true);
        assert!(check_filter(&*policy, &other_bot));

        // Self message → fails
        let self_msg = author_id_msg("99", "hello", false);
        assert!(!check_filter(&*policy, &self_msg));

        // Human with empty content → fails
        let empty = author_id_msg("2", "", false);
        assert!(!check_filter(&*policy, &empty));
    }

    #[test]
    fn drops_bot_named_jeff_on_friday_or_tuesday() {
        use chrono::Weekday;
        use context::on_weekdays;

        // Not(AllOf(IS_BOT, AuthorNamed("Jeff"), OnWeekdays(Friday, Tuesday)))
        let reject_jeff = not(all_of(vec![
            IS_BOT.clone(),
            author::author_named("Jeff"),
            on_weekdays([Weekday::Fri, Weekday::Tue]),
        ]));

        // 2024-01-02 is a Tuesday, 2024-01-05 is Friday
        fn msg_with_opts(
            author_id: &str,
            username: &str,
            is_bot: bool,
            timestamp: &str,
            guild_id: Option<&str>,
        ) -> Message {
            let mut val = serde_json::json!({
                "id": "1",
                "channel_id": "1",
                "author": {
                    "id": author_id,
                    "username": username,
                    "bot": is_bot,
                    "discriminator": "0",
                    "public_flags": 0
                },
                "content": "hi",
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
            serde_json::from_value(val).expect("msg")
        }

        let jeff_bot_on_friday =
            msg_with_opts("1", "Jeff", true, "2024-01-05T12:00:00+00:00", None);
        let jeff_bot_on_tuesday =
            msg_with_opts("1", "Jeff", true, "2024-01-02T12:00:00+00:00", None);
        let jeff_bot_on_monday =
            msg_with_opts("1", "Jeff", true, "2024-01-01T12:00:00+00:00", None);
        let human_jeff_on_friday =
            msg_with_opts("2", "Jeff", false, "2024-01-05T12:00:00+00:00", None);

        assert!(!check_filter(&*reject_jeff, &jeff_bot_on_friday)); // blocked
        assert!(!check_filter(&*reject_jeff, &jeff_bot_on_tuesday)); // blocked
        assert!(check_filter(&*reject_jeff, &jeff_bot_on_monday)); // Jeff bot on safe day
        assert!(check_filter(&*reject_jeff, &human_jeff_on_friday)); // human Jeff passes
    }

    #[test]
    fn scenario_2_human_passes_bingo_always_passes_bot_22222_on_lucky_roll() {
        // AnyOf(NOT_BOT, AllOf(author_id("22222"), chance), ContentContains("bingo"))
        let winning_roll = random::chance(1.0);
        let losing_roll = random::chance(0.0);

        let make_policy = |roll: Arc<dyn MessageFilter>| {
            any_of(vec![
                NOT_BOT.clone(),
                all_of(vec![author::author_id("22222"), roll]),
                content::content_contains("bingo"),
            ])
        };

        // Human passes regardless
        let human = author_id_msg("100", "hi", false);
        assert!(check_filter(&*make_policy(winning_roll.clone()), &human));

        // Bot with "bingo" — passes via content branch
        let bot_bingo = author_id_msg("999", "bingo", true);
        assert!(check_filter(&*make_policy(losing_roll.clone()), &bot_bingo));

        // Bot 22222 on winning roll
        let bot_22222 = author_id_msg("22222", "hi", true);
        assert!(check_filter(
            &*make_policy(winning_roll.clone()),
            &bot_22222
        ));

        // Bot 22222 on losing roll without bingo — blocked
        assert!(!check_filter(
            &*make_policy(losing_roll.clone()),
            &bot_22222
        ));

        // Bot 22222 on losing roll with bingo — passes via content
        let bot_22222_bingo = author_id_msg("22222", "bingo", true);
        assert!(check_filter(
            &*make_policy(losing_roll.clone()),
            &bot_22222_bingo
        ));

        // Unrelated bot, losing roll — blocked
        let other_bot = author_id_msg("999", "hi", true);
        assert!(!check_filter(
            &*make_policy(losing_roll.clone()),
            &other_bot
        ));
    }
}
