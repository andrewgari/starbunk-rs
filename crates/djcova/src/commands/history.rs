use crate::manager::GuildAudioManager;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn history_command() -> CreateCommand {
    CreateCommand::new("history").description("Show session playback history")
}

pub async fn handle(
    ctx: &Context,
    cmd: &CommandInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    let m = mgr.lock().await;
    let history = m.get_history();

    if history.is_empty() {
        let _ = cmd
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("No history yet this session."),
                ),
            )
            .await;
    } else {
        let mut list = history
            .iter()
            .enumerate()
            .map(|(i, item)| {
                format!(
                    "{}. {} (requested by {})\n",
                    i + 1,
                    item.title,
                    item.requester
                )
            })
            .collect::<String>();

        // Discord embed description limit is 4096 characters.
        if list.len() > 4000 {
            list.truncate(4000);
            list.push_str("\n… (history truncated)");
        }

        let embed = CreateEmbed::new().title("History").description(list);
        let _ = cmd
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().embed(embed),
                ),
            )
            .await;
    }

    Ok(())
}
