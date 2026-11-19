//! Postgres-backed [`QuestionRepository`].
//!
//! TDD PR 1: method bodies are stubs. The implementation lands in PR 2.

use super::QuestionRepository;
use crate::question::{NewQuestion, Question, QuestionId, QuestionUpdate};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct PgQuestionRepository {
    #[allow(dead_code)] // used once PR 2 implements the queries
    pool: PgPool,
}

impl PgQuestionRepository {
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl QuestionRepository for PgQuestionRepository {
    async fn list(&self) -> anyhow::Result<Vec<Question>> {
        todo!("ragtime: PgQuestionRepository::list (PR 2)")
    }

    async fn get(&self, _id: QuestionId) -> anyhow::Result<Option<Question>> {
        todo!("ragtime: PgQuestionRepository::get (PR 2)")
    }

    async fn create(&self, _new: NewQuestion) -> anyhow::Result<Question> {
        todo!("ragtime: PgQuestionRepository::create (PR 2)")
    }

    async fn update(
        &self,
        _id: QuestionId,
        _changes: QuestionUpdate,
    ) -> anyhow::Result<Option<Question>> {
        todo!("ragtime: PgQuestionRepository::update (PR 2)")
    }

    async fn delete(&self, _id: QuestionId) -> anyhow::Result<bool> {
        todo!("ragtime: PgQuestionRepository::delete (PR 2)")
    }

    async fn random(&self, _n: usize) -> anyhow::Result<Vec<Question>> {
        todo!("ragtime: PgQuestionRepository::random (PR 2)")
    }
}
