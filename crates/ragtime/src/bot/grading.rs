//! Production answer grader: fuzzy matching with a configurable threshold.

use crate::round::Grader;
use std::sync::Arc;

/// Grader backed by [`crate::grade_with_threshold`].
pub fn fuzzy_grader(threshold: f64) -> Arc<dyn Grader> {
    Arc::new(move |reply: &str, display: &str, ok: &[String]| {
        crate::grade_with_threshold(reply, display, ok, threshold)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn honours_threshold() {
        let ok = vec!["JFK".to_string()];
        let lenient = fuzzy_grader(0.85);
        assert!(lenient.grade("jfk", "John F. Kennedy", &ok));
        assert!(lenient.grade("kenedy", "Kennedy", &[]));
        assert!(!lenient.grade("Lincoln", "Kennedy", &[]));
        assert!(!fuzzy_grader(1.0).grade("kenedy", "Kennedy", &[]));
    }
}
