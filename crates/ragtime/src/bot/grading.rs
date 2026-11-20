//! Interim answer grader.
//!
//! The fuzzy `grade()` function is a separate card (t_a1639804) that has not
//! landed yet. Until it does, the bot grades with a normalised exact match so
//! the round is playable. When `grade()` lands, replace the body of
//! [`interim_grader`] with
//! `Arc::new(|reply: &str, display: &str, ok: &[String]| crate::grade(reply, display, ok))`.

use crate::round::Grader;
use std::sync::Arc;

fn normalise(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Case- and whitespace-insensitive exact match against the display answer and
/// every acceptable answer. Empty replies are never correct.
pub fn interim_grader() -> Arc<dyn Grader> {
    Arc::new(|reply: &str, display: &str, acceptable: &[String]| {
        let reply = normalise(reply);
        !reply.is_empty()
            && std::iter::once(display)
                .chain(acceptable.iter().map(String::as_str))
                .any(|candidate| normalise(candidate) == reply)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_display_or_acceptable_ignoring_case_and_spacing() {
        let g = interim_grader();
        let ok = vec!["JFK".to_string()];
        assert!(g.grade("  john   f. KENNEDY ", "John F. Kennedy", &ok));
        assert!(g.grade("jfk", "John F. Kennedy", &ok));
        assert!(!g.grade("Lincoln", "John F. Kennedy", &ok));
        assert!(!g.grade("   ", "John F. Kennedy", &ok));
    }
}
