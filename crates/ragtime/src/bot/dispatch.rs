//! Turning [`RoundEvent`]s into [`super::Outbox`] posts.

use super::engine::RagtimeEngine;
use super::replies;
use crate::presentation::{RevealInfo, TallyRow};
use crate::round::{CleanupReason, RoundEvent};

/// Public notice for a cleanup reason, if it warrants one.
fn cleanup_notice(reason: CleanupReason) -> Option<&'static str> {
    match reason {
        CleanupReason::Finished | CleanupReason::NoPlayers => None,
        CleanupReason::NoQuestions => Some(replies::NO_QUESTIONS),
        CleanupReason::LobbyTimeout => Some(replies::LOBBY_TIMED_OUT),
        CleanupReason::Abandoned => Some(replies::ABANDONED),
        CleanupReason::Cancelled => Some(replies::CANCELLED),
    }
}

impl RagtimeEngine {
    /// Post `events` in order. Callers hold `seq` (not `state`). A failed post
    /// is logged and skipped: the round keeps going, and the next post may succeed.
    pub(super) async fn dispatch(&self, events: Vec<RoundEvent>) {
        for event in events {
            let (channel, result) = match event {
                RoundEvent::PostQuestion {
                    channel,
                    index,
                    total,
                    question,
                    ..
                } => (
                    channel,
                    self.outbox
                        .post_question(channel, index, total, &question.question)
                        .await,
                ),
                RoundEvent::PostReveal(reveal) => (
                    reveal.channel,
                    self.outbox
                        .post_reveal(
                            reveal.channel,
                            RevealInfo {
                                display_answer: reveal.display_answer,
                                correct_users: reveal.correct,
                            },
                        )
                        .await,
                ),
                RoundEvent::PostTally { channel, standings } => {
                    let rows = standings
                        .into_iter()
                        .map(|s| TallyRow {
                            rank: s.rank,
                            user: s.user,
                            score: s.score,
                        })
                        .collect();
                    (channel, self.outbox.post_tally(channel, rows).await)
                }
                RoundEvent::Cleanup { channel, reason } => {
                    tracing::info!(bot = "ragtime", channel, ?reason, "round ended");
                    match cleanup_notice(reason) {
                        Some(text) => (channel, self.outbox.post_notice(channel, text).await),
                        None => (channel, Ok(())),
                    }
                }
            };
            if let Err(e) = result {
                tracing::error!(bot = "ragtime", channel, err = %e, "failed to post round event");
            }
        }
    }
}
