use crate::config::{BotConfig, ConditionNode, IdentityConfig};
use rand::Rng;
use regex::Regex;
use serenity::all::Message;

/// Mirrors `ConditionNode` with all regex patterns pre-compiled at load time.
///
/// `ContainsWord` and `MatchesRegex` both store a `Regex` built during
/// `BunkBotEngine::new` rather than on every message evaluation. Invalid
/// patterns cause the owning bot to be skipped at load time (logged as error).
pub(super) enum CompiledNode {
    ContainsPhrase(String),
    ContainsWord(Regex),
    MatchesRegex(Regex),
    /// Stored as the raw snowflake string for a cheap string comparison.
    FromUser(String),
    /// Percentage 0–100; values >100 are clamped at eval time.
    WithChance(u8),
    Always(bool),
    AllOf(Vec<CompiledNode>),
    AnyOf(Vec<CompiledNode>),
    NoneOf(Vec<CompiledNode>),
}

impl TryFrom<ConditionNode> for CompiledNode {
    type Error = regex::Error;

    fn try_from(node: ConditionNode) -> Result<Self, regex::Error> {
        match node {
            ConditionNode::ContainsPhrase(s) => Ok(CompiledNode::ContainsPhrase(s)),
            ConditionNode::ContainsWord(word) => {
                let pattern = format!(r"(?i)\b{}\b", regex::escape(&word));
                Ok(CompiledNode::ContainsWord(Regex::new(&pattern)?))
            }
            ConditionNode::MatchesRegex(pattern) => {
                let pattern = if pattern.starts_with("(?") {
                    pattern
                } else {
                    format!("(?i){}", pattern)
                };
                Ok(CompiledNode::MatchesRegex(Regex::new(&pattern)?))
            }
            ConditionNode::FromUser(id) => Ok(CompiledNode::FromUser(id.0)),
            ConditionNode::WithChance(pct) => Ok(CompiledNode::WithChance(pct)),
            ConditionNode::Always(b) => Ok(CompiledNode::Always(b)),
            ConditionNode::AllOf(v) => Ok(CompiledNode::AllOf(
                v.into_iter()
                    .map(CompiledNode::try_from)
                    .collect::<Result<_, _>>()?,
            )),
            ConditionNode::AnyOf(v) => Ok(CompiledNode::AnyOf(
                v.into_iter()
                    .map(CompiledNode::try_from)
                    .collect::<Result<_, _>>()?,
            )),
            ConditionNode::NoneOf(v) => Ok(CompiledNode::NoneOf(
                v.into_iter()
                    .map(CompiledNode::try_from)
                    .collect::<Result<_, _>>()?,
            )),
        }
    }
}

pub(super) struct CompiledTrigger {
    pub responses: Vec<String>,
    pub conditions: CompiledNode,
}

pub(super) struct CompiledBot {
    pub name: String,
    pub identity: IdentityConfig,
    pub responses: Vec<String>,
    pub ignore_bots: bool,
    pub ignore_humans: bool,
    pub ignore_self: bool,
    pub frequency: u8,
    pub triggers: Vec<CompiledTrigger>,
}

impl TryFrom<BotConfig> for CompiledBot {
    type Error = regex::Error;

