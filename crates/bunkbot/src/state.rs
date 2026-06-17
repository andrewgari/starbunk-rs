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
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct InMemoryBotStateManager {
    states: RwLock<HashMap<String, bool>>,
    frequencies: RwLock<HashMap<String, FrequencyOverride>>,
}

impl InMemoryBotStateManager {
    pub fn new() -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            frequencies: RwLock::new(HashMap::new()),
        }
    }
}

impl BotStateService for InMemoryBotStateManager {
    fn enable_bot(&self, _bot_name: &str) {
        // Stub for TDD PR 1
    }

    fn disable_bot(&self, _bot_name: &str) {
        // Stub for TDD PR 1
    }

    fn is_bot_enabled(&self, _bot_name: &str) -> bool {
        // Stub for TDD PR 1: always defaults to true
        true
    }

    fn set_frequency(
        &self,
        _bot_name: &str,
        _frequency: u8,
        _admin_user_id: &str,
        _original_frequency: u8,
    ) {
        // Stub for TDD PR 1
    }

    fn get_frequency(&self, _bot_name: &str) -> Option<u8> {
        // Stub for TDD PR 1
        None
    }

    fn get_original_frequency(&self, _bot_name: &str) -> Option<u8> {
        // Stub for TDD PR 1
        None
    }

    fn reset_frequency(&self, _bot_name: &str) -> Option<u8> {
        // Stub for TDD PR 1
        None
    }

    fn get_all_states(&self) -> HashMap<String, bool> {
        // Stub for TDD PR 1
        HashMap::new()
    }

    fn get_all_frequencies(&self) -> HashMap<String, FrequencyOverride> {
        // Stub for TDD PR 1
        HashMap::new()
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
}
