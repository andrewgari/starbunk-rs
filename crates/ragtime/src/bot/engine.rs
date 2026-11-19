//! [`RagtimeEngine`]: the Discord-free orchestrator behind the handler.

use super::outbox::Outbox;
use super::{clock, replies};
use crate::presentation::{already_answered_text, not_in_round_text, AnswerRoute};
use crate::repository::QuestionRepository;
use crate::round::{Grader, RoundError, RoundRegistry, QUESTIONS_PER_ROUND};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::AbortHandle;

/// Result of an @mention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MentionOutcome {
    /// A lobby was opened and posted; this is its message id.
    Created { lobby_message: u64 },
    /// The channel already has a round. Reply with [`replies::ALREADY_RUNNING`].
    AlreadyRunning,
}

/// Result of an \[Answer\] button click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonOutcome {
    /// Open the answer modal for this route.
    OpenModal(AnswerRoute),
    /// Respond ephemerally with this text.
    Ephemeral(&'static str),
}

/// Mutable state guarded by the registry lock.
pub(super) struct State {
    pub registry: RoundRegistry,
    /// Lobby message per channel, so only reactions on *that* message join.
    pub lobby_messages: HashMap<u64, u64>,
    /// Pending timer per channel.
    pub timers: HashMap<u64, AbortHandle>,
    pub rng: StdRng,
}

/// Owns the round registry, the question source and the outbox.
///
/// Always used as `Arc<RagtimeEngine>` because timers hold a clone.
pub struct RagtimeEngine {
    /// Serialises every state change together with the posts it causes.
    /// Lock order: `seq` → `state`. `state` is never held across an `.await`.
    pub(super) seq: Arc<Mutex<()>>,
    pub(super) state: Mutex<State>,
    pub(super) repo: Arc<dyn QuestionRepository>,
    pub(super) outbox: Arc<dyn Outbox>,
}

impl std::fmt::Debug for RagtimeEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RagtimeEngine").finish_non_exhaustive()
    }
}

fn error_text(err: RoundError) -> &'static str {
    match err {
        RoundError::RoundAlreadyActive => replies::ALREADY_RUNNING,
        RoundError::NoActiveRound => replies::NO_ROUND,
        RoundError::NotHost => replies::NOT_HOST,
        RoundError::AlreadyStarted => replies::ALREADY_STARTED,
        RoundError::NotInRound => not_in_round_text(),
        RoundError::AlreadyAnswered => already_answered_text(),
        RoundError::NotInProgress | RoundError::StaleQuestion => replies::QUESTION_CLOSED,
    }
}

impl RagtimeEngine {
    /// A new engine with an entropy-seeded RNG.
    pub fn new(
        repo: Arc<dyn QuestionRepository>,
        outbox: Arc<dyn Outbox>,
        grader: Arc<dyn Grader>,
    ) -> Arc<Self> {
        Self::with_rng(repo, outbox, grader, StdRng::from_entropy())
    }

    /// A new engine with an explicit RNG (deterministic question order in tests).
    pub fn with_rng(
        repo: Arc<dyn QuestionRepository>,
        outbox: Arc<dyn Outbox>,
        grader: Arc<dyn Grader>,
        rng: StdRng,
    ) -> Arc<Self> {
        Arc::new(Self {
            seq: Arc::new(Mutex::new(())),
            state: Mutex::new(State {
                registry: RoundRegistry::new(grader),
                lobby_messages: HashMap::new(),
                timers: HashMap::new(),
                rng,
            }),
            repo,
            outbox,
        })
    }

    /// The bot was @mentioned in `channel` by `user`: open a lobby hosted by
    /// them, post it and add the join reaction.
    #[tracing::instrument(skip(self))]
    pub async fn mention(
        self: &Arc<Self>,
        channel: u64,
        user: u64,
    ) -> anyhow::Result<MentionOutcome> {
        let _seq = self.seq.lock().await;
        {
            let mut st = self.state.lock().await;
            match st.registry.create(channel, user, clock::now()) {
                Ok(()) => {}
                Err(RoundError::RoundAlreadyActive) => return Ok(MentionOutcome::AlreadyRunning),
                Err(e) => return Err(e.into()),
            }
        }

        let lobby_message = match self.outbox.post_lobby(channel, user).await {
            Ok(id) => id,
            Err(e) => {
                // Nobody can see the lobby, so don't leave a ghost round behind.
                let mut st = self.state.lock().await;
                let _ = st.registry.cancel(channel, user);
                return Err(e);
            }
        };
        {
            let mut st = self.state.lock().await;
            st.lobby_messages.insert(channel, lobby_message);
            self.reschedule(&mut st, channel);
        }
        if let Err(e) = self.outbox.add_join_reaction(channel, lobby_message).await {
            tracing::warn!(bot = "ragtime", channel, err = %e, "failed to add join reaction");
        }
        tracing::info!(bot = "ragtime", channel, host = user, "lobby opened");
        Ok(MentionOutcome::Created { lobby_message })
    }

