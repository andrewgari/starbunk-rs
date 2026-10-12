//! Per-channel registry of rounds — the public entry point of the state machine.

use super::game::Game;
use super::{
    select_questions, ChannelId, CleanupReason, Grader, Phase, RoundError, RoundEvent, UserId,
    QUESTIONS_PER_ROUND,
};
use crate::question::Question;
use rand::Rng;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

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
    /// `BTreeMap` so [`Self::tick`] visits channels in a deterministic order.
    rounds: BTreeMap<ChannelId, Game>,
    grader: Arc<dyn Grader>,
}

impl std::fmt::Debug for RoundRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoundRegistry")
            .field("rounds", &self.rounds)
            .finish_non_exhaustive()
    }
}

fn cleanup(channel: ChannelId, reason: CleanupReason) -> RoundEvent {
    RoundEvent::Cleanup { channel, reason }
}

impl RoundRegistry {
    /// An empty registry that grades answers with `grader`.
    pub fn new(grader: Arc<dyn Grader>) -> Self {
        Self {
            rounds: BTreeMap::new(),
            grader,
        }
    }

    fn game(&self, channel: ChannelId) -> Result<&Game, RoundError> {
        self.rounds.get(&channel).ok_or(RoundError::NoActiveRound)
    }

    fn game_mut(&mut self, channel: ChannelId) -> Result<&mut Game, RoundError> {
        self.rounds
            .get_mut(&channel)
            .ok_or(RoundError::NoActiveRound)
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
        if self.rounds.contains_key(&channel) {
            return Err(RoundError::RoundAlreadyActive);
        }
        self.rounds.insert(channel, Game::new(host, now));
        Ok(())
    }

    /// Add `user` to the lobby. Idempotent.
    ///
    /// Late joins are **rejected** with [`RoundError::AlreadyStarted`]: the
    /// early-close rule waits for every joined player, and a player joining
    /// mid-question would also start behind on points with no way to catch up.
    pub fn join(&mut self, channel: ChannelId, user: UserId) -> Result<JoinOutcome, RoundError> {
        let game = self.game_mut(channel)?;
        if game.phase != Phase::Lobby {
            return Err(RoundError::AlreadyStarted);
        }
        if game.is_player(user) {
            return Ok(JoinOutcome::AlreadyJoined);
        }
        game.players.push(user);
        Ok(JoinOutcome::Joined)
    }

    /// Check that `user` may start the round in `channel` without changing
    /// anything. Call this before loading questions so a non-host gets
    /// [`RoundError::NotHost`] without a database round-trip.
    pub fn ensure_can_start(&self, channel: ChannelId, user: UserId) -> Result<(), RoundError> {
        let game = self.game(channel)?;
        if game.phase != Phase::Lobby {
            return Err(RoundError::AlreadyStarted);
        }
        if game.host != user {
            return Err(RoundError::NotHost);
        }
        Ok(())
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
        self.ensure_can_start(channel, user)?;
        let game = self.game_mut(channel)?;

        if game.players.is_empty() {
            self.rounds.remove(&channel);
            return Ok(vec![cleanup(channel, CleanupReason::NoPlayers)]);
        }
        let questions = select_questions(pool, QUESTIONS_PER_ROUND, rng);
        let Some(first) = questions.first().cloned() else {
            self.rounds.remove(&channel);
            return Ok(vec![cleanup(channel, CleanupReason::NoQuestions)]);
        };

        let total = questions.len();
        game.questions = questions;
        game.last_activity = now;
        let deadline = game.open_question(0, now);
        Ok(vec![RoundEvent::PostQuestion {
            channel,
            index: 0,
            total,
            question: first,
            deadline,
        }])
    }

    /// Whether `user` may answer the open question. Returns its 0-based index.
    ///
    /// Use on \[Answer\] button clicks: [`RoundError::NotInRound`] → ephemeral
    /// "not in this round"; [`RoundError::AlreadyAnswered`] → ephemeral notice;
    /// `Ok` → open the modal.
    pub fn can_answer(&self, channel: ChannelId, user: UserId) -> Result<usize, RoundError> {
        let game = self.game(channel)?;
        let index = game.open_index().ok_or(RoundError::NotInProgress)?;
        if !game.is_player(user) {
            return Err(RoundError::NotInRound);
        }
        if game.answers.contains_key(&user) {
            return Err(RoundError::AlreadyAnswered);
        }
        Ok(index)
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
        let grader = Arc::clone(&self.grader);
        let game = self.game_mut(channel)?;
        let open = game.open_index().ok_or(RoundError::NotInProgress)?;
        if !game.is_player(user) {
            return Err(RoundError::NotInRound);
        }
        if index != open {
            return Err(RoundError::StaleQuestion);
        }
        if game.answers.contains_key(&user) {
            return Err(RoundError::AlreadyAnswered);
        }

        let question = &game.questions[open];
        let correct = grader.grade(
            reply,
            &question.display_answer,
            &question.acceptable_answers,
        );
        game.answers.insert(user, correct);
        if correct {
            *game.scores.entry(user).or_insert(0) += 1;
        }
        game.last_activity = now;

        if !game.everyone_answered() {
            return Ok(Vec::new());
        }
        let closed = game.close_question(channel, now);
        if closed.finished {
            self.rounds.remove(&channel);
        }
        Ok(closed.events)
    }

    /// Host-only: cancel the round in any phase. Returns
    /// `[Cleanup { reason: Cancelled }]`.
    pub fn cancel(
        &mut self,
        channel: ChannelId,
        user: UserId,
    ) -> Result<Vec<RoundEvent>, RoundError> {
        if self.game(channel)?.host != user {
            return Err(RoundError::NotHost);
        }
        self.rounds.remove(&channel);
        Ok(vec![cleanup(channel, CleanupReason::Cancelled)])
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
        let mut events = Vec::new();
        let mut removed = Vec::new();
        for (&channel, game) in &mut self.rounds {
            if game.lobby_expired(now) {
                game.phase = Phase::Abandoned;
                events.push(cleanup(channel, CleanupReason::LobbyTimeout));
                removed.push(channel);
            } else if game.abandoned(now) {
                game.phase = Phase::Abandoned;
                events.push(cleanup(channel, CleanupReason::Abandoned));
                removed.push(channel);
            } else if game.question_expired(now) {
                let closed = game.close_question(channel, now);
                events.extend(closed.events);
                if closed.finished {
                    removed.push(channel);
                }
            }
        }
        for channel in removed {
            self.rounds.remove(&channel);
        }
        events
    }

    /// When [`Self::tick`] should next be called for `channel`, or `None` if
    /// there is no round there.
    pub fn next_deadline(&self, channel: ChannelId) -> Option<Instant> {
        self.rounds.get(&channel).and_then(Game::next_deadline)
    }

    /// Current phase of the round in `channel`, or `None` if there is none.
    pub fn phase(&self, channel: ChannelId) -> Option<Phase> {
        self.rounds.get(&channel).map(|g| g.phase)
    }

    /// Host of the round in `channel`.
    pub fn host(&self, channel: ChannelId) -> Option<UserId> {
        self.rounds.get(&channel).map(|g| g.host)
    }

    /// Joined players of the round in `channel`, in join order.
    pub fn players(&self, channel: ChannelId) -> Option<&[UserId]> {
        self.rounds.get(&channel).map(|g| g.players.as_slice())
    }
}
