//! Discord-wiring tests: drive [`RagtimeEngine`] the way the serenity handler
//! does, with an in-memory question bank, a recording outbox and paused tokio
//! time (so the 2-minute / 6-minute / 10-minute timers run instantly).

use chrono::Utc;
use ragtime::bot::{replies, ButtonOutcome, MentionOutcome, Outbox, RagtimeEngine};
use ragtime::presentation::{
    already_answered_text, not_in_round_text, AnswerRoute, RevealInfo, TallyRow,
};
use ragtime::question::{NewQuestion, Question, QuestionId, QuestionUpdate};
use ragtime::repository::QuestionRepository;
use ragtime::round::{HOST_ABANDON_TIMEOUT, LOBBY_TIMEOUT, QUESTION_TIMEOUT};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const CH: u64 = 100;
const HOST: u64 = 1;
const ALICE: u64 = 2;
const BOB: u64 = 3;
const STRANGER: u64 = 9;

// ---------------------------------------------------------------- fakes

#[derive(Debug, Clone, PartialEq, Eq)]
enum Post {
    Lobby {
        host: u64,
    },
    Reaction {
        message: u64,
    },
    Question {
        index: usize,
        total: usize,
        text: String,
    },
    Reveal(RevealInfo),
    Tally(Vec<TallyRow>),
    Notice(String),
}

#[derive(Default)]
struct FakeOutbox {
    posts: Mutex<Vec<(u64, Post)>>,
    next_id: AtomicU64,
}

impl FakeOutbox {
    fn posts(&self) -> Vec<Post> {
        self.posts
            .lock()
            .unwrap()
            .iter()
            .map(|(_, p)| p.clone())
            .collect()
    }
    fn take(&self) -> Vec<Post> {
        std::mem::take(&mut *self.posts.lock().unwrap())
            .into_iter()
            .map(|(_, p)| p)
            .collect()
    }
    fn push(&self, channel: u64, post: Post) {
        self.posts.lock().unwrap().push((channel, post));
    }
}

#[async_trait::async_trait]
impl Outbox for FakeOutbox {
    async fn post_lobby(&self, channel: u64, host: u64) -> anyhow::Result<u64> {
        self.push(channel, Post::Lobby { host });
        Ok(5000 + self.next_id.fetch_add(1, Ordering::SeqCst))
    }
    async fn add_join_reaction(&self, channel: u64, message: u64) -> anyhow::Result<()> {
        self.push(channel, Post::Reaction { message });
        Ok(())
    }
    async fn post_question(
        &self,
        channel: u64,
        index: usize,
        total: usize,
        text: &str,
    ) -> anyhow::Result<()> {
        // Simulate network latency so ordering bugs would show up.
        tokio::task::yield_now().await;
        self.push(
            channel,
            Post::Question {
                index,
                total,
                text: text.to_string(),
            },
        );
        Ok(())
    }
    async fn post_reveal(&self, channel: u64, reveal: RevealInfo) -> anyhow::Result<()> {
        tokio::task::yield_now().await;
        self.push(channel, Post::Reveal(reveal));
        Ok(())
    }
    async fn post_tally(&self, channel: u64, standings: Vec<TallyRow>) -> anyhow::Result<()> {
        self.push(channel, Post::Tally(standings));
        Ok(())
    }
    async fn post_notice(&self, channel: u64, text: &str) -> anyhow::Result<()> {
        self.push(channel, Post::Notice(text.to_string()));
        Ok(())
    }
}

struct MemRepo {
    questions: Vec<Question>,
    random_calls: AtomicU64,
}

impl MemRepo {
    fn with(n: usize) -> Self {
        let now = Utc::now();
        let questions = (1..=n as i64)
            .map(|id| Question {
                id,
                question: format!("Q{id}?"),
                display_answer: format!("Answer {id}"),
                acceptable_answers: vec![format!("a{id}")],
                created_at: now,
                updated_at: now,
            })
            .collect();
        Self {
            questions,
            random_calls: AtomicU64::new(0),
        }
    }
}

