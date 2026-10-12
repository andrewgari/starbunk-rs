//! The per-question embed and its \[Answer\] button.

use serenity::all::{ButtonStyle, CreateActionRow, CreateButton, CreateEmbed};

use super::custom_id::{encode_button, AnswerRoute};

/// Build the embed for question `index` (0-based) of `total`.
pub fn question_embed(index: usize, total: usize, question_text: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title(format!("Question {}/{total}", index + 1))
        .description(question_text)
}

/// Build the \[Answer\] button for question `route.index` of the round
/// running in channel `route.channel`.
pub fn answer_button(route: AnswerRoute) -> CreateButton {
    CreateButton::new(encode_button(route))
        .label("Answer")
        .style(ButtonStyle::Primary)
}

/// Convenience: `answer_button` wrapped in a single-button action row, ready
/// to attach to the question message.
pub fn question_action_row(route: AnswerRoute) -> CreateActionRow {
    CreateActionRow::Buttons(vec![answer_button(route)])
}

#[cfg(test)]
mod tests {
    use super::super::custom_id::parse_button;
    use super::*;

    fn embed_json(embed: &CreateEmbed) -> serde_json::Value {
        serde_json::to_value(embed).expect("CreateEmbed must serialize")
    }

    fn button_json(button: &CreateButton) -> serde_json::Value {
        serde_json::to_value(button).expect("CreateButton must serialize")
    }

    #[test]
    fn title_shows_one_based_question_number_and_total() {
        let embed = question_embed(0, 20, "What is the capital of France?");
        let json = embed_json(&embed);
        assert_eq!(json["title"], "Question 1/20");
    }

    #[test]
    fn title_tracks_the_index_correctly() {
        let embed = question_embed(19, 20, "last one");
        let json = embed_json(&embed);
        assert_eq!(json["title"], "Question 20/20");
    }

    #[test]
    fn description_is_the_formatted_question_text() {
        let embed = question_embed(0, 1, "What is 2 + 2?");
        let json = embed_json(&embed);
        assert_eq!(json["description"], "What is 2 + 2?");
    }

    #[test]
    fn answer_button_has_a_label() {
        let button = answer_button(AnswerRoute::new(1, 0));
        let json = button_json(&button);
        assert_eq!(json["label"], "Answer");
    }

    #[test]
    fn answer_button_custom_id_round_trips_to_the_route() {
        let route = AnswerRoute::new(987, 3);
        let button = answer_button(route);
        let json = button_json(&button);
        let custom_id = json["custom_id"].as_str().unwrap();
        assert_eq!(parse_button(custom_id), Ok(route));
    }

    #[test]
    fn action_row_contains_exactly_the_answer_button() {
        let route = AnswerRoute::new(1, 0);
        let row = question_action_row(route);
        match row {
            CreateActionRow::Buttons(buttons) => assert_eq!(buttons.len(), 1),
            _ => panic!("expected a Buttons action row"),
        }
    }
}
