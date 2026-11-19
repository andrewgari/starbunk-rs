//! Errors returned by [`super::RoundRegistry`].

use std::fmt;

/// Why a round operation was rejected. No state changes when an error is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundError {
    /// [`super::RoundRegistry::create`]: this channel already has an active round.
    RoundAlreadyActive,
    /// There is no active round in this channel.
    NoActiveRound,
    /// Only the host (the user who created the round) may start or cancel it.
    NotHost,
    /// The round has already started: it can't be started again or joined late.
    AlreadyStarted,
    /// The round is still in its lobby, so there is no open question.
    NotInProgress,
    /// The user did not join this round. The Discord layer shows the ephemeral
    /// "not in this round" message.
    NotInRound,
    /// The user already answered the current question.
    AlreadyAnswered,
    /// The answer targets a question that is no longer open (e.g. a modal
    /// submitted after the question closed).
    StaleQuestion,
}

impl fmt::Display for RoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::RoundAlreadyActive => "a round is already running in this channel",
            Self::NoActiveRound => "there is no active round in this channel",
            Self::NotHost => "only the host can do that",
            Self::AlreadyStarted => "the round has already started",
            Self::NotInProgress => "the round hasn't started yet",
            Self::NotInRound => "you're not in this round",
            Self::AlreadyAnswered => "you already answered this question",
            Self::StaleQuestion => "that question is already closed",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for RoundError {}
