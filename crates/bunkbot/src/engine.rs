mod compiled;

use crate::config::{BotConfig, IdentityConfig};
use crate::state::BotStateService;
use crate::template::resolve_template;
use compiled::{eval, CompiledBot};
use rand::Rng;
use regex::Regex;
use serenity::all::{Context, Message, UserId};
use starbunk::discord::{Identity, IdentityProvider, MessageService};
use std::sync::{Arc, LazyLock};

/// YAML-driven reply bot engine.
///
/// Converts all `BotConfig` entries to a pre-compiled internal form during
/// `new()` (regex patterns compiled once, invalid patterns logged and skipped).
/// Each call to `handle` iterates all loaded bots, applies per-bot author
/// filters + frequency gate, evaluates trigger conditions, and sends the first
/// matching response.
use starbunk::audit::AuditStore;

pub struct BunkBotEngine {
    configs: Vec<BotConfig>,
    bots: Vec<CompiledBot>,
    sender: Arc<dyn MessageService>,
    identity_provider: Arc<dyn IdentityProvider>,
    state_service: Arc<dyn BotStateService>,
    audit: Arc<AuditStore>,
}

impl BunkBotEngine {
    pub fn new(
        bots: Vec<BotConfig>,
        sender: Arc<dyn MessageService>,
        identity_provider: Arc<dyn IdentityProvider>,
        state_service: Arc<dyn BotStateService>,
        audit: Arc<AuditStore>,
    ) -> Self {
        let configs = bots.clone();
        let compiled = bots
            .into_iter()
            .filter_map(|config| {
                let name = config.name.clone();
                match CompiledBot::try_from(config) {
                    Ok(b) => Some(b),
                    Err(e) => {
                        tracing::error!(
                            bot = %name,
                            "invalid regex in bot config, skipping: {}", e
                        );
                        None
                    }
                }
            })
            .collect();

        Self {
            configs,
            bots: compiled,
            sender,
            identity_provider,
            state_service,
            audit,
        }
    }
    pub fn reload_bots(&mut self, bots: Vec<BotConfig>) {
        let compiled = bots
            .iter()
            .filter_map(|config| {
                let name = config.name.clone();
                match CompiledBot::try_from(config.clone()) {
                    Ok(b) => Some(b),
                    Err(e) => {
                        tracing::error!(bot = %name, "failed to compile bot config: {}", e);
                        None
                    }
                }
            })
            .collect();
        self.configs = bots;
        self.bots = compiled;
    }

    pub fn bot_configs(&self) -> &[BotConfig] {
        &self.configs
    }

    pub fn active_bots(&self) -> Vec<(String, u8)> {
        self.bots
            .iter()
            .map(|b| (b.name.clone(), b.frequency))
            .collect()
    }

    pub fn state_service(&self) -> Arc<dyn BotStateService> {
        self.state_service.clone()
    }

    #[tracing::instrument(skip(self, ctx, msg), fields(channel = %msg.channel_id))]
    pub async fn handle(&self, ctx: &Context, msg: &Message, self_id: UserId) {
        for bot in &self.bots {
            if !should_process(bot, msg, self_id, &*self.state_service) {
                continue;
            }
            self.dispatch_bot(ctx, msg, bot).await;
        }
    }

    async fn dispatch_bot(&self, ctx: &Context, msg: &Message, bot: &CompiledBot) {
        // Strip URLs once per dispatch, not per trigger.
        let stripped = strip_urls(&msg.content);

        for trigger in &bot.triggers {
            if !eval(&trigger.conditions, msg, &stripped) {
                continue;
            }

            let pool = if !trigger.responses.is_empty() {
                &trigger.responses
            } else {
                &bot.responses
            };

            let Some(template) = pick_response(pool) else {
                continue;
            };

            let response = resolve_template(template, &msg.content);
            let identity =
                resolve_identity(&bot.identity, ctx, msg, &*self.identity_provider).await;

            let result = match identity {
                Some(id) => self
                    .sender
                    .send_with_identity(msg.channel_id, &response, id)
                    .await
                    .map(|_| ()),
                None => self
                    .sender
                    .send(msg.channel_id, &response)
                    .await
                    .map(|_| ()),
            };

            if let Err(e) = result {
                tracing::error!(
                    bot = %bot.name,
                    channel = %msg.channel_id,
                    "send failed: {}", e
                );
            } else {
                let _ = self
                    .audit
                    .log_event(&bot.name, &msg.content, &response, None)
                    .await;
            }

            return; // first matching trigger wins
        }
    }
}

