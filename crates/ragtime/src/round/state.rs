//! Round phases.

/// Lifecycle of a round.
///
/// `Lobby → InProgress { index } → Finished`, or `Lobby → Abandoned` (lobby
/// timeout) / `InProgress → Abandoned` (no answers) / `Cancelled` (host).
/// Terminal phases are never observable through
/// [`super::RoundRegistry::phase`]: the round is removed at the same moment and
/// a [`super::RoundEvent::Cleanup`] reports the reason instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Waiting for players; only the host can start.
    Lobby,
    /// Question `index` (0-based) is open for answers.
    InProgress { index: usize },
    /// All questions were played.
    Finished,
    /// Timed out (lobby never started, or nobody answering).
    Abandoned,
    /// Cancelled by the host.
    Cancelled,
}
