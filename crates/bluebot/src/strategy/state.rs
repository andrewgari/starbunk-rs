use chrono::{DateTime, Utc};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared state for BlueBot strategies to track reply windows and cooldowns.
#[derive(Debug, Clone)]
pub struct SharedState {
    pub last_blue_response: Option<DateTime<Utc>>,
    pub last_murder_response: Option<DateTime<Utc>>,
    pub reply_window_ms: i64,
    pub murder_window_ms: i64,
    pub enemy_user_id: u64,
}

impl SharedState {
    pub fn new() -> Arc<RwLock<Self>> {
        let reply_window_ms = env::var("BLUEBOT_REPLY_WINDOW_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5 * 60 * 1000); // 5 mins

        let murder_window_ms = env::var("BLUEBOT_MURDER_WINDOW_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24 * 60 * 60 * 1000); // 24 hrs

        let enemy_user_id = env::var("BLUEBOT_ENEMY_USER_ID")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        Arc::new(RwLock::new(Self {
            last_blue_response: None,
            last_murder_response: None,
            reply_window_ms,
            murder_window_ms,
            enemy_user_id,
        }))
    }

    pub fn open_reply_window(&mut self, now: DateTime<Utc>) {
        self.last_blue_response = Some(now);
    }

    pub fn clear_reply_window(&mut self) {
        self.last_blue_response = None;
    }

    pub fn open_murder_window(&mut self, now: DateTime<Utc>) {
        self.last_murder_response = Some(now);
    }

    #[allow(dead_code)]
    pub fn clear_murder_window(&mut self) {
        self.last_murder_response = None;
    }

    pub fn is_within_reply_window(&self, now: DateTime<Utc>) -> bool {
        if let Some(last) = self.last_blue_response {
            let diff = now.signed_duration_since(last).num_milliseconds();
            diff < self.reply_window_ms && diff >= 0
        } else {
            false
        }
    }

    pub fn is_within_murder_window(&self, now: DateTime<Utc>) -> bool {
        if let Some(last) = self.last_murder_response {
            let diff = now.signed_duration_since(last).num_milliseconds();
            diff < self.murder_window_ms && diff >= 0
        } else {
            false
        }
    }
}