/// Returns `true` if this bot should process the message.
///
/// Applies `ignore_self`, `ignore_bots`, `ignore_humans`, and the `frequency`
/// gate in that order.
///
/// **Webhook limitation**: `ignore_self` compares `msg.author.id` against
/// `self_id` (the bot's own Discord user ID). Messages sent via
/// `WebhookService::execute` carry the webhook's user ID instead of the bot's
/// user ID, so `ignore_self: true` does not filter the bot's own webhook
/// responses. When `ignore_bots: true` (the default), this is moot — webhook
/// messages have `author.bot = true` and are caught by `ignore_bots`. Only
/// bots that set `ignore_bots: false` are exposed to this edge case.
fn should_process(
    bot: &CompiledBot,
    msg: &Message,
    self_id: UserId,
    state_service: &dyn BotStateService,
) -> bool {
    if !state_service.is_bot_enabled(&bot.name) {
        return false;
    }
    if bot.ignore_self && msg.author.id == self_id {
        return false;
    }
    if bot.ignore_bots && msg.author.bot {
        return false;
    }
    if bot.ignore_humans && !msg.author.bot {
        return false;
    }
    // Clamp frequency so values like 200 don't silently bypass the gate.
    let frequency = state_service
        .get_frequency(&bot.name)
        .unwrap_or(bot.frequency)
        .min(100);
    if frequency < 100 {
        let roll: u8 = rand::thread_rng().gen_range(0..100);
        if roll >= frequency {
            return false;
        }
    }
    true
}

static URL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://\S+").expect("URL_RE is valid"));

fn strip_urls(text: &str) -> String {
    URL_RE.replace_all(text, "").to_string()
}

fn pick_response(pool: &[String]) -> Option<&str> {
    if pool.is_empty() {
        return None;
    }
    Some(&pool[rand::thread_rng().gen_range(0..pool.len())])
}

async fn resolve_identity(
    identity: &IdentityConfig,
    ctx: &Context,
    msg: &Message,
    provider: &dyn IdentityProvider,
) -> Option<Identity> {
    match identity {
        IdentityConfig::Static {
            bot_name,
            avatar_url,
        } => Some(Identity {
            username: bot_name.clone(),
            avatar_url: avatar_url.clone(),
            ..Default::default()
        }),

        IdentityConfig::Mimic { user_id } => {
            let uid: u64 = user_id.0.parse().ok()?;
            match provider.get_identity(UserId::new(uid), msg.guild_id).await {
                Ok(id) => Some(id),
                Err(e) => {
                    tracing::warn!(user_id = %user_id, "failed to resolve mimic user: {}", e);
                    None
                }
            }
        }

        IdentityConfig::Random => {
            let guild_id = msg.guild_id?;
            // Hold the cache lock only long enough to copy the data out.
            let (username, avatar_url) = {
                let guild = ctx.cache.guild(guild_id)?;
                let members: Vec<_> = guild.members.values().collect();
                if members.is_empty() {
                    return None;
                }
                let m = members[rand::thread_rng().gen_range(0..members.len())];
                (m.display_name().to_string(), m.face())
            };
            Some(Identity {
                username,
                avatar_url,
                ..Default::default()
            })
        }

        IdentityConfig::MimicPoster => Some(Identity {
            username: msg.author.name.clone(),
            avatar_url: msg.author.face(),
            ..Default::default()
        }),
    }
}

#[cfg(test)]
pub mod tests {
    use super::compiled::{CompiledBot, CompiledNode};
    use super::*;
    use crate::config::IdentityConfig;

