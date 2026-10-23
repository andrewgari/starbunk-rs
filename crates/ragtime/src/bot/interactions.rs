//! Small serenity helpers for the interaction handlers.

use crate::presentation::ANSWER_INPUT_CUSTOM_ID;
use serenity::all::{
    ActionRow, ActionRowComponent, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse,
};

/// The text the player typed into the answer modal's input, if present.
pub(super) fn answer_text(rows: &[ActionRow]) -> Option<String> {
    rows.iter()
        .flat_map(|row| row.components.iter())
        .find_map(|component| match component {
            ActionRowComponent::InputText(input) if input.custom_id == ANSWER_INPUT_CUSTOM_ID => {
                input.value.clone()
            }
            _ => None,
        })
}

/// An immediate ephemeral text response.
pub(super) fn ephemeral(text: &str) -> CreateInteractionResponse {
    CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(text)
            .ephemeral(true),
    )
}

/// Content for editing a deferred (ephemeral) response.
pub(super) fn edit(text: &str) -> EditInteractionResponse {
    EditInteractionResponse::new().content(text)
}
