use crate::gif_client::GifService;
use crate::voice::VoiceService;
use serenity::all::{ChannelId, GuildId, MessageId, UserId};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    Song,
    Queue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueItem {
    pub id: u64,
    pub title: String,
    pub url: String,
    /// Display name of the requester (may change; use `requester_id` for identity checks).
    pub requester: String,
    /// Stable Discord user ID — use this for ownership/permission comparisons.
    pub requester_id: UserId,
    pub duration: Option<Duration>,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug)]
pub struct GuildAudioManager {
    guild_id: GuildId,
    text_channel_id: Option<ChannelId>,
    pub(crate) voice_channel_id: Option<ChannelId>,
    queue: VecDeque<QueueItem>,
    history: Vec<QueueItem>,
    current_track: Option<QueueItem>,
    volume: u8,
    repeat_mode: RepeatMode,
    is_paused: bool,
    voice: Arc<dyn VoiceService>,
    gif: Arc<dyn GifService>,
    pub idle_timer_active: bool,
    pub leave_timer_active: bool,
    gif_task: Option<tokio::task::JoinHandle<()>>,
    next_item_id: u64,
}

impl GuildAudioManager {
    pub fn new(guild_id: GuildId, voice: Arc<dyn VoiceService>, gif: Arc<dyn GifService>) -> Self {
        Self {
            guild_id,
            text_channel_id: None,
            voice_channel_id: None,
            queue: VecDeque::new(),
            history: Vec::new(),
            current_track: None,
            volume: 50,
            repeat_mode: RepeatMode::Off,
            is_paused: false,
            voice,
            gif,
            idle_timer_active: false,
            leave_timer_active: false,
            gif_task: None,
            next_item_id: 0,
        }
    }

    pub async fn play(
        &mut self,
        http: Option<Arc<serenity::all::Http>>,
        text_channel: ChannelId,
        voice_channel: ChannelId,
        input: String,
        requester: String,
        requester_id: UserId,
    ) -> anyhow::Result<String> {
        tracing::info!(
            bot = "djcova",
            guild = %self.guild_id,
            text_channel = %text_channel,
            voice_channel = %voice_channel,
            track = %input,
            requester = %requester,
            requester_id = %requester_id,
            "Audio play request received"
        );

        self.text_channel_id = Some(text_channel);
        self.voice_channel_id = Some(voice_channel);
        self.next_item_id += 1;
        let item_id = self.next_item_id;

        // Search for existing resolved metadata for the same URL
        let mut existing_metadata = None;
        if let Some(track) = &self.current_track {
            if track.url == input && track.title != "Loading..." {
                existing_metadata = Some((
                    track.title.clone(),
                    track.duration,
                    track.thumbnail_url.clone(),
                ));
            }
        }
        if existing_metadata.is_none() {
            for track in &self.queue {
                if track.url == input && track.title != "Loading..." {
                    existing_metadata = Some((
                        track.title.clone(),
                        track.duration,
                        track.thumbnail_url.clone(),
                    ));
                    break;
                }
            }
        }
        if existing_metadata.is_none() {
            for track in &self.history {
                if track.url == input && track.title != "Loading..." {
                    existing_metadata = Some((
                        track.title.clone(),
                        track.duration,
                        track.thumbnail_url.clone(),
                    ));
                    break;
                }
            }
        }

        let (title, duration, thumbnail_url) = match existing_metadata {
            Some((t, d, th)) => {
                tracing::info!(
                    bot = "djcova",
                    guild = %self.guild_id,
                    track = %input,
                    title = %t,
                    "Reusing cached track metadata"
                );
                (t, d, th)
            }
            None => ("Loading...".to_string(), None, None),
        };

        if self.current_track.is_none() {
            if let Err(e) = self.voice.join(self.guild_id, voice_channel).await {
                tracing::error!(
                    bot = "djcova",
                    guild = %self.guild_id,
                    voice_channel = %voice_channel,
                    err = %e,
                    "Failed to join voice channel"
                );
                crate::record_error("voice_join_failed");
                return Err(e);
            }
            if let Err(e) = self.voice.play(self.guild_id, &input).await {
                tracing::error!(
                    bot = "djcova",
                    guild = %self.guild_id,
                    track = %input,
                    err = %e,
                    "Failed to start track playback"
                );
                crate::record_error("voice_play_failed");
                return Err(e);
            }
            let item = QueueItem {
                id: item_id,
                title,
                url: input.clone(),
                requester,
                requester_id,
                duration,
                thumbnail_url,
            };
            self.current_track = Some(item.clone());
            let _ = self
                .voice
                .set_volume(self.guild_id, self.volume as f32 / 100.0)
                .await;
            self.idle_timer_active = false;
            if let Some(h) = http {
                self.start_gif_loop(h, text_channel);
            }
            tracing::info!(
                bot = "djcova",
                guild = %self.guild_id,
                track = %input,
                "Playback started successfully"
            );
            Ok(format!(
                "Now playing: {} requested by {}",
                item.title, item.requester
            ))
        } else {
            let item = QueueItem {
                id: item_id,
                title,
                url: input.clone(),
                requester,
                requester_id,
                duration,
                thumbnail_url,
            };
            self.queue.push_back(item.clone());
            // Restart the GIF loop so it posts to the new text channel.
            if let Some(h) = http {
                self.stop_gif_loop();
                self.start_gif_loop(h, text_channel);
            }
            tracing::info!(
                bot = "djcova",
                guild = %self.guild_id,
                track = %input,
                "Track queued successfully"
            );
            Ok(format!(
                "Queued: {} requested by {}",
                item.title, item.requester
            ))
        }
    }

