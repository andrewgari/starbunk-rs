use crate::manager::QueueItem;
use serenity::all::{ButtonStyle, CreateActionRow, CreateButton, CreateEmbed};

pub fn create_now_playing_embed(track: &QueueItem) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title("Now Playing")
        .description(format!("**{}**", track.title))
        .field("Requested By", &track.requester, true);

    if let Some(dur) = track.duration {
        embed = embed.field(
            "Duration",
            format!("{}:{:02}", dur.as_secs() / 60, dur.as_secs() % 60),
            true,
        );
    }

    if let Some(ref thumb) = track.thumbnail_url {
        embed = embed.thumbnail(thumb);
    } else {
        embed = embed.thumbnail("https://cdn.discordapp.com/embed/avatars/0.png");
    }

    embed
}

pub fn create_buttons() -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        CreateButton::new("djcova_stop")
            .label("Stop")
            .style(ButtonStyle::Danger),
        CreateButton::new("djcova_skip")
            .label("Skip")
            .style(ButtonStyle::Primary),
        CreateButton::new("djcova_restart")
            .label("Restart")
            .style(ButtonStyle::Secondary),
        CreateButton::new("djcova_requeue")
            .label("Re-queue")
            .style(ButtonStyle::Secondary),
    ])
}
