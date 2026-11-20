//! serenity implementation of [`Outbox`].

use super::outbox::Outbox;
use crate::presentation::{
    lobby_embed, question::question_action_row, question_embed, reveal_embed, tally_embed,
    AnswerRoute, RevealInfo, TallyRow, JOIN_EMOJI,
};
use serenity::all::{
    ChannelId, CreateAllowedMentions, CreateMessage, Http, MessageId, ReactionType,
};
use std::sync::Arc;

/// Posts to Discord channels over the REST API.
#[derive(Debug, Clone)]
pub struct DiscordOutbox {
    http: Arc<Http>,
}

impl DiscordOutbox {
    pub fn new(http: Arc<Http>) -> Self {
        Self { http }
    }

    async fn send(&self, channel: u64, msg: CreateMessage) -> anyhow::Result<u64> {
        let sent = ChannelId::new(channel)
            .send_message(&self.http, msg)
            .await?;
        Ok(sent.id.get())
    }
}

#[async_trait::async_trait]
impl Outbox for DiscordOutbox {
    async fn post_lobby(&self, channel: u64, host: u64) -> anyhow::Result<u64> {
        self.send(channel, CreateMessage::new().embed(lobby_embed(host)))
            .await
    }

    async fn add_join_reaction(&self, channel: u64, message: u64) -> anyhow::Result<()> {
        ChannelId::new(channel)
            .create_reaction(
                &self.http,
                MessageId::new(message),
                ReactionType::Unicode(JOIN_EMOJI.to_string()),
            )
            .await?;
        Ok(())
    }

    async fn post_question(
        &self,
        channel: u64,
        index: usize,
        total: usize,
        text: &str,
    ) -> anyhow::Result<()> {
        let msg = CreateMessage::new()
            .embed(question_embed(index, total, text))
            .components(vec![question_action_row(AnswerRoute::new(channel, index))]);
        self.send(channel, msg).await.map(|_| ())
    }

    async fn post_reveal(&self, channel: u64, reveal: RevealInfo) -> anyhow::Result<()> {
        self.send(channel, CreateMessage::new().embed(reveal_embed(&reveal)))
            .await
            .map(|_| ())
    }

    async fn post_tally(&self, channel: u64, standings: Vec<TallyRow>) -> anyhow::Result<()> {
        self.send(channel, CreateMessage::new().embed(tally_embed(&standings)))
            .await
            .map(|_| ())
    }

    async fn post_notice(&self, channel: u64, text: &str) -> anyhow::Result<()> {
        let msg = CreateMessage::new()
            .content(text)
            .allowed_mentions(CreateAllowedMentions::new());
        self.send(channel, msg).await.map(|_| ())
    }
}
