use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SocialBatteryConfig {
    pub max: i32,
    pub starting_value: i32,
    pub depletion_rate: i32,
    pub recharge_rate: i32,
    pub recharge_interval_minutes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Profile {
    pub name_aliases: Vec<String>,
    pub system_prompt: String,
    pub speech_patterns: Vec<String>,
    pub topic_affinities: Vec<String>,
    pub self_facts: Vec<String>,
    pub relationships: HashMap<String, String>,
    pub social_battery_config: SocialBatteryConfig,
}

impl Profile {
    pub fn load(yaml_content: &str) -> Result<Self> {
        let profile = serde_yaml::from_str(yaml_content)?;
        Ok(profile)
    }

    pub fn merge(&mut self, other: Profile) {
        self.name_aliases.extend(other.name_aliases);
        if !other.system_prompt.is_empty() {
            if !self.system_prompt.is_empty() {
                self.system_prompt.push_str("\n\n");
            }
            self.system_prompt.push_str(&other.system_prompt);
        }
        self.speech_patterns.extend(other.speech_patterns);
        self.topic_affinities.extend(other.topic_affinities);
        self.self_facts.extend(other.self_facts);
        self.relationships.extend(other.relationships);

        // Overwrite social battery config if provided (assuming max > 0 means it's non-default)
        if other.social_battery_config.max > 0 {
            self.social_battery_config = other.social_battery_config;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_profile() {
        let yaml = r#"
name_aliases: ["Cova", "covabot"]
system_prompt: "You are a friendly bot."
speech_patterns: ["Use emoji"]
topic_affinities: ["Cheeseburgers"]
self_facts: ["Created in 2024"]
relationships:
  "1234": "Is your best friend"
social_battery_config:
  max: 100
  starting_value: 80
  depletion_rate: 10
  recharge_rate: 5
  recharge_interval_minutes: 5
        "#;

        let profile = Profile::load(yaml).unwrap();
        assert_eq!(profile.name_aliases.len(), 2);
        assert_eq!(profile.topic_affinities[0], "Cheeseburgers");
        assert_eq!(
            profile.relationships.get("1234").unwrap(),
            "Is your best friend"
        );
        assert_eq!(profile.social_battery_config.max, 100);
    }
}
