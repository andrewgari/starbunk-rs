use anyhow::Context as _;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub struct ConfigRecord {
    pub name: String,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait ConfigStore: Send + Sync {
    async fn upsert_bot(&self, name: &str, config: serde_json::Value) -> anyhow::Result<()>;
    async fn get_all_bots(&self) -> anyhow::Result<Vec<ConfigRecord>>;
    async fn delete_bot(&self, name: &str) -> anyhow::Result<()>;
}

pub struct PgConfigStore {
    pool: PgPool,
}

impl PgConfigStore {
    pub async fn new(pool: PgPool) -> anyhow::Result<Self> {
        let store = Self { pool };
        store.init_schema().await?;
        Ok(store)
    }

    async fn init_schema(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS bunkbot_configs (
                name VARCHAR(255) PRIMARY KEY,
                config JSONB NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )"#,
        )
        .execute(&self.pool)
        .await
        .context("failed to create bunkbot_configs table")?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ConfigStore for PgConfigStore {
    async fn upsert_bot(&self, name: &str, config: serde_json::Value) -> anyhow::Result<()> {
        sqlx::query(
            r#"INSERT INTO bunkbot_configs (name, config, updated_at) 
               VALUES ($1, $2, CURRENT_TIMESTAMP)
               ON CONFLICT (name) DO UPDATE 
               SET config = EXCLUDED.config, updated_at = CURRENT_TIMESTAMP"#,
        )
        .bind(name)
        .bind(config)
        .execute(&self.pool)
        .await
        .context("failed to upsert bot config")?;

        Ok(())
    }

    async fn get_all_bots(&self) -> anyhow::Result<Vec<ConfigRecord>> {
        let rows = sqlx::query_as::<_, (String, serde_json::Value, DateTime<Utc>, DateTime<Utc>)>(
            r#"SELECT name, config, created_at, updated_at
               FROM bunkbot_configs
               ORDER BY name ASC"#,
        )
        .fetch_all(&self.pool)
        .await
        .context("failed to query all bot configs")?;

        Ok(rows
            .into_iter()
            .map(|(name, config, created_at, updated_at)| ConfigRecord {
                name,
                config,
                created_at,
                updated_at,
            })
            .collect())
    }

    async fn delete_bot(&self, name: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM bunkbot_configs WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .context("failed to delete bot config")?;

        Ok(())
    }
}
