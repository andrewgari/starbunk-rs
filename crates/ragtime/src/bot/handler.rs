//! The serenity [`EventHandler`] for Ragtime.

use super::commands::{register_commands, RagtimeCommand};
use super::discord_outbox::DiscordOutbox;
use super::engine::{ButtonOutcome, MentionOutcome, RagtimeEngine};
use super::interactions::{answer_text, edit, ephemeral};
use super::replies;
use crate::presentation::{
    answer_modal, custom_id::parse_button, modal::parse_answer_modal, JOIN_EMOJI,
};
use crate::repository::QuestionRepository;
use crate::round::Grader;
use serenity::all::{
    CommandInteraction, ComponentInteraction, Context, CreateInteractionResponse, CreateMessage,
    EventHandler, Interaction, Message, ModalInteraction, Reaction, ReactionType, Ready, UserId,
};
use starbunk::discord::StarbunkMessage;
use std::sync::{Arc, OnceLock};

/// Discord event handler. The engine is built on the first `ready`, once the
/// client's HTTP handle exists.
pub struct RagtimeHandler {
    repo: Arc<dyn QuestionRepository>,
    grader: Arc<dyn Grader>,
    engine: OnceLock<Arc<RagtimeEngine>>,
}

impl std::fmt::Debug for RagtimeHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RagtimeHandler").finish_non_exhaustive()
    }
}

impl RagtimeHandler {
    pub fn new(repo: Arc<dyn QuestionRepository>, grader: Arc<dyn Grader>) -> Self {
        Self {
            repo,
            grader,
            engine: OnceLock::new(),
        }
    }

    fn engine(&self, ctx: &Context) -> &Arc<RagtimeEngine> {
        self.engine.get_or_init(|| {
            RagtimeEngine::new(
                Arc::clone(&self.repo),
                Arc::new(DiscordOutbox::new(Arc::clone(&ctx.http))),
                Arc::clone(&self.grader),
            )
        })
    }
}

/// Whether `content` contains a direct `<@id>` / `<@!id>` mention of `me`
/// (so reply-pings and @everyone don't open lobbies).
fn mentions_directly(content: &str, me: UserId) -> bool {
    content.contains(&format!("<@{me}>")) || content.contains(&format!("<@!{me}>"))
}

impl RagtimeHandler {
    #[tracing::instrument(skip(self, ctx, command), fields(channel = %command.channel_id))]
    async fn on_command(&self, ctx: &Context, command: &CommandInteraction) {
        let Some(sub) = RagtimeCommand::from_interaction(command) else {
            return;
        };
        // Start loads questions and posts question 1; defer so we never miss
        // Discord's 3-second response window.
        if let Err(e) = command.defer_ephemeral(&ctx.http).await {
            tracing::error!(bot = "ragtime", err = %e, "failed to defer command");
            return;
        }
        let (channel, user) = (command.channel_id.get(), command.user.id.get());
        let engine = self.engine(ctx);
        let reply = match sub {
            RagtimeCommand::Start => engine.start(channel, user).await,
            RagtimeCommand::Cancel => engine.cancel(channel, user).await,
        };
        if let Err(e) = command.edit_response(&ctx.http, edit(reply)).await {
            tracing::error!(bot = "ragtime", err = %e, "failed to answer command");
        }
    }

    #[tracing::instrument(skip(self, ctx, component), fields(channel = %component.channel_id))]
    async fn on_component(&self, ctx: &Context, component: &ComponentInteraction) {
        let Ok(route) = parse_button(&component.data.custom_id) else {
            return;
        };
        let response = match self
            .engine(ctx)
            .answer_button(route, component.user.id.get())
            .await
        {
            ButtonOutcome::OpenModal(route) => {
                CreateInteractionResponse::Modal(answer_modal(route))
            }
            ButtonOutcome::Ephemeral(text) => ephemeral(text),
        };
        if let Err(e) = component.create_response(&ctx.http, response).await {
            tracing::error!(bot = "ragtime", err = %e, "failed to answer button");
        }
    }

