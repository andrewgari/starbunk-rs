//! Answer grading abstraction.

/// Grades a player's reply. Implemented for any
/// `Fn(&str, &str, &[String]) -> bool`, so the Discord layer can pass the
/// crate's `grade()` function directly:
/// `Arc::new(|reply: &str, display: &str, ok: &[String]| grade(reply, display, ok))`.
pub trait Grader: Send + Sync {
    /// `true` if `reply` is a correct answer.
    fn grade(&self, reply: &str, display_answer: &str, acceptable_answers: &[String]) -> bool;
}

impl<F> Grader for F
where
    F: Fn(&str, &str, &[String]) -> bool + Send + Sync,
{
    fn grade(&self, reply: &str, display_answer: &str, acceptable_answers: &[String]) -> bool {
        self(reply, display_answer, acceptable_answers)
    }
}
