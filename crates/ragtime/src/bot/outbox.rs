//! The engine's only way to talk to Discord.

use crate::presentation::{RevealInfo, TallyRow};

/// Channel-level posts the engine performs. Interaction responses (ephemeral
/// replies, modals) are not here: the engine returns them as values and the
/// handler sends them on the interaction.
///
/// `DiscordOutbox` is the serenity implementation; tests use a recording fake.
#[async_trait::async_trait]
pub trait Outbox: Send + Sync {
    /// Post the lobby embed hosted by `host`. Returns the new message's id.
    async fn post_lobby(&self, channel: u64, host: u64) -> anyhow::Result<u64>;

    /// React to the lobby message with the join emoji.
    async fn add_join_reaction(&self, channel: u64, message: u64) -> anyhow::Result<()>;

    /// Post question `index` (0-based) of `total` with its \[Answer\] button.
    async fn post_question(
        &self,
        channel: u64,
        index: usize,
        total: usize,
        text: &str,
    ) -> anyhow::Result<()>;

    /// Post the reveal for the question that just closed.
    async fn post_reveal(&self, channel: u64, reveal: RevealInfo) -> anyhow::Result<()>;

    /// Post the final leaderboard.
    async fn post_tally(&self, channel: u64, standings: Vec<TallyRow>) -> anyhow::Result<()>;

    /// Post a short plain-text notice (timeouts, cancellation).
    async fn post_notice(&self, channel: u64, text: &str) -> anyhow::Result<()>;
}
