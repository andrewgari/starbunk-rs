use crate::manager::GuildAudioManager;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn queue_command() -> CreateCommand {
    CreateCommand::new("queue").description("Show the current playback queue")
}

pub async fn handle(
    ctx: &Context,
    cmd: &CommandInteraction,
    mgr: Arc<Mutex<GuildAudioManager>>,
) -> anyhow::Result<()> {
    let m = mgr.lock().await;
    let queue = m.get_queue();

    if queue.is_empty() {
        let _ = cmd
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("The queue is empty."),
                ),
            )
            .await;
    } else {
        let list = queue
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

        let embed = CreateEmbed::new().title("Queue").description(list);
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
