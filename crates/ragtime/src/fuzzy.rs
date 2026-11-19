//! Fuzzy answer grading.
//!
//! Normalisation: lowercase, drop punctuation, collapse whitespace, drop one
//! leading article ("the ", "a ", "an "). An empty normalised reply is never
//! correct. Normalised strings of [`SHORT_ANSWER_MAX_CHARS`] chars or fewer
//! (on either side) must match exactly; longer ones are accepted when the
//! `strsim` normalised Levenshtein similarity is `>= threshold`.

use strsim::normalized_levenshtein;

/// Default similarity threshold used by [`grade`].
pub const DEFAULT_GRADE_THRESHOLD: f64 = 0.85;

/// Normalised answers this short (or shorter) require an exact match.
pub const SHORT_ANSWER_MAX_CHARS: usize = 4;

fn normalize(input: &str) -> String {
    let cleaned: String = input
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    let words: Vec<&str> = cleaned.split_whitespace().collect();
    let words = match words.as_slice() {
        ["the" | "a" | "an", rest @ ..] if !rest.is_empty() => rest,
        all => all,
    };
    words.join(" ")
}

fn matches(reply: &str, candidate: &str, threshold: f64) -> bool {
    if candidate.is_empty() {
        return false;
    }
    if reply == candidate {
        return true;
    }
    if reply.chars().count() <= SHORT_ANSWER_MAX_CHARS
        || candidate.chars().count() <= SHORT_ANSWER_MAX_CHARS
    {
        return false;
    }
    normalized_levenshtein(reply, candidate) >= threshold
}

/// Grade `reply` against the display answer and acceptable answers using
/// [`DEFAULT_GRADE_THRESHOLD`].
pub fn grade(reply: &str, display_answer: &str, acceptable_answers: &[String]) -> bool {
    grade_with_threshold(
        reply,
        display_answer,
        acceptable_answers,
        DEFAULT_GRADE_THRESHOLD,
    )
}

/// Grade with an explicit similarity threshold in `0.0..=1.0`.
pub fn grade_with_threshold(
    reply: &str,
    display_answer: &str,
    acceptable_answers: &[String],
    threshold: f64,
) -> bool {
    let reply = normalize(reply);
    if reply.is_empty() {
        return false;
    }
    std::iter::once(display_answer)
        .chain(acceptable_answers.iter().map(String::as_str))
        .any(|c| matches(&reply, &normalize(c), threshold))
}
