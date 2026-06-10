use super::identity::Identity;
use anyhow::{anyhow, Context as _};
use serenity::all::{ChannelId, CreateWebhook, ExecuteWebhook, Http, Message, Webhook};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const WEBHOOK_NAME: &str = "Starbunk Webhook";
const WEBHOOK_TTL: Duration = Duration::from_secs(5 * 60);
const REAPER_INTERVAL: Duration = Duration::from_secs(60);

struct ChannelEntry {
    webhook: Webhook,
    last_used: Instant,
}

struct Inner {
    http: Arc<Http>,
    entries: Mutex<HashMap<ChannelId, ChannelEntry>>,
}

/// Manages per-channel Discord webhooks: lazy creation, caching, and idle cleanup.
pub struct WebhookService {
    inner: Arc<Inner>,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
}

impl WebhookService {
    /// Create a new WebhookService and spawn the background reaper.
    /// Must be called from within a tokio runtime context.
    pub fn new(http: Arc<Http>) -> Self {
        let inner = Arc::new(Inner {
            http,
            entries: Mutex::new(HashMap::new()),
        });

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);

        let inner_clone = inner.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(REAPER_INTERVAL);
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        inner_clone.reap(false).await;
                    }
                    _ = shutdown_rx.changed() => {
                        inner_clone.reap(true).await;
                        break;
                    }
                }
            }
        });

        Self { inner, shutdown_tx }
    }

    /// Execute a webhook in `channel_id` with content appearing as `identity`.
    /// Identity must be valid (username and avatar_url non-empty).
    pub async fn execute(
        &self,
        channel_id: ChannelId,
        content: &str,
        identity: &Identity,
    ) -> anyhow::Result<Message> {
        if !identity.is_valid() {
            return Err(anyhow!(
                "webhook: Identity.username and avatar_url are required"
            ));
        }

        let webhook = self.get_or_create(channel_id).await?;

        let params = ExecuteWebhook::new()
            .content(content)
            .username(&identity.username)
            .avatar_url(&identity.avatar_url);

        let msg = webhook
            .execute(&self.inner.http, true, params)
            .await
            .context("webhook execute failed")?
            .ok_or_else(|| anyhow!("webhook: no message returned"))?;

        // Reset idle timer after confirmed send.
        let mut entries = self.inner.entries.lock().await;
        if let Some(entry) = entries.get_mut(&channel_id) {
            entry.last_used = Instant::now();
        }

        Ok(msg)
    }

    /// Signal the reaper to delete all webhooks and stop.
    pub async fn close(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    async fn get_or_create(&self, channel_id: ChannelId) -> anyhow::Result<Webhook> {
        let mut entries = self.inner.entries.lock().await;

        if let Some(entry) = entries.get_mut(&channel_id) {
            entry.last_used = Instant::now();
            return Ok(entry.webhook.clone());
        }

        let webhook = fetch_or_create(&self.inner.http, channel_id).await?;
        entries.insert(
            channel_id,
            ChannelEntry {
                webhook: webhook.clone(),
                last_used: Instant::now(),
            },
        );
        Ok(webhook)
    }
}

async fn fetch_or_create(http: &Http, channel_id: ChannelId) -> anyhow::Result<Webhook> {
    let webhooks = channel_id
        .webhooks(http)
        .await
        .context("failed to list channel webhooks")?;

    if let Some(wh) = webhooks
        .into_iter()
        .find(|w| w.name.as_deref() == Some(WEBHOOK_NAME))
    {
        return Ok(wh);
    }

    channel_id
        .create_webhook(http, CreateWebhook::new(WEBHOOK_NAME))
        .await
        .context("failed to create webhook")
}

impl Inner {
    async fn reap(&self, all: bool) {
        let deadline = Instant::now() - WEBHOOK_TTL;
        let mut entries = self.entries.lock().await;

        let stale: Vec<ChannelId> = entries
            .iter()
            .filter(|(_, e)| all || e.last_used < deadline)
            .map(|(id, _)| *id)
            .collect();

        for channel_id in stale {
            if let Some(entry) = entries.remove(&channel_id) {
                if let Err(e) = entry.webhook.delete(&self.http).await {
                    tracing::warn!(
                        channel = %channel_id,
                        webhook_id = %entry.webhook.id,
                        "failed to delete idle webhook: {}",
                        e
                    );
                }
            }
        }
    }
}
