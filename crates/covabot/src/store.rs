use anyhow::Result;
use sqlx::PgPool;

#[allow(dead_code)]
pub struct CovaBotStore {
    pool: PgPool,
}

impl CovaBotStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn init(&self) -> Result<()> {
        Ok(())
    }

    pub async fn load_profile_yaml(&self) -> Result<String> {
        // Stub for TDD
        anyhow::bail!("not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn it_loads_profile_from_db() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/starbunk_memory".to_string()
        });
        let pool = PgPoolOptions::new()
            .connect(&db_url)
            .await
            .expect("Failed to connect to DB");

        let store = CovaBotStore::new(pool);
        store.init().await.unwrap();

        // This should fail in TDD since it's not implemented yet
        let yaml = store.load_profile_yaml().await.unwrap();
        assert!(!yaml.is_empty());
    }
}
