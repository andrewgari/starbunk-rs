pub mod identity;
pub mod message_service;
pub mod webhook_service;

pub use identity::Identity;
pub use message_service::{DiscordMessageService, MessageService};
pub use webhook_service::WebhookService;

use serenity::all::{GuildId, Http, UserId};
use std::sync::Arc;

/// Resolves a Discord user's identity on demand.
pub struct IdentityProvider {
    http: Arc<Http>,
}

impl IdentityProvider {
    pub fn new(http: Arc<Http>) -> Self {
        Self { http }
    }

    /// Query Discord for `user_id`. When `guild_id` is provided it prefers
    /// guild-member details (nick, server avatar) over the global profile.
    pub async fn get_identity(
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
