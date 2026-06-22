use crate::personality::SocialBatteryConfig;
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

#[derive(Clone)]
struct ChannelState {
    muted: bool,
    dampened: bool,
    last_spoke_at: Option<Instant>,
    battery: i32,
    last_battery_update: Instant,
}

impl ChannelState {
    fn new(starting_value: i32, config: &SocialBatteryConfig) -> Self {
        Self {
            muted: false,
            dampened: false,
            last_spoke_at: None,
            battery: starting_value.min(config.max),
            last_battery_update: Instant::now(),
        }
    }

    fn recharge(&mut self, config: &SocialBatteryConfig) {
        if config.recharge_rate <= 0 || config.recharge_interval_minutes <= 0 {
            return;
        }
        let elapsed_secs = self.last_battery_update.elapsed().as_secs();
        let interval_secs = (config.recharge_interval_minutes as u64) * 60;
        let intervals = elapsed_secs / interval_secs;

        if intervals > 0 {
            let add = (intervals as i32) * config.recharge_rate;
            self.battery = (self.battery + add).min(config.max);
            self.last_battery_update += Duration::from_secs(intervals * interval_secs);
        }
    }
}

/// Tracks engagement state per channel and decides if CovaBot should respond.
pub struct Manager {
    states: Mutex<HashMap<String, ChannelState>>,
    topic_affinities: Vec<String>,
    battery_config: SocialBatteryConfig,
}

