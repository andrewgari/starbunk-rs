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

/// Copies the cookies file to a writable temp path and returns that path.
///
/// Kubernetes secret volumes are always read-only (projected tmpfs). yt-dlp tries to save
/// updated session cookies back to the `--cookies` file on teardown; passing the K8s secret
/// path directly causes an EROFS crash. Copying to /tmp gives yt-dlp a writable location.
fn writable_cookies_path(source_path: &str) -> String {
    const TMP_COOKIES: &str = "/tmp/yt-dlp-cookies.txt";
    match std::fs::copy(source_path, TMP_COOKIES) {
        Ok(_) => TMP_COOKIES.to_string(),
        Err(e) => {
            tracing::warn!(source = source_path, err = %e, "failed to copy cookies to /tmp, falling back to source path");
            source_path.to_string()
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
        if let Ok(cookies_path) = std::env::var("YOUTUBE_COOKIES_PATH") {
            let trimmed = cookies_path.trim();
            if !trimmed.is_empty() {
                ytdl = ytdl.user_args(vec![
                    "--cookies".to_string(),
                    writable_cookies_path(trimmed),
                ]);
            }
        }
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
        if let Ok(cookies_path) = std::env::var("YOUTUBE_COOKIES_PATH") {
            let trimmed = cookies_path.trim();
            if !trimmed.is_empty() {
                source = source.user_args(vec![
                    "--cookies".to_string(),
                    writable_cookies_path(trimmed),
                ]);
            }
        }
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

    static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct EnvGuard {
        original_path: String,
        original_cookies: Option<String>,
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            std::env::set_var("PATH", &self.original_path);
            if let Some(val) = &self.original_cookies {
                std::env::set_var("YOUTUBE_COOKIES_PATH", val);
            } else {
                std::env::remove_var("YOUTUBE_COOKIES_PATH");
            }
        }
    }

    #[tokio::test]
    #[ignore]
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

    #[tokio::test]
    #[ignore]
    async fn test_symphonia_probe_youtube_dl() {
        let yt_dlp_exists = std::process::Command::new("yt-dlp")
            .arg("--version")
            .output()
            .is_ok();
        if !yt_dlp_exists {
            return;
        }

        let client = reqwest::Client::new();
        let mut source =
            songbird::input::YoutubeDl::new(client, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        let audio_stream = source.create_async().await;
        assert!(
            audio_stream.is_ok(),
            "create_async failed: {:?}",
            audio_stream.err()
        );
        let stream = audio_stream.unwrap();

        // Run symphonia probe in spawn_blocking to prevent deadlocking the tokio runtime thread
        let probed = tokio::task::spawn_blocking(move || {
            use symphonia::core::io::MediaSourceStream;
            use symphonia::core::probe::Hint;

            let mss = MediaSourceStream::new(stream.input, Default::default());
            let mut hint = Hint::new();
            hint.with_extension("webm");

            symphonia::default::get_probe().format(
                &hint,
                mss,
                &Default::default(),
                &Default::default(),
            )
        })
        .await
        .unwrap();

        assert!(probed.is_ok(), "Symphonia probe failed: {:?}", probed.err());
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_youtube_cookies_path_env_var() {
        let _lock = ENV_MUTEX.lock().unwrap();

        use std::fs::File;
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;

        // Create a temporary directory in target (inside workspace)
        let temp_dir = std::env::current_dir()
            .unwrap()
            .join("target")
            .join("test_temp_ytdl");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let mock_ytdl_path = temp_dir.join("yt-dlp");
        let arg_log_path = temp_dir.join("args.txt");

        // Write a mock yt-dlp script that logs its arguments
        let script_content = format!(
            "#!/bin/sh\necho \"$@\" > \"{}\"\nexit 1\n",
            arg_log_path.to_str().unwrap()
        );
        {
            let mut file = File::create(&mock_ytdl_path).unwrap();
            file.write_all(script_content.as_bytes()).unwrap();
        }

        // Make the script executable
        let mut perms = std::fs::metadata(&mock_ytdl_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&mock_ytdl_path, perms).unwrap();

        // Backup current PATH and YOUTUBE_COOKIES_PATH
        let original_path = std::env::var("PATH").unwrap_or_default();
        let original_cookies = std::env::var("YOUTUBE_COOKIES_PATH").ok();

        // Create RAII guard to restore PATH and YOUTUBE_COOKIES_PATH even if panic occurs
        let _guard = EnvGuard {
            original_path: original_path.clone(),
            original_cookies: original_cookies.clone(),
        };

        // Update PATH to put our mock yt-dlp first
        let new_path = format!("{}:{}", temp_dir.to_str().unwrap(), original_path);
        std::env::set_var("PATH", new_path);
        std::env::set_var("YOUTUBE_COOKIES_PATH", "mock_cookies.txt");

        // Construct service
        let songbird = songbird::Songbird::serenity();
        let service = DiscordVoiceService::new(songbird);

        // Run resolve_metadata, which will call our mock yt-dlp
        let _ = service.resolve_metadata("test_query").await;

        // Read the logged arguments
        let logged_args =
            std::fs::read_to_string(&arg_log_path).expect("Mock yt-dlp was not executed");

        // Clean up temp dir
        let _ = std::fs::remove_dir_all(&temp_dir);

        assert!(
            logged_args.contains("--cookies mock_cookies.txt"),
            "yt-dlp was not called with cookies option. Args: {}",
            logged_args
        );
    }
}
