//! Postgres-backed [`QuestionRepository`].

use super::QuestionRepository;
use crate::question::{NewQuestion, Question, QuestionId, QuestionUpdate};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct PgQuestionRepository {
    pool: PgPool,
}

impl PgQuestionRepository {
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    question: String,
    display_answer: String,
    acceptable_answers: Vec<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<Row> for Question {
    fn from(r: Row) -> Self {
        Question {
            id: r.id,
            question: r.question,
            display_answer: r.display_answer,
            acceptable_answers: r.acceptable_answers,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

/// Column list, as a macro so queries stay `&'static str` literals.
macro_rules! cols {
    () => {
        "id, question, display_answer, acceptable_answers, created_at, updated_at"
    };
}

#[async_trait::async_trait]
impl QuestionRepository for PgQuestionRepository {
    async fn list(&self) -> anyhow::Result<Vec<Question>> {
        let rows: Vec<Row> = sqlx::query_as(concat!(
            "SELECT ",
            cols!(),
            " FROM ragtime_questions ORDER BY id"
        ))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get(&self, id: QuestionId) -> anyhow::Result<Option<Question>> {
        let row: Option<Row> = sqlx::query_as(concat!(
            "SELECT ",
            cols!(),
            " FROM ragtime_questions WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn create(&self, new: NewQuestion) -> anyhow::Result<Question> {
        let row: Row = sqlx::query_as(concat!(
            "INSERT INTO ragtime_questions (question, display_answer, acceptable_answers) \
             VALUES ($1, $2, $3) RETURNING ",
            cols!(),
            ""
        ))
        .bind(new.question)
        .bind(new.display_answer)
        .bind(new.acceptable_answers)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn update(
        &self,
        id: QuestionId,
        changes: QuestionUpdate,
    ) -> anyhow::Result<Option<Question>> {
        // clock_timestamp(), not now(): now() is fixed per transaction.
        let row: Option<Row> = sqlx::query_as(concat!(
            "UPDATE ragtime_questions SET \
                question           = COALESCE($2, question), \
                display_answer     = COALESCE($3, display_answer), \
                acceptable_answers = COALESCE($4, acceptable_answers), \
                updated_at         = clock_timestamp() \
             WHERE id = $1 RETURNING ",
            cols!(),
            ""
        ))
        .bind(id)
        .bind(changes.question)
        .bind(changes.display_answer)
        .bind(changes.acceptable_answers)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn delete(&self, id: QuestionId) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM ragtime_questions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn random(&self, n: usize) -> anyhow::Result<Vec<Question>> {
        if n == 0 {
            return Ok(Vec::new());
        }
        let limit = i64::try_from(n).unwrap_or(i64::MAX);
        let rows: Vec<Row> = sqlx::query_as(concat!(
            "SELECT ",
            cols!(),
            " FROM ragtime_questions ORDER BY random() LIMIT $1"
        ))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}
