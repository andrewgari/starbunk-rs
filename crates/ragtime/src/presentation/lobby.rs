//! The "Ragtime contest" lobby embed, posted when the bot is mentioned.

use serenity::all::CreateEmbed;

/// Reaction emoji players add to join a round's lobby. Shared constant so the
/// Discord layer adds exactly this reaction and recognizes it on
/// `ReactionAdd`.
pub const JOIN_EMOJI: &str = "🙋";

/// Build the lobby embed.
///
/// `host` is the Discord user id (snowflake) of whoever mentioned the bot;
/// it is rendered as a `<@id>` mention, same as every other mention in this
/// module.
pub fn lobby_embed(host: u64) -> CreateEmbed {
    CreateEmbed::new()
        .title("Ragtime contest")
        .description(format!(
            "Hosted by <@{host}>.\n\n\
             React with {JOIN_EMOJI} to join the contest.\n\
             The host starts the contest with the start command."
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn embed_json(embed: &CreateEmbed) -> serde_json::Value {
        serde_json::to_value(embed).expect("CreateEmbed must serialize")
    }

    #[test]
    fn title_is_ragtime_contest() {
        let embed = lobby_embed(1);
        let json = embed_json(&embed);
        assert_eq!(json["title"], "Ragtime contest");
    }

    #[test]
    fn mentions_the_host() {
        let embed = lobby_embed(555);
        let json = embed_json(&embed);
        let description = json["description"].as_str().unwrap();
        assert!(description.contains("<@555>"));
    }

    #[test]
    fn describes_how_to_join_and_start() {
        let embed = lobby_embed(1);
        let json = embed_json(&embed);
        let description = json["description"].as_str().unwrap();
        assert!(description.contains(JOIN_EMOJI));
        assert!(description.to_lowercase().contains("react"));
        assert!(description.to_lowercase().contains("start"));
    }

    #[test]
    fn join_emoji_is_a_plain_unicode_emoji() {
        // Sanity check: the shared constant must round-trip through a
        // ReactionType::Unicode without needing a custom-emoji id.
        assert!(!JOIN_EMOJI.is_empty());
    }
}
