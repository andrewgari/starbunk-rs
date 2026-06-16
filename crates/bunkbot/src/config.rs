use serde::Deserialize;

/// Top-level wrapper matching the `reply-bots:` YAML key.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ReplyBotsFile {
    #[serde(rename = "reply-bots")]
    pub reply_bots: Vec<BotConfig>,
}

/// Configuration for a single reply bot loaded from YAML.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct BotConfig {
    pub name: String,
    pub identity: IdentityConfig,
    /// Bot-level response pool. Used when a trigger has no responses of its own.
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub responses: Vec<String>,
    /// When true (default), the bot ignores messages from other bots.
    #[serde(default = "default_true")]
    pub ignore_bots: bool,
    /// When true, the bot ignores messages from human users.
    #[serde(default)]
    pub ignore_humans: bool,
    pub triggers: Vec<TriggerConfig>,
}

fn default_true() -> bool {
    true
}

/// The persona a bot assumes when posting a response via webhook.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IdentityConfig {
    /// Fixed display name and avatar.
    Static {
        bot_name: String,
        avatar_url: String,
    },
    /// Mirrors a specific Discord member's display name and avatar.
    Mimic { as_member: String },
    /// Picks a random guild member each time the bot fires.
    Random,
}

/// A single named trigger within a bot definition.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TriggerConfig {
    /// Optional label used in logs and admin commands.
    pub name: Option<String>,
    pub conditions: ConditionNode,
    /// Trigger-level response pool. When non-empty, overrides the bot-level pool.
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub responses: Vec<String>,
}

/// Recursive condition tree.
///
/// Leaf nodes evaluate a single property of the incoming message.
/// Compound nodes (`all_of`, `any_of`, `none_of`) combine other nodes.
///
/// YAML uses a single-key mapping for each node:
/// ```yaml
/// contains_phrase: "banana"
/// all_of:
///   - contains_phrase: "banana"
///   - with_chance: 25
/// ```
///
/// serde_yaml 0.9 no longer supports JSON-style external enum tagging, so
/// deserialization is implemented manually via a `MapAccess` visitor.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionNode {
    // --- leaf conditions ---
    /// Case-insensitive substring match. URLs are stripped before matching.
    ContainsPhrase(String),
    /// Case-insensitive whole-word match (word-boundary anchored). URLs stripped.
    ContainsWord(String),
    /// Case-insensitive regex match. URLs stripped before matching.
    /// Also accepted as `matches_pattern` for compatibility.
    MatchesRegex(String),
    /// Matches only messages from this Discord user (string or integer snowflake).
    FromUser(Snowflake),
    /// Fires with N% probability (0 = never, 100 = always).
    WithChance(u8),
    /// Unconditionally passes. Typically paired with `with_chance` inside `all_of`.
    Always(bool),
    // --- compound conditions ---
    /// Passes only when every child passes (AND, short-circuits).
    AllOf(Vec<ConditionNode>),
    /// Passes when at least one child passes (OR, short-circuits).
    AnyOf(Vec<ConditionNode>),
    /// Passes when no child passes (NOR).
    NoneOf(Vec<ConditionNode>),
}

impl<'de> Deserialize<'de> for ConditionNode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{MapAccess, Visitor};

        struct ConditionVisitor;

        impl<'de> Visitor<'de> for ConditionVisitor {
            type Value = ConditionNode;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "a single-key condition mapping, e.g. `contains_phrase: \"text\"`"
                )
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<ConditionNode, A::Error> {
                let key: String = map
                    .next_key()?
                    .ok_or_else(|| serde::de::Error::custom("condition mapping is empty"))?;

                let node = match key.as_str() {
                    "contains_phrase" => ConditionNode::ContainsPhrase(map.next_value()?),
                    "contains_word" => ConditionNode::ContainsWord(map.next_value()?),
                    "matches_regex" | "matches_pattern" => {
                        ConditionNode::MatchesRegex(map.next_value()?)
                    }
                    "from_user" => ConditionNode::FromUser(map.next_value()?),
                    "with_chance" => ConditionNode::WithChance(map.next_value()?),
                    "always" => ConditionNode::Always(map.next_value()?),
                    "all_of" => ConditionNode::AllOf(map.next_value()?),
                    "any_of" => ConditionNode::AnyOf(map.next_value()?),
                    "none_of" => ConditionNode::NoneOf(map.next_value()?),
                    other => {
                        return Err(serde::de::Error::unknown_field(
                            other,
                            &[
                                "contains_phrase",
                                "contains_word",
                                "matches_regex",
                                "matches_pattern",
                                "from_user",
                                "with_chance",
                                "always",
                                "all_of",
                                "any_of",
                                "none_of",
                            ],
                        ))
                    }
                };

                if map.next_key::<String>()?.is_some() {
                    return Err(serde::de::Error::custom(
                        "condition mapping must have exactly one key",
                    ));
                }

                Ok(node)
            }
        }

        deserializer.deserialize_map(ConditionVisitor)
    }
}

/// A Discord snowflake ID that safely deserializes from either a quoted string
/// (`"113035990725066752"`) or a bare YAML integer (`113035990725066752`).
///
/// Both forms appear in the production bots.yml.
#[derive(Debug, Clone, PartialEq)]
pub struct Snowflake(pub String);

impl<'de> Deserialize<'de> for Snowflake {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SnowflakeVisitor;

        impl<'de> serde::de::Visitor<'de> for SnowflakeVisitor {
            type Value = Snowflake;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "a Discord snowflake as an integer or quoted string")
            }

            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Snowflake, E> {
                Ok(Snowflake(v.to_string()))
            }

            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Snowflake, E> {
                if v < 0 {
                    return Err(E::custom(format!(
                        "Discord snowflake must be non-negative, got {v}"
                    )));
                }
                Ok(Snowflake(v.to_string()))
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Snowflake, E> {
                Ok(Snowflake(v.to_string()))
            }

            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Snowflake, E> {
                Ok(Snowflake(v))
            }
        }

        deserializer.deserialize_any(SnowflakeVisitor)
    }
}

impl AsRef<str> for Snowflake {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Snowflake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Deserialize a YAML field that may be a single string or a sequence of strings.
///
/// Production YAML has both forms:
/// ```yaml
/// responses: "Nice."                      # single
/// responses:                              # array
///   - "Hello!"
///   - "Hi there!"
/// ```
fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        Single(String),
        Multiple(Vec<String>),
    }

    match StringOrVec::deserialize(deserializer)? {
        StringOrVec::Single(s) => Ok(vec![s]),
        StringOrVec::Multiple(v) => Ok(v),
    }
}

/// Parse a YAML string containing a `reply-bots:` document into a list of bot configs.
pub fn parse_bots(yaml: &str) -> Result<Vec<BotConfig>, serde_yaml::Error> {
    let file: ReplyBotsFile = serde_yaml::from_str(yaml)?;
    Ok(file.reply_bots)
}

#[cfg(test)]
mod tests;
