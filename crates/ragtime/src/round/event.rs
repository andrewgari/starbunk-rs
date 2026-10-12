//! Effects the Discord layer must perform after a state transition.

use super::{ChannelId, Standing, UserId};
use crate::question::Question;
use std::time::Instant;

/// Something the Discord layer should do. Events are returned in the order
/// they must be performed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoundEvent {
    /// Post a question embed with an \[Answer\] button.
    PostQuestion {
        channel: ChannelId,
        /// 0-based question index. Encode it in the button id and pass it back
        /// to [`super::RoundRegistry::submit_answer`].
        index: usize,
        /// Number of questions in this round (show as `index + 1` of `total`).
        total: usize,
        question: Question,
        /// When the question times out. Schedule a [`super::RoundRegistry::tick`] for it.
        deadline: Instant,
    },
    /// Post the reveal embed for a question that just closed.
    PostReveal(Reveal),
    /// Post the final leaderboard. Always followed by
    /// [`RoundEvent::Cleanup`] with [`CleanupReason::Finished`].
    PostTally {
        channel: ChannelId,
        standings: Vec<Standing>,
    },
    /// The round was removed from the registry. Drop timers for this channel.
    /// Only [`CleanupReason::LobbyTimeout`], [`CleanupReason::Abandoned`] and
    /// [`CleanupReason::Cancelled`] warrant a (short, optional) notice.
    Cleanup {
        channel: ChannelId,
        reason: CleanupReason,
    },
}

/// Payload of the reveal embed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reveal {
    pub channel: ChannelId,
    /// 0-based index of the question that closed.
    pub index: usize,
    pub question: Question,
    /// The canonical answer to show (`question.display_answer`).
    pub display_answer: String,
    /// Players who answered correctly, in join order. Mention these.
    pub correct: Vec<UserId>,
}

/// Why a round was removed from the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupReason {
    /// All questions were played and the tally was posted.
    Finished,
    /// The host started with nobody joined. The round ends silently.
    NoPlayers,
    /// The host started but the question pool was empty.
    NoQuestions,
    /// The lobby was never started within [`super::LOBBY_TIMEOUT`].
    LobbyTimeout,
    /// No answers for [`super::HOST_ABANDON_TIMEOUT`] while in progress.
    Abandoned,
    /// The host cancelled the round.
    Cancelled,
}
