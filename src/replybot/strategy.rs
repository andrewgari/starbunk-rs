use crate::discord::Identity;
use crate::middleware::MessageFilter;
use async_trait::async_trait;
use serenity::all::{Context, Message};
use std::sync::Arc;

/// Core extensibility seam for reply bots. Implement this to add a trigger
/// mechanism (regex, keyword, LLM) or a response style (static, random, LLM).
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Identifies this strategy in logs.
    fn name(&self) -> &str;

    /// Returns true when this strategy wants to respond to `msg`.
    async fn should_trigger(&self, ctx: &Context, msg: &Message) -> bool;

    /// Returns the text to send. Only called after `should_trigger` returns true.
    async fn response(&self, ctx: &Context, msg: &Message) -> String;

    /// Optional pre-condition. When Some, the Bot checks this before
    /// `should_trigger`. Messages that fail are silently skipped.
    fn condition(&self) -> Option<&dyn MessageFilter> {
        None
    }

    /// Optional persona. When Some, the response is sent via webhook with
    /// the returned identity instead of the bot's own account.
    async fn identity(&self, _ctx: &Context, _msg: &Message) -> Option<Identity> {
        None
    }
}

/// Wraps any Strategy with a pre-condition filter without modifying the
/// strategy struct itself. Returned condition takes precedence over Bot-level
/// filtering but runs before `should_trigger`.
pub struct WithCondition<S> {
    condition: Arc<dyn MessageFilter>,
    inner: S,
}

impl<S: Strategy> WithCondition<S> {
    pub fn new(condition: Arc<dyn MessageFilter>, inner: S) -> Self {
        Self { condition, inner }
    }
}

#[async_trait]
impl<S: Strategy + 'static> Strategy for WithCondition<S> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn condition(&self) -> Option<&dyn MessageFilter> {
        Some(&*self.condition)
    }

    async fn should_trigger(&self, ctx: &Context, msg: &Message) -> bool {
        self.inner.should_trigger(ctx, msg).await
    }

    async fn response(&self, ctx: &Context, msg: &Message) -> String {
        self.inner.response(ctx, msg).await
    }

    async fn identity(&self, ctx: &Context, msg: &Message) -> Option<Identity> {
        self.inner.identity(ctx, msg).await
    }
}
