//! The reveal embed posted when a question closes.

use serenity::all::CreateEmbed;

/// Plain-data payload for the reveal embed. Mirrors the shape of
/// `ragtime::round::event::Reveal` without depending on that module — the
/// Discord layer maps one to the other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevealInfo {
    /// The canonical answer to show.
    pub display_answer: String,
    /// User ids of everyone who answered correctly, in the order they
    /// should be listed (join order upstream).
    pub correct_users: Vec<u64>,
}

/// Room for the mention list inside the 4096-character description limit,
/// leaving headroom for the answer text and the "...and N more" marker.
const MENTION_BUDGET: usize = 3000;

/// Build the reveal embed for a question that just closed.
pub fn reveal_embed(reveal: &RevealInfo) -> CreateEmbed {
    let who = if reveal.correct_users.is_empty() {
        "Nobody answered correctly.".to_string()
    } else {
        let mut shown: Vec<String> = Vec::new();
        let mut len = "Correct: ".len();
        for id in &reveal.correct_users {
            let m = format!("<@{id}>");
            if len + m.len() + 2 > MENTION_BUDGET {
                break;
            }
            len += m.len() + 2;
            shown.push(m);
        }
        let hidden = reveal.correct_users.len() - shown.len();
        let mut line = format!("Correct: {}", shown.join(", "));
        if hidden > 0 {
            line.push_str(&format!(" ...and {hidden} more"));
        }
        line
    };

    CreateEmbed::new().title("Reveal").description(format!(
        "The answer was **{}**.\n\n{who}",
        reveal.display_answer
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn embed_json(embed: &CreateEmbed) -> serde_json::Value {
        serde_json::to_value(embed).expect("CreateEmbed must serialize")
    }

    #[test]
    fn huge_correct_list_is_truncated_under_discord_limit() {
        let reveal = RevealInfo {
            display_answer: "Paris".to_string(),
            correct_users: (0..1000u64).map(|n| 100_000_000_000_000_000 + n).collect(),
        };
        let json = embed_json(&reveal_embed(&reveal));
        let description = json["description"].as_str().unwrap();
        assert!(description.chars().count() <= 4096);
        assert!(description.contains("more"));
    }

    #[test]
    fn shows_the_display_answer() {
        let reveal = RevealInfo {
            display_answer: "Paris".to_string(),
            correct_users: vec![],
        };
        let json = embed_json(&reveal_embed(&reveal));
        let description = json["description"].as_str().unwrap();
        assert!(description.contains("Paris"));
    }

    #[test]
    fn zero_correct_users_says_so_explicitly() {
        let reveal = RevealInfo {
            display_answer: "Paris".to_string(),
            correct_users: vec![],
        };
        let json = embed_json(&reveal_embed(&reveal));
        let description = json["description"].as_str().unwrap();
        assert!(description.to_lowercase().contains("nobody"));
        assert!(!description.contains("<@"));
    }

    #[test]
    fn one_correct_user_is_mentioned() {
        let reveal = RevealInfo {
            display_answer: "Paris".to_string(),
            correct_users: vec![111],
        };
        let json = embed_json(&reveal_embed(&reveal));
        let description = json["description"].as_str().unwrap();
        assert!(description.contains("<@111>"));
        assert!(!description.to_lowercase().contains("nobody"));
    }

    #[test]
    fn many_correct_users_are_all_mentioned_in_order() {
        let reveal = RevealInfo {
            display_answer: "Paris".to_string(),
            correct_users: vec![1, 2, 3],
        };
        let json = embed_json(&reveal_embed(&reveal));
        let description = json["description"].as_str().unwrap();
        let pos1 = description.find("<@1>").unwrap();
        let pos2 = description.find("<@2>").unwrap();
        let pos3 = description.find("<@3>").unwrap();
        assert!(pos1 < pos2 && pos2 < pos3);
    }
}
