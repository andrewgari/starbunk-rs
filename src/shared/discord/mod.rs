pub mod identity;
pub mod message_service;
pub mod webhook_service;

pub use identity::Identity;
pub use message_service::{DiscordMessageService, MessageService};
pub use webhook_service::WebhookService;

use async_trait::async_trait;
use serenity::all::{GuildId, Http, UserId};
use std::sync::Arc;

/// Resolves a Discord user identity on demand.
#[async_trait]
pub trait IdentityProvider: Send + Sync {
    async fn get_identity(
        &self,
        user_id: UserId,
        guild_id: Option<GuildId>,
    ) -> anyhow::Result<Identity>;
}

/// Discord-backed identity provider. Queries Discord for guild-member details
/// (nick, server avatar) when a guild_id is provided, falling back to the
/// global user profile.
pub struct DiscordIdentityProvider {
    http: Arc<Http>,
}

impl DiscordIdentityProvider {
    pub fn new(http: Arc<Http>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl IdentityProvider for DiscordIdentityProvider {
    /// Query Discord for `user_id`. When `guild_id` is provided it prefers
    /// guild-member details (nick, server avatar) over the global profile.
    async fn get_identity(
        &self,
        user_id: UserId,
        guild_id: Option<GuildId>,
    ) -> anyhow::Result<Identity> {
        if let Some(gid) = guild_id {
            if let Ok(member) = gid.member(&self.http, user_id).await {
                let avatar_url = member
                    .avatar_url()
                    .unwrap_or_else(|| member.user.face());
                return Ok(Identity {
                    nickname: member.nick.unwrap_or_default(),
                    username: member.user.name.clone(),
                    avatar_url,
                    ..Default::default()
                });
            }
        }

        let user = user_id.to_user(&self.http).await?;
        Ok(Identity {
            username: user.name.clone(),
            avatar_url: user.face(),
            ..Default::default()
        })
    }
}