impl Manager {
    pub fn new(battery_config: SocialBatteryConfig) -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
            topic_affinities: vec![],
            battery_config,
        }
    }

    pub fn with_affinities(mut self, affinities: Vec<String>) -> Self {
        self.topic_affinities = affinities;
        self
    }

    pub fn deplete(&self, channel_id: &str) {
        let mut states = self.states.lock().expect("mutex poisoned");
        let state = states.entry(channel_id.to_string()).or_insert_with(|| {
            ChannelState::new(self.battery_config.starting_value, &self.battery_config)
        });
        state.recharge(&self.battery_config);
        state.battery -= self.battery_config.depletion_rate;
        if state.battery < 0 {
            state.battery = 0;
        }
    }

    pub fn should_respond(&self, input: &MessageInput) -> EngagementResult {
        let states = self.states.lock().expect("engagement mutex poisoned");

        let (muted, dampened, recently_spoke, battery_low) =
            if let Some(state) = states.get(&input.channel_id) {
                let mut virtual_state = state.clone();
                virtual_state.recharge(&self.battery_config);
                let recently_spoke = virtual_state
                    .last_spoke_at
                    .map(|t| t.elapsed() < RECENT_SPEAK_WINDOW)
                    .unwrap_or(false);
                let threshold = (self.battery_config.max as f32 * 0.20) as i32;
                (
                    virtual_state.muted,
                    virtual_state.dampened,
                    recently_spoke,
                    virtual_state.battery <= threshold,
                )
            } else {
                let threshold = (self.battery_config.max as f32 * 0.20) as i32;
                let battery = self
                    .battery_config
                    .starting_value
                    .min(self.battery_config.max);
                (false, false, false, battery <= threshold)
            };

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
        let matches_affinity = input.topical_tags.iter().any(|t| {
            self.topic_affinities
                .iter()
                .any(|a| a.eq_ignore_ascii_case(t))
        });
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
        let state = states.entry(channel_id.to_string()).or_insert_with(|| {
            ChannelState::new(self.battery_config.starting_value, &self.battery_config)
        });
        state.last_spoke_at = Some(Instant::now());
    }

    /// Temporarily raise the pull floor; silences non-directed responses.
    pub fn dampen(&self, channel_id: &str) {
        let mut states = self.states.lock().expect("engagement mutex poisoned");
        let state = states.entry(channel_id.to_string()).or_insert_with(|| {
            ChannelState::new(self.battery_config.starting_value, &self.battery_config)
        });
        state.dampened = true;
    }

    pub fn set_dampen(&self, channel_id: &str, dampened: bool) {
        let mut states = self.states.lock().expect("engagement mutex poisoned");
        let state = states.entry(channel_id.to_string()).or_insert_with(|| {
            ChannelState::new(self.battery_config.starting_value, &self.battery_config)
        });
        state.dampened = dampened;
    }

    /// Apply a hard floor. Only direct addresses pass through when muted.
    pub fn set_mute(&self, channel_id: &str, muted: bool) {
        let mut states = self.states.lock().expect("engagement mutex poisoned");
        let state = states.entry(channel_id.to_string()).or_insert_with(|| {
            ChannelState::new(self.battery_config.starting_value, &self.battery_config)
        });
        state.muted = muted;
    }

    // For testing
    #[cfg(test)]
    pub fn get_battery(&self, channel_id: &str) -> i32 {
        let mut states = self.states.lock().expect("mutex poisoned");
        let state = states.entry(channel_id.to_string()).or_insert_with(|| {
            ChannelState::new(self.battery_config.starting_value, &self.battery_config)
        });
        state.recharge(&self.battery_config);
        state.battery
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> SocialBatteryConfig {
        SocialBatteryConfig {
            max: 100,
            starting_value: 100,
            depletion_rate: 10,
            recharge_rate: 5,
            recharge_interval_minutes: 5,
        }
    }

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
        let mgr = Manager::new(default_config());
        let mut i = input("ch1");
        i.is_mentioned = true;
        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::DirectMention));
        assert_eq!(result.energy, Some(GateEnergy::Invested));
    }

    #[test]
    fn direct_mention_overrides_mute() {
        let mgr = Manager::new(default_config());
        mgr.set_mute("ch1", true);
        let mut i = input("ch1");
        i.is_mentioned = true;
        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::DirectMention));
    }

    #[test]
    fn direct_mention_overrides_dampener() {
        let mgr = Manager::new(default_config());
        mgr.set_dampen("ch1", true);
        let mut i = input("ch1");
        i.is_mentioned = true;
        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::DirectMention));
    }

    #[test]
    fn reply_to_cova_responds() {
        let mgr = Manager::new(default_config());
        let mut i = input("ch1");
        i.is_reply_to_me = true;
        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::ReplyToCova));
        assert_eq!(result.energy, Some(GateEnergy::Normal));
    }

    #[test]
    fn reply_overrides_dampener() {
        let mgr = Manager::new(default_config());
        mgr.set_dampen("ch1", true);
        let mut i = input("ch1");
        i.is_reply_to_me = true;
        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::ReplyToCova));
    }

    #[test]
    fn reply_blocked_by_mute() {
        let mgr = Manager::new(default_config());
        mgr.set_mute("ch1", true);
        let mut i = input("ch1");
        i.is_reply_to_me = true;
        let result = mgr.should_respond(&i);
        assert!(!result.respond);
    }

    #[test]
    fn engagement_continuity_after_cova_speaks() {
        let mgr = Manager::new(default_config());
        mgr.record_cova_speak("ch1");
        let result = mgr.should_respond(&input("ch1"));
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::EngagementContinuity));
    }

    #[test]
    fn no_response_without_continuity() {
        let mgr = Manager::new(default_config());
        let result = mgr.should_respond(&input("ch1"));
        assert!(!result.respond);
    }

    #[test]
    fn dampener_suppresses_continuity() {
        let mgr = Manager::new(default_config());
        mgr.record_cova_speak("ch1");
        mgr.set_dampen("ch1", true);
        let result = mgr.should_respond(&input("ch1"));
        assert!(!result.respond);
    }

    #[test]
    fn addressee_self_triggers_reply() {
        let mgr = Manager::new(default_config());
        let mut i = input("ch1");
        i.is_addressee_self = true;
        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::ReplyToCova));
    }

    #[test]
    fn no_crossover_between_channels() {
        let mgr = Manager::new(default_config());
        mgr.record_cova_speak("ch1");
        // ch2 has no activity — should not respond
        let result = mgr.should_respond(&input("ch2"));
        assert!(!result.respond);
    }

    #[test]
    fn topic_affinity_pulls_response() {
        let mgr = Manager::new(default_config()).with_affinities(vec!["Cheeseburgers".to_string()]);
        let mut i = input("ch1");
        i.topical_tags = vec!["Cheeseburgers".to_string()];

        let result = mgr.should_respond(&i);
        assert!(result.respond);
        assert_eq!(result.reason, Some(GateReason::TopicAffinity));
    }

    #[test]
    fn low_social_battery_dampens_ambient_responses() {
        let mgr = Manager::new(default_config());
        for _ in 0..10 {
            mgr.deplete("ch1"); // Depletes by 10 each time, down to 0
        }
        let i = input("ch1"); // ambient
        let result = mgr.should_respond(&i);
        assert!(!result.respond);
    }

    #[test]
    fn recharge_works_over_time() {
        let config = default_config();
        let mut state = ChannelState::new(0, &config); // Start at 0
                                                       // Shift time back by 11 minutes
        state.last_battery_update -= Duration::from_secs(11 * 60);
        state.recharge(&config);

        // 2 intervals of 5 minutes = 10 minutes.
        // Recharge is 5 per interval, so 2 * 5 = 10.
        assert_eq!(state.battery, 10);

        // last_battery_update should be shifted forward by 10 minutes (leaving 1 minute remainder).
        let elapsed = state.last_battery_update.elapsed().as_secs();
        assert!((60..120).contains(&elapsed));
    }
}
