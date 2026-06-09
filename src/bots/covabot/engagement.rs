use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How long engagement continuity stays active after Cova speaks.
const RECENT_SPEAK_WINDOW: Duration = Duration::from_secs(5 * 60);

pub struct MessageInput {
    pub channel_id: String,
    pub author_id: String,
    pub is_mentioned: bool,
    pub is_reply_to_me: bool,
    pub is_addressee_self: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateReason {
    DirectMention,
    ReplyToCova,
    EngagementContinuity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateEnergy {
    QuickJab,
    Normal,
    Invested,
}

#[derive(Debug, Clone)]
pub struct EngagementResult {
    pub respond: bool,
    pub reason: Option<GateReason>,
    pub energy: Option<GateEnergy>,
}

struct ChannelState {
    muted: bool,
    dampened: bool,
    last_spoke_at: Option<Instant>,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            muted: false,
            dampened: false,
            last_spoke_at: None,
        }
    }
}

/// Tracks engagement state per channel and decides if CovaBot should respond.
pub struct Manager {
    states: Mutex<HashMap<String, ChannelState>>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }

    pub fn should_respond(&self, input: &MessageInput) -> EngagementResult {
        let states = self.states.lock().expect("engagement mutex poisoned");
        let state = states.get(&input.channel_id);

        let muted = state.map(|s| s.muted).unwrap_or(false);
        let dampened = state.map(|s| s.dampened).unwrap_or(false);
        let recently_spoke = state
            .and_then(|s| s.last_spoke_at)
            .map(|t| t.elapsed() < RECENT_SPEAK_WINDOW)
            .unwrap_or(false);

        // 1. Direct Mention — highest pull, clears all restraints.
        if input.is_mentioned {
            return EngagementResult {
                respond: true,
                reason: Some(GateReason::DirectMention),
                energy: Some(GateEnergy::Invested),
            };
        }

        // 2. Mute stops everything except direct mention.
        if muted {
            return EngagementResult { respond: false, reason: None, energy: None };
        }

        // 3. Direct Reply or Addressee==Self — high pull, clears dampener.
        if input.is_reply_to_me || input.is_addressee_self {
            return EngagementResult {
                respond: true,
                reason: Some(GateReason::ReplyToCova),
                energy: Some(GateEnergy::Normal),
            };
        }

        // 4. Dampener stops ambient/continuity responses.
        if dampened {
            return EngagementResult { respond: false, reason: None, energy: None };
        }

        // 5. Engagement continuity — active for RECENT_SPEAK_WINDOW after Cova last spoke.
        if recently_spoke {
            return EngagementResult {
                respond: true,
                reason: Some(GateReason::EngagementContinuity),
                energy: Some(GateEnergy::Normal),
            };
        }

        EngagementResult { respond: false, reason: None, energy: None }
    }

    /// Record that CovaBot just spoke in `channel_id`.
    pub fn record_cova_speak(&self, channel_id: &str) {
        let mut states = self.states.lock().expect("engagement mutex poisoned");
        let state = states.entry(channel_id.to_string()).or_default();
        state.last_spoke_at = Some(Instant::now());
    }

    /// Temporarily raise the pull floor; silences non-directed responses.
    pub fn dampen(&self, channel_id: &str) {
        let mut states = self.states.lock().expect("engagement mutex poisoned");
        let state = states.entry(channel_id.to_string()).or_default();
        state.dampened = true;
    }

    pub fn set_dampen(&self, channel_id: &str, dampened: bool) {
        let mut states = self.states.lock().expect("engagement mutex poisoned");
        let state = states.entry(channel_id.to_string()).or_default();
        state.dampened = dampened;
    }

    /// Apply a hard floor. Only direct addresses pass through when muted.
    pub fn set_mute(&self, channel_id: &str, muted: bool) {
        let mut states = self.states.lock().expect("engagement mutex poisoned");
        let state = states.entry(channel_id.to_string()).or_default();
        state.muted = muted;
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}
