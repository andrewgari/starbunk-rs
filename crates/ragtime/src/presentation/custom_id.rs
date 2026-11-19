//! Encoding/parsing for the routing info embedded in component and modal
//! `custom_id`s.
//!
//! A button click or a modal submit arrives at the interaction handler with
//! nothing but a `custom_id` string — this module is the single place that
//! knows how to pack "which channel/round and which question" into that
//! string and unpack it again, so the two call sites (button build +
//! interaction routing, modal build + submit routing) can never drift apart.

use std::fmt;

/// The channel/round and question index encoded in an answer button or
/// answer modal `custom_id`.
///
/// `channel` is the Discord channel id (snowflake) that owns the round — one
/// round per channel, so the channel id doubles as the round id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnswerRoute {
    pub channel: u64,
    /// 0-based question index within the round.
    pub index: usize,
}

impl AnswerRoute {
    pub fn new(channel: u64, index: usize) -> Self {
        Self { channel, index }
    }
}

/// A `custom_id` that could not be parsed back into an [`AnswerRoute`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomIdError {
    /// The id doesn't carry a namespace this module recognizes, or is
    /// missing fields.
    Malformed(String),
    /// A field that should have been an integer wasn't.
    InvalidInteger(String),
}

impl fmt::Display for CustomIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomIdError::Malformed(id) => write!(f, "malformed custom_id: {id:?}"),
            CustomIdError::InvalidInteger(id) => {
                write!(f, "custom_id has a non-integer field: {id:?}")
            }
        }
    }
}

impl std::error::Error for CustomIdError {}

const BUTTON_PREFIX: &str = "ragtime:answer";
const MODAL_PREFIX: &str = "ragtime:modal";

fn encode(prefix: &str, route: AnswerRoute) -> String {
    format!("{prefix}:{}:{}", route.channel, route.index)
}

fn decode(prefix: &str, custom_id: &str) -> Result<AnswerRoute, CustomIdError> {
    let rest = custom_id
        .strip_prefix(prefix)
        .and_then(|s| s.strip_prefix(':'))
        .ok_or_else(|| CustomIdError::Malformed(custom_id.to_string()))?;

    let mut parts = rest.split(':');
    let channel = parts
        .next()
        .ok_or_else(|| CustomIdError::Malformed(custom_id.to_string()))?;
    let index = parts
        .next()
        .ok_or_else(|| CustomIdError::Malformed(custom_id.to_string()))?;
    if parts.next().is_some() {
        return Err(CustomIdError::Malformed(custom_id.to_string()));
    }

    let channel: u64 = channel
        .parse()
        .map_err(|_| CustomIdError::InvalidInteger(custom_id.to_string()))?;
    let index: usize = index
        .parse()
        .map_err(|_| CustomIdError::InvalidInteger(custom_id.to_string()))?;

    Ok(AnswerRoute::new(channel, index))
}

/// Encode the `custom_id` for an \[Answer\] button.
pub fn encode_button(route: AnswerRoute) -> String {
    encode(BUTTON_PREFIX, route)
}

/// Parse the `custom_id` of a clicked \[Answer\] button.
pub fn parse_button(custom_id: &str) -> Result<AnswerRoute, CustomIdError> {
    decode(BUTTON_PREFIX, custom_id)
}

/// Encode the `custom_id` for the answer modal.
pub fn encode_modal(route: AnswerRoute) -> String {
    encode(MODAL_PREFIX, route)
}

/// Parse the `custom_id` of a submitted answer modal.
pub fn parse_modal(custom_id: &str) -> Result<AnswerRoute, CustomIdError> {
    decode(MODAL_PREFIX, custom_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_round_trips() {
        let route = AnswerRoute::new(123_456_789, 7);
        let id = encode_button(route);
        assert_eq!(parse_button(&id), Ok(route));
    }

    #[test]
    fn modal_round_trips() {
        let route = AnswerRoute::new(42, 0);
        let id = encode_modal(route);
        assert_eq!(parse_modal(&id), Ok(route));
    }

    #[test]
    fn button_and_modal_ids_are_distinct_for_the_same_route() {
        let route = AnswerRoute::new(1, 2);
        assert_ne!(encode_button(route), encode_modal(route));
    }

    #[test]
    fn button_parser_rejects_a_modal_id() {
        let route = AnswerRoute::new(1, 2);
        let modal_id = encode_modal(route);
        assert!(parse_button(&modal_id).is_err());
    }

    #[test]
    fn modal_parser_rejects_a_button_id() {
        let route = AnswerRoute::new(1, 2);
        let button_id = encode_button(route);
        assert!(parse_modal(&button_id).is_err());
    }

    #[test]
    fn parser_rejects_unrelated_custom_ids() {
        assert!(parse_button("some_other_button").is_err());
        assert!(parse_modal("some_other_modal").is_err());
    }

    #[test]
    fn parser_rejects_missing_fields() {
        assert!(parse_button("ragtime:answer:1").is_err());
        assert!(parse_button("ragtime:answer").is_err());
    }

    #[test]
    fn parser_rejects_extra_fields() {
        assert!(parse_button("ragtime:answer:1:2:3").is_err());
    }

    #[test]
    fn parser_rejects_non_integer_fields() {
        assert!(parse_button("ragtime:answer:not-a-number:2").is_err());
        assert!(parse_button("ragtime:answer:1:not-a-number").is_err());
    }

    #[test]
    fn large_channel_ids_and_index_zero_round_trip() {
        // Discord snowflakes don't fit in u32; make sure we use the full u64 range.
        let route = AnswerRoute::new(u64::MAX, 0);
        assert_eq!(parse_button(&encode_button(route)), Ok(route));
        assert_eq!(parse_modal(&encode_modal(route)), Ok(route));
    }
}
