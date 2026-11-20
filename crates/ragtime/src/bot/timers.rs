//! tokio timers for the lobby, question and abandon timeouts.
//!
//! Each channel has at most one pending timer, sleeping until
//! [`crate::round::RoundRegistry::next_deadline`]. Whenever a channel's state
//! changes its timer is replaced (the old task is aborted). A timer that fires
//! anyway after its question already closed is harmless: `tick` is a no-op
//! until a real deadline has passed, and the timer simply re-arms.

use super::clock;
use super::engine::{RagtimeEngine, State};
use crate::round::RoundEvent;
use std::collections::BTreeSet;
use std::sync::Arc;

fn event_channel(event: &RoundEvent) -> u64 {
    match event {
        RoundEvent::PostQuestion { channel, .. }
        | RoundEvent::PostTally { channel, .. }
        | RoundEvent::Cleanup { channel, .. } => *channel,
        RoundEvent::PostReveal(reveal) => reveal.channel,
    }
}

impl RagtimeEngine {
    /// Replace `channel`'s timer with one for its next deadline, or drop all
    /// per-channel state if the round is gone. Call with the state lock held.
    pub(super) fn reschedule(self: &Arc<Self>, st: &mut State, channel: u64) {
        if let Some(old) = st.timers.remove(&channel) {
            old.abort();
        }
        let Some(deadline) = st.registry.next_deadline(channel) else {
            st.lobby_messages.remove(&channel);
            return;
        };
        let engine = Arc::clone(self);
        let handle = tokio::spawn(async move {
            clock::sleep_until(deadline).await;
            engine.on_timer(channel).await;
        });
        st.timers.insert(channel, handle.abort_handle());
    }

    /// A channel's deadline passed: advance the state machine and post the result.
    #[tracing::instrument(skip(self))]
    async fn on_timer(self: Arc<Self>, channel: u64) {
        let _seq = self.seq.lock().await;
        let events = {
            let mut st = self.state.lock().await;
            // Forget our own handle without aborting it: we are that task, and
            // aborting ourselves would cancel the posts below.
            let me = tokio::task::try_id();
            if me.is_some() && st.timers.get(&channel).map(|h| h.id()) == me {
                st.timers.remove(&channel);
            }
            let events = st.registry.tick(clock::now());
            // `tick` advances every channel, so re-arm all it touched, plus
            // this one (it re-arms itself even if it fired early or stale).
            let mut channels: BTreeSet<u64> = events.iter().map(event_channel).collect();
            channels.insert(channel);
            for ch in channels {
                self.reschedule(&mut st, ch);
            }
            events
        };
        self.dispatch(events).await;
    }
}
