//! Discord wiring for the Ragtime contest.
//!
//! Two layers:
//!
//! * [`RagtimeEngine`] — Discord-free orchestration. It owns the
//!   [`crate::round::RoundRegistry`] behind an async lock, loads questions from
//!   the [`crate::repository::QuestionRepository`], schedules tokio timers and
//!   turns every [`crate::round::RoundEvent`] into an [`Outbox`] call. All ids
//!   are plain `u64`s, so the whole game loop is exercised in tests with a fake
//!   outbox and paused tokio time (`crates/ragtime/tests/discord_wiring.rs`).
//! * [`RagtimeHandler`] — the serenity `EventHandler`. It only translates
//!   gateway events into engine calls and engine outcomes into Discord
//!   responses ([`DiscordOutbox`] does the channel posts).
//!
//! # Commands and inputs
//!
//! | Input | Effect |
//! |---|---|
//! | @mention the bot | open a lobby in the channel with the mentioner as host (or "already running") |
//! | react with [`crate::presentation::JOIN_EMOJI`] on the lobby message | join the round |
//! | `/ragtime start` (host only) | load up to [`crate::round::QUESTIONS_PER_ROUND`] questions and post question 1 |
//! | `/ragtime cancel` (host only) | end the round in this channel |
//! | \[Answer\] button | open the answer modal (ephemeral notice if not joined / already answered) |
//! | answer modal submit | grade and acknowledge ephemerally, never revealing correctness |
//!
//! # Locking
//!
//! Two locks, always taken in this order: `seq` (serialises state changes and
//! the posts they cause, so a channel's messages appear in event order), then
//! `registry`. The registry lock is **never held across an `.await`**: it is
//! taken, the synchronous registry call runs, and it is dropped before any
//! Discord I/O. The \[Answer\] button path only reads the registry and never
//! takes `seq`, so modal opening stays within Discord's 3-second window.

mod clock;
mod commands;
mod discord_outbox;
mod dispatch;
mod engine;
mod grading;
mod handler;
mod interactions;
mod outbox;
pub mod replies;
mod timers;

pub use commands::{register_commands, RagtimeCommand, COMMAND_NAME};
pub use discord_outbox::DiscordOutbox;
pub use engine::{ButtonOutcome, MentionOutcome, RagtimeEngine};
pub use grading::interim_grader;
pub use handler::RagtimeHandler;
pub use outbox::Outbox;