    #[tracing::instrument(skip(self, ctx, modal), fields(channel = %modal.channel_id))]
    async fn on_modal(&self, ctx: &Context, modal: &ModalInteraction) {
        let Ok(route) = parse_answer_modal(&modal.data.custom_id) else {
            return;
        };
        if let Err(e) = modal.defer_ephemeral(&ctx.http).await {
            tracing::error!(bot = "ragtime", err = %e, "failed to defer modal");
            return;
        }
        let text = answer_text(&modal.data.components).unwrap_or_default();
        let reply = self
            .engine(ctx)
            .submit_answer(route, modal.user.id.get(), &text)
            .await;
        if let Err(e) = modal.edit_response(&ctx.http, edit(reply)).await {
            tracing::error!(bot = "ragtime", err = %e, "failed to acknowledge answer");
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for RagtimeHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!(bot = "ragtime", user = %ready.user.name, "Ragtime connected");
        self.engine(&ctx);
        if let Err(e) = register_commands(&ctx).await {
            tracing::error!(bot = "ragtime", err = %e, "failed to register commands");
        }
    }

    #[tracing::instrument(skip(self, ctx, msg), fields(channel = %msg.channel_id))]
    async fn message(&self, ctx: Context, msg: Message) {
        let me = ctx.cache.current_user().id;
        // `from_serenity` classifies the author: real users stay `User`, and in
        // E2E mode the runner's webhook posts (`[E2E_HUMAN]` / `[E2E_BOT]` /
        // `[E2E_MOCK_USER:id]`) are unwrapped to `E2eUser` / `E2eBot` with the
        // prefix stripped. Outside E2E mode every bot/webhook stays `Bot`.
        let msg = StarbunkMessage::from_serenity(msg);
        if msg.sender.is_bot() || msg.author.id == me || msg.guild_id.is_none() {
            return;
        }
        if !mentions_directly(&msg.content, me) {
            return;
        }
        let channel = msg.channel_id.get();
        match self
            .engine(&ctx)
            .mention(channel, msg.author.id.get())
            .await
        {
            Ok(MentionOutcome::Created { .. }) => {}
            Ok(MentionOutcome::AlreadyRunning) => {
                let reply = CreateMessage::new()
                    .content(replies::ALREADY_RUNNING)
                    .reference_message(&msg.inner);
                if let Err(e) = msg.channel_id.send_message(&ctx.http, reply).await {
                    tracing::error!(bot = "ragtime", err = %e, "failed to reply to mention");
                }
            }
            Err(e) => tracing::error!(bot = "ragtime", err = %e, "failed to open lobby"),
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        if !matches!(&reaction.emoji, ReactionType::Unicode(e) if e == JOIN_EMOJI) {
            return;
        }
        let Some(user) = reaction.user_id else {
            return;
        };
        if user == ctx.cache.current_user().id {
            return;
        }
        let is_bot = match &reaction.member {
            Some(member) => member.user.bot,
            None => match reaction.user(&ctx).await {
                Ok(u) => u.bot,
                Err(e) => {
                    tracing::warn!(bot = "ragtime", err = %e, "could not resolve reacting user");
                    return;
                }
            },
        };
        if is_bot {
            return;
        }
        let joined = self
            .engine(&ctx)
            .join_reaction(
                reaction.channel_id.get(),
                reaction.message_id.get(),
                user.get(),
            )
            .await;
        tracing::debug!(bot = "ragtime", channel = %reaction.channel_id, %user, joined, "join reaction");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => self.on_command(&ctx, &command).await,
            Interaction::Component(component) => self.on_component(&ctx, &component).await,
            Interaction::Modal(modal) => self.on_modal(&ctx, &modal).await,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_direct_mentions_count() {
        let me = UserId::new(42);
        assert!(mentions_directly("<@42> trivia?", me));
        assert!(mentions_directly("hey <@!42>", me));
        assert!(!mentions_directly("<@421> hi", me));
        assert!(!mentions_directly("@everyone", me));
        assert!(!mentions_directly("<@43>", me));
    }
}
