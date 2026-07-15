use anyhow::Context as _;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub struct AuditRecord {
    pub id: i32,
    pub bot_name: String,
    pub trigger_condition: String,
    pub output_message: String,
    pub expected: Option<bool>,
    pub created_at: DateTime<Utc>,
}

pub struct AuditStore {
    pool: PgPool,
}

impl AuditStore {
    pub async fn new(pool: PgPool) -> anyhow::Result<Self> {
        let store = Self { pool };
        store.init_schema().await?;
        Ok(store)
    }

    async fn init_schema(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS bot_audit_history (
                id SERIAL PRIMARY KEY,
                bot_name VARCHAR(255) NOT NULL,
                trigger_condition TEXT NOT NULL,
                output_message TEXT NOT NULL,
                expected BOOLEAN,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )"#,
        )
        .execute(&self.pool)
        .await
        .context("failed to create bot_audit_history table")?;

        Ok(())
    }

    pub async fn log_event(
        &self,
        bot_name: &str,
        trigger_condition: &str,
        output_message: &str,
        expected: Option<bool>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO bot_audit_history (bot_name, trigger_condition, output_message, expected) VALUES ($1, $2, $3, $4)"
        )
        .bind(bot_name)
        .bind(trigger_condition)
        .bind(output_message)
        .bind(expected)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    #[ignore = "requires a live Postgres connection — run with DATABASE_URL set"]
    async fn it_initializes_audit_schema() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/starbunk_memory".to_string()
        });
        let pool = PgPoolOptions::new()
            .connect(&db_url)
            .await
            .expect("Failed to connect to DB");

        let _store = AuditStore::new(pool).await.expect("schema init failed");
    }
}
