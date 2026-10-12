//! Contract tests for `PgQuestionRepository`.
//!
//! These run against a real Postgres via `#[sqlx::test]`, which creates a fresh
//! database per test and applies `crates/ragtime/migrations/` automatically.
//!
//! They are `#[ignore]`d because CI has no Postgres service.
//! Run them locally with:
//!
//! ```bash
//! DATABASE_URL=postgres://starbunk:starbunk@localhost:5432/starbunk_memory \
//!     cargo test -p ragtime -- --ignored
//! ```

use ragtime::question::{NewQuestion, Question, QuestionUpdate};
use ragtime::repository::{PgQuestionRepository, QuestionRepository};
use sqlx::PgPool;
use std::collections::HashSet;

fn new_q(n: usize) -> NewQuestion {
    NewQuestion {
        question: format!("Question {n}?"),
        display_answer: format!("Answer {n}"),
        acceptable_answers: vec![format!("answer {n}"), format!("ans{n}")],
    }
}

async fn seed(repo: &PgQuestionRepository, count: usize) -> anyhow::Result<Vec<Question>> {
    let mut out = Vec::with_capacity(count);
    for n in 0..count {
        out.push(repo.create(new_q(n)).await?);
    }
    Ok(out)
}

fn ids(qs: &[Question]) -> HashSet<i64> {
    qs.iter().map(|q| q.id).collect()
}

#[sqlx::test]
#[ignore = "needs Postgres (DATABASE_URL)"]
async fn create_then_get_round_trips_every_field(pool: PgPool) -> anyhow::Result<()> {
    let repo = PgQuestionRepository::from_pool(pool);
    let input = NewQuestion {
        question: "What is the capital of France?".into(),
        display_answer: "Paris".into(),
        acceptable_answers: vec!["paris".into(), "Paris, France".into(), "パリ".into()],
    };

    let created = repo.create(input.clone()).await?;
    assert_eq!(created.question, input.question);
    assert_eq!(created.display_answer, input.display_answer);
    assert_eq!(created.acceptable_answers, input.acceptable_answers);
    assert!(created.updated_at >= created.created_at);

    let fetched = repo
        .get(created.id)
        .await?
        .expect("created question must be retrievable");
    assert_eq!(
        fetched, created,
        "get must return exactly what create stored"
    );
    Ok(())
}

#[sqlx::test]
#[ignore = "needs Postgres (DATABASE_URL)"]
async fn create_preserves_empty_acceptable_answers(pool: PgPool) -> anyhow::Result<()> {
    let repo = PgQuestionRepository::from_pool(pool);
    let created = repo
        .create(NewQuestion {
            question: "Q".into(),
            display_answer: "A".into(),
            acceptable_answers: vec![],
        })
        .await?;
    let fetched = repo.get(created.id).await?.expect("row exists");
    assert!(fetched.acceptable_answers.is_empty());
    Ok(())
}

#[sqlx::test]
#[ignore = "needs Postgres (DATABASE_URL)"]
async fn list_returns_all_rows(pool: PgPool) -> anyhow::Result<()> {
    let repo = PgQuestionRepository::from_pool(pool);
    assert!(repo.list().await?.is_empty(), "fresh table lists nothing");

    let seeded = seed(&repo, 3).await?;
    let listed = repo.list().await?;
    assert_eq!(listed.len(), 3);
    assert_eq!(listed, seeded, "list returns every row ordered by id");
    Ok(())
}

#[sqlx::test]
#[ignore = "needs Postgres (DATABASE_URL)"]
async fn update_changes_fields_and_bumps_updated_at(pool: PgPool) -> anyhow::Result<()> {
    let repo = PgQuestionRepository::from_pool(pool);
    let original = repo.create(new_q(1)).await?;

    // now() has microsecond resolution; make sure the clock visibly advances.
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    let updated = repo
        .update(
            original.id,
            QuestionUpdate {
                question: Some("Updated question?".into()),
                display_answer: Some("Updated answer".into()),
                acceptable_answers: Some(vec!["updated".into()]),
            },
        )
        .await?
        .expect("existing row must update");

    assert_eq!(updated.id, original.id);
    assert_eq!(updated.question, "Updated question?");
    assert_eq!(updated.display_answer, "Updated answer");
    assert_eq!(updated.acceptable_answers, vec!["updated".to_string()]);
    assert_eq!(
        updated.created_at, original.created_at,
        "created_at is immutable"
    );
    assert!(
        updated.updated_at > original.updated_at,
        "updated_at must be bumped: {} !> {}",
        updated.updated_at,
        original.updated_at
    );

    let fetched = repo.get(original.id).await?.expect("row still exists");
    assert_eq!(fetched, updated, "update is persisted");
    Ok(())
}

