//! Discord-free round state machine for the Ragtime contest.
//!
//! This module owns the whole game loop — lobby, joining, question rotation,
//! grading, early/timeout closes, tally and abandonment — without touching any
//! Discord type. The Discord layer feeds it inputs and turns the returned
//! [`RoundEvent`]s into messages.
//!
//! # Wiring guide (for the Discord layer)
//!
//! Hold one [`RoundRegistry`] behind an async-safe lock (e.g.
//! `Arc<tokio::sync::Mutex<RoundRegistry>>`). Every method is synchronous, so
//! **never hold the lock across an `.await`**: lock, call, collect the events,
//! drop the lock, then send messages.
//!
//! | Discord input | Call | Notes |
//! |---|---|---|
//! | bot @mentioned | [`RoundRegistry::create`] | mentioner becomes host; [`RoundError::RoundAlreadyActive`] → "a round is already running" |
//! | join reaction added | [`RoundRegistry::join`] | idempotent; rejected after start ([`RoundError::AlreadyStarted`]) |
//! | host start command | [`RoundRegistry::ensure_can_start`], then load questions **with the lock released**, then [`RoundRegistry::start`] | non-host → [`RoundError::NotHost`] |
//! | \[Answer\] button | [`RoundRegistry::can_answer`] | `NotInRound` → ephemeral "not in this round"; `AlreadyAnswered` → ephemeral notice; `Ok` → open modal |
//! | modal submit | [`RoundRegistry::submit_answer`] | pass the question index encoded in the button id; acknowledge ephemerally |
//! | host cancel (optional) | [`RoundRegistry::cancel`] | |
//! | timer fired / periodic sweep | [`RoundRegistry::tick`] | use [`RoundRegistry::next_deadline`] to schedule; stale timers are harmless |
//!
//! Every call that can change the round returns `Vec<RoundEvent>`; post them in
//! order. A [`RoundEvent::Cleanup`] always means the round has been removed
//! from the registry and any timers for that channel can be dropped.
//!
//! # Rules and timeouts
//!
//! * One active round per channel.
//! * The host (creator) is **not** auto-joined; they join like everyone else.
//! * Up to [`QUESTIONS_PER_ROUND`] questions are drawn at random from the pool
//!   passed to [`RoundRegistry::start`]; a smaller pool is used in full.
//! * A question closes when every joined player has answered, or after
//!   [`QUESTION_TIMEOUT`], whichever comes first.
//! * 1 point per correct answer; ties share a rank (see [`rank`]).
//! * [`LOBBY_TIMEOUT`] and [`HOST_ABANDON_TIMEOUT`] clean up dead rounds.

mod advance;
mod error;
mod event;
mod game;
mod grader;
mod registry;
mod select;
mod state;
mod tally;

pub use error::RoundError;
pub use event::{CleanupReason, Reveal, RoundEvent};
pub use grader::Grader;
pub use registry::{JoinOutcome, RoundRegistry};
pub use select::select_questions;
pub use state::Phase;
pub use tally::{rank, Standing};

use std::time::Duration;

/// Discord user id (snowflake), kept as a plain integer so this module stays Discord-free.
pub type UserId = u64;

/// Discord channel id (snowflake), kept as a plain integer.
pub type ChannelId = u64;

/// Maximum number of questions in one round.
pub const QUESTIONS_PER_ROUND: usize = 20;

/// How long a question stays open before it closes on its own.
pub const QUESTION_TIMEOUT: Duration = Duration::from_secs(2 * 60);

/// A lobby that the host never starts is removed this long after creation.
pub const LOBBY_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// An in-progress round with no answer from any player for this long is
/// abandoned and removed.
///
/// Once a round starts the host has no further role, so "the host abandoned
/// the round" in practice means everyone walked away. 6 minutes is three full
/// [`QUESTION_TIMEOUT`] windows with zero answers: long enough that a quiet
/// question or two (hard questions, players typing slowly) never kills a live
/// round, but short enough that a dead round stops after 3 questions instead of
/// grinding through up to 40 minutes of unanswered questions in the channel.
/// The clock starts at [`RoundRegistry::start`] and resets on every accepted answer.
pub const HOST_ABANDON_TIMEOUT: Duration = Duration::from_secs(6 * 60);