#[async_trait::async_trait]
impl QuestionRepository for MemRepo {
    async fn list(&self) -> anyhow::Result<Vec<Question>> {
        Ok(self.questions.clone())
    }
    async fn get(&self, id: QuestionId) -> anyhow::Result<Option<Question>> {
        Ok(self.questions.iter().find(|q| q.id == id).cloned())
    }
    async fn create(&self, _new: NewQuestion) -> anyhow::Result<Question> {
        anyhow::bail!("read-only")
    }
    async fn update(&self, _: QuestionId, _: QuestionUpdate) -> anyhow::Result<Option<Question>> {
        anyhow::bail!("read-only")
    }
    async fn delete(&self, _: QuestionId) -> anyhow::Result<bool> {
        anyhow::bail!("read-only")
    }
    async fn random(&self, n: usize) -> anyhow::Result<Vec<Question>> {
        self.random_calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.questions.iter().take(n).cloned().collect())
    }
}

struct Harness {
    engine: Arc<RagtimeEngine>,
    outbox: Arc<FakeOutbox>,
    repo: Arc<MemRepo>,
}

fn harness(n_questions: usize) -> Harness {
    let outbox = Arc::new(FakeOutbox::default());
    let repo = Arc::new(MemRepo::with(n_questions));
    let engine = RagtimeEngine::with_rng(
        repo.clone(),
        outbox.clone(),
        ragtime::bot::interim_grader(),
        StdRng::seed_from_u64(1),
    );
    Harness {
        engine,
        outbox,
        repo,
    }
}

/// Let spawned tasks (background dispatch, timers that are due) run.
async fn settle(h: &Harness) {
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
    h.engine.flush().await;
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
}

async fn lobby_with(h: &Harness, players: &[u64]) -> u64 {
    let MentionOutcome::Created { lobby_message } = h.engine.mention(CH, HOST).await.unwrap()
    else {
        panic!("expected a new lobby");
    };
    for &p in players {
        assert!(h.engine.join_reaction(CH, lobby_message, p).await);
    }
    lobby_message
}

/// Answer for the question whose text is `Q{id}?`.
fn right_answer(text: &str) -> String {
    let id = text.trim_start_matches('Q').trim_end_matches('?');
    format!("a{id}")
}

fn open_question(posts: &[Post]) -> (usize, String) {
    posts
        .iter()
        .rev()
        .find_map(|p| match p {
            Post::Question { index, text, .. } => Some((*index, text.clone())),
            _ => None,
        })
        .expect("a question was posted")
}

// ---------------------------------------------------------------- tests

