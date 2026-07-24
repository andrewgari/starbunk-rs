use serde::{Deserialize, Serialize};

/// Top-level wrapper matching the `reply-bots:` YAML key.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReplyBotsFile {
    #[serde(rename = "reply-bots")]
    pub reply_bots: Vec<BotConfig>,
}

/// Configuration for a single reply bot loaded from YAML.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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
    /// When true (default), the bot ignores messages that came from itself.
    #[serde(default = "default_true")]
    pub ignore_self: bool,
    /// When true (default), the bot ignores messages sent via any webhook.
    /// Set to false only for bots that deliberately respond to webhook messages,
    /// and ensure those bots cannot trigger themselves (e.g. via `ignore_self`).
    #[serde(default = "default_true")]
    pub ignore_webhooks: bool,
    /// Probability (0–100) that the bot fires on any given trigger match.
    /// 100 = always fire (default), 0 = never fire.
    #[serde(default = "default_frequency")]
    pub frequency: u8,
    pub triggers: Vec<TriggerConfig>,
}

fn default_true() -> bool {
    true
}

fn default_frequency() -> u8 {
    100
}

/// The persona a bot assumes when posting a response via webhook.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IdentityConfig {
    /// Fixed display name and avatar.
    Static {
        #[serde(alias = "botName")]
        bot_name: String,
        #[serde(alias = "avatarUrl")]
        avatar_url: String,
    },
    /// Mirrors a specific Discord member by their user ID.
    Mimic {
        #[serde(alias = "as_member")]
        user_id: Snowflake,
    },
    /// Picks a random guild member each time the bot fires.
    Random,
    /// Adopts the identity of whoever sent the triggering message.
    MimicPoster,
}

/// A single named trigger within a bot definition.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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

impl serde::Serialize for ConditionNode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            ConditionNode::ContainsPhrase(s) => map.serialize_entry("contains_phrase", s)?,
            ConditionNode::ContainsWord(s) => map.serialize_entry("contains_word", s)?,
            ConditionNode::MatchesRegex(s) => map.serialize_entry("matches_regex", s)?,
            ConditionNode::FromUser(s) => map.serialize_entry("from_user", s)?,
            ConditionNode::WithChance(n) => map.serialize_entry("with_chance", n)?,
            ConditionNode::Always(b) => map.serialize_entry("always", b)?,
            ConditionNode::AllOf(v) => map.serialize_entry("all_of", v)?,
            ConditionNode::AnyOf(v) => map.serialize_entry("any_of", v)?,
            ConditionNode::NoneOf(v) => map.serialize_entry("none_of", v)?,
        }
        map.end()
    }
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
#[derive(Debug, Clone, PartialEq, Serialize)]
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
