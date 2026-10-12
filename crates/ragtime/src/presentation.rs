//! Pure Discord message builders for the Ragtime contest.
//!
//! Every function here is a pure, Discord-I/O-free mapping from plain data
//! (user ids as `u64`, strings, scores) to a `serenity` builder type
//! (`CreateEmbed`, `CreateButton`, `CreateModal`, ...). Nothing in this module
//! performs network calls or depends on `crate::round` — the Discord layer
//! (wired in a later task) is responsible for calling these with data derived
//! from the round state machine's `RoundEvent`/`Reveal`/`Standing` payloads.
//!
//! # Routing
//!
//! The [`custom_id`] submodule encodes "which channel/round and which
//! question index" into the button and modal custom ids so the interaction
//! handler can route a click or a modal submit back to the right round
//! without any extra lookup table.
//!
//! # Wiring guide (for the Discord layer)
//!
//! | Round event / interaction | Call | Notes |
//! |---|---|---|
//! | `RoundRegistry::create` succeeds | [`lobby_embed`] | send it, then react with [`JOIN_EMOJI`] |
//! | `RoundEvent::PostQuestion { channel, index, total, question, .. }` | [`question::question_embed`] + [`question::question_action_row`] with `AnswerRoute::new(channel, index)` | attach the row to the question message |
//! | \[Answer\] button clicked, `can_answer` → `NotInRound` | [`not_in_round_text`] | send ephemeral |
//! | \[Answer\] button clicked, `can_answer` → `AlreadyAnswered` | [`already_answered_text`] | send ephemeral |
//! | \[Answer\] button clicked, `can_answer` → `Ok` | [`custom_id::parse_button`] the clicked button's id to recover the `AnswerRoute`, then [`modal::answer_modal`] with the same route | open as the response to the component interaction |
//! | modal submitted | [`modal::parse_answer_modal`] the modal's `custom_id`, read the input via [`ANSWER_INPUT_CUSTOM_ID`], then call `RoundRegistry::submit_answer` | acknowledge ephemerally without revealing correctness |
//! | `RoundEvent::PostReveal(reveal)` | map `reveal.correct` (`Vec<round::UserId>`, which is a `u64`) and `reveal.display_answer` into [`RevealInfo`], then [`reveal_embed`] | |
//! | `RoundEvent::PostTally { standings, .. }` | map each `round::Standing` into a [`TallyRow`] (same field names), then [`tally_embed`] | `standings` from `round::rank` is already ordered; pass it straight through |

pub mod custom_id;
pub mod lobby;
pub mod modal;
pub mod question;
pub mod response;
pub mod reveal;
pub mod tally;

pub use custom_id::{AnswerRoute, CustomIdError};
pub use lobby::{lobby_embed, JOIN_EMOJI};
pub use modal::{answer_modal, ANSWER_INPUT_CUSTOM_ID};
pub use question::{answer_button, question_embed};
pub use response::{already_answered_text, not_in_round_text};
pub use reveal::{reveal_embed, RevealInfo};
pub use tally::{tally_embed, TallyRow};
