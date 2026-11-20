//! The answer modal opened by the \[Answer\] button.

use serenity::all::{CreateActionRow, CreateInputText, CreateModal, InputTextStyle};

use super::custom_id::{encode_modal, parse_modal, AnswerRoute, CustomIdError};

/// `custom_id` of the modal's single text input. Fixed (not per-route)
/// because the modal's own `custom_id` already carries the routing info; the
/// interaction handler reads this input by this constant id.
pub const ANSWER_INPUT_CUSTOM_ID: &str = "ragtime:answer_text";

/// Build the single-text-input modal shown after an \[Answer\] button click.
pub fn answer_modal(route: AnswerRoute) -> CreateModal {
    let input = CreateInputText::new(InputTextStyle::Short, "Your answer", ANSWER_INPUT_CUSTOM_ID)
        .required(true);

    CreateModal::new(encode_modal(route), "Ragtime — Answer")
        .components(vec![CreateActionRow::InputText(input)])
}

/// Parse the `custom_id` of a submitted answer modal back into its route.
///
/// Thin re-export so callers only need to import `presentation::modal`
/// instead of reaching into `presentation::custom_id` directly.
pub fn parse_answer_modal(custom_id: &str) -> Result<AnswerRoute, CustomIdError> {
    parse_modal(custom_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modal_json(modal: &CreateModal) -> serde_json::Value {
        serde_json::to_value(modal).expect("CreateModal must serialize")
    }

    #[test]
    fn modal_has_exactly_one_text_input() {
        let modal = answer_modal(AnswerRoute::new(1, 0));
        let json = modal_json(&modal);
        let rows = json["components"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        let components = rows[0]["components"].as_array().unwrap();
        assert_eq!(components.len(), 1);
    }

    #[test]
    fn modal_custom_id_round_trips_to_the_route() {
        let route = AnswerRoute::new(42, 5);
        let modal = answer_modal(route);
        let json = modal_json(&modal);
        let custom_id = json["custom_id"].as_str().unwrap();
        assert_eq!(parse_answer_modal(custom_id), Ok(route));
    }

    #[test]
    fn input_text_uses_the_shared_custom_id() {
        let modal = answer_modal(AnswerRoute::new(1, 0));
        let json = modal_json(&modal);
        let input = &json["components"][0]["components"][0];
        assert_eq!(input["custom_id"], ANSWER_INPUT_CUSTOM_ID);
    }

    #[test]
    fn input_text_is_required() {
        let modal = answer_modal(AnswerRoute::new(1, 0));
        let json = modal_json(&modal);
        let input = &json["components"][0]["components"][0];
        assert_eq!(input["required"], true);
    }
}
