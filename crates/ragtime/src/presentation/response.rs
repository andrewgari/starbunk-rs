//! Ephemeral response text for answer-button edge cases.
//!
//! These are plain `&'static str`s rather than full `CreateInteractionResponseMessage`
//! builders: the interaction handler already knows it needs an ephemeral
//! message (`CreateInteractionResponseMessage::new().ephemeral(true).content(..)`),
//! so this module only owns the copy, keeping it consistent and testable
//! without a serenity `Context`.

/// Shown when a user who never joined the round clicks \[Answer\].
pub const NOT_IN_ROUND_TEXT: &str =
    "You're not in this round. React to the lobby message to join the next one.";

/// Shown when a user who already answered this question clicks \[Answer\] again.
pub const ALREADY_ANSWERED_TEXT: &str = "You've already answered this question.";

/// Function form, for call sites that prefer calling over importing a const.
pub fn not_in_round_text() -> &'static str {
    NOT_IN_ROUND_TEXT
}

/// Function form, for call sites that prefer calling over importing a const.
pub fn already_answered_text() -> &'static str {
    ALREADY_ANSWERED_TEXT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_in_round_text_is_non_empty_and_mentions_the_round() {
        let text = not_in_round_text();
        assert!(!text.is_empty());
        assert!(text.to_lowercase().contains("round"));
    }

    #[test]
    fn already_answered_text_is_non_empty_and_distinct() {
        let text = already_answered_text();
        assert!(!text.is_empty());
        assert_ne!(text, not_in_round_text());
    }
}