    pub async fn skip(&mut self, http: Option<Arc<serenity::all::Http>>) -> anyhow::Result<String> {
        let Some(old_track) = self.current_track.clone() else {
            anyhow::bail!("Nothing is currently playing.");
        };

        tracing::info!(
            bot = "djcova",
            guild = %self.guild_id,
            track = %old_track.title,
            "Skipping current track"
        );

        self.history.push(old_track.clone());

        let next_track = match self.repeat_mode {
            RepeatMode::Song => Some(old_track),
            RepeatMode::Queue => {
                self.queue.push_back(old_track);
                self.queue.pop_front()
            }
            RepeatMode::Off => self.queue.pop_front(),
        };

        self.current_track = next_track;

        if let Some(track) = self.current_track.clone() {
            if let Err(e) = self.voice.play(self.guild_id, &track.url).await {
                tracing::error!(
                    bot = "djcova",
                    guild = %self.guild_id,
                    track = %track.title,
                    err = %e,
                    "Failed to play next track after skip"
                );
                crate::record_error("voice_play_failed");
                return Err(e);
            }
            let _ = self
                .voice
                .set_volume(self.guild_id, self.volume as f32 / 100.0)
                .await;
            if let Some(h) = http {
                if let Some(tc) = self.text_channel_id {
                    self.start_gif_loop(h, tc);
                }
            }
            tracing::info!(
                bot = "djcova",
                guild = %self.guild_id,
                track = %track.title,
                "Skipped to next track successfully"
            );
            Ok(format!("Skipped to: {}", track.title))
        } else {
            if let Err(e) = self.voice.stop(self.guild_id).await {
                tracing::warn!(
                    bot = "djcova",
                    guild = %self.guild_id,
                    err = %e,
                    "Failed to stop voice service during skip-to-empty"
                );
            }
            self.stop_gif_loop();
            self.idle_timer_active = true;
            tracing::info!(
                bot = "djcova",
                guild = %self.guild_id,
                "Playback stopped, queue is empty"
            );
            Ok("Playback stopped, queue is empty.".to_string())
        }
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        tracing::info!(
            bot = "djcova",
            guild = %self.guild_id,
            "Stopping playback and leaving voice channel"
        );
        if let Err(e) = self.voice.leave(self.guild_id).await {
            tracing::warn!(
                bot = "djcova",
                guild = %self.guild_id,
                err = %e,
                "Error leaving voice channel"
            );
        }
        self.queue.clear();
        self.current_track = None;
        self.voice_channel_id = None;
        self.text_channel_id = None;
        self.idle_timer_active = false;
        self.leave_timer_active = false;
        self.stop_gif_loop();
        Ok(())
    }