    fn try_from(c: BotConfig) -> Result<Self, regex::Error> {
        let triggers = c
            .triggers
            .into_iter()
            .map(|t| {
                Ok(CompiledTrigger {
                    responses: t.responses,
                    conditions: CompiledNode::try_from(t.conditions)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(CompiledBot {
            name: c.name,
            identity: c.identity,
            responses: c.responses,
            ignore_bots: c.ignore_bots,
            ignore_humans: c.ignore_humans,
            ignore_self: c.ignore_self,
            frequency: c.frequency,
            triggers,
        })
    }
}

/// Evaluate a compiled condition tree against a message.
/// `stripped` is the URL-stripped message content (computed once per dispatch).
pub(super) fn eval(node: &CompiledNode, msg: &Message, stripped: &str) -> bool {
    match node {
        CompiledNode::ContainsPhrase(phrase) => {
            stripped.to_lowercase().contains(&phrase.to_lowercase())
        }
        CompiledNode::ContainsWord(re) => re.is_match(stripped),
        CompiledNode::MatchesRegex(re) => re.is_match(stripped),
        CompiledNode::FromUser(id) => msg.author.id.to_string() == *id,
        // Clamp to 100 so values like 200 don't silently become "always fire".
        CompiledNode::WithChance(pct) => rand::thread_rng().gen_range(0u8..100) < (*pct).min(100),
        CompiledNode::Always(b) => *b,
        CompiledNode::AllOf(v) => v.iter().all(|c| eval(c, msg, stripped)),
        CompiledNode::AnyOf(v) => v.iter().any(|c| eval(c, msg, stripped)),
        CompiledNode::NoneOf(v) => v.iter().all(|c| !eval(c, msg, stripped)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_msg(content: &str, author_id: &str) -> Message {
        serde_json::from_value(serde_json::json!({
            "id": "1", "channel_id": "1",
            "author": {
                "id": author_id, "username": "testuser",
                "bot": false, "discriminator": "0", "public_flags": 0
            },
            "content": content,
            "timestamp": "2024-01-01T12:00:00+00:00",
            "edited_timestamp": null, "tts": false, "mention_everyone": false,
            "mentions": [], "mention_roles": [], "attachments": [], "embeds": [],
            "pinned": false, "type": 0
        }))
        .expect("test message")
    }

    fn check(node: &CompiledNode, content: &str) -> bool {
        let msg = build_msg(content, "1");
        eval(node, &msg, content)
    }

    #[test]
    fn contains_phrase_case_insensitive() {
        let node = CompiledNode::ContainsPhrase("banana".into());
        assert!(check(&node, "I like BANANA today"));
    }

    #[test]
    fn contains_phrase_no_match() {
        let node = CompiledNode::ContainsPhrase("banana".into());
        assert!(!check(&node, "I like apples"));
    }

    #[test]
    fn contains_word_word_boundary() {
        let node =
            CompiledNode::try_from(crate::config::ConditionNode::ContainsWord("check".into()))
                .unwrap();
        assert!(check(&node, "please check this"));
    }

    #[test]
    fn contains_word_does_not_match_substring() {
        let node =
            CompiledNode::try_from(crate::config::ConditionNode::ContainsWord("check".into()))
                .unwrap();
        assert!(!check(&node, "checkout the store"));
    }

    #[test]
    fn matches_regex_fires_on_match() {
        let node =
            CompiledNode::try_from(crate::config::ConditionNode::MatchesRegex(r"\d+".into()))
                .unwrap();
        assert!(check(&node, "3 bananas"));
    }

    #[test]
    fn matches_regex_no_fire_on_no_match() {
        let node =
            CompiledNode::try_from(crate::config::ConditionNode::MatchesRegex(r"\d+".into()))
                .unwrap();
        assert!(!check(&node, "no digits here"));
    }

    #[test]
    fn matches_regex_case_insensitive_by_default() {
        let node = CompiledNode::try_from(crate::config::ConditionNode::MatchesRegex(
            r"spider[^-]*man".into(),
        ))
        .unwrap();
        assert!(check(&node, "SpiderMan"));
        assert!(check(&node, "spiderman"));
        assert!(!check(&node, "Spider-Man"));
    }

    #[test]
    fn from_user_matches_author_id() {
        let node = CompiledNode::FromUser("42".into());
        let msg = build_msg("hi", "42");
        assert!(eval(&node, &msg, "hi"));
    }

    #[test]
    fn from_user_rejects_wrong_id() {
        let node = CompiledNode::FromUser("42".into());
        let msg = build_msg("hi", "99");
        assert!(!eval(&node, &msg, "hi"));
    }

    #[test]
    fn with_chance_100_always_passes() {
        let node = CompiledNode::WithChance(100);
        for _ in 0..50 {
            assert!(check(&node, ""));
        }
    }

    #[test]
    fn with_chance_0_never_passes() {
        let node = CompiledNode::WithChance(0);
        for _ in 0..50 {
            assert!(!check(&node, ""));
        }
    }

    #[test]
    fn with_chance_above_100_treated_as_100() {
        // Values > 100 must be clamped to 100, not become "always-fire" silently.
        let node = CompiledNode::WithChance(200);
        for _ in 0..50 {
            // After clamping to 100, gen_range(0..100) < 100 is always true.
            assert!(check(&node, ""));
        }
    }

    #[test]
    fn always_true_passes() {
        assert!(check(&CompiledNode::Always(true), ""));
    }

    #[test]
    fn always_false_blocks() {
        assert!(!check(&CompiledNode::Always(false), ""));
    }

    #[test]
    fn all_of_passes_when_all_pass() {
        let node =
            CompiledNode::AllOf(vec![CompiledNode::Always(true), CompiledNode::Always(true)]);
        assert!(check(&node, ""));
    }

    #[test]
    fn all_of_fails_when_any_fails() {
        let node = CompiledNode::AllOf(vec![
            CompiledNode::Always(true),
            CompiledNode::Always(false),
        ]);
        assert!(!check(&node, ""));
    }

    #[test]
    fn none_of_passes_when_all_fail() {
        let node = CompiledNode::NoneOf(vec![
            CompiledNode::Always(false),
            CompiledNode::Always(false),
        ]);
        assert!(check(&node, ""));
    }

    #[test]
    fn none_of_fails_when_any_passes() {
        let node = CompiledNode::NoneOf(vec![
            CompiledNode::Always(false),
            CompiledNode::Always(true),
        ]);
        assert!(!check(&node, ""));
    }
}
