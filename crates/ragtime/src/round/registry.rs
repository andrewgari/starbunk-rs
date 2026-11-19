//! Per-channel registry of rounds — the public entry point of the state machine.

use super::{ChannelId, Phase, RoundError, RoundEvent, UserId};
use crate::question::Question;
use rand::Rng;
use std::sync::Arc;
use std::time::Instant;

/// Grades a player's reply. Implemented for any
/// `Fn(&str, &str, &[String]) -> bool`, so the Discord layer can pass the
/// crate's `grade()` function directly:
/// `Arc::new(|reply: &str, display: &str, ok: &[String]| grade(reply, display, ok))`.
pub trait Grader: Send + Sync {
    /// `true` if `reply` is a correct answer.
    fn grade(&self, reply: &str, display_answer: &str, acceptable_answers: &[String]) -> bool;
}

impl<F> Grader for F
where
    F: Fn(&str, &str, &[String]) -> bool + Send + Sync,
{
    fn grade(&self, reply: &str, display_answer: &str, acceptable_answers: &[String]) -> bool {
        self(reply, display_answer, acceptable_answers)
    }
}

/// Result of a successful [`RoundRegistry::join`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinOutcome {
    /// The user was added to the round.
    Joined,
    /// The user was already in the round; nothing changed.
    AlreadyJoined,
}

/// All active rounds, keyed by channel. At most one round per channel.
///
/// Time is always passed in explicitly (`now: Instant`) so tests never sleep.
pub struct RoundRegistry {
    // TDD PR 1: fields (per-channel rounds + grader) land with the implementation.
    _private: (),
}

impl std::fmt::Debug for RoundRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoundRegistry").finish_non_exhaustive()
    }
}

impl RoundRegistry {
    /// An empty registry that grades answers with `grader`.
    pub fn new(grader: Arc<dyn Grader>) -> Self {
        let _ = grader;
        todo!("ragtime: RoundRegistry::new (PR 2)")
    }

    /// Open a lobby in `channel` hosted by `host`.
    ///
    /// The host is **not** auto-joined. Errors with
    /// [`RoundError::RoundAlreadyActive`] if the channel already has a round.
    pub fn create(
        &mut self,
        channel: ChannelId,
        host: UserId,
        now: Instant,
    ) -> Result<(), RoundError> {
        let _ = (channel, host, now);
        todo!("ragtime: RoundRegistry::create (PR 2)")
    }

    /// Add `user` to the lobby. Idempotent.
    ///
    /// Late joins are **rejected** with [`RoundError::AlreadyStarted`]: the
    /// early-close rule waits for every joined player, and a player joining
    /// mid-question would also start behind on points with no way to catch up.
    pub fn join(&mut self, channel: ChannelId, user: UserId) -> Result<JoinOutcome, RoundError> {
        let _ = (channel, user);
        todo!("ragtime: RoundRegistry::join (PR 2)")
    }

    /// Check that `user` may start the round in `channel` without changing
    /// anything. Call this before loading questions so a non-host gets
    /// [`RoundError::NotHost`] without a database round-trip.
    pub fn ensure_can_start(&self, channel: ChannelId, user: UserId) -> Result<(), RoundError> {
        let _ = (channel, user);
        todo!("ragtime: RoundRegistry::ensure_can_start (PR 2)")
    }

    /// Start the round: pick up to [`super::QUESTIONS_PER_ROUND`] questions
    /// from `pool` with `rng` (see [`super::select_questions`]) and open the first.
    ///
    /// * Only the host may start ([`RoundError::NotHost`]).
    /// * No players joined → the round is removed and the only event is
    ///   [`RoundEvent::Cleanup`] with [`super::CleanupReason::NoPlayers`]
    ///   (post nothing).
    /// * Empty `pool` → removed with [`super::CleanupReason::NoQuestions`].
    /// * Otherwise returns `[PostQuestion { index: 0, .. }]`.
    pub fn start<R: Rng + ?Sized>(
        &mut self,
        channel: ChannelId,
        user: UserId,
        pool: Vec<Question>,
        rng: &mut R,
        now: Instant,
    ) -> Result<Vec<RoundEvent>, RoundError> {
        let _ = (channel, user, pool, rng, now);
        todo!("ragtime: RoundRegistry::start (PR 2)")
    }

