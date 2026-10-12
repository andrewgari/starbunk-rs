//! Question close / advance transitions on a [`Game`].

use super::game::Game;
use super::{rank, ChannelId, CleanupReason, Phase, Reveal, RoundEvent};
use std::time::Instant;

/// Outcome of closing the open question.
pub(super) struct Closed {
    pub events: Vec<RoundEvent>,
    /// The round is over and must be removed from the registry.
    pub finished: bool,
}

impl Game {
    /// Close the open question at `now`: emit the reveal, then open the next
    /// question or post the tally. Must only be called while in progress.
    pub fn close_question(&mut self, channel: ChannelId, now: Instant) -> Closed {
        let Some(index) = self.open_index() else {
            return Closed {
                events: Vec::new(),
                finished: false,
            };
        };
        let question = self.questions[index].clone();
        let mut events = vec![RoundEvent::PostReveal(Reveal {
            channel,
            index,
            display_answer: question.display_answer.clone(),
            correct: self.correct_players(),
            question,
        })];

        let next = index + 1;
        if let Some(question) = self.questions.get(next).cloned() {
            let deadline = self.open_question(next, now);
            events.push(RoundEvent::PostQuestion {
                channel,
                index: next,
                total: self.questions.len(),
                question,
                deadline,
            });
            return Closed {
                events,
                finished: false,
            };
        }

        self.phase = Phase::Finished;
        self.question_deadline = None;
        events.push(RoundEvent::PostTally {
            channel,
            standings: rank(&self.scores_in_join_order()),
        });
        events.push(RoundEvent::Cleanup {
            channel,
            reason: CleanupReason::Finished,
        });
        Closed {
            events,
            finished: true,
        }
    }
}
