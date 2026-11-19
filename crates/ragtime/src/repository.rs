//! Persistence for the trivia question bank.

mod postgres;

pub use postgres::PgQuestionRepository;

use crate::question::{NewQuestion, Question, QuestionId, QuestionUpdate};

/// CRUD + random sampling over `ragtime_questions`.
///
/// Missing rows follow the workspace convention (see `ratbot::store::Store`):
/// lookups return `Ok(None)` rather than an error.
#[async_trait::async_trait]
pub trait QuestionRepository: Send + Sync {
    /// All questions, ordered by `id` ascending.
    async fn list(&self) -> anyhow::Result<Vec<Question>>;

    /// A single question, or `None` if `id` does not exist.
    async fn get(&self, id: QuestionId) -> anyhow::Result<Option<Question>>;

    /// Insert a question and return the stored row (with id and timestamps).
    async fn create(&self, new: NewQuestion) -> anyhow::Result<Question>;

    /// Apply `changes` and bump `updated_at`. Returns `None` if `id` does not exist.
    async fn update(
        &self,
        id: QuestionId,
        changes: QuestionUpdate,
    ) -> anyhow::Result<Option<Question>>;

    /// Delete a question. Returns `true` if a row was removed, `false` if `id` did not exist.
    async fn delete(&self, id: QuestionId) -> anyhow::Result<bool>;

    /// Up to `n` distinct questions in random order. `n == 0` returns an empty vec.
    async fn random(&self, n: usize) -> anyhow::Result<Vec<Question>>;
}
