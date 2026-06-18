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
    pub topical_tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateReason {
    DirectMention,
    ReplyToCova,
    EngagementContinuity,
    TopicAffinity,
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

#[derive(Default)]
struct ChannelState {
    muted: bool,
    dampened: bool,
    last_spoke_at: Option<Instant>,
}

/// Tracks engagement state per channel and decides if CovaBot should respond.
pub struct Manager {
    states: Mutex<HashMap<String, ChannelState>>,
    topic_affinities: Vec<String>,
    battery: Mutex<i32>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
            topic_affinities: vec![],
            battery: Mutex::new(100),
        }
    }

    pub fn with_affinities(mut self, affinities: Vec<String>) -> Self {
        self.topic_affinities = affinities;
        self
    }

    pub fn deplete_battery(&self, amount: i32) {
        let mut b = self.battery.lock().expect("mutex poisoned");
        *b -= amount;
        if *b < 0 {
            *b = 0;
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

        let battery_level = *self.battery.lock().expect("mutex poisoned");
        let battery_low = battery_level <= 20;

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
            return EngagementResult {
                respond: false,
                reason: None,
                energy: None,
            };
        }

        // 3. Direct Reply or Addressee==Self — high pull, clears dampener and battery limits.
        if input.is_reply_to_me || input.is_addressee_self {
            return EngagementResult {
                respond: true,
                reason: Some(GateReason::ReplyToCova),
                energy: Some(GateEnergy::Normal),
            };
        }

        // 4. Dampener or Low Battery stops ambient/continuity responses.
        if dampened || battery_low {
            return EngagementResult {
                respond: false,
                reason: None,
                energy: None,
            };
        }

        // 5. Topic Affinity - ambient pull if a topic matches
        let matches_affinity = input
            .topical_tags
            .iter()
            .any(|t| self.topic_affinities.contains(t));
        if matches_affinity {
            return EngagementResult {
                respond: true,
                reason: Some(GateReason::TopicAffinity),
                energy: Some(GateEnergy::Invested),
            };
        }

        // 6. Engagement continuity — active for RECENT_SPEAK_WINDOW after Cova last spoke.
        if recently_spoke {
            return EngagementResult {
                respond: true,
                reason: Some(GateReason::EngagementContinuity),
                energy: Some(GateEnergy::Normal),
            };
        }

        EngagementResult {
            respond: false,
            reason: None,
            energy: None,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn input(channel_id: &str) -> MessageInput {
        MessageInput {
            channel_id: channel_id.to_string(),
            author_id: "user1".to_string(),
            is_mentioned: false,
            is_reply_to_me: false,
            is_addressee_self: false,
            topical_tags: vec![],
        }
    }

    #[test]
    fn direct_mention_always_responds() {
        let mgr = Manager::new();
        let mut i = input("ch1");
        i.is_mentioned = true;
        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::DirectMention));
        assert_eq!(result.energy, Some(GateEnergy::Invested));
    }

    #[test]
    fn direct_mention_overrides_mute() {
        let mgr = Manager::new();
        mgr.set_mute("ch1", true);
        let mut i = input("ch1");
        i.is_mentioned = true;
        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::DirectMention));
    }

    #[test]
    fn direct_mention_overrides_dampener() {
        let mgr = Manager::new();
        mgr.set_dampen("ch1", true);
        let mut i = input("ch1");
        i.is_mentioned = true;
        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::DirectMention));
    }

    #[test]
    fn reply_to_cova_responds() {
        let mgr = Manager::new();
        let mut i = input("ch1");
        i.is_reply_to_me = true;
        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::ReplyToCova));
        assert_eq!(result.energy, Some(GateEnergy::Normal));
    }

    #[test]
    fn reply_overrides_dampener() {
        let mgr = Manager::new();
        mgr.set_dampen("ch1", true);
        let mut i = input("ch1");
        i.is_reply_to_me = true;
        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::ReplyToCova));
    }

    #[test]
    fn reply_blocked_by_mute() {
        let mgr = Manager::new();
        mgr.set_mute("ch1", true);
        let mut i = input("ch1");
        i.is_reply_to_me = true;
        let result = mgr.should_respond(&i);
        assert!(!result.respond);
    }

    #[test]
    fn engagement_continuity_after_cova_speaks() {
        let mgr = Manager::new();
        mgr.record_cova_speak("ch1");
        let result = mgr.should_respond(&input("ch1"));
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::EngagementContinuity));
    }

    #[test]
    fn no_response_without_continuity() {
        let mgr = Manager::new();
        let result = mgr.should_respond(&input("ch1"));
        assert!(!result.respond);
    }

    #[test]
    fn dampener_suppresses_continuity() {
        let mgr = Manager::new();
        mgr.record_cova_speak("ch1");
        mgr.set_dampen("ch1", true);
        let result = mgr.should_respond(&input("ch1"));
        assert!(!result.respond);
    }

    #[test]
    fn addressee_self_triggers_reply() {
        let mgr = Manager::new();
        let mut i = input("ch1");
        i.is_addressee_self = true;
        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::ReplyToCova));
    }

    #[test]
    fn no_crossover_between_channels() {
        let mgr = Manager::new();
        mgr.record_cova_speak("ch1");
        // ch2 has no activity — should not respond
        let result = mgr.should_respond(&input("ch2"));
        assert!(!result.respond);
    }

    #[test]
    fn topic_affinity_pulls_response() {
        let mgr = Manager::new().with_affinities(vec!["Cheeseburgers".to_string()]);
        let mut i = input("ch1");
        i.topical_tags = vec!["Cheeseburgers".to_string()];

        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::TopicAffinity));
    }

    #[test]
    fn low_social_battery_dampens_ambient_responses() {
        let mgr = Manager::new();
        mgr.deplete_battery(100);
        let i = input("ch1"); // ambient
        let result = mgr.should_respond(&i);
        assert!(!result.respond);
    }
}
