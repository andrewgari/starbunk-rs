//! Time source shared by the engine and its timers.
//!
//! The round state machine takes `std::time::Instant`s. Reading them through
//! tokio's clock means `tokio::time::pause()` / `advance()` in tests move the
//! state machine's notion of "now" and the timers together.

use std::time::Instant;

/// Current time on tokio's (possibly paused) clock.
pub(super) fn now() -> Instant {
    tokio::time::Instant::now().into_std()
}

/// Sleep until `deadline` on tokio's clock.
pub(super) async fn sleep_until(deadline: Instant) {
    tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
}