    /// Whether `user` may answer the open question. Returns its 0-based index.
    ///
    /// Use on \[Answer\] button clicks: [`RoundError::NotInRound`] → ephemeral
    /// "not in this round"; [`RoundError::AlreadyAnswered`] → ephemeral notice;
    /// `Ok` → open the modal.
    pub fn can_answer(&self, channel: ChannelId, user: UserId) -> Result<usize, RoundError> {
        let _ = (channel, user);
        todo!("ragtime: RoundRegistry::can_answer (PR 2)")
    }

    /// Record and grade `user`'s `reply` to question `index`.
    ///
    /// One answer per player per question. Errors, checked in this order:
    /// [`RoundError::NoActiveRound`], [`RoundError::NotInProgress`],
    /// [`RoundError::NotInRound`], [`RoundError::StaleQuestion`] (`index` is
    /// not the open question), [`RoundError::AlreadyAnswered`].
    ///
    /// Returns no events while others still have to answer. When this was the
    /// last outstanding answer the question closes early and the events are
    /// `[PostReveal, PostQuestion]`, or `[PostReveal, PostTally, Cleanup]`
    /// after the last question.
    pub fn submit_answer(
        &mut self,
        channel: ChannelId,
        user: UserId,
        index: usize,
        reply: &str,
        now: Instant,
    ) -> Result<Vec<RoundEvent>, RoundError> {
        let _ = (channel, user, index, reply, now);
        todo!("ragtime: RoundRegistry::submit_answer (PR 2)")
    }

    /// Host-only: cancel the round in any phase. Returns
    /// `[Cleanup { reason: Cancelled }]`.
    pub fn cancel(
        &mut self,
        channel: ChannelId,
        user: UserId,
    ) -> Result<Vec<RoundEvent>, RoundError> {
        let _ = (channel, user);
        todo!("ragtime: RoundRegistry::cancel (PR 2)")
    }

    /// Advance time to `now` for every round, in channel-id order:
    ///
    /// * lobby older than [`super::LOBBY_TIMEOUT`] → `Cleanup { LobbyTimeout }`
    /// * in progress with no answers for [`super::HOST_ABANDON_TIMEOUT`] →
    ///   `Cleanup { Abandoned }` (checked before the question timeout)
    /// * open question past [`super::QUESTION_TIMEOUT`] → closes it, same
    ///   events as an early close; the next question's deadline is `now + QUESTION_TIMEOUT`
    ///
    /// Safe to call at any time and as often as you like; a stale timer is a no-op.
    pub fn tick(&mut self, now: Instant) -> Vec<RoundEvent> {
        let _ = now;
        todo!("ragtime: RoundRegistry::tick (PR 2)")
    }

    /// When [`Self::tick`] should next be called for `channel`, or `None` if
    /// there is no round there.
    pub fn next_deadline(&self, channel: ChannelId) -> Option<Instant> {
        let _ = channel;
        todo!("ragtime: RoundRegistry::next_deadline (PR 2)")
    }

    /// Current phase of the round in `channel`, or `None` if there is none.
    pub fn phase(&self, channel: ChannelId) -> Option<Phase> {
        let _ = channel;
        todo!("ragtime: RoundRegistry::phase (PR 2)")
    }

    /// Host of the round in `channel`.
    pub fn host(&self, channel: ChannelId) -> Option<UserId> {
        let _ = channel;
        todo!("ragtime: RoundRegistry::host (PR 2)")
    }

    /// Joined players of the round in `channel`, in join order.
    pub fn players(&self, channel: ChannelId) -> Option<&[UserId]> {
        let _ = channel;
        todo!("ragtime: RoundRegistry::players (PR 2)")
    }
}