    /// `user` added the join reaction to `message` in `channel`. Reactions on
    /// any other message, or after the round started, are ignored.
    /// Returns whether the user is now in the round.
    #[tracing::instrument(skip(self))]
    pub async fn join_reaction(&self, channel: u64, message: u64, user: u64) -> bool {
        let mut st = self.state.lock().await;
        if st.lobby_messages.get(&channel) != Some(&message) {
            return false;
        }
        match st.registry.join(channel, user) {
            Ok(_) => true,
            Err(e) => {
                tracing::debug!(bot = "ragtime", channel, user, err = %e, "join rejected");
                false
            }
        }
    }

    /// Host-only start. Returns the ephemeral reply for the invoker.
    #[tracing::instrument(skip(self))]
    pub async fn start(self: &Arc<Self>, channel: u64, user: u64) -> &'static str {
        let _seq = self.seq.lock().await;
        let has_players = {
            let st = self.state.lock().await;
            if let Err(e) = st.registry.ensure_can_start(channel, user) {
                return error_text(e);
            }
            st.registry.players(channel).is_some_and(|p| !p.is_empty())
        };

        // No joiners: skip the database; `start` cleans up with NoPlayers.
        let pool = if has_players {
            match self.repo.random(QUESTIONS_PER_ROUND).await {
                Ok(pool) => pool,
                Err(e) => {
                    tracing::error!(bot = "ragtime", channel, err = %e, "failed to load questions");
                    return replies::LOAD_FAILED;
                }
            }
        } else {
            Vec::new()
        };

        let events = {
            let mut st = self.state.lock().await;
            let State { registry, rng, .. } = &mut *st;
            let events = match registry.start(channel, user, pool, rng, clock::now()) {
                Ok(events) => events,
                Err(e) => return error_text(e),
            };
            // Lobby timer → question timer (or drop everything on cleanup).
            self.reschedule(&mut st, channel);
            events
        };
        let reply = match events.first() {
            Some(crate::round::RoundEvent::Cleanup { reason, .. }) => match reason {
                crate::round::CleanupReason::NoPlayers => replies::NO_PLAYERS,
                _ => replies::NO_QUESTIONS,
            },
            _ => replies::STARTED,
        };
        tracing::info!(bot = "ragtime", channel, reply, "start");
        self.dispatch(events).await;
        reply
    }

    /// Host-only cancel. Returns the ephemeral reply for the invoker.
    #[tracing::instrument(skip(self))]
    pub async fn cancel(self: &Arc<Self>, channel: u64, user: u64) -> &'static str {
        let _seq = self.seq.lock().await;
        let events = {
            let mut st = self.state.lock().await;
            let events = match st.registry.cancel(channel, user) {
                Ok(events) => events,
                Err(e) => return error_text(e),
            };
            self.reschedule(&mut st, channel);
            events
        };
        self.dispatch(events).await;
        replies::CANCELLED_ACK
    }

    /// \[Answer\] clicked for `route` by `user`. Read-only and lock-light so
    /// the modal opens inside Discord's response window.
    pub async fn answer_button(&self, route: AnswerRoute, user: u64) -> ButtonOutcome {
        let st = self.state.lock().await;
        match st.registry.can_answer(route.channel, user) {
            Ok(open) if open == route.index => ButtonOutcome::OpenModal(route),
            Ok(_) => ButtonOutcome::Ephemeral(replies::QUESTION_CLOSED),
            Err(e) => ButtonOutcome::Ephemeral(error_text(e)),
        }
    }

    /// Answer modal submitted. Returns the ephemeral acknowledgement right
    /// away; if this closed the question, the reveal/next question are posted
    /// in the background (still in order, under `seq`).
    #[tracing::instrument(skip(self, reply))]
    pub async fn submit_answer(
        self: &Arc<Self>,
        route: AnswerRoute,
        user: u64,
        reply: &str,
    ) -> &'static str {
        let seq = Arc::clone(&self.seq).lock_owned().await;
        let events = {
            let mut st = self.state.lock().await;
            match st
                .registry
                .submit_answer(route.channel, user, route.index, reply, clock::now())
            {
                Ok(events) => {
                    // An accepted answer moves the abandon deadline.
                    self.reschedule(&mut st, route.channel);
                    events
                }
                Err(e) => return error_text(e),
            }
        };
        if !events.is_empty() {
            let engine = Arc::clone(self);
            tokio::spawn(async move {
                let _seq = seq;
                engine.dispatch(events).await;
            });
        }
        replies::ANSWER_RECEIVED
    }

    /// Wait until every in-flight state change and its posts have finished.
    pub async fn flush(&self) {
        drop(self.seq.lock().await);
    }

    /// Whether `channel` currently has a round.
    pub async fn has_round(&self, channel: u64) -> bool {
        self.state.lock().await.registry.phase(channel).is_some()
    }
}
