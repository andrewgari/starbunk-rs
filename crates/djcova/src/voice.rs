use async_trait::async_trait;
use serenity::all::{ChannelId, GuildId};
use songbird::input::Compose;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackInfo {
    pub title: String,
    pub duration: Option<Duration>,
    pub thumbnail_url: Option<String>,
}

#[async_trait]
pub trait VoiceService: Send + Sync + std::fmt::Debug {
    async fn join(&self, guild_id: GuildId, channel_id: ChannelId) -> anyhow::Result<()>;
    async fn leave(&self, guild_id: GuildId) -> anyhow::Result<()>;
    async fn resolve_metadata(&self, input: &str) -> anyhow::Result<TrackInfo>;
    /// Resolves metadata and starts playback in a single yt-dlp invocation.
    async fn play(&self, guild_id: GuildId, track_url: &str) -> anyhow::Result<TrackInfo>;
    async fn stop(&self, guild_id: GuildId) -> anyhow::Result<()>;
    async fn pause(&self, guild_id: GuildId) -> anyhow::Result<()>;
    async fn resume(&self, guild_id: GuildId) -> anyhow::Result<()>;
    async fn set_volume(&self, guild_id: GuildId, volume: f32) -> anyhow::Result<()>;
    async fn get_track_position(&self, guild_id: GuildId) -> anyhow::Result<Option<Duration>>;
}

#[derive(Debug)]
pub struct DiscordVoiceService {
    songbird: Arc<songbird::Songbird>,
    client: reqwest::Client,
}

impl DiscordVoiceService {
    pub fn new(songbird: Arc<songbird::Songbird>) -> Self {
        Self {
            songbird,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }
}

#[async_trait]
impl VoiceService for DiscordVoiceService {
    async fn join(&self, guild_id: GuildId, channel_id: ChannelId) -> anyhow::Result<()> {
        self.songbird
            .join(guild_id, channel_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to join voice channel: {e}"))?;
        Ok(())
    }

    async fn leave(&self, guild_id: GuildId) -> anyhow::Result<()> {
        let _ = self.songbird.remove(guild_id).await;
        Ok(())
    }

    async fn resolve_metadata(&self, input: &str) -> anyhow::Result<TrackInfo> {
        let url_or_query = if input.starts_with("http://") || input.starts_with("https://") {
            input.to_string()
        } else {
            format!("ytsearch:{}", input)
        };

        let mut ytdl = songbird::input::YoutubeDl::new(self.client.clone(), url_or_query);
        let metadata = ytdl.aux_metadata().await?;

        let title = metadata
            .title
            .or(metadata.track)
            .unwrap_or_else(|| "Unknown Title".to_string());

        Ok(TrackInfo {
            title,
            duration: metadata.duration,
            thumbnail_url: metadata.thumbnail,
        })
    }

    async fn play(&self, guild_id: GuildId, track_url: &str) -> anyhow::Result<TrackInfo> {
        let url_or_query = if track_url.starts_with("http://") || track_url.starts_with("https://")
        {
            track_url.to_string()
        } else {
            format!("ytsearch:{}", track_url)
        };

        // Resolve metadata before acquiring the handler lock — yt-dlp can take seconds
        // and holding the per-guild lock across that await would block all other voice ops.
        let mut source = songbird::input::YoutubeDl::new(self.client.clone(), url_or_query);
        let metadata = source.aux_metadata().await?;
        let title = metadata
            .title
            .or(metadata.track)
            .unwrap_or_else(|| "Unknown Title".to_string());

        if let Some(handler_lock) = self.songbird.get(guild_id) {
            let mut handler = handler_lock.lock().await;
            let queue = handler.queue().clone();
            queue.stop();
            queue.add_source(source.into(), &mut handler).await;
            Ok(TrackInfo {
                title,
                duration: metadata.duration,
                thumbnail_url: metadata.thumbnail,
            })
        } else {
            anyhow::bail!("Bot is not connected to a voice channel in this guild.")
        }
    }

    async fn stop(&self, guild_id: GuildId) -> anyhow::Result<()> {
        if let Some(handler_lock) = self.songbird.get(guild_id) {
            let handler = handler_lock.lock().await;
            handler.queue().stop();
        }
        Ok(())
    }

    async fn pause(&self, guild_id: GuildId) -> anyhow::Result<()> {
        let handler_lock = self
            .songbird
            .get(guild_id)
            .ok_or_else(|| anyhow::anyhow!("Not connected to a voice channel"))?;
        let handler = handler_lock.lock().await;
        let track = handler
            .queue()
            .current()
            .ok_or_else(|| anyhow::anyhow!("Nothing is currently playing"))?;
        track
            .pause()
            .map_err(|e| anyhow::anyhow!("Failed to pause track: {e}"))
    }

    async fn resume(&self, guild_id: GuildId) -> anyhow::Result<()> {
        let handler_lock = self
            .songbird
            .get(guild_id)
            .ok_or_else(|| anyhow::anyhow!("Not connected to a voice channel"))?;
        let handler = handler_lock.lock().await;
        let track = handler
            .queue()
            .current()
            .ok_or_else(|| anyhow::anyhow!("Nothing is currently playing"))?;
        track
            .play()
            .map_err(|e| anyhow::anyhow!("Failed to resume track: {e}"))
    }

    async fn set_volume(&self, guild_id: GuildId, volume: f32) -> anyhow::Result<()> {
        if let Some(handler_lock) = self.songbird.get(guild_id) {
            let handler = handler_lock.lock().await;
            if let Some(track) = handler.queue().current() {
                let _ = track.set_volume(volume);
            }
        }
        Ok(())
    }

    async fn get_track_position(&self, guild_id: GuildId) -> anyhow::Result<Option<Duration>> {
        if let Some(handler_lock) = self.songbird.get(guild_id) {
            let handler = handler_lock.lock().await;
            if let Some(track) = handler.queue().current() {
                if let Ok(info) = track.get_info().await {
                    return Ok(Some(info.position));
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discord_voice_service_resolve_metadata() {
        // Skip test if yt-dlp is not present on the system path (e.g. in CI)
        let yt_dlp_exists = std::process::Command::new("yt-dlp")
            .arg("--version")
            .output()
            .is_ok();
        if !yt_dlp_exists {
            eprintln!("yt-dlp is not installed. Skipping live resolve_metadata test.");
            return;
        }

        // Check if we can construct Songbird with default or if we need a custom constructor
        let songbird = songbird::Songbird::serenity();
        let service = DiscordVoiceService::new(songbird);

        // Resolve metadata for a public YouTube URL
        let res = service
            .resolve_metadata("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
            .await;

        assert!(res.is_ok(), "resolve_metadata failed: {:?}", res.err());
        let info = res.unwrap();
        assert!(!info.title.is_empty());
        let title_lower = info.title.to_lowercase();
        assert!(
            title_lower.contains("rick astley") || title_lower.contains("never gonna give you up"),
            "Title did not contain expected terms: {}",
            info.title
        );
    }
}
