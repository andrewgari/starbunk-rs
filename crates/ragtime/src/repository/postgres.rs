//! Postgres-backed [`QuestionRepository`].

use super::QuestionRepository;
use crate::question::{NewQuestion, Question, QuestionId, QuestionUpdate};
use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Columns selected for every [`Question`] read, in [`QuestionRow`] order.
/// A macro (not a `const`) so it can be `concat!`ed into `&'static` SQL.
macro_rules! columns {
    () => {
        "id, question, display_answer, acceptable_answers, created_at, updated_at"
    };
}

type QuestionRow = (
    i64,
    String,
    String,
    Vec<String>,
    DateTime<Utc>,
    DateTime<Utc>,
);

fn to_question(row: QuestionRow) -> Question {
    let (id, question, display_answer, acceptable_answers, created_at, updated_at) = row;
    Question {
        id,
        question,
        display_answer,
        acceptable_answers,
        created_at,
        updated_at,
    }
}

/// sqlx implementation of [`QuestionRepository`] over `ragtime_questions`.
#[derive(Debug, Clone)]
pub struct PgQuestionRepository {
    pool: PgPool,
}

impl PgQuestionRepository {
    /// Connect to `conn_str` and apply the crate's migrations.
    pub async fn connect(conn_str: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(conn_str)
            .await
            .context("failed to connect to postgres")?;
        let repo = Self::from_pool(pool);
        repo.migrate().await?;
        Ok(repo)
    }

    /// Wrap an existing pool. Does not run migrations.
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Apply `crates/ragtime/migrations/` (idempotent).
    #[tracing::instrument(skip(self))]
    pub async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .context("failed to run ragtime migrations")?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl QuestionRepository for PgQuestionRepository {
    #[tracing::instrument(skip(self))]
    async fn list(&self) -> anyhow::Result<Vec<Question>> {
        let rows = sqlx::query_as::<_, QuestionRow>(concat!(
            "SELECT ",
            columns!(),
            " FROM ragtime_questions ORDER BY id"
        ))
        .fetch_all(&self.pool)
        .await
        .context("failed to list questions")?;
        Ok(rows.into_iter().map(to_question).collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: QuestionId) -> anyhow::Result<Option<Question>> {
        let row = sqlx::query_as::<_, QuestionRow>(concat!(
            "SELECT ",
            columns!(),
            " FROM ragtime_questions WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("failed to get question")?;
        Ok(row.map(to_question))
    }

    #[tracing::instrument(skip(self, new))]
    async fn create(&self, new: NewQuestion) -> anyhow::Result<Question> {
        let row = sqlx::query_as::<_, QuestionRow>(concat!(
            "INSERT INTO ragtime_questions (question, display_answer, acceptable_answers)
             VALUES ($1, $2, $3)
             RETURNING ",
            columns!()
        ))
        .bind(&new.question)
        .bind(&new.display_answer)
        .bind(&new.acceptable_answers)
        .fetch_one(&self.pool)
        .await
        .context("failed to create question")?;
        Ok(to_question(row))
    }

    #[tracing::instrument(skip(self, changes))]
    async fn update(
        &self,
        id: QuestionId,
        changes: QuestionUpdate,
    ) -> anyhow::Result<Option<Question>> {
        // clock_timestamp() rather than now(): now() is fixed per transaction.
        let row = sqlx::query_as::<_, QuestionRow>(concat!(
            "UPDATE ragtime_questions SET
                 question           = COALESCE($2, question),
                 display_answer     = COALESCE($3, display_answer),
                 acceptable_answers = COALESCE($4, acceptable_answers),
                 updated_at         = clock_timestamp()
             WHERE id = $1
             RETURNING ",
            columns!()
        ))
        .bind(id)
        .bind(changes.question)
        .bind(changes.display_answer)
        .bind(changes.acceptable_answers)
        .fetch_optional(&self.pool)
        .await
        .context("failed to update question")?;
        Ok(row.map(to_question))
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: QuestionId) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM ragtime_questions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("failed to delete question")?;
        Ok(result.rows_affected() > 0)
    }

    #[tracing::instrument(skip(self))]
    async fn random(&self, n: usize) -> anyhow::Result<Vec<Question>> {
        if n == 0 {
            return Ok(Vec::new());
        }
        let limit = i64::try_from(n).unwrap_or(i64::MAX);
        let rows = sqlx::query_as::<_, QuestionRow>(concat!(
            "SELECT ",
            columns!(),
            " FROM ragtime_questions ORDER BY random() LIMIT $1"
        ))
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("failed to sample random questions")?;
        Ok(rows.into_iter().map(to_question).collect())
    }
}
