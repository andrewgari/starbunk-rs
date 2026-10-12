//! Internal per-round state (not part of the public API).

use super::{Phase, UserId, HOST_ABANDON_TIMEOUT, LOBBY_TIMEOUT, QUESTION_TIMEOUT};
use crate::question::Question;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// State of a single round in one channel.
#[derive(Debug)]
pub(super) struct Game {
    pub host: UserId,
    pub phase: Phase,
    /// Joined players in join order (no duplicates).
    pub players: Vec<UserId>,
    pub created_at: Instant,
    /// Questions in play order, fixed at start.
    pub questions: Vec<Question>,
    /// Players who answered the open question → whether they were correct.
    pub answers: HashMap<UserId, bool>,
    /// Correct-answer count per player.
    pub scores: HashMap<UserId, u32>,
    /// When the open question times out.
    pub question_deadline: Option<Instant>,
    /// Start time or last accepted answer, for [`HOST_ABANDON_TIMEOUT`].
    pub last_activity: Instant,
}

impl Game {
    pub fn new(host: UserId, now: Instant) -> Self {
        Self {
            host,
            phase: Phase::Lobby,
            players: Vec::new(),
            created_at: now,
            questions: Vec::new(),
            answers: HashMap::new(),
            scores: HashMap::new(),
            question_deadline: None,
            last_activity: now,
        }
    }

    pub fn is_player(&self, user: UserId) -> bool {
        self.players.contains(&user)
    }

    /// Index of the open question, if the round is in progress.
    pub fn open_index(&self) -> Option<usize> {
        match self.phase {
            Phase::InProgress { index } => Some(index),
            _ => None,
        }
    }

    /// Open question `index` at `now`; returns its deadline.
    pub fn open_question(&mut self, index: usize, now: Instant) -> Instant {
        self.phase = Phase::InProgress { index };
        self.answers.clear();
        let deadline = now + QUESTION_TIMEOUT;
        self.question_deadline = Some(deadline);
        deadline
    }

    pub fn everyone_answered(&self) -> bool {
        self.players.iter().all(|p| self.answers.contains_key(p))
    }

    /// Players who answered the open question correctly, in join order.
    pub fn correct_players(&self) -> Vec<UserId> {
        let correct: HashSet<UserId> = self
            .answers
            .iter()
            .filter_map(|(user, ok)| ok.then_some(*user))
            .collect();
        self.players
            .iter()
            .copied()
            .filter(|p| correct.contains(p))
            .collect()
    }

    /// `(user, score)` for every player, in join order.
    pub fn scores_in_join_order(&self) -> Vec<(UserId, u32)> {
        self.players
            .iter()
            .map(|p| (*p, self.scores.get(p).copied().unwrap_or(0)))
            .collect()
    }

    pub fn lobby_expired(&self, now: Instant) -> bool {
        self.phase == Phase::Lobby && now >= self.created_at + LOBBY_TIMEOUT
    }

    pub fn abandoned(&self, now: Instant) -> bool {
        self.open_index().is_some() && now >= self.last_activity + HOST_ABANDON_TIMEOUT
    }

    pub fn question_expired(&self, now: Instant) -> bool {
        self.open_index().is_some() && self.question_deadline.is_some_and(|d| now >= d)
    }

    /// The earliest moment a `tick` can change this round.
    pub fn next_deadline(&self) -> Option<Instant> {
        match self.phase {
            Phase::Lobby => Some(self.created_at + LOBBY_TIMEOUT),
            Phase::InProgress { .. } => {
                let abandon = self.last_activity + HOST_ABANDON_TIMEOUT;
                Some(self.question_deadline.map_or(abandon, |q| q.min(abandon)))
            }
            Phase::Finished | Phase::Abandoned | Phase::Cancelled => None,
        }
    }
}
