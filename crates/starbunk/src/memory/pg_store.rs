use anyhow::Context as _;
use chrono::{DateTime, Utc};
use pgvector::Vector;
use sqlx::PgPool;

pub struct MemoryRecord {
    pub id: i32,
    pub user_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait Store: Send + Sync {
    async fn save_memory(
        &self,
        user_id: &str,
        content: &str,
        embedding: Vec<f32>,
    ) -> anyhow::Result<()>;

    async fn find_similar(
        &self,
        user_id: &str,
        embedding: Vec<f32>,
        limit: i32,
    ) -> anyhow::Result<Vec<MemoryRecord>>;
}

pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    pub async fn new(conn_str: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(conn_str)
            .await
            .context("failed to connect to postgres")?;

        let store = Self { pool };
        store.init_schema().await?;
        Ok(store)
    }

    async fn init_schema(&self) -> anyhow::Result<()> {
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(&self.pool)
            .await
            .context("failed to create vector extension")?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS memories (
                id SERIAL PRIMARY KEY,
                user_id VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                embedding vector(1536),
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )"#,
        )
        .execute(&self.pool)
        .await
        .context("failed to create memories table")?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Store for PgStore {
    async fn save_memory(
        &self,
        user_id: &str,
        content: &str,
        embedding: Vec<f32>,
    ) -> anyhow::Result<()> {
        let vec = Vector::from(embedding);
        sqlx::query("INSERT INTO memories (user_id, content, embedding) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(content)
            .bind(vec)
            .execute(&self.pool)
            .await
            .context("failed to save memory")?;

        tracing::debug!("saved memory to pgvector");
        Ok(())
    }

    async fn find_similar(
        &self,
        user_id: &str,
        embedding: Vec<f32>,
        limit: i32,
    ) -> anyhow::Result<Vec<MemoryRecord>> {
        let vec = Vector::from(embedding);
        let rows = sqlx::query_as::<_, (i32, String, String, DateTime<Utc>)>(
            r#"SELECT id, user_id, content, created_at
               FROM memories
               WHERE user_id = $1
               ORDER BY embedding <=> $2
               LIMIT $3"#,
        )
        .bind(user_id)
        .bind(vec)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("failed to query similar memories")?;

        Ok(rows
            .into_iter()
            .map(|(id, user_id, content, created_at)| MemoryRecord {
                id,
                user_id,
                content,
                created_at,
            })
            .collect())
    }
}
