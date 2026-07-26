use async_trait::async_trait;
use regex::Regex;
use serenity::all::{Context, Message};
use starbunk::replybot::Strategy;
use std::sync::LazyLock;

/// Matches any message containing a "sheesh"-like word where the total
/// number of 'e' characters in the matched token is >= 2.
///
/// Valid examples: "sheesh", "sheeeesh", "SHEESH", "oh sheesh that's wild"
/// Invalid examples: "shed", "she", "shell" (fewer than 2 e's in the token)
static SHEESH_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // sh + 2 or more e's + sh, case-insensitive
    Regex::new(r"(?i)\bsh(e{2,})sh\b").expect("sheesh regex")
});

#[derive(Debug)]
pub struct SheeshStrategy;

impl SheeshStrategy {
    pub fn new() -> Self {
        Self
    }

    /// Returns true if the message content contains a sheesh-like pattern
    /// with at least 2 'e' characters.
    pub fn matches(content: &str) -> bool {
        SHEESH_PATTERN.is_match(content)
    }

    /// Generates a reply of the form `sh{N}sh 😤` where N is a random
    /// number of 'e' characters between 2 and 20 (inclusive).
    #[allow(dead_code)] // implemented in PR 2
    pub fn build_reply(_n: usize) -> String {
        // Stub — real implementation in PR 2
        unimplemented!("sheeshbot reply not yet implemented")
    }
}

impl Default for SheeshStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Strategy for SheeshStrategy {
    fn name(&self) -> &str {
        "SheeshStrategy"
    }

    async fn should_trigger(&self, _ctx: &Context, msg: &Message) -> bool {
        Self::matches(&msg.content)
    }

    async fn response(&self, _ctx: &Context, _msg: &Message) -> String {
        // Stub — real implementation in PR 2
        unimplemented!("sheeshbot response not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Trigger matching ──────────────────────────────────────────────────────

    #[test]
    fn triggers_on_exact_sheesh() {
        assert!(SheeshStrategy::matches("sheesh"));
    }

    #[test]
    fn triggers_on_extended_sheesh() {
        assert!(SheeshStrategy::matches("sheeeesh"));
        assert!(SheeshStrategy::matches("sheeeeeeeeesh"));
    }

    #[test]
    fn triggers_case_insensitive() {
        assert!(SheeshStrategy::matches("SHEESH"));
        assert!(SheeshStrategy::matches("Sheesh"));
        assert!(SheeshStrategy::matches("sHeEsH"));
    }

    #[test]
    fn triggers_when_embedded_in_sentence() {
        assert!(SheeshStrategy::matches("oh sheesh that's wild"));
        assert!(SheeshStrategy::matches("wow sheesh man"));
    }

    #[test]
    fn does_not_trigger_on_shed() {
        assert!(!SheeshStrategy::matches("shed"));
    }

    #[test]
    fn does_not_trigger_on_she() {
        assert!(!SheeshStrategy::matches("she"));
    }

    #[test]
    fn does_not_trigger_on_shell() {
        assert!(!SheeshStrategy::matches("shell"));
    }

    #[test]
    fn does_not_trigger_on_empty() {
        assert!(!SheeshStrategy::matches(""));
    }

    #[test]
    fn does_not_trigger_on_unrelated_content() {
        assert!(!SheeshStrategy::matches("hello world"));
        assert!(!SheeshStrategy::matches("blue is my favourite colour"));
    }

    #[test]
    fn does_not_trigger_on_single_e_sheesh_variant() {
        // "shesh" has only one 'e' between the sh's — should not trigger
        assert!(!SheeshStrategy::matches("shesh"));
    }

    // ── Reply format ──────────────────────────────────────────────────────────
    //
    // These contract tests define the *expected* output of build_reply once
    // the implementation lands in PR 2.  They are marked #[ignore] so that CI
    // stays green while the stub still panics; remove #[ignore] in PR 2 after
    // the real implementation is written.

    #[test]
    #[ignore = "contract test — enable in PR 2 when build_reply is implemented"]
    fn reply_format_contract_n_2() {
        let reply = SheeshStrategy::build_reply(2);
        assert_eq!(
            reply, "sheesh 😤",
            "build_reply(2) must produce exactly 2 e's"
        );
    }

    #[test]
    #[ignore = "contract test — enable in PR 2 when build_reply is implemented"]
    fn reply_format_contract_n_20() {
        let reply = SheeshStrategy::build_reply(20);
        // "sh" + 20 × "e" + "sh 😤"
        let expected = format!("sh{}sh 😤", "e".repeat(20));
        assert_eq!(
            reply, expected,
            "build_reply(20) must produce exactly 20 e's"
        );
    }

    #[test]
    #[ignore = "contract test — enable in PR 2 when build_reply is implemented"]
    fn reply_starts_with_sh_and_ends_with_sh_emoji() {
        let reply = SheeshStrategy::build_reply(5);
        assert!(reply.starts_with("sh"), "reply must start with 'sh'");
        assert!(reply.ends_with("sh 😤"), "reply must end with 'sh 😤'");
    }

    #[test]
    #[ignore = "contract test — enable in PR 2 when build_reply is implemented"]
    fn reply_e_count_matches_n() {
        for n in [2, 5, 10, 20] {
            let reply = SheeshStrategy::build_reply(n);
            let inner = &reply[2..reply.len() - "sh 😤".len()];
            assert_eq!(
                inner.len(),
                n,
                "build_reply({n}) should have exactly {n} e's in the middle"
            );
            assert!(
                inner.chars().all(|c| c == 'e'),
                "build_reply({n}) middle must be all 'e' characters"
            );
        }
    }

    // ── Strategy name ─────────────────────────────────────────────────────────

    #[test]
    fn strategy_name_is_sheesh_strategy() {
        let s = SheeshStrategy::new();
        assert_eq!(s.name(), "SheeshStrategy");
    }
}
