use super::identity::Identity;
use super::webhook_service::WebhookService;
use anyhow::Context as _;
use async_trait::async_trait;
use serenity::all::{
    ChannelId, CreateMessage, EditMessage, Http, Message, MessageId, MessageReference,
};
use std::sync::Arc;

/// Caller-facing send API. Callers declare what to send and as whom;
/// the implementation decides how to deliver it.
#[async_trait]
pub trait MessageService: Send + Sync {
    async fn send(&self, channel_id: ChannelId, content: &str) -> anyhow::Result<Message>;
    async fn send_with_identity(
        &self,
        channel_id: ChannelId,
        content: &str,
        identity: Identity,
    ) -> anyhow::Result<Message>;
    async fn reply(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
    ) -> anyhow::Result<Message>;
    async fn edit(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
    ) -> anyhow::Result<Message>;
    async fn delete(&self, channel_id: ChannelId, message_id: MessageId) -> anyhow::Result<()>;
    async fn close(&self);
}

pub struct DiscordMessageService {
    http: Arc<Http>,
    webhooks: Arc<WebhookService>,
}

impl DiscordMessageService {
    pub fn new(http: Arc<Http>, webhooks: Arc<WebhookService>) -> Self {
        Self { http, webhooks }
    }
}

#[async_trait]
impl MessageService for DiscordMessageService {
    async fn send(&self, channel_id: ChannelId, content: &str) -> anyhow::Result<Message> {
        channel_id
            .say(&self.http, content)
            .await
            .context("send failed")
    }

    async fn send_with_identity(
        &self,
        channel_id: ChannelId,
        content: &str,
        identity: Identity,
    ) -> anyhow::Result<Message> {
        self.webhooks.execute(channel_id, content, &identity).await
    }

    async fn reply(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
    ) -> anyhow::Result<Message> {
        channel_id
            .send_message(
                &self.http,
                CreateMessage::new()
                    .content(content)
                    .reference_message(MessageReference::from((channel_id, message_id))),
            )
            .await
            .context("reply failed")
    }

    async fn edit(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
    ) -> anyhow::Result<Message> {
        channel_id
            .edit_message(&self.http, message_id, EditMessage::new().content(content))
            .await
            .context("edit failed")
    }

    async fn delete(&self, channel_id: ChannelId, message_id: MessageId) -> anyhow::Result<()> {
        channel_id
            .delete_message(&self.http, message_id)
            .await
            .context("delete failed")
    }

    async fn close(&self) {
        self.webhooks.close().await;
    }
}
