use crate::assignment::Assignment;
use anyhow::Context as _;
use chrono::{DateTime, Utc};
use serenity::all::{GuildId, RoleId, UserId};
use sqlx::PgPool;

#[derive(Debug, PartialEq, Eq)]
pub enum EventStatus {
    Initializing,
    Assigned,
}

impl From<String> for EventStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "assigned" => EventStatus::Assigned,
            _ => EventStatus::Initializing,
        }
    }
}

impl From<&EventStatus> for String {
    fn from(s: &EventStatus) -> Self {
        match s {
            EventStatus::Initializing => "initializing".to_string(),
            EventStatus::Assigned => "assigned".to_string(),
        }
    }
}

pub struct RatmasEvent {
    pub guild_id: GuildId,
    pub participant_role_id: RoleId,
    pub status: EventStatus,
    pub updated_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait Store: Send + Sync {
    async fn init_event(&self, guild_id: GuildId, role_id: RoleId) -> anyhow::Result<()>;
    async fn get_event(&self, guild_id: GuildId) -> anyhow::Result<Option<RatmasEvent>>;
    async fn update_event_status(
        &self,
        guild_id: GuildId,
        status: EventStatus,
    ) -> anyhow::Result<()>;
    async fn cancel_event(&self, guild_id: GuildId) -> anyhow::Result<()>;

    async fn save_assignments(
        &self,
        guild_id: GuildId,
        assignments: &[Assignment],
    ) -> anyhow::Result<()>;
    async fn get_assignments(&self, guild_id: GuildId) -> anyhow::Result<Vec<Assignment>>;
    async fn get_active_guilds_for_user(&self, user_id: UserId) -> anyhow::Result<Vec<GuildId>>;
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

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn init_schema(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS ratmas_events (
                guild_id BIGINT PRIMARY KEY,
                participant_role_id BIGINT NOT NULL,
                status VARCHAR(50) NOT NULL DEFAULT 'initializing',
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )"#,
        )
        .execute(&self.pool)
        .await
        .context("failed to create ratmas_events table")?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS ratmas_assignments (
                id SERIAL PRIMARY KEY,
                guild_id BIGINT NOT NULL REFERENCES ratmas_events(guild_id) ON DELETE CASCADE,
                gifter_id BIGINT NOT NULL,
                recipient_id BIGINT NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                UNIQUE (guild_id, gifter_id),
                UNIQUE (guild_id, recipient_id)
            )"#,
        )
        .execute(&self.pool)
        .await
        .context("failed to create ratmas_assignments table")?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Store for PgStore {
    async fn init_event(&self, guild_id: GuildId, role_id: RoleId) -> anyhow::Result<()> {
        sqlx::query(
            r#"INSERT INTO ratmas_events (guild_id, participant_role_id, status)
               VALUES ($1, $2, 'initializing')
               ON CONFLICT (guild_id) DO UPDATE SET 
                 participant_role_id = EXCLUDED.participant_role_id,
                 status = 'initializing',
                 updated_at = CURRENT_TIMESTAMP"#,
        )
        .bind(guild_id.get() as i64)
        .bind(role_id.get() as i64)
        .execute(&self.pool)
        .await
        .context("failed to init event")?;

        // Clear any old assignments for this guild
        sqlx::query("DELETE FROM ratmas_assignments WHERE guild_id = $1")
            .bind(guild_id.get() as i64)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_event(&self, guild_id: GuildId) -> anyhow::Result<Option<RatmasEvent>> {
        let row = sqlx::query_as::<_, (i64, i64, String, DateTime<Utc>)>(
            "SELECT guild_id, participant_role_id, status, updated_at FROM ratmas_events WHERE guild_id = $1"
        )
        .bind(guild_id.get() as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(gid, rid, status_str, updated_at)| RatmasEvent {
            guild_id: GuildId::new(gid as u64),
            participant_role_id: RoleId::new(rid as u64),
            status: EventStatus::from(status_str),
            updated_at,
        }))
    }

    async fn update_event_status(
        &self,
        guild_id: GuildId,
        status: EventStatus,
    ) -> anyhow::Result<()> {
        let status_str: String = (&status).into();
        sqlx::query("UPDATE ratmas_events SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE guild_id = $2")
            .bind(status_str)
            .bind(guild_id.get() as i64)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn cancel_event(&self, guild_id: GuildId) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM ratmas_events WHERE guild_id = $1")
            .bind(guild_id.get() as i64)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn save_assignments(
        &self,
        guild_id: GuildId,
        assignments: &[Assignment],
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM ratmas_assignments WHERE guild_id = $1")
            .bind(guild_id.get() as i64)
            .execute(&mut *tx)
            .await?;

        for a in assignments {
            sqlx::query(
                "INSERT INTO ratmas_assignments (guild_id, gifter_id, recipient_id) VALUES ($1, $2, $3)"
            )
            .bind(guild_id.get() as i64)
            .bind(a.gifter.get() as i64)
            .bind(a.recipient.get() as i64)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_assignments(&self, guild_id: GuildId) -> anyhow::Result<Vec<Assignment>> {
        let rows = sqlx::query_as::<_, (i64, i64)>(
            "SELECT gifter_id, recipient_id FROM ratmas_assignments WHERE guild_id = $1",
        )
        .bind(guild_id.get() as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(g, r)| Assignment {
                gifter: UserId::new(g as u64),
                recipient: UserId::new(r as u64),
            })
            .collect())
    }

    async fn get_active_guilds_for_user(&self, user_id: UserId) -> anyhow::Result<Vec<GuildId>> {
        let rows = sqlx::query_as::<_, (i64,)>(
            r#"SELECT DISTINCT a.guild_id 
               FROM ratmas_assignments a
               JOIN ratmas_events e ON a.guild_id = e.guild_id
               WHERE (a.gifter_id = $1 OR a.recipient_id = $1)
               AND e.status = 'assigned'"#,
        )
        .bind(user_id.get() as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(g,)| GuildId::new(g as u64))
            .collect())
    }
}
