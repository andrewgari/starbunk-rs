//! Fuzzy answer grading (TDD stub — PR 1).
//!
//! The behaviour is pinned by `tests/fuzzy_grader.rs`; the implementation
//! lands in PR 2 (card t_87abbc89). Until then every call panics so the
//! table-driven suite fails loudly instead of passing by accident.

/// Default similarity threshold used by [`grade`].
pub const DEFAULT_GRADE_THRESHOLD: f64 = 0.85;

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
    _reply: &str,
    _display_answer: &str,
    _acceptable_answers: &[String],
    _threshold: f64,
) -> bool {
    todo!("fuzzy grade() — implemented in PR 2 (t_87abbc89)")
}
