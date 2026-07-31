use sqlx::PgPool;
use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GuildInfo {
    pub bot_name: String,
    pub guild_id: u64,
    pub guild_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChannelInfo {
    pub bot_name: String,
    pub guild_id: u64,
    pub channel_id: u64,
    pub channel_name: String,
}

#[async_trait::async_trait]
pub trait BotTrackingStore: Send + Sync {
    async fn upsert_guild(
        &self,
        bot_name: &str,
        guild_id: u64,
        guild_name: &str,
    ) -> anyhow::Result<()>;
    async fn remove_guild(&self, bot_name: &str, guild_id: u64) -> anyhow::Result<()>;
    async fn upsert_channel(
        &self,
        bot_name: &str,
        guild_id: u64,
        channel_id: u64,
        channel_name: &str,
    ) -> anyhow::Result<()>;
    async fn remove_channel(&self, bot_name: &str, channel_id: u64) -> anyhow::Result<()>;
    async fn get_all_guilds(&self) -> anyhow::Result<HashMap<String, Vec<GuildInfo>>>;
    async fn get_all_channels(&self) -> anyhow::Result<HashMap<String, Vec<ChannelInfo>>>;
}

pub struct PgBotTrackingStore {
    pool: PgPool,
}

impl PgBotTrackingStore {
    pub async fn new(pool: PgPool) -> anyhow::Result<Self> {
        let store = Self { pool };
        store.init_schema().await?;
        Ok(store)
    }

    async fn init_schema(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS bot_guilds (
                bot_name TEXT NOT NULL,
                guild_id BIGINT NOT NULL,
                guild_name TEXT NOT NULL,
                PRIMARY KEY (bot_name, guild_id)
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS bot_channels (
                bot_name TEXT NOT NULL,
                guild_id BIGINT NOT NULL,
                channel_id BIGINT NOT NULL,
                channel_name TEXT NOT NULL,
                PRIMARY KEY (bot_name, channel_id)
            )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl BotTrackingStore for PgBotTrackingStore {
    async fn upsert_guild(
        &self,
        bot_name: &str,
        guild_id: u64,
        guild_name: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO bot_guilds (bot_name, guild_id, guild_name) VALUES ($1, $2, $3)
             ON CONFLICT (bot_name, guild_id) DO UPDATE SET guild_name = EXCLUDED.guild_name",
        )
        .bind(bot_name)
        .bind(guild_id as i64)
        .bind(guild_name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn remove_guild(&self, bot_name: &str, guild_id: u64) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM bot_guilds WHERE bot_name = $1 AND guild_id = $2")
            .bind(bot_name)
            .bind(guild_id as i64)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn upsert_channel(
        &self,
        bot_name: &str,
        guild_id: u64,
        channel_id: u64,
        channel_name: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO bot_channels (bot_name, guild_id, channel_id, channel_name) VALUES ($1, $2, $3, $4)
             ON CONFLICT (bot_name, channel_id) DO UPDATE SET channel_name = EXCLUDED.channel_name, guild_id = EXCLUDED.guild_id"
        )
        .bind(bot_name)
        .bind(guild_id as i64)
        .bind(channel_id as i64)
        .bind(channel_name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn remove_channel(&self, bot_name: &str, channel_id: u64) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM bot_channels WHERE bot_name = $1 AND channel_id = $2")
            .bind(bot_name)
            .bind(channel_id as i64)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_all_guilds(&self) -> anyhow::Result<HashMap<String, Vec<GuildInfo>>> {
        let records = sqlx::query("SELECT bot_name, guild_id, guild_name FROM bot_guilds")
            .fetch_all(&self.pool)
            .await?;

        let mut map: HashMap<String, Vec<GuildInfo>> = HashMap::new();
        for r in records {
            use sqlx::Row;
            let bot_name: String = r.try_get("bot_name")?;
            let guild_id: i64 = r.try_get("guild_id")?;
            let guild_name: String = r.try_get("guild_name")?;
            map.entry(bot_name.clone()).or_default().push(GuildInfo {
                bot_name,
                guild_id: guild_id as u64,
                guild_name,
            });
        }
        Ok(map)
    }

    async fn get_all_channels(&self) -> anyhow::Result<HashMap<String, Vec<ChannelInfo>>> {
        let records =
            sqlx::query("SELECT bot_name, guild_id, channel_id, channel_name FROM bot_channels")
                .fetch_all(&self.pool)
                .await?;

        let mut map: HashMap<String, Vec<ChannelInfo>> = HashMap::new();
        for r in records {
            use sqlx::Row;
            let bot_name: String = r.try_get("bot_name")?;
            let guild_id: i64 = r.try_get("guild_id")?;
            let channel_id: i64 = r.try_get("channel_id")?;
            let channel_name: String = r.try_get("channel_name")?;
            map.entry(bot_name.clone()).or_default().push(ChannelInfo {
                bot_name,
                guild_id: guild_id as u64,
                channel_id: channel_id as u64,
                channel_name,
            });
        }
        Ok(map)
    }
}

pub struct BotTrackingHandler {
    pub bot_name: String,
    pub store: std::sync::Arc<dyn BotTrackingStore>,
}

#[serenity::async_trait]
impl serenity::all::EventHandler for BotTrackingHandler {
    async fn ready(&self, _ctx: serenity::all::Context, ready: serenity::all::Ready) {
        for guild in ready.guilds {
            let _ = self
                .store
                .upsert_guild(&self.bot_name, guild.id.get(), "")
                .await;
        }
    }

    async fn guild_create(
        &self,
        _ctx: serenity::all::Context,
        guild: serenity::all::Guild,
        _is_new: Option<bool>,
    ) {
        let _ = self
            .store
            .upsert_guild(&self.bot_name, guild.id.get(), &guild.name)
            .await;
        for (channel_id, channel) in guild.channels {
            let _ = self
                .store
                .upsert_channel(
                    &self.bot_name,
                    guild.id.get(),
                    channel_id.get(),
                    &channel.name,
                )
                .await;
        }
    }

    async fn guild_delete(
        &self,
        _ctx: serenity::all::Context,
        incomplete: serenity::all::UnavailableGuild,
        _full: Option<serenity::all::Guild>,
    ) {
        let _ = self
            .store
            .remove_guild(&self.bot_name, incomplete.id.get())
            .await;
    }

    async fn channel_create(
        &self,
        _ctx: serenity::all::Context,
        channel: serenity::all::GuildChannel,
    ) {
        let _ = self
            .store
            .upsert_channel(
                &self.bot_name,
                channel.guild_id.get(),
                channel.id.get(),
                &channel.name,
            )
            .await;
    }

    async fn channel_delete(
        &self,
        _ctx: serenity::all::Context,
        channel: serenity::all::GuildChannel,
        _messages: Option<Vec<serenity::all::Message>>,
    ) {
        let _ = self
            .store
            .remove_channel(&self.bot_name, channel.id.get())
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock store for testing
    pub struct MockTrackingStore {
        pub guilds: std::sync::Mutex<HashMap<(String, u64), String>>,
        pub channels: std::sync::Mutex<HashMap<(String, u64), (u64, String)>>,
    }

    impl MockTrackingStore {
        pub fn new() -> Self {
            Self {
                guilds: std::sync::Mutex::new(HashMap::new()),
                channels: std::sync::Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl BotTrackingStore for MockTrackingStore {
        async fn upsert_guild(
            &self,
            bot_name: &str,
            guild_id: u64,
            guild_name: &str,
        ) -> anyhow::Result<()> {
            let mut guilds = self.guilds.lock().unwrap();
            guilds.insert((bot_name.to_string(), guild_id), guild_name.to_string());
            Ok(())
        }

        async fn remove_guild(&self, bot_name: &str, guild_id: u64) -> anyhow::Result<()> {
            let mut guilds = self.guilds.lock().unwrap();
            guilds.remove(&(bot_name.to_string(), guild_id));
            Ok(())
        }

        async fn upsert_channel(
            &self,
            bot_name: &str,
            guild_id: u64,
            channel_id: u64,
            channel_name: &str,
        ) -> anyhow::Result<()> {
            let mut channels = self.channels.lock().unwrap();
            channels.insert(
                (bot_name.to_string(), channel_id),
                (guild_id, channel_name.to_string()),
            );
            Ok(())
        }

        async fn remove_channel(&self, bot_name: &str, channel_id: u64) -> anyhow::Result<()> {
            let mut channels = self.channels.lock().unwrap();
            channels.remove(&(bot_name.to_string(), channel_id));
            Ok(())
        }

        async fn get_all_guilds(&self) -> anyhow::Result<HashMap<String, Vec<GuildInfo>>> {
            let guilds = self.guilds.lock().unwrap();
            let mut result: HashMap<String, Vec<GuildInfo>> = HashMap::new();
            for ((bot_name, guild_id), guild_name) in guilds.iter() {
                result.entry(bot_name.clone()).or_default().push(GuildInfo {
                    bot_name: bot_name.clone(),
                    guild_id: *guild_id,
                    guild_name: guild_name.clone(),
                });
            }
            Ok(result)
        }

        async fn get_all_channels(&self) -> anyhow::Result<HashMap<String, Vec<ChannelInfo>>> {
            let channels = self.channels.lock().unwrap();
            let mut result: HashMap<String, Vec<ChannelInfo>> = HashMap::new();
            for ((bot_name, channel_id), (guild_id, channel_name)) in channels.iter() {
                result
                    .entry(bot_name.clone())
                    .or_default()
                    .push(ChannelInfo {
                        bot_name: bot_name.clone(),
                        guild_id: *guild_id,
                        channel_id: *channel_id,
                        channel_name: channel_name.clone(),
                    });
            }
            Ok(result)
        }
    }

    #[tokio::test]
    async fn test_upsert_and_get_guilds() {
        let store = MockTrackingStore::new();
        store
            .upsert_guild("test_bot", 123, "Test Guild")
            .await
            .unwrap();

        let guilds = store.get_all_guilds().await.unwrap();
        assert!(guilds.contains_key("test_bot"));
        let bot_guilds = guilds.get("test_bot").unwrap();
        assert_eq!(bot_guilds.len(), 1);
        assert_eq!(bot_guilds[0].guild_id, 123);
        assert_eq!(bot_guilds[0].guild_name, "Test Guild");
    }

    #[tokio::test]
    async fn test_remove_guild() {
        let store = MockTrackingStore::new();
        store
            .upsert_guild("test_bot", 123, "Test Guild")
            .await
            .unwrap();
        store.remove_guild("test_bot", 123).await.unwrap();

        let guilds = store.get_all_guilds().await.unwrap();
        assert!(guilds.get("test_bot").unwrap_or(&vec![]).is_empty());
    }

    #[tokio::test]
    async fn test_upsert_and_get_channels() {
        let store = MockTrackingStore::new();
        store
            .upsert_channel("test_bot", 123, 456, "Test Channel")
            .await
            .unwrap();

        let channels = store.get_all_channels().await.unwrap();
        assert!(channels.contains_key("test_bot"));
        let bot_channels = channels.get("test_bot").unwrap();
        assert_eq!(bot_channels.len(), 1);
        assert_eq!(bot_channels[0].channel_id, 456);
        assert_eq!(bot_channels[0].channel_name, "Test Channel");
    }

    #[tokio::test]
    async fn test_remove_channel() {
        let store = MockTrackingStore::new();
        store
            .upsert_channel("test_bot", 123, 456, "Test Channel")
            .await
            .unwrap();
        store.remove_channel("test_bot", 456).await.unwrap();

        let channels = store.get_all_channels().await.unwrap();
        assert!(channels.get("test_bot").unwrap_or(&vec![]).is_empty());
    }
}
