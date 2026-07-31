use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::Row;
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CovabotPersonalityYml {
    pub identity: String,
    pub speech_patterns: Vec<String>,
    pub affinities: Vec<String>,
    pub relationships: HashMap<String, String>,
}

pub struct PersonalityStore {
    pool: sqlx::PgPool,
}

impl PersonalityStore {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS covabot_personality (
                id INTEGER PRIMARY KEY DEFAULT 1,
                identity TEXT NOT NULL,
                speech_patterns JSONB NOT NULL DEFAULT '[]'::jsonb,
                affinities JSONB NOT NULL DEFAULT '[]'::jsonb,
                relationships JSONB NOT NULL DEFAULT '{}'::jsonb
            )"#,
        )
        .execute(&self.pool)
        .await?;

        // Ensure there is a row
        sqlx::query(
            r#"INSERT INTO covabot_personality (id, identity) VALUES (1, '') ON CONFLICT DO NOTHING"#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_personality(&self) -> Result<CovabotPersonalityYml> {
        let row = sqlx::query("SELECT identity, speech_patterns, affinities, relationships FROM covabot_personality WHERE id = 1")
            .fetch_one(&self.pool)
            .await?;

        let identity: String = row.try_get("identity")?;
        let speech_patterns: serde_json::Value = row.try_get("speech_patterns")?;
        let affinities: serde_json::Value = row.try_get("affinities")?;
        let relationships: serde_json::Value = row.try_get("relationships")?;

        Ok(CovabotPersonalityYml {
            identity,
            speech_patterns: serde_json::from_value(speech_patterns)?,
            affinities: serde_json::from_value(affinities)?,
            relationships: serde_json::from_value(relationships)?,
        })
    }

    pub async fn update_personality(&self, personality: &CovabotPersonalityYml) -> Result<()> {
        let sp = serde_json::to_value(&personality.speech_patterns)?;
        let aff = serde_json::to_value(&personality.affinities)?;
        let rel = serde_json::to_value(&personality.relationships)?;

        sqlx::query(
            "UPDATE covabot_personality SET identity = $1, speech_patterns = $2, affinities = $3, relationships = $4 WHERE id = 1"
        )
        .bind(&personality.identity)
        .bind(sp)
        .bind(aff)
        .bind(rel)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_identity(&self, identity: &str) -> Result<()> {
        sqlx::query("UPDATE covabot_personality SET identity = $1 WHERE id = 1")
            .bind(identity)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn add_speech_pattern(&self, pattern: &str) -> Result<()> {
        sqlx::query("UPDATE covabot_personality SET speech_patterns = speech_patterns || $1::jsonb WHERE id = 1")
            .bind(serde_json::to_value(vec![pattern])?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn remove_speech_pattern(&self, pattern: &str) -> Result<()> {
        sqlx::query(
            "UPDATE covabot_personality SET speech_patterns = speech_patterns - $1 WHERE id = 1",
        )
        .bind(pattern)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn add_affinity(&self, affinity: &str) -> Result<()> {
        sqlx::query(
            "UPDATE covabot_personality SET affinities = affinities || $1::jsonb WHERE id = 1",
        )
        .bind(serde_json::to_value(vec![affinity])?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_affinity(&self, affinity: &str) -> Result<()> {
        sqlx::query("UPDATE covabot_personality SET affinities = affinities - $1 WHERE id = 1")
            .bind(affinity)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_relationship(&self, user_id: &str, description: &str) -> Result<()> {
        let val = serde_json::to_value(description)?;
        sqlx::query("UPDATE covabot_personality SET relationships = jsonb_set(relationships, ARRAY[$1::text], $2::jsonb, true) WHERE id = 1")
            .bind(user_id)
            .bind(val)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn sync_from_yaml(&self, yaml_content: &str) -> Result<()> {
        let parsed: CovabotPersonalityYml = serde_yaml::from_str(yaml_content)?;
        self.update_personality(&parsed).await
    }

    pub async fn sync_to_yaml(&self) -> Result<String> {
        let personality = self.get_personality().await?;
        let yaml = serde_yaml::to_string(&personality)?;
        Ok(yaml)
    }
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

    #[test]
    fn parses_covabot_personality_yml() {
        let yaml = r#"
identity: "You are a cool bot."
speech_patterns: ["Use emoji"]
affinities: ["Cheeseburgers"]
relationships:
  "1234": "Best friend"
"#;
        let parsed: CovabotPersonalityYml = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(parsed.identity, "You are a cool bot.");
        assert_eq!(parsed.speech_patterns, vec!["Use emoji"]);
        assert_eq!(parsed.affinities, vec!["Cheeseburgers"]);
        assert_eq!(parsed.relationships.get("1234").unwrap(), "Best friend");
    }

    #[test]
    fn serializes_covabot_personality_yml() {
        let mut relationships = HashMap::new();
        relationships.insert("1234".to_string(), "Enemy".to_string());

        let p = CovabotPersonalityYml {
            identity: "Test".to_string(),
            speech_patterns: vec!["Test Pattern".to_string()],
            affinities: vec!["Test Affinity".to_string()],
            relationships,
        };

        let yaml = serde_yaml::to_string(&p).unwrap();
        assert!(yaml.contains("identity: Test"));
        assert!(yaml.contains("speech_patterns:"));
        assert!(yaml.contains("- Test Pattern"));
    }

    use tokio::sync::OnceCell;
    static POOL: OnceCell<sqlx::PgPool> = OnceCell::const_new();

    async fn setup_store() -> PersonalityStore {
        let pool = POOL
            .get_or_init(|| async {
                let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgresql://starbunk:starbunk@localhost:5432/starbunk_memory".to_string()
                });
                let p = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(5)
                    .connect(&db_url)
                    .await
                    .unwrap();

                let store = PersonalityStore::new(p.clone());
                store.init_schema().await.unwrap();
                p
            })
            .await;

        PersonalityStore::new(pool.clone())
    }

    #[tokio::test]
    async fn db_crud_personality_get() {
        let store = setup_store().await;
        store.get_personality().await.unwrap();
    }

    #[tokio::test]
    async fn db_crud_personality_update() {
        let store = setup_store().await;
        store
            .update_personality(&CovabotPersonalityYml::default())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn db_crud_personality_update_identity() {
        let store = setup_store().await;
        store.update_identity("New Identity").await.unwrap();
    }

    #[tokio::test]
    async fn db_crud_personality_add_speech_pattern() {
        let store = setup_store().await;
        store.add_speech_pattern("pattern").await.unwrap();
    }

    #[tokio::test]
    async fn db_crud_personality_add_affinity() {
        let store = setup_store().await;
        store.add_affinity("affinity").await.unwrap();
    }

    #[tokio::test]
    async fn db_crud_personality_set_relationship() {
        let store = setup_store().await;
        store.set_relationship("user_id", "friend").await.unwrap();
    }

    #[tokio::test]
    async fn db_crud_personality_sync_from_yaml() {
        let store = setup_store().await;
        let yaml_content = r#"
identity: "Test"
speech_patterns: []
affinities: []
relationships: {}
"#;
        store.sync_from_yaml(yaml_content).await.unwrap();
    }

    #[tokio::test]
    async fn db_crud_personality_sync_to_yaml() {
        let store = setup_store().await;
        store.sync_to_yaml().await.unwrap();
    }
}