#[tokio::test(start_paused = true)]
async fn mention_posts_lobby_and_join_reaction() {
    let h = harness(3);
    let lobby = lobby_with(&h, &[]).await;
    assert_eq!(
        h.outbox.posts(),
        vec![
            Post::Lobby { host: HOST },
            Post::Reaction { message: lobby }
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn second_mention_in_same_channel_is_already_running() {
    let h = harness(3);
    lobby_with(&h, &[]).await;
    assert_eq!(
        h.engine.mention(CH, ALICE).await.unwrap(),
        MentionOutcome::AlreadyRunning
    );
    // Other channels are independent.
    assert!(matches!(
        h.engine.mention(CH + 1, ALICE).await.unwrap(),
        MentionOutcome::Created { .. }
    ));
}

#[tokio::test(start_paused = true)]
async fn reactions_on_other_messages_do_not_join() {
    let h = harness(3);
    let lobby = lobby_with(&h, &[]).await;
    assert!(!h.engine.join_reaction(CH, lobby + 999, ALICE).await);
    assert!(h.engine.join_reaction(CH, lobby, ALICE).await);
}

#[tokio::test(start_paused = true)]
async fn non_host_start_is_rejected() {
    let h = harness(3);
    lobby_with(&h, &[ALICE]).await;
    assert_eq!(h.engine.start(CH, ALICE).await, replies::NOT_HOST);
    assert_eq!(h.repo.random_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test(start_paused = true)]
async fn start_with_nobody_joined_posts_nothing_and_cleans_up() {
    let h = harness(3);
    lobby_with(&h, &[]).await;
    h.outbox.take();
    assert_eq!(h.engine.start(CH, HOST).await, replies::NO_PLAYERS);
    settle(&h).await;
    assert!(h.outbox.posts().is_empty());
    assert!(!h.engine.has_round(CH).await);
    assert_eq!(h.repo.random_calls.load(Ordering::SeqCst), 0, "no DB hit");
}

#[tokio::test(start_paused = true)]
async fn start_with_empty_bank_posts_notice_and_cleans_up() {
    let h = harness(0);
    lobby_with(&h, &[ALICE]).await;
    h.outbox.take();
    assert_eq!(h.engine.start(CH, HOST).await, replies::NO_QUESTIONS);
    assert_eq!(
        h.outbox.posts(),
        vec![Post::Notice(replies::NO_QUESTIONS.into())]
    );
    assert!(!h.engine.has_round(CH).await);
}

#[tokio::test(start_paused = true)]
async fn answer_button_edge_cases() {
    let h = harness(3);
    lobby_with(&h, &[ALICE, BOB]).await;
    h.engine.start(CH, HOST).await;
    let (index, text) = open_question(&h.outbox.posts());
    let route = AnswerRoute::new(CH, index);

    assert_eq!(
        h.engine.answer_button(route, STRANGER).await,
        ButtonOutcome::Ephemeral(not_in_round_text())
    );
    assert_eq!(
        h.engine.answer_button(route, ALICE).await,
        ButtonOutcome::OpenModal(route)
    );
    assert_eq!(
        h.engine
            .submit_answer(route, ALICE, &right_answer(&text))
            .await,
        replies::ANSWER_RECEIVED
    );
    assert_eq!(
        h.engine.answer_button(route, ALICE).await,
        ButtonOutcome::Ephemeral(already_answered_text())
    );
    // Second submit (e.g. two modals open) is rejected too.
    assert_eq!(
        h.engine.submit_answer(route, ALICE, "again").await,
        already_answered_text()
    );
    // A button from an old question is reported as closed.
    assert_eq!(
        h.engine
            .answer_button(AnswerRoute::new(CH, index + 1), BOB)
            .await,
        ButtonOutcome::Ephemeral(replies::QUESTION_CLOSED)
    );
}

#[tokio::test(start_paused = true)]
async fn wrong_and_right_answers_get_the_same_acknowledgement() {
    let h = harness(3);
    lobby_with(&h, &[ALICE, BOB]).await;
    h.engine.start(CH, HOST).await;
    let (index, text) = open_question(&h.outbox.posts());
    let route = AnswerRoute::new(CH, index);
    let right = h
        .engine
        .submit_answer(route, ALICE, &right_answer(&text))
        .await;
    let wrong = h.engine.submit_answer(route, BOB, "definitely wrong").await;
    assert_eq!(right, wrong);
}

#[tokio::test(start_paused = true)]
async fn full_round_lobby_to_tally() {
    let h = harness(20);
    lobby_with(&h, &[ALICE, BOB]).await;
    assert_eq!(h.engine.start(CH, HOST).await, replies::STARTED);
    settle(&h).await;

    let mut alice_points = 0;
    for q in 0..20 {
        let posts = h.outbox.take();
        let (index, text) = open_question(&posts);
        assert_eq!(index, q, "questions are posted in order");
        let route = AnswerRoute::new(CH, index);
        // Alice is right on even questions, Bob always wrong.
        let alice_reply = if q % 2 == 0 {
            alice_points += 1;
            right_answer(&text)
        } else {
            "nope".to_string()
        };
        h.engine.submit_answer(route, ALICE, &alice_reply).await;
        h.engine.submit_answer(route, BOB, "nope").await;
        settle(&h).await;

        let posts = h.outbox.posts();
        let reveal = posts
            .iter()
            .find_map(|p| match p {
                Post::Reveal(r) => Some(r.clone()),
                _ => None,
            })
            .expect("question closed early once both answered");
        let expected: Vec<u64> = if q % 2 == 0 { vec![ALICE] } else { vec![] };
        assert_eq!(reveal.correct_users, expected);
        assert_eq!(
            reveal.display_answer,
            text.replace('Q', "Answer ").replace('?', "")
        );
        // Reveal comes before the next question / tally.
        let reveal_at = posts
            .iter()
            .position(|p| matches!(p, Post::Reveal(_)))
            .unwrap();
        let next_at = posts
            .iter()
            .position(|p| matches!(p, Post::Question { .. } | Post::Tally(_)))
            .unwrap();
        assert!(
            reveal_at < next_at,
            "reveal must precede the next post: {posts:?}"
        );
        if q < 19 {
            h.outbox
                .posts
                .lock()
                .unwrap()
                .retain(|(_, p)| !matches!(p, Post::Reveal(_)));
        }
    }

    let tally = h
        .outbox
        .posts()
        .into_iter()
        .find_map(|p| match p {
            Post::Tally(rows) => Some(rows),
            _ => None,
        })
        .expect("final tally posted");
    assert_eq!(
        tally,
        vec![
            TallyRow {
                rank: 1,
                user: ALICE,
                score: alice_points
            },
            TallyRow {
                rank: 2,
                user: BOB,
                score: 0
            },
        ]
    );
    assert!(!h.engine.has_round(CH).await, "round cleaned up");
    // The channel is free for a new round.
    assert!(matches!(
        h.engine.mention(CH, BOB).await.unwrap(),
        MentionOutcome::Created { .. }
    ));
}

#[tokio::test(start_paused = true)]
async fn uses_all_questions_when_bank_is_small() {
    let h = harness(2);
    lobby_with(&h, &[ALICE]).await;
    h.engine.start(CH, HOST).await;
    let (_, _) = open_question(&h.outbox.posts());
    assert!(h.outbox.posts().iter().any(|p| matches!(
        p,
        Post::Question {
            index: 0,
            total: 2,
            ..
        }
    )));
}

#[tokio::test(start_paused = true)]
async fn question_times_out_after_two_minutes() {
    let h = harness(3);
    lobby_with(&h, &[ALICE, BOB]).await;
    h.engine.start(CH, HOST).await;
    let (index, text) = open_question(&h.outbox.take());
    // Only Alice answers; the question must wait for the timer.
    h.engine
        .submit_answer(AnswerRoute::new(CH, index), ALICE, &right_answer(&text))
        .await;
    settle(&h).await;
    assert!(
        h.outbox.posts().is_empty(),
        "still open: Bob hasn't answered"
    );

    tokio::time::sleep(QUESTION_TIMEOUT - Duration::from_secs(1)).await;
    settle(&h).await;
    assert!(h.outbox.posts().is_empty(), "not yet");

    tokio::time::sleep(Duration::from_secs(2)).await;
    settle(&h).await;
    let posts = h.outbox.take();
    assert!(matches!(&posts[0], Post::Reveal(r) if r.correct_users == vec![ALICE]));
    assert!(matches!(&posts[1], Post::Question { index: 1, .. }));
}

#[tokio::test(start_paused = true)]
async fn early_close_does_not_let_the_old_timer_close_the_next_question() {
    let h = harness(3);
    lobby_with(&h, &[ALICE]).await;
    h.engine.start(CH, HOST).await;

    // Q1 open for 90s, then answered → closes early, Q2 opens with a fresh 2 min.
    tokio::time::sleep(Duration::from_secs(90)).await;
    let (index, text) = open_question(&h.outbox.take());
    h.engine
        .submit_answer(AnswerRoute::new(CH, index), ALICE, &right_answer(&text))
        .await;
    settle(&h).await;
    assert!(matches!(
        &h.outbox.take()[1],
        Post::Question { index: 1, .. }
    ));

    // When Q1's original deadline passes (30s later), nothing must happen to Q2.
    tokio::time::sleep(Duration::from_secs(45)).await;
    settle(&h).await;
    assert!(
        h.outbox.posts().is_empty(),
        "stale timer ignored: {:?}",
        h.outbox.posts()
    );

    // Q2 times out on its own schedule.
    tokio::time::sleep(QUESTION_TIMEOUT - Duration::from_secs(45)).await;
    settle(&h).await;
    assert!(matches!(&h.outbox.posts()[0], Post::Reveal(_)));
}

#[tokio::test(start_paused = true)]
async fn lobby_times_out() {
    let h = harness(3);
    lobby_with(&h, &[ALICE]).await;
    h.outbox.take();
    tokio::time::sleep(LOBBY_TIMEOUT + Duration::from_secs(1)).await;
    settle(&h).await;
    assert_eq!(
        h.outbox.posts(),
        vec![Post::Notice(replies::LOBBY_TIMED_OUT.into())]
    );
    assert!(!h.engine.has_round(CH).await);
}

#[tokio::test(start_paused = true)]
async fn round_with_no_answers_is_abandoned() {
    let h = harness(20);
    lobby_with(&h, &[ALICE]).await;
    h.engine.start(CH, HOST).await;
    h.outbox.take();
    tokio::time::sleep(HOST_ABANDON_TIMEOUT + Duration::from_secs(1)).await;
    settle(&h).await;
    let posts = h.outbox.posts();
    assert_eq!(
        posts.last(),
        Some(&Post::Notice(replies::ABANDONED.into())),
        "{posts:?}"
    );
    // Questions 1 and 2 timed out (reveal + next) before the abandon at 6 min.
    let reveals = posts
        .iter()
        .filter(|p| matches!(p, Post::Reveal(_)))
        .count();
    assert_eq!(reveals, 2);
    assert!(!h.engine.has_round(CH).await);

    // Nothing more happens afterwards.
    h.outbox.take();
    tokio::time::sleep(Duration::from_secs(600)).await;
    settle(&h).await;
    assert!(h.outbox.posts().is_empty());
}

#[tokio::test(start_paused = true)]
async fn host_can_cancel_and_others_cannot() {
    let h = harness(3);
    lobby_with(&h, &[ALICE]).await;
    assert_eq!(h.engine.cancel(CH, ALICE).await, replies::NOT_HOST);
    h.outbox.take();
    assert_eq!(h.engine.cancel(CH, HOST).await, replies::CANCELLED_ACK);
    assert_eq!(
        h.outbox.posts(),
        vec![Post::Notice(replies::CANCELLED.into())]
    );
    assert!(!h.engine.has_round(CH).await);
    // The cancelled lobby's timer must not fire later.
    h.outbox.take();
    tokio::time::sleep(LOBBY_TIMEOUT * 2).await;
    settle(&h).await;
    assert!(h.outbox.posts().is_empty());
}

#[tokio::test(start_paused = true)]
async fn late_join_reactions_are_ignored() {
    let h = harness(3);
    let lobby = lobby_with(&h, &[ALICE]).await;
    h.engine.start(CH, HOST).await;
    assert!(!h.engine.join_reaction(CH, lobby, BOB).await);
}

/// The Ragtime e2e round-flow entry, step by step, with a 2-question bank.
///
/// `crates/e2e/suites/ragtime.json` covers the steps a webhook can drive on
/// live Discord (mention → lobby, mention guards). Everything after the lobby
/// needs a real user client (reactions from a second account, slash commands,
/// buttons, modals, ephemeral replies), so this test is that flow's
/// end-to-end check, at the engine boundary the serenity handler calls.
#[tokio::test(start_paused = true)]
async fn e2e_round_flow_two_questions() {
    let h = harness(2);

    // 1. A mention creates a lobby hosted by the mentioner.
    let MentionOutcome::Created { lobby_message } = h.engine.mention(CH, HOST).await.unwrap()
    else {
        panic!("mention must open a lobby");
    };
    assert_eq!(
        h.outbox.take(),
        vec![
            Post::Lobby { host: HOST },
            Post::Reaction {
                message: lobby_message
            }
        ]
    );

    // 2. Joining via the lobby reaction registers the players.
    assert!(h.engine.join_reaction(CH, lobby_message, ALICE).await);
    assert!(h.engine.join_reaction(CH, lobby_message, BOB).await);

    // 3. A non-host start is rejected and posts nothing.
    assert_eq!(h.engine.start(CH, ALICE).await, replies::NOT_HOST);
    assert!(h.outbox.posts().is_empty());

    // 4. The host's start posts question 1 of 2.
    assert_eq!(h.engine.start(CH, HOST).await, replies::STARTED);
    settle(&h).await;
    let posts = h.outbox.take();
    assert!(
        matches!(
            posts.as_slice(),
            [Post::Question {
                index: 0,
                total: 2,
                ..
            }]
        ),
        "{posts:?}"
    );
    let (index, text) = open_question(&posts);
    let q1 = AnswerRoute::new(CH, index);

    // 6. Someone who didn't join gets "not in this round", on the button and
    //    on a (forged) modal submit alike.
    assert_eq!(
        h.engine.answer_button(q1, STRANGER).await,
        ButtonOutcome::Ephemeral(not_in_round_text())
    );
    assert_eq!(
        h.engine.submit_answer(q1, STRANGER, "anything").await,
        not_in_round_text()
    );

    // 5. A joined player's modal answer is recorded: the button opens the
    //    modal, the submit is acknowledged, and a second answer is refused.
    assert_eq!(
        h.engine.answer_button(q1, ALICE).await,
        ButtonOutcome::OpenModal(q1)
    );
    assert_eq!(
        h.engine
            .submit_answer(q1, ALICE, &right_answer(&text))
            .await,
        replies::ANSWER_RECEIVED
    );
    assert_eq!(
        h.engine.answer_button(q1, ALICE).await,
        ButtonOutcome::Ephemeral(already_answered_text())
    );
    settle(&h).await;
    assert!(
        h.outbox.posts().is_empty(),
        "question stays open until every player answered"
    );

    // 7. The last player's answer closes the question early (no timer
    //    elapsed); the reveal mentions only the correct player and comes
    //    before question 2.
    h.engine.submit_answer(q1, BOB, "wrong").await;
    settle(&h).await;
    let posts = h.outbox.take();
    match posts.as_slice() {
        [Post::Reveal(reveal), Post::Question {
            index: 1,
            total: 2,
            text: q2_text,
        }] => {
            assert_eq!(reveal.correct_users, vec![ALICE]);
            let (index, _) = open_question(&posts);
            let q2 = AnswerRoute::new(CH, index);

            // Both right on question 2.
            h.engine
                .submit_answer(q2, BOB, &right_answer(q2_text))
                .await;
            h.engine
                .submit_answer(q2, ALICE, &right_answer(q2_text))
                .await;
        }
        other => panic!("expected reveal then question 2, got {other:?}"),
    }
    settle(&h).await;

    // 8. After the last question: its reveal, then the final tally.
    let posts = h.outbox.take();
    match posts.as_slice() {
        [Post::Reveal(reveal), Post::Tally(rows)] => {
            // Mentions are listed in join order, not answer order.
            assert_eq!(reveal.correct_users, vec![ALICE, BOB]);
            assert_eq!(
                rows,
                &vec![
                    TallyRow {
                        rank: 1,
                        user: ALICE,
                        score: 2
                    },
                    TallyRow {
                        rank: 2,
                        user: BOB,
                        score: 1
                    },
                ]
            );
        }
        other => panic!("expected final reveal then tally, got {other:?}"),
    }
    assert!(!h.engine.has_round(CH).await, "round cleaned up");
}

#[tokio::test(start_paused = true, flavor = "current_thread")]
async fn concurrent_submits_do_not_deadlock() {
    let h = harness(20);
    let players: Vec<u64> = (10..30).collect();
    lobby_with(&h, &players).await;
    h.engine.start(CH, HOST).await;
    let (index, _) = open_question(&h.outbox.take());
    let route = AnswerRoute::new(CH, index);
    let tasks: Vec<_> = players
        .iter()
        .map(|&p| {
            let engine = Arc::clone(&h.engine);
            tokio::spawn(async move { engine.submit_answer(route, p, "x").await })
        })
        .collect();
    for t in tasks {
        assert_eq!(t.await.unwrap(), replies::ANSWER_RECEIVED);
    }
    settle(&h).await;
    let posts = h.outbox.posts();
    assert_eq!(
        posts
            .iter()
            .filter(|p| matches!(p, Post::Reveal(_)))
            .count(),
        1,
        "exactly one close: {posts:?}"
    );
}