    pub fn get_voice_service(&self) -> Arc<dyn VoiceService> {
        self.voice.clone()
    }

    pub fn get_queue(&self) -> Vec<QueueItem> {
        self.queue.iter().cloned().collect()
    }

    pub fn get_history(&self) -> Vec<QueueItem> {
        self.history.clone()
    }

    pub fn shuffle(&mut self) {
        tracing::info!(bot = "djcova", guild = %self.guild_id, "Shuffling remaining queue");
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let mut vec: Vec<QueueItem> = self.queue.drain(..).collect();
        vec.shuffle(&mut rng);
        self.queue = vec.into();
    }

    pub fn set_volume(&mut self, volume: u8) {
        tracing::info!(bot = "djcova", guild = %self.guild_id, volume = %volume, "Setting volume");
        self.volume = volume;
        let vol_float = volume as f32 / 100.0;
        let voice = self.voice.clone();
        let guild_id = self.guild_id;
        tokio::spawn(async move {
            if let Err(e) = voice.set_volume(guild_id, vol_float).await {
                tracing::error!(bot = "djcova", guild = %guild_id, err = %e, "Failed to set volume in voice handler");
            }
        });
    }

    pub fn get_volume(&self) -> u8 {
        self.volume
    }

    pub fn clear(&mut self) {
        tracing::info!(bot = "djcova", guild = %self.guild_id, "Clearing queue");
        self.queue.clear();
    }

    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        tracing::info!(bot = "djcova", guild = %self.guild_id, repeat_mode = ?mode, "Setting repeat mode");
        self.repeat_mode = mode;
    }

    pub fn get_repeat_mode(&self) -> RepeatMode {
        self.repeat_mode
    }

    pub fn get_current_track(&self) -> Option<QueueItem> {
        self.current_track.clone()
    }

    /// Pauses the current track without clearing the queue.
    pub async fn pause(&mut self) -> anyhow::Result<()> {
        if self.current_track.is_none() {
            anyhow::bail!("Nothing is currently playing.");
        }
        if let Err(e) = self.voice.pause(self.guild_id).await {
            tracing::error!(bot = "djcova", guild = %self.guild_id, err = %e, "Failed to pause voice playback");
            crate::record_error("pause_failed");
            return Err(e);
        }
        self.is_paused = true;
        tracing::info!(bot = "djcova", guild = %self.guild_id, "Playback paused successfully");
        Ok(())
    }

    /// Resumes a paused track.
    pub async fn resume(&mut self) -> anyhow::Result<()> {
        if !self.is_paused {
            anyhow::bail!("Playback is not paused.");
        }
        if self.current_track.is_none() {
            // is_paused flag became stale (track stopped/cleared while paused)
            self.is_paused = false;
            anyhow::bail!("No active track to resume.");
        }
        if let Err(e) = self.voice.resume(self.guild_id).await {
            tracing::error!(bot = "djcova", guild = %self.guild_id, err = %e, "Failed to resume voice playback");
            crate::record_error("resume_failed");
            return Err(e);
        }
        self.is_paused = false;
        tracing::info!(bot = "djcova", guild = %self.guild_id, "Playback resumed successfully");
        Ok(())
    }

    /// Returns whether playback is currently paused.
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    /// Removes and returns the title of the first queued song requested by `requester_id`.
    /// Returns `None` if no matching item exists.
    pub fn skip_next_by(&mut self, requester_id: UserId) -> Option<String> {
        let pos = self
            .queue
            .iter()
            .position(|item| item.requester_id == requester_id)?;
        self.queue.remove(pos).map(|item| item.title)
    }

    /// Removes and returns the title of the last queued song requested by `requester_id`.
    /// Returns `None` if no matching item exists.
    pub fn skip_last_by(&mut self, requester_id: UserId) -> Option<String> {
        let pos = self
            .queue
            .iter()
            .rposition(|item| item.requester_id == requester_id)?;
        self.queue.remove(pos).map(|item| item.title)
    }

    pub fn tick_idle_timer(&mut self) -> bool {
        if self.idle_timer_active {
            tracing::info!(bot = "djcova", guild = %self.guild_id, "Idle disconnect timer expired, leaving voice channel");
            self.current_track = None;
            self.queue.clear();
            self.idle_timer_active = false;
            self.voice_channel_id = None;
            self.text_channel_id = None;
            self.stop_gif_loop();
            let voice = self.voice.clone();
            let guild_id = self.guild_id;
            tokio::spawn(async move {
                let _ = voice.leave(guild_id).await;
            });
            true
        } else {
            false
        }
    }

    pub fn tick_leave_timer(&mut self) -> bool {
        if self.leave_timer_active {
            tracing::info!(bot = "djcova", guild = %self.guild_id, "Empty channel disconnect timer expired, leaving voice channel");
            self.current_track = None;
            self.queue.clear();
            self.leave_timer_active = false;
            self.voice_channel_id = None;
            self.text_channel_id = None;
            self.stop_gif_loop();
            let voice = self.voice.clone();
            let guild_id = self.guild_id;
            tokio::spawn(async move {
                let _ = voice.leave(guild_id).await;
            });
            true
        } else {
            false
        }
    }

    pub fn user_left_voice_channel(&mut self) {
        tracing::info!(bot = "djcova", guild = %self.guild_id, "Voice channel empty of non-bot members, activating leave timer");
        self.leave_timer_active = true;
    }

    pub fn user_returned_to_voice_channel(&mut self) {
        tracing::info!(bot = "djcova", guild = %self.guild_id, "Non-bot member returned to voice channel, deactivating leave timer");
        self.leave_timer_active = false;
    }

    fn start_gif_loop(&mut self, http: Arc<serenity::all::Http>, text_channel: ChannelId) {
        if self.gif_task.as_ref().is_some_and(|h| !h.is_finished()) {
            return;
        }

        tracing::info!(bot = "djcova", guild = %self.guild_id, channel = %text_channel, "Starting Tenor gif reaction loop");
        let gif = self.gif.clone();
        let mut last_msg_id: Option<MessageId> = None;
        let guild_id = self.guild_id;

        let handle = tokio::spawn(async move {
            use rand::Rng;
            loop {
                let delay = {
                    let mut rng = rand::thread_rng();
                    rng.gen_range(45..=90)
                };
                tokio::time::sleep(Duration::from_secs(delay)).await;

                let gif_url = match gif.fetch_dancing_gif().await {
                    Ok(url) => url,
                    Err(e) => {
                        tracing::warn!(bot = "djcova", guild = %guild_id, err = %e, "Failed to fetch dancing gif");
                        continue;
                    }
                };

                let messages = match text_channel
                    .messages(&http, serenity::all::GetMessages::new().limit(1))
                    .await
                {
                    Ok(msgs) => msgs,
                    Err(e) => {
                        tracing::warn!(bot = "djcova", guild = %guild_id, channel = %text_channel, err = %e, "Failed to retrieve messages for gif check");
                        continue;
                    }
                };

                let should_edit = if let Some(last_posted) = last_msg_id {
                    messages.first().is_some_and(|m| m.id == last_posted)
                } else {
                    false
                };

                if should_edit {
                    let last_posted = last_msg_id.expect("should_edit guarantees Some");
                    let edit_res = text_channel
                        .edit_message(
                            &http,
                            last_posted,
                            serenity::all::EditMessage::new().content(&gif_url),
                        )
                        .await;
                    if let Err(e) = edit_res {
                        tracing::warn!(
                            bot = "djcova",
                            guild = %guild_id,
                            channel = %text_channel,
                            err = %e,
                            "Failed to edit gif message in place, posting new one"
                        );
                        if let Ok(new_msg) = text_channel.say(&http, &gif_url).await {
                            last_msg_id = Some(new_msg.id);
                        }
                    }
                } else if let Ok(new_msg) = text_channel.say(&http, &gif_url).await {
                    last_msg_id = Some(new_msg.id);
                }
            }
        });

        self.gif_task = Some(handle);
    }

    pub async fn restart(&self) -> anyhow::Result<String> {
        match &self.current_track {
            Some(track) => {
                tracing::info!(bot = "djcova", guild = %self.guild_id, track = %track.title, "Restarting current track");
                if let Err(e) = self.voice.play(self.guild_id, &track.url).await {
                    tracing::error!(
                        bot = "djcova",
                        guild = %self.guild_id,
                        track = %track.title,
                        err = %e,
                        "Failed to restart track"
                    );
                    crate::record_error("voice_play_failed");
                    return Err(e);
                }
                let _ = self
                    .voice
                    .set_volume(self.guild_id, self.volume as f32 / 100.0)
                    .await;
                Ok(format!("Restarted: {}", track.title))
            }
            None => anyhow::bail!("Nothing is currently playing."),
        }
    }

    pub fn update_track_metadata(
        &mut self,
        id: u64,
        title: String,
        duration: Option<Duration>,
        thumbnail_url: Option<String>,
    ) -> bool {
        let mut updated = false;
        if let Some(track) = &mut self.current_track {
            if track.id == id {
                tracing::info!(
                    bot = "djcova",
                    guild = %self.guild_id,
                    track_id = id,
                    title = %title,
                    "Resolved metadata for current playing track"
                );
                track.title = title.clone();
                track.duration = duration;
                track.thumbnail_url = thumbnail_url.clone();
                updated = true;
            }
        }
        for track in &mut self.queue {
            if track.id == id {
                tracing::info!(
                    bot = "djcova",
                    guild = %self.guild_id,
                    track_id = id,
                    title = %title,
                    "Resolved metadata for queued track"
                );
                track.title = title.clone();
                track.duration = duration;
                track.thumbnail_url = thumbnail_url.clone();
                updated = true;
            }
        }
        updated
    }

    fn stop_gif_loop(&mut self) {
        if let Some(handle) = self.gif_task.take() {
            tracing::info!(bot = "djcova", guild = %self.guild_id, "Stopping gif loop");
            handle.abort();
        }
    }
}

