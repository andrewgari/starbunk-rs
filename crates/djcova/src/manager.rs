use crate::gif_client::GifService;
use crate::voice::VoiceService;
use serenity::all::{ChannelId, GuildId, MessageId};
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
    pub title: String,
    pub url: String,
    pub requester: String,
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
    voice: Arc<dyn VoiceService>,
    gif: Arc<dyn GifService>,
    pub idle_timer_active: bool,
    pub leave_timer_active: bool,
    gif_task: Option<tokio::task::JoinHandle<()>>,
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
            voice,
            gif,
            idle_timer_active: false,
            leave_timer_active: false,
            gif_task: None,
        }
    }

    pub async fn play(
        &mut self,
        http: Option<Arc<serenity::all::Http>>,
        text_channel: ChannelId,
        voice_channel: ChannelId,
        input: String,
        requester: String,
    ) -> anyhow::Result<String> {
        self.text_channel_id = Some(text_channel);
        self.voice_channel_id = Some(voice_channel);

        if self.current_track.is_none() {
            // Join voice then resolve metadata + start playback in a single yt-dlp invocation.
            self.voice.join(self.guild_id, voice_channel).await?;
            let info = self.voice.play(self.guild_id, &input).await?;
            let item = QueueItem {
                title: info.title,
                url: input,
                requester,
                duration: info.duration,
                thumbnail_url: info.thumbnail_url,
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
            Ok(format!(
                "Now playing: {} requested by {}",
                item.title, item.requester
            ))
        } else {
            // Resolve metadata only — playback starts when this item reaches the front of the queue.
            let resolved = self.voice.resolve_metadata(&input).await?;
            let item = QueueItem {
                title: resolved.title,
                url: input,
                requester,
                duration: resolved.duration,
                thumbnail_url: resolved.thumbnail_url,
            };
            self.queue.push_back(item.clone());
            // Restart the GIF loop so it posts to the new text channel.
            if let Some(h) = http {
                self.stop_gif_loop();
                self.start_gif_loop(h, text_channel);
            }
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
            self.voice.play(self.guild_id, &track.url).await?; // metadata return ignored; track info already known
            let _ = self
                .voice
                .set_volume(self.guild_id, self.volume as f32 / 100.0)
                .await;
            if let Some(h) = http {
                if let Some(tc) = self.text_channel_id {
                    self.start_gif_loop(h, tc);
                }
            }
            Ok(format!("Skipped to: {}", track.title))
        } else {
            self.voice.stop(self.guild_id).await?;
            self.stop_gif_loop();
            self.idle_timer_active = true;
            Ok("Playback stopped, queue is empty.".to_string())
        }
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        self.voice.leave(self.guild_id).await?;
        self.queue.clear();
        self.current_track = None;
        self.voice_channel_id = None;
        self.text_channel_id = None;
        self.idle_timer_active = false;
        self.leave_timer_active = false;
        self.stop_gif_loop();
        Ok(())
    }

    pub fn get_queue(&self) -> Vec<QueueItem> {
        self.queue.iter().cloned().collect()
    }

    pub fn get_history(&self) -> Vec<QueueItem> {
        self.history.clone()
    }

    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let mut vec: Vec<QueueItem> = self.queue.drain(..).collect();
        vec.shuffle(&mut rng);
        self.queue = vec.into();
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume;
        let vol_float = volume as f32 / 100.0;
        let voice = self.voice.clone();
        let guild_id = self.guild_id;
        tokio::spawn(async move {
            let _ = voice.set_volume(guild_id, vol_float).await;
        });
    }

    pub fn get_volume(&self) -> u8 {
        self.volume
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
    }

    pub fn get_repeat_mode(&self) -> RepeatMode {
        self.repeat_mode
    }

    pub fn get_current_track(&self) -> Option<QueueItem> {
        self.current_track.clone()
    }

    pub fn tick_idle_timer(&mut self) -> bool {
        if self.idle_timer_active {
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
        self.leave_timer_active = true;
    }

    pub fn user_returned_to_voice_channel(&mut self) {
        self.leave_timer_active = false;
    }

    fn start_gif_loop(&mut self, http: Arc<serenity::all::Http>, text_channel: ChannelId) {
        if self.gif_task.as_ref().is_some_and(|h| !h.is_finished()) {
            return;
        }

        let gif = self.gif.clone();
        let mut last_msg_id: Option<MessageId> = None;

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
                        tracing::warn!("Failed to fetch dancing gif: {}", e);
                        continue;
                    }
                };

                let messages = match text_channel
                    .messages(&http, serenity::all::GetMessages::new().limit(1))
                    .await
                {
                    Ok(msgs) => msgs,
                    Err(e) => {
                        tracing::warn!("Failed to retrieve channel messages for gif check: {}", e);
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
                            "Failed to edit gif message in place: {}, posting new one",
                            e
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
                self.voice.play(self.guild_id, &track.url).await?;
                let _ = self
                    .voice
                    .set_volume(self.guild_id, self.volume as f32 / 100.0)
                    .await;
                Ok(format!("Restarted: {}", track.title))
            }
            None => anyhow::bail!("Nothing is currently playing."),
        }
    }

    fn stop_gif_loop(&mut self) {
        if let Some(handle) = self.gif_task.take() {
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
        async fn play(&self, _guild_id: GuildId, _track_url: &str) -> anyhow::Result<TrackInfo> {
            Ok(TrackInfo {
                title: "Stub Track".to_string(),
                duration: Some(Duration::from_secs(180)),
                thumbnail_url: None,
            })
        }
        async fn stop(&self, _guild_id: GuildId) -> anyhow::Result<()> {
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
            )
            .await
            .unwrap();

        assert!(resp.contains("Queued") || resp.contains("Added"));
        assert_eq!(manager.get_current_track().unwrap().title, "Stub Track"); // resolved from MockVoiceService stub
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
}
