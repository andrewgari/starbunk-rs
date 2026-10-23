//! Table-driven spec for the fuzzy answer grader (TDD PR 1).
//!
//! Normalisation: lowercase, trim, strip punctuation, collapse whitespace,
//! drop one leading article ("the ", "a ", "an "). Empty reply => false.
//! Short strings (named cutoff) require an exact normalised match.
//! Otherwise strsim similarity >= threshold (default 0.85) against the
//! display answer or any acceptable answer.

use ragtime::{grade, grade_with_threshold, DEFAULT_GRADE_THRESHOLD};

struct Case {
    name: &'static str,
    reply: &'static str,
    display: &'static str,
    acceptable: &'static [&'static str],
    expected: bool,
}

fn owned(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

const JFK: &str = "John F. Kennedy";

const CASES: &[Case] = &[
    // JFK
    Case {
        name: "jfk acceptable alias",
        reply: "JFK",
        display: JFK,
        acceptable: &["JFK"],
        expected: true,
    },
    Case {
        name: "jfk alias lowercase",
        reply: "jfk",
        display: JFK,
        acceptable: &["JFK"],
        expected: true,
    },
    Case {
        name: "jfk alias punctuated",
        reply: "J.F.K.",
        display: JFK,
        acceptable: &["JFK"],
        expected: true,
    },
    Case {
        name: "full display answer",
        reply: "John F. Kennedy",
        display: JFK,
        acceptable: &["JFK"],
        expected: true,
    },
    Case {
        name: "jfc short near-miss rejected",
        reply: "JFC",
        display: JFK,
        acceptable: &["JFK"],
        expected: false,
    },
    // kenedy vs Kennedy
    Case {
        name: "kenedy typo accepted",
        reply: "kenedy",
        display: "Kennedy",
        acceptable: &[],
        expected: true,
    },
    Case {
        name: "kenedy typo vs acceptable",
        reply: "Kenedy",
        display: JFK,
        acceptable: &["Kennedy"],
        expected: true,
    },
    // Lincoln vs Kennedy
    Case {
        name: "lincoln rejected",
        reply: "Lincoln",
        display: "Kennedy",
        acceptable: &[],
        expected: false,
    },
    Case {
        name: "lincoln rejected vs jfk set",
        reply: "Abraham Lincoln",
        display: JFK,
        acceptable: &["JFK", "Kennedy"],
        expected: false,
    },
    // case / punctuation / article noise
    Case {
        name: "case noise",
        reply: "jOhN f KeNnEdY",
        display: JFK,
        acceptable: &[],
        expected: true,
    },
    Case {
        name: "punctuation noise",
        reply: "John, F. Kennedy!!!",
        display: JFK,
        acceptable: &[],
        expected: true,
    },
    Case {
        name: "whitespace noise",
        reply: "  john   f   kennedy  ",
        display: JFK,
        acceptable: &[],
        expected: true,
    },
    Case {
        name: "leading 'the' dropped",
        reply: "The Beatles",
        display: "Beatles",
        acceptable: &[],
        expected: true,
    },
    Case {
        name: "article on candidate dropped",
        reply: "beatles",
        display: "The Beatles",
        acceptable: &[],
        expected: true,
    },
    Case {
        name: "leading 'a' dropped",
        reply: "a tomato",
        display: "Tomato",
        acceptable: &[],
        expected: true,
    },
    Case {
        name: "leading 'an' dropped",
        reply: "An Apple",
        display: "apple",
        acceptable: &[],
        expected: true,
    },
    Case {
        name: "all noise combined",
        reply: "  THE   beatles!? ",
        display: "Beatles",
        acceptable: &[],
        expected: true,
    },
    // empty / whitespace
    Case {
        name: "empty reply",
        reply: "",
        display: JFK,
        acceptable: &["JFK"],
        expected: false,
    },
    Case {
        name: "whitespace reply",
        reply: "   \t\n ",
        display: JFK,
        acceptable: &["JFK"],
        expected: false,
    },
    Case {
        name: "punctuation-only reply",
        reply: "?!.",
        display: JFK,
        acceptable: &["JFK"],
        expected: false,
    },
    // 4 vs 5 / 4 vs 4
    Case {
        name: "4 vs 5 rejected",
        reply: "4",
        display: "5",
        acceptable: &[],
        expected: false,
    },
    Case {
        name: "4 vs 4 accepted",
        reply: "4",
        display: "4",
        acceptable: &[],
        expected: true,
    },
    Case {
        name: "4 vs four-acceptable",
        reply: "4",
        display: "Four",
        acceptable: &["4"],
        expected: true,
    },
    Case {
        name: "1945 vs 1946 rejected",
        reply: "1946",
        display: "1945",
        acceptable: &[],
        expected: false,
    },
];

#[test]
fn default_threshold_is_0_85() {
    assert!((DEFAULT_GRADE_THRESHOLD - 0.85).abs() < f64::EPSILON);
}

#[test]
fn table_default_threshold() {
    let mut failures = Vec::new();
    for c in CASES {
        let got = std::panic::catch_unwind(|| grade(c.reply, c.display, &owned(c.acceptable)));
        match got {
            Ok(g) if g == c.expected => {}
            other => failures.push(format!(
                "{}: reply={:?} display={:?} expected={} got={:?}",
                c.name,
                c.reply,
                c.display,
                c.expected,
                other.map_err(|_| "panic")
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "{} case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn grade_matches_grade_with_default_threshold() {
    for c in CASES {
        let ok = owned(c.acceptable);
        assert_eq!(
            grade(c.reply, c.display, &ok),
            grade_with_threshold(c.reply, c.display, &ok, DEFAULT_GRADE_THRESHOLD),
            "{}",
            c.name
        );
    }
}

struct ThresholdCase {
    name: &'static str,
    reply: &'static str,
    display: &'static str,
    threshold: f64,
    expected: bool,
}

const THRESHOLD_CASES: &[ThresholdCase] = &[
    ThresholdCase {
        name: "strict rejects kenedy",
        reply: "kenedy",
        display: "Kennedy",
        threshold: 0.99,
        expected: false,
    },
    ThresholdCase {
        name: "strict still accepts exact",
        reply: "KENNEDY",
        display: "Kennedy",
        threshold: 1.0,
        expected: true,
    },
    ThresholdCase {
        name: "lenient accepts heavier typo",
        reply: "kanedi",
        display: "Kennedy",
        threshold: 0.5,
        expected: true,
    },
    ThresholdCase {
        name: "lenient still rejects lincoln",
        reply: "Lincoln",
        display: "Kennedy",
        threshold: 0.5,
        expected: false,
    },
    ThresholdCase {
        name: "zero threshold keeps short-string exact rule",
        reply: "4",
        display: "5",
        threshold: 0.0,
        expected: false,
    },
    ThresholdCase {
        name: "zero threshold keeps empty rule",
        reply: "  ",
        display: "Kennedy",
        threshold: 0.0,
        expected: false,
    },
];

#[test]
fn table_custom_threshold() {
    let mut failures = Vec::new();
    for c in THRESHOLD_CASES {
        let got =
            std::panic::catch_unwind(|| grade_with_threshold(c.reply, c.display, &[], c.threshold));
        match got {
            Ok(g) if g == c.expected => {}
            other => failures.push(format!(
                "{}: reply={:?} display={:?} t={} expected={} got={:?}",
                c.name,
                c.reply,
                c.display,
                c.threshold,
                c.expected,
                other.map_err(|_| "panic")
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "{} case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