#[sqlx::test]
#[ignore = "needs Postgres (DATABASE_URL)"]
async fn partial_update_leaves_other_fields_unchanged(pool: PgPool) -> anyhow::Result<()> {
    let repo = PgQuestionRepository::from_pool(pool);
    let original = repo.create(new_q(1)).await?;

    let updated = repo
        .update(
            original.id,
            QuestionUpdate {
                display_answer: Some("Only this changed".into()),
                ..Default::default()
            },
        )
        .await?
        .expect("existing row must update");

    assert_eq!(updated.question, original.question);
    assert_eq!(updated.display_answer, "Only this changed");
    assert_eq!(updated.acceptable_answers, original.acceptable_answers);
    Ok(())
}

#[sqlx::test]
#[ignore = "needs Postgres (DATABASE_URL)"]
async fn delete_removes_the_row(pool: PgPool) -> anyhow::Result<()> {
    let repo = PgQuestionRepository::from_pool(pool);
    let seeded = seed(&repo, 2).await?;

    assert!(
        repo.delete(seeded[0].id).await?,
        "delete reports a removed row"
    );
    assert!(repo.get(seeded[0].id).await?.is_none());

    let remaining = repo.list().await?;
    assert_eq!(remaining, vec![seeded[1].clone()], "other rows untouched");
    Ok(())
}

#[sqlx::test]
#[ignore = "needs Postgres (DATABASE_URL)"]
async fn missing_id_returns_none_or_false(pool: PgPool) -> anyhow::Result<()> {
    let repo = PgQuestionRepository::from_pool(pool);
    let missing = 9_999_999;

    assert!(repo.get(missing).await?.is_none(), "get(missing) -> None");
    assert!(!repo.delete(missing).await?, "delete(missing) -> false");
    assert!(
        repo.update(missing, QuestionUpdate::default())
            .await?
            .is_none(),
        "update(missing) -> None"
    );

    // Deleting twice: second call reports nothing removed.
    let q = repo.create(new_q(1)).await?;
    assert!(repo.delete(q.id).await?);
    assert!(!repo.delete(q.id).await?);
    Ok(())
}

#[sqlx::test]
#[ignore = "needs Postgres (DATABASE_URL)"]
async fn random_returns_at_most_n_distinct_rows(pool: PgPool) -> anyhow::Result<()> {
    let repo = PgQuestionRepository::from_pool(pool);
    let seeded = seed(&repo, 10).await?;
    let all_ids = ids(&seeded);

    for _ in 0..5 {
        let picked = repo.random(3).await?;
        assert_eq!(picked.len(), 3, "10 rows available, so exactly 3 returned");
        let picked_ids = ids(&picked);
        assert_eq!(picked_ids.len(), picked.len(), "rows are distinct");
        assert!(picked_ids.is_subset(&all_ids), "rows come from the table");
    }
    Ok(())
}

#[sqlx::test]
#[ignore = "needs Postgres (DATABASE_URL)"]
async fn random_with_n_larger_than_table_returns_all_rows(pool: PgPool) -> anyhow::Result<()> {
    let repo = PgQuestionRepository::from_pool(pool);
    let seeded = seed(&repo, 4).await?;

    let picked = repo.random(50).await?;
    assert_eq!(picked.len(), 4);
    assert_eq!(ids(&picked), ids(&seeded));
    Ok(())
}

#[sqlx::test]
#[ignore = "needs Postgres (DATABASE_URL)"]
async fn random_zero_returns_empty(pool: PgPool) -> anyhow::Result<()> {
    let repo = PgQuestionRepository::from_pool(pool);
    seed(&repo, 3).await?;

    assert!(repo.random(0).await?.is_empty());
    Ok(())
}

#[sqlx::test]
#[ignore = "needs Postgres (DATABASE_URL)"]
async fn random_on_empty_table_returns_empty(pool: PgPool) -> anyhow::Result<()> {
    let repo = PgQuestionRepository::from_pool(pool);
    assert!(repo.random(5).await?.is_empty());
    Ok(())
}
