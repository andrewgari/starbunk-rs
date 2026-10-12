//! Startup migration behaviour for `PgQuestionRepository`.
//!
//! `#[ignore]`d in CI (no Postgres). Run locally with:
//!
//! ```bash
//! DATABASE_URL=postgres://... cargo test -p ragtime -- --ignored
//! ```

use ragtime::question::NewQuestion;
use ragtime::repository::{PgQuestionRepository, QuestionRepository};
use sqlx::PgPool;

#[sqlx::test(migrations = false)]
#[ignore = "needs Postgres (DATABASE_URL)"]
async fn migrate_creates_table_and_is_idempotent(pool: PgPool) -> anyhow::Result<()> {
    let repo = PgQuestionRepository::from_pool(pool);

    // Startup runs migrations on every boot; a second run must be a no-op.
    repo.migrate().await?;
    repo.migrate().await?;

    let q = repo
        .create(NewQuestion {
            question: "Q".into(),
            display_answer: "A".into(),
            acceptable_answers: vec!["a".into()],
        })
        .await?;
    assert_eq!(repo.list().await?, vec![q]);
    Ok(())
}
