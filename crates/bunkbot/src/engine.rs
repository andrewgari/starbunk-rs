use crate::config::{BotConfig, ConditionNode, IdentityConfig};
use crate::template::resolve_template;
use rand::Rng;
use regex::Regex;
use serenity::all::{Context, Message, UserId};
use starbunk::discord::{Identity, IdentityProvider, MessageService};
use std::sync::{Arc, LazyLock};

/// YAML-driven reply bot engine.
///
/// Iterates all loaded `BotConfig` entries for every incoming message.
/// Each bot applies its own author filters, frequency gate, and trigger
/// conditions. The first trigger that matches fires its response; remaining
/// triggers for that bot are skipped.
pub struct BunkBotEngine {
    bots: Vec<BotConfig>,
    sender: Arc<dyn MessageService>,
    identity_provider: Arc<dyn IdentityProvider>,
}

impl BunkBotEngine {
    pub fn new(
        bots: Vec<BotConfig>,
        sender: Arc<dyn MessageService>,
        identity_provider: Arc<dyn IdentityProvider>,
    ) -> Self {
        Self {
            bots,
            sender,
            identity_provider,
        }
    }

    #[tracing::instrument(skip(self, ctx, msg), fields(channel = %msg.channel_id))]
    pub async fn handle(&self, ctx: &Context, msg: &Message, self_id: UserId) {
        for bot in &self.bots {
            if !should_process(bot, msg, self_id) {
                continue;
            }
            self.dispatch_bot(ctx, msg, bot).await;
        }
    }

    async fn dispatch_bot(&self, ctx: &Context, msg: &Message, bot: &BotConfig) {
        for trigger in &bot.triggers {
            if !eval_condition(&trigger.conditions, msg) {
                continue;
            }

            // Trigger-level pool wins over bot-level pool.
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
            }

            return; // first matching trigger wins
        }
    }
}

/// Returns `true` if this bot should process the message, based on author
/// filters (`ignore_self`, `ignore_bots`, `ignore_humans`) and the bot's
/// fire-rate `frequency` (0–100%).
fn should_process(bot: &BotConfig, msg: &Message, self_id: UserId) -> bool {
    if bot.ignore_self && msg.author.id == self_id {
        return false;
    }
    if bot.ignore_bots && msg.author.bot {
        return false;
    }
    if bot.ignore_humans && !msg.author.bot {
        return false;
    }
    // frequency: 100 → always fire, 0 → never fire.
    if bot.frequency < 100 {
        let roll: u8 = rand::thread_rng().gen_range(0..100);
        if roll >= bot.frequency {
            return false;
        }
    }
    true
}

/// Evaluate a `ConditionNode` tree against a Discord message.
/// URLs are stripped from the content before phrase/word/regex matching.
fn eval_condition(node: &ConditionNode, msg: &Message) -> bool {
    let stripped = strip_urls(&msg.content);
    eval_node(node, msg, &stripped)
}

fn eval_node(node: &ConditionNode, msg: &Message, stripped: &str) -> bool {
    match node {
        ConditionNode::ContainsPhrase(phrase) => {
            stripped.to_lowercase().contains(&phrase.to_lowercase())
        }
        ConditionNode::ContainsWord(word) => {
            let pattern = format!(r"(?i)\b{}\b", regex::escape(word));
            Regex::new(&pattern)
                .map(|re| re.is_match(stripped))
                .unwrap_or(false)
        }
        ConditionNode::MatchesRegex(pattern) => Regex::new(pattern)
            .map(|re| re.is_match(stripped))
            .unwrap_or(false),
        ConditionNode::FromUser(id) => msg.author.id.to_string() == id.as_ref(),
        ConditionNode::WithChance(pct) => rand::thread_rng().gen_range(0u8..100) < *pct,
        ConditionNode::Always(b) => *b,
        ConditionNode::AllOf(children) => children.iter().all(|c| eval_node(c, msg, stripped)),
        ConditionNode::AnyOf(children) => children.iter().any(|c| eval_node(c, msg, stripped)),
        ConditionNode::NoneOf(children) => children.iter().all(|c| !eval_node(c, msg, stripped)),
    }
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

/// Resolve a bot's `IdentityConfig` into a sendable `Identity`.
///
/// Returns `None` for `Random` when not in a guild or guild has no cached
/// members — the caller falls back to a plain (non-webhook) send in that case.
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
            // Extract member data while holding the cache lock, drop before any await.
            let (username, avatar_url) = {
                let guild = ctx.cache.guild(guild_id)?;
                let members: Vec<_> = guild.members.values().collect();
                if members.is_empty() {
                    return None;
                }
                let idx = rand::thread_rng().gen_range(0..members.len());
                let m = members[idx];
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
