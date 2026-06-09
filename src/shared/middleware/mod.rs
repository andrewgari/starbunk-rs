//! Composable gate primitives for evaluating Discord messages.
//!
//! Every gate implements [`MessageFilter`], so any combination can be
//! composed freely using [`all_of`], [`any_of`], and [`not`]:
//!
//! ```no_run
//! use starbunk::middleware::{all_of, not, NOT_BOT, NOT_SELF, GUILD_ONLY, HAS_CONTENT};
//! let filter = all_of(vec![NOT_SELF, NOT_BOT, GUILD_ONLY, HAS_CONTENT]);
//! ```

pub mod author;
pub mod content;
pub mod context;
pub mod random;

use serenity::all::{Context, Message};
use std::sync::Arc;

/// Gate for an incoming Discord message. Returns `true` to allow processing,
/// `false` to drop silently.
pub trait MessageFilter: Send + Sync {
    fn check(&self, ctx: &Context, msg: &Message) -> bool;
}

/// Passes only when every child passes. Short-circuits on first failure.
/// Passes vacuously when given no children.
pub fn all_of(filters: Vec<Arc<dyn MessageFilter>>) -> Arc<dyn MessageFilter> {
    Arc::new(AllOf(filters))
}

/// Passes when at least one child passes. Short-circuits on first success.
/// Fails vacuously when given no children.
pub fn any_of(filters: Vec<Arc<dyn MessageFilter>>) -> Arc<dyn MessageFilter> {
    Arc::new(AnyOf(filters))
}

/// Inverts a filter.
pub fn not(f: Arc<dyn MessageFilter>) -> Arc<dyn MessageFilter> {
    Arc::new(Not(f))
}

// Re-export the named filter constants for ergonomic use.
pub use author::{author_has_role, author_id, author_named, IS_BOT, NOT_BOT, NOT_SELF};
pub use content::{content_contains, content_matches, HAS_ATTACHMENT, HAS_CONTENT};
pub use context::{in_channel, on_weekdays, DM_ONLY, GUILD_ONLY};
pub use random::chance;

struct AllOf(Vec<Arc<dyn MessageFilter>>);
struct AnyOf(Vec<Arc<dyn MessageFilter>>);
struct Not(Arc<dyn MessageFilter>);

impl MessageFilter for AllOf {
    fn check(&self, ctx: &Context, msg: &Message) -> bool {
        self.0.iter().all(|f| f.check(ctx, msg))
    }
}

impl MessageFilter for AnyOf {
    fn check(&self, ctx: &Context, msg: &Message) -> bool {
        self.0.iter().any(|f| f.check(ctx, msg))
    }
}

impl MessageFilter for Not {
    fn check(&self, ctx: &Context, msg: &Message) -> bool {
        !self.0.check(ctx, msg)
    }
}

/// Helper: wrap a plain filter function as a [`MessageFilter`].
pub fn filter_fn(f: impl Fn(&Context, &Message) -> bool + Send + Sync + 'static) -> Arc<dyn MessageFilter> {
    Arc::new(FnFilter(Box::new(f)))
}

struct FnFilter(Box<dyn Fn(&Context, &Message) -> bool + Send + Sync>);

impl MessageFilter for FnFilter {
    fn check(&self, ctx: &Context, msg: &Message) -> bool {
        (self.0)(ctx, msg)
    }
}