pub fn spawn_idle_timer(mgr: Arc<Mutex<GuildAudioManager>>) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(120)).await;
        let mut m = mgr.lock().await;
        if m.idle_timer_active {
            let _ = m.stop().await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voice::TrackInfo;
    use async_trait::async_trait;

    // Stable test user IDs — never use usernames for comparisons in production code.
    const USER_A: UserId = UserId::new(1);
    const USER_B: UserId = UserId::new(2);
    const USER_C: UserId = UserId::new(3);
    const USER: UserId = UserId::new(4);

    #[derive(Debug)]
    struct MockVoiceService;

    #[async_trait]
    impl VoiceService for MockVoiceService {
        async fn join(&self, _guild_id: GuildId, _channel_id: ChannelId) -> anyhow::Result<()> {
            Ok(())
        }
        async fn leave(&self, _guild_id: GuildId) -> anyhow::Result<()> {
            Ok(())
        }
        async fn resolve_metadata(&self, _input: &str) -> anyhow::Result<TrackInfo> {
            Ok(TrackInfo {
                title: "Stub Track".to_string(),
                duration: Some(Duration::from_secs(180)),
                thumbnail_url: None,
            })
        }
        async fn play(&self, _guild_id: GuildId, _track_url: &str) -> anyhow::Result<()> {
            Ok(())
        }
        async fn stop(&self, _guild_id: GuildId) -> anyhow::Result<()> {
            Ok(())
        }
        async fn pause(&self, _guild_id: GuildId) -> anyhow::Result<()> {
            Ok(())
        }
        async fn resume(&self, _guild_id: GuildId) -> anyhow::Result<()> {
            Ok(())
        }
        async fn set_volume(&self, _guild_id: GuildId, _volume: f32) -> anyhow::Result<()> {
            Ok(())
        }
        async fn get_track_position(&self, _guild_id: GuildId) -> anyhow::Result<Option<Duration>> {
            Ok(None)
        }
    }

    #[derive(Debug)]
    struct MockGifService;

    #[async_trait]
    impl GifService for MockGifService {
        async fn fetch_dancing_gif(&self) -> anyhow::Result<String> {
            Ok("https://media.tenor.com/stub_gif.gif".to_string())
        }
    }

    fn setup() -> GuildAudioManager {
        GuildAudioManager::new(
            GuildId::new(1),
            Arc::new(MockVoiceService),
            Arc::new(MockGifService),
        )
    }

    #[tokio::test]
    async fn test_queue_add_and_playback_starts_immediately() {
        let mut manager = setup();
        let resp = manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "https://youtube.com/watch?v=123".to_string(),
                "UserA".to_string(),
                USER_A,
            )
            .await
            .unwrap();

        assert!(resp.contains("Now playing") || resp.contains("Playing"));
        assert!(manager.get_current_track().is_some());
        assert_eq!(manager.get_current_track().unwrap().requester, "UserA");
        assert!(manager.get_queue().is_empty());
    }

    #[tokio::test]
    async fn test_queue_add_when_playing_queues_and_does_not_interrupt() {
        let mut manager = setup();
        // Start playing first song
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "UserA".to_string(),
                USER_A,
            )
            .await
            .unwrap();

        // Queue second song
        let resp = manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song2".to_string(),
                "UserB".to_string(),
                USER_B,
            )
            .await
            .unwrap();

        assert!(resp.contains("Queued") || resp.contains("Added"));
        assert_eq!(manager.get_current_track().unwrap().title, "Loading...");
        assert_eq!(manager.get_queue().len(), 1);
        assert_eq!(manager.get_queue()[0].requester, "UserB");
    }

    #[tokio::test]
    async fn test_skip_advances_queue() {
        let mut manager = setup();
        // Play song1
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "UserA".to_string(),
                USER_A,
            )
            .await
            .unwrap();
        // Queue song2
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song2".to_string(),
                "UserB".to_string(),
                USER_B,
            )
            .await
            .unwrap();

        let current_track = manager.get_current_track().unwrap();
        manager.skip(None).await.unwrap();

        assert_eq!(manager.get_history().len(), 1);
        assert_eq!(manager.get_history()[0].title, current_track.title);
        assert!(manager.get_queue().is_empty());
    }

    #[tokio::test]
    async fn test_repeat_mode_song() {
        let mut manager = setup();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "UserA".to_string(),
                USER_A,
            )
            .await
            .unwrap();
        manager.set_repeat_mode(RepeatMode::Song);

        let initial_track = manager.get_current_track().unwrap();
        manager.skip(None).await.unwrap();

        // Should repeat same track
        assert_eq!(manager.get_current_track().unwrap(), initial_track);
        assert!(manager.get_queue().is_empty());
    }

    #[tokio::test]
    async fn test_repeat_mode_queue() {
        let mut manager = setup();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "UserA".to_string(),
                USER_A,
            )
            .await
            .unwrap();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song2".to_string(),
                "UserB".to_string(),
                USER_B,
            )
            .await
            .unwrap();
        manager.set_repeat_mode(RepeatMode::Queue);

        let first_track = manager.get_current_track().unwrap();
        manager.skip(None).await.unwrap();

        // The first track should now be at the end of the queue
        let queue = manager.get_queue();
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0], first_track);
    }

    #[tokio::test]
    async fn test_shuffle_shuffles_queue() {
        let mut manager = setup();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "User".to_string(),
                USER,
            )
            .await
            .unwrap();
        for i in 2..10 {
            manager
                .play(
                    None,
                    ChannelId::new(1),
                    ChannelId::new(2),
                    format!("song{}", i),
                    "User".to_string(),
                    USER,
                )
                .await
                .unwrap();
        }

        let original_queue = manager.get_queue();
        manager.shuffle();
        let shuffled_queue = manager.get_queue();

        assert_eq!(original_queue.len(), shuffled_queue.len());
        // Shuffled queue might happen to be the same, but for 8 elements it's highly unlikely.
        // We assert they contain the same items.
        for item in &original_queue {
            assert!(shuffled_queue.contains(item));
        }
    }

    #[tokio::test]
    async fn test_idle_timer_tick_cleans_up_state() {
        let mut manager = setup();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "User".to_string(),
                USER,
            )
            .await
            .unwrap();

        // Skip so queue is empty
        manager.skip(None).await.unwrap();
        assert!(manager.idle_timer_active);

        let disconnected = manager.tick_idle_timer();
        assert!(disconnected);
        assert!(manager.get_current_track().is_none());
    }

    #[tokio::test]
    async fn test_user_leaving_voice_starts_timer_and_disconnects_if_not_returned() {
        let mut manager = setup();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "User".to_string(),
                USER,
            )
            .await
            .unwrap();

        manager.user_left_voice_channel();
        assert!(manager.leave_timer_active);

        let disconnected = manager.tick_leave_timer();
        assert!(disconnected);
    }

    #[tokio::test]
    async fn test_user_returning_before_timeout_cancels_leave_timer() {
        let mut manager = setup();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "User".to_string(),
                USER,
            )
            .await
            .unwrap();

        manager.user_left_voice_channel();
        assert!(manager.leave_timer_active);

        manager.user_returned_to_voice_channel();
        assert!(!manager.leave_timer_active);

        let disconnected = manager.tick_leave_timer();
        assert!(!disconnected);
    }

    // ── skip_next_by tests ──────────────────────────────────────────────────

    #[tokio::test]
    async fn test_skip_next_by_removes_first_matching_item() {
        let mut manager = setup();
        // current: UserA | queue: [UserB, UserA, UserC]
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "UserA".to_string(),
                USER_A,
            )
            .await
            .unwrap();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song2".to_string(),
                "UserB".to_string(),
                USER_B,
            )
            .await
            .unwrap();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song3".to_string(),
                "UserA".to_string(),
                USER_A,
            )
            .await
            .unwrap();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song4".to_string(),
                "UserC".to_string(),
                USER_C,
            )
            .await
            .unwrap();

        let skipped = manager.skip_next_by(USER_A);
        assert!(
            skipped.is_some(),
            "should return the title of the removed song"
        );

        // Queue should now have 2 items: [UserB, UserC] — first UserA item was removed
        let queue = manager.get_queue();
        assert_eq!(queue.len(), 2);
        assert_eq!(queue[0].requester, "UserB");
        assert_eq!(queue[1].requester, "UserC");
    }

    #[tokio::test]
    async fn test_skip_next_by_no_match_returns_none_and_queue_unchanged() {
        let mut manager = setup();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "UserA".to_string(),
                USER_A,
            )
            .await
            .unwrap();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song2".to_string(),
                "UserB".to_string(),
                USER_B,
            )
            .await
            .unwrap();

        let skipped = manager.skip_next_by(USER_C);
        assert!(skipped.is_none());
        assert_eq!(manager.get_queue().len(), 1);
    }

    // ── skip_last_by tests ──────────────────────────────────────────────────

    #[tokio::test]
    async fn test_skip_last_by_removes_last_matching_item() {
        let mut manager = setup();
        // current: UserA | queue: [UserA, UserB, UserA]
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "UserA".to_string(),
                USER_A,
            )
            .await
            .unwrap();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song2".to_string(),
                "UserA".to_string(),
                USER_A,
            )
            .await
            .unwrap();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song3".to_string(),
                "UserB".to_string(),
                USER_B,
            )
            .await
            .unwrap();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song4".to_string(),
                "UserA".to_string(),
                USER_A,
            )
            .await
            .unwrap();

        let skipped = manager.skip_last_by(USER_A);
        assert!(
            skipped.is_some(),
            "should return the title of the removed song"
        );

        // Queue should now have 2 items: [UserA, UserB] — last UserA item was removed
        let queue = manager.get_queue();
        assert_eq!(queue.len(), 2);
        assert_eq!(queue[0].requester, "UserA");
        assert_eq!(queue[1].requester, "UserB");
    }

    #[tokio::test]
    async fn test_skip_last_by_no_match_returns_none_and_queue_unchanged() {
        let mut manager = setup();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "UserA".to_string(),
                USER_A,
            )
            .await
            .unwrap();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song2".to_string(),
                "UserB".to_string(),
                USER_B,
            )
            .await
            .unwrap();

        let skipped = manager.skip_last_by(USER_C);
        assert!(skipped.is_none());
        assert_eq!(manager.get_queue().len(), 1);
    }

    // ── pause / resume tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_pause_sets_paused_state_and_preserves_queue() {
        let mut manager = setup();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "User".to_string(),
                USER,
            )
            .await
            .unwrap();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song2".to_string(),
                "User".to_string(),
                USER,
            )
            .await
            .unwrap();

        manager.pause().await.unwrap();

        assert!(
            manager.is_paused(),
            "manager should report paused after pause()"
        );
        assert!(
            manager.get_current_track().is_some(),
            "current track must be preserved"
        );
        assert_eq!(manager.get_queue().len(), 1, "queue must be preserved");
    }

    #[tokio::test]
    async fn test_resume_clears_paused_state() {
        let mut manager = setup();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "User".to_string(),
                USER,
            )
            .await
            .unwrap();

        manager.pause().await.unwrap();
        assert!(manager.is_paused());

        manager.resume().await.unwrap();
        assert!(
            !manager.is_paused(),
            "manager should not report paused after resume()"
        );
    }

    #[tokio::test]
    async fn test_update_track_metadata() {
        let mut manager = setup();
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "User".to_string(),
                USER,
            )
            .await
            .unwrap();

        let track = manager.get_current_track().unwrap();
        assert_eq!(track.title, "Loading...");
        let track_id = track.id;
        assert!(track_id > 0);

        // Update the metadata
        let updated = manager.update_track_metadata(
            track_id,
            "Updated Title".to_string(),
            Some(Duration::from_secs(200)),
            Some("http://thumb.url".to_string()),
        );

        assert!(
            updated,
            "update_track_metadata should return true for valid id"
        );
        let updated_track = manager.get_current_track().unwrap();
        assert_eq!(updated_track.title, "Updated Title");
        assert_eq!(updated_track.duration, Some(Duration::from_secs(200)));
        assert_eq!(
            updated_track.thumbnail_url,
            Some("http://thumb.url".to_string())
        );
    }

    #[tokio::test]
    async fn test_metadata_caching() {
        let mut manager = setup();

        // Play first track (starts with Loading...)
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "User".to_string(),
                USER,
            )
            .await
            .unwrap();

        let track = manager.get_current_track().unwrap();
        let track_id = track.id;

        // Update the metadata (simulating resolution)
        manager.update_track_metadata(
            track_id,
            "Real Song Title".to_string(),
            Some(Duration::from_secs(120)),
            Some("http://image.url".to_string()),
        );

        // Play the same song again
        manager
            .play(
                None,
                ChannelId::new(1),
                ChannelId::new(2),
                "song1".to_string(),
                "User".to_string(),
                USER,
            )
            .await
            .unwrap();

        // Verify that it immediately reuses the resolved metadata
        let queue = manager.get_queue();
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].title, "Real Song Title");
        assert_eq!(queue[0].duration, Some(Duration::from_secs(120)));
        assert_eq!(queue[0].thumbnail_url, Some("http://image.url".to_string()));
    }
}