    fn build_msg(content: &str, is_bot: bool, author_id: &str) -> Message {
        serde_json::from_value(serde_json::json!({
            "id": "1", "channel_id": "1",
            "author": {
                "id": author_id, "username": "user",
                "bot": is_bot, "discriminator": "0", "public_flags": 0
            },
            "content": content,
            "timestamp": "2024-01-01T12:00:00+00:00",
            "edited_timestamp": null, "tts": false, "mention_everyone": false,
            "mentions": [], "mention_roles": [], "attachments": [], "embeds": [],
            "pinned": false, "type": 0
        }))
        .expect("test message")
    }

    fn bot_cfg(
        ignore_self: bool,
        ignore_bots: bool,
        ignore_humans: bool,
        frequency: u8,
    ) -> CompiledBot {
        CompiledBot {
            name: "test".into(),
            identity: IdentityConfig::Random,
            responses: vec![],
            ignore_self,
            ignore_bots,
            ignore_humans,
            frequency,
            triggers: vec![],
        }
    }

    use crate::state::InMemoryBotStateManager;

    const SELF_ID: UserId = UserId::new(99);

    // --- should_process ---

    #[test]
    fn ignore_self_drops_own_message() {
        let bot = bot_cfg(true, false, false, 100);
        let msg = build_msg("hi", false, "99"); // author.id == SELF_ID
        let state = InMemoryBotStateManager::new();
        assert!(!should_process(&bot, &msg, SELF_ID, &state));
    }

    #[test]
    fn ignore_self_allows_other_message() {
        let bot = bot_cfg(true, false, false, 100);
        let msg = build_msg("hi", false, "1"); // author.id != SELF_ID
        let state = InMemoryBotStateManager::new();
        assert!(should_process(&bot, &msg, SELF_ID, &state));
    }

    #[test]
    fn ignore_bots_drops_bot_message() {
        let bot = bot_cfg(false, true, false, 100);
        let msg = build_msg("hi", true, "2"); // is_bot = true
        let state = InMemoryBotStateManager::new();
        assert!(!should_process(&bot, &msg, SELF_ID, &state));
    }

    #[test]
    fn ignore_bots_false_allows_bot_message() {
        let bot = bot_cfg(false, false, false, 100);
        let msg = build_msg("hi", true, "2");
        let state = InMemoryBotStateManager::new();
        assert!(should_process(&bot, &msg, SELF_ID, &state));
    }

    #[test]
    fn ignore_humans_drops_human_message() {
        let bot = bot_cfg(false, false, true, 100);
        let msg = build_msg("hi", false, "1"); // is_bot = false
        let state = InMemoryBotStateManager::new();
        assert!(!should_process(&bot, &msg, SELF_ID, &state));
    }

    #[test]
    fn frequency_0_never_fires() {
        let bot = bot_cfg(false, false, false, 0);
        let msg = build_msg("hi", false, "1");
        let state = InMemoryBotStateManager::new();
        for _ in 0..50 {
            assert!(!should_process(&bot, &msg, SELF_ID, &state));
        }
    }

    #[test]
    fn frequency_100_always_fires() {
        let bot = bot_cfg(false, false, false, 100);
        let msg = build_msg("hi", false, "1");
        let state = InMemoryBotStateManager::new();
        for _ in 0..50 {
            assert!(should_process(&bot, &msg, SELF_ID, &state));
        }
    }

    #[test]
    fn frequency_above_100_clamped_to_always_fire() {
        // A misconfigured frequency of 200 must behave like 100 (always fire),
        // not skip the gate entirely due to `200 < 100` being false.
        let bot = bot_cfg(false, false, false, 200);
        let msg = build_msg("hi", false, "1");
        let state = InMemoryBotStateManager::new();
        for _ in 0..50 {
            assert!(should_process(&bot, &msg, SELF_ID, &state));
        }
    }

    #[test]
    fn test_should_process_bot_disabled() {
        let bot = bot_cfg(false, false, false, 100);
        let msg = build_msg("hi", false, "1");
        let state = InMemoryBotStateManager::new();
        state.disable_bot(&bot.name);

        assert!(!should_process(&bot, &msg, SELF_ID, &state));
    }

    #[test]
    fn test_should_process_frequency_override() {
        // Bot defaults to 0% (never fires)
        let bot = bot_cfg(false, false, false, 0);
        let msg = build_msg("hi", false, "1");
        let state = InMemoryBotStateManager::new();

        // Override to 100% (always fires)
        state.set_frequency(&bot.name, 100, "admin", 0);

        assert!(should_process(&bot, &msg, SELF_ID, &state));
    }

