//! Trivia question model and its create/update input types.

use chrono::{DateTime, Utc};

/// Primary key of a row in `ragtime_questions`.
pub type QuestionId = i64;

/// A stored trivia question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub id: QuestionId,
    /// The prompt shown to players.
    pub question: String,
    /// The canonical answer revealed after the round.
    pub display_answer: String,
    /// Every answer string that should be graded as correct.
    pub acceptable_answers: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for [`crate::repository::QuestionRepository::create`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewQuestion {
    pub question: String,
    pub display_answer: String,
    pub acceptable_answers: Vec<String>,
}

/// Partial update for [`crate::repository::QuestionRepository::update`].
///
/// `None` leaves the field unchanged. Every successful update bumps `updated_at`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QuestionUpdate {
    pub question: Option<String>,
    pub display_answer: Option<String>,
    pub acceptable_answers: Option<Vec<String>>,
}
