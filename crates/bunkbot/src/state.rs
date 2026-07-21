use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub struct FrequencyOverride {
    pub bot_name: String,
    pub original_frequency: u8,
    pub current_frequency: u8,
    pub set_at: DateTime<Utc>,
    pub set_by: String,
}

pub trait BotStateService: Send + Sync + std::fmt::Debug {
    fn enable_bot(&self, bot_name: &str);
    fn disable_bot(&self, bot_name: &str);
    fn is_bot_enabled(&self, bot_name: &str) -> bool;
    fn set_frequency(
        &self,
        bot_name: &str,
        frequency: u8,
        admin_user_id: &str,
        original_frequency: u8,
    );
    fn get_frequency(&self, bot_name: &str) -> Option<u8>;
    fn get_original_frequency(&self, bot_name: &str) -> Option<u8>;
    fn reset_frequency(&self, bot_name: &str) -> Option<u8>;
    fn get_all_states(&self) -> HashMap<String, bool>;
    fn get_all_frequencies(&self) -> HashMap<String, FrequencyOverride>;
    fn increment_trigger(&self, bot_name: &str);
    fn get_triggers_today(&self, bot_name: &str) -> u64;
    fn get_all_triggers_today(&self) -> HashMap<String, u64>;
}

#[allow(dead_code)]
#[derive(Debug, Default)]
/// In-memory state manager for reply bot toggle statuses and frequency overrides.
///
/// NOTE: Uses `std::sync::RwLock` because all operations are simple in-memory
/// synchronous map lookups/writes with tiny lock hold times that never block
/// Tokio worker threads or span `.await` boundaries.
pub struct InMemoryBotStateManager {
    states: RwLock<HashMap<String, bool>>,
    frequencies: RwLock<HashMap<String, FrequencyOverride>>,
    triggers: RwLock<HashMap<String, u64>>,
}

impl InMemoryBotStateManager {
    pub fn new() -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            frequencies: RwLock::new(HashMap::new()),
            triggers: RwLock::new(HashMap::new()),
        }
    }
}

impl BotStateService for InMemoryBotStateManager {
    fn enable_bot(&self, bot_name: &str) {
        let mut states = self.states.write().unwrap_or_else(|e| e.into_inner());
        states.insert(bot_name.to_string(), true);
    }

    fn disable_bot(&self, bot_name: &str) {
        let mut states = self.states.write().unwrap_or_else(|e| e.into_inner());
        states.insert(bot_name.to_string(), false);
    }

    fn is_bot_enabled(&self, bot_name: &str) -> bool {
        let states = self.states.read().unwrap_or_else(|e| e.into_inner());
        *states.get(bot_name).unwrap_or(&true)
    }

    fn set_frequency(
        &self,
        bot_name: &str,
        frequency: u8,
        admin_user_id: &str,
        original_frequency: u8,
    ) {
        let mut frequencies = self.frequencies.write().unwrap_or_else(|e| e.into_inner());
        frequencies.insert(
            bot_name.to_string(),
            FrequencyOverride {
                bot_name: bot_name.to_string(),
                original_frequency,
                current_frequency: frequency,
                set_at: Utc::now(),
                set_by: admin_user_id.to_string(),
            },
        );
    }

    fn get_frequency(&self, bot_name: &str) -> Option<u8> {
        let frequencies = self.frequencies.read().unwrap_or_else(|e| e.into_inner());
        frequencies.get(bot_name).map(|f| f.current_frequency)
    }

    fn get_original_frequency(&self, bot_name: &str) -> Option<u8> {
        let frequencies = self.frequencies.read().unwrap_or_else(|e| e.into_inner());
        frequencies.get(bot_name).map(|f| f.original_frequency)
    }

    fn reset_frequency(&self, bot_name: &str) -> Option<u8> {
        let mut frequencies = self.frequencies.write().unwrap_or_else(|e| e.into_inner());
        frequencies.remove(bot_name).map(|f| f.original_frequency)
    }

    fn get_all_states(&self) -> HashMap<String, bool> {
        self.states
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    fn get_all_frequencies(&self) -> HashMap<String, FrequencyOverride> {
        self.frequencies
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    fn increment_trigger(&self, bot_name: &str) {
        let mut triggers = self.triggers.write().unwrap_or_else(|e| e.into_inner());
        let count = triggers.entry(bot_name.to_string()).or_insert(0);
        *count += 1;
    }

    fn get_triggers_today(&self, bot_name: &str) -> u64 {
        let triggers = self.triggers.read().unwrap_or_else(|e| e.into_inner());
        *triggers.get(bot_name).unwrap_or(&0)
    }

    fn get_all_triggers_today(&self) -> HashMap<String, u64> {
        self.triggers
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enable_disable_bot() {
        let manager = InMemoryBotStateManager::new();
        let bot = "test_bot";

        // Defaults to enabled
        assert!(manager.is_bot_enabled(bot));

        // Disable it
        manager.disable_bot(bot);
        assert!(!manager.is_bot_enabled(bot));
        assert_eq!(manager.get_all_states().get(bot), Some(&false));

        // Enable it again
        manager.enable_bot(bot);
        assert!(manager.is_bot_enabled(bot));
        assert_eq!(manager.get_all_states().get(bot), Some(&true));
    }

    #[test]
    fn test_frequency_override() {
        let manager = InMemoryBotStateManager::new();
        let bot = "test_bot";

        // No override initially
        assert_eq!(manager.get_frequency(bot), None);

        // Set override
        let start_time = Utc::now();
        manager.set_frequency(bot, 50, "admin123", 100);
        assert_eq!(manager.get_frequency(bot), Some(50));
        assert_eq!(manager.get_original_frequency(bot), Some(100));

        let all_freqs = manager.get_all_frequencies();
        let ovr = all_freqs.get(bot).expect("override exists");
        assert_eq!(ovr.bot_name, bot);
        assert_eq!(ovr.current_frequency, 50);
        assert_eq!(ovr.original_frequency, 100);
        assert_eq!(ovr.set_by, "admin123");
        assert!(ovr.set_at >= start_time);

        // Reset override
        let orig = manager.reset_frequency(bot);
        assert_eq!(orig, Some(100));
        assert_eq!(manager.get_frequency(bot), None);
    }

    #[test]
    fn test_reset_nonexistent_override() {
        let manager = InMemoryBotStateManager::new();
        assert_eq!(manager.reset_frequency("nonexistent"), None);
    }

    #[test]
    fn test_increment_triggers() {
        let manager = InMemoryBotStateManager::new();
        assert_eq!(manager.get_triggers_today("bot1"), 0);
        manager.increment_trigger("bot1");
        assert_eq!(manager.get_triggers_today("bot1"), 1);
        manager.increment_trigger("bot1");
        assert_eq!(manager.get_triggers_today("bot1"), 2);

        let all = manager.get_all_triggers_today();
        assert_eq!(all.get("bot1"), Some(&2));
        assert_eq!(all.get("bot2"), None);
    }
}