    // --- strip_urls ---

    #[test]
    fn strip_urls_removes_http_link() {
        let result = strip_urls("check https://example.com out");
        assert!(!result.contains("https://"), "URL must be stripped");
        assert!(result.contains("check"), "surrounding text preserved");
    }

    #[test]
    fn strip_urls_leaves_plain_text_unchanged() {
        let text = "no links here";
        assert_eq!(strip_urls(text), text);
    }

    #[test]
    fn url_stripped_before_condition_eval() {
        // A URL containing the phrase must not trigger ContainsPhrase.
        let node = CompiledNode::ContainsPhrase("banana".into());
        let msg = build_msg("https://banana.example.com", false, "1");
        let stripped = strip_urls(&msg.content);
        assert!(!eval(&node, &msg, &stripped));
    }

    // --- pick_response ---

    #[test]
    fn pick_response_empty_returns_none() {
        assert!(pick_response(&[]).is_none());
    }

    #[test]
    fn pick_response_single_returns_it() {
        let pool = vec!["only".to_string()];
        assert_eq!(pick_response(&pool), Some("only"));
    }

    #[test]
    fn pick_response_always_within_pool() {
        let pool: Vec<String> = (0..5).map(|i| i.to_string()).collect();
        for _ in 0..100 {
            let r = pick_response(&pool).unwrap();
            assert!(pool.iter().any(|s| s == r));
        }
    }

    #[derive(Clone)]
    pub(crate) struct DummySender;
    #[async_trait::async_trait]
    impl starbunk::discord::MessageService for DummySender {
        async fn send(&self, _c: serenity::all::ChannelId, _m: &str) -> anyhow::Result<Message> {
            unimplemented!()
        }
        async fn send_with_identity(
            &self,
            _c: serenity::all::ChannelId,
            _m: &str,
            _i: starbunk::discord::Identity,
        ) -> anyhow::Result<Message> {
            unimplemented!()
        }
        async fn reply(
            &self,
            _c: serenity::all::ChannelId,
            _m: serenity::all::MessageId,
            _co: &str,
        ) -> anyhow::Result<Message> {
            unimplemented!()
        }
        async fn edit(
            &self,
            _c: serenity::all::ChannelId,
            _m: serenity::all::MessageId,
            _co: &str,
        ) -> anyhow::Result<Message> {
            unimplemented!()
        }
        async fn delete(
            &self,
            _c: serenity::all::ChannelId,
            _m: serenity::all::MessageId,
        ) -> anyhow::Result<()> {
            unimplemented!()
        }
        async fn close(&self) {}
    }

    pub(crate) struct DummyIdentity;
    #[async_trait::async_trait]
    impl starbunk::discord::IdentityProvider for DummyIdentity {
        async fn get_identity(
            &self,
            _u: UserId,
            _g: Option<serenity::all::GuildId>,
        ) -> anyhow::Result<starbunk::discord::Identity> {
            unimplemented!()
        }
    }

    // --- reload_bots ---
    #[tokio::test]
    async fn test_reload_bots_updates_internal_bots_list() {
        use std::sync::Arc;
        let mut engine = BunkBotEngine {
            configs: vec![],
            bots: vec![],
            sender: Arc::new(DummySender),
            identity_provider: Arc::new(DummyIdentity),
            state_service: Arc::new(InMemoryBotStateManager::new()),
            audit: Arc::new(starbunk::audit::AuditStore::dummy()),
        };

        let new_bot = crate::config::BotConfig {
            name: "new_bot".into(),
            identity: IdentityConfig::Random,
            ignore_self: false,
            ignore_bots: false,
            ignore_humans: false,
            frequency: 100,
            triggers: vec![],
            responses: vec!["hello".into()],
        };

        engine.reload_bots(vec![new_bot.clone()]);

        let configs = engine.bot_configs();
        // Failing condition: the stub does nothing, so configs is empty. We expect it to have new_bot
        assert_eq!(configs.len(), 1, "Expected reload_bots to add the new bot");
        assert_eq!(configs[0].name, "new_bot");
    }
}
