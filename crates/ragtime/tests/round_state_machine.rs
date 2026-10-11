//! Unit tests for the Discord-free round state machine (`ragtime::round`).
//!
//! Time is injected as explicit `Instant`s and randomness via a seeded `StdRng`,
//! so nothing here sleeps or touches Discord. A stand-in grader (exact,
//! case-insensitive match against the acceptable answers) is injected because
//! the fuzzy `grade()` function is owned by a separate card.
//!
//! TDD PR 1: every behaviour test is `#[ignore]`d so CI stays green while the
//! module is `todo!()` (sheeshbot precedent). Run the red suite with
//! `cargo test -p ragtime --test round_state_machine -- --ignored`.
//! PR 2 removes the `#[ignore]` attributes; the tests must then pass unchanged.

use chrono::Utc;
use ragtime::question::Question;
use ragtime::round::{
    rank, select_questions, CleanupReason, JoinOutcome, Phase, Reveal, RoundError, RoundEvent,
    RoundRegistry, Standing, HOST_ABANDON_TIMEOUT, LOBBY_TIMEOUT, QUESTIONS_PER_ROUND,
    QUESTION_TIMEOUT,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

const CH: u64 = 100;
const HOST: u64 = 1;
const ALICE: u64 = 10;
const BOB: u64 = 20;
const CAROL: u64 = 30;

fn q(id: i64) -> Question {
    let now = Utc::now();
    Question {
        id,
        question: format!("Question {id}?"),
        display_answer: format!("Answer {id}"),
        acceptable_answers: vec![format!("a{id}")],
        created_at: now,
        updated_at: now,
    }
}

fn pool(n: usize) -> Vec<Question> {
    (1..=n as i64).map(q).collect()
}

fn registry() -> RoundRegistry {
    RoundRegistry::new(Arc::new(|reply: &str, _display: &str, ok: &[String]| {
        ok.iter().any(|a| a.eq_ignore_ascii_case(reply.trim()))
    }))
}

fn rng() -> StdRng {
    StdRng::seed_from_u64(7)
}

/// Lobby in `CH` hosted by `HOST` with `players` joined.
fn lobby(players: &[u64], t0: Instant) -> RoundRegistry {
    let mut reg = registry();
    reg.create(CH, HOST, t0).unwrap();
    for &p in players {
        reg.join(CH, p).unwrap();
    }
    reg
}

/// Started round at `t0`; returns the registry and the first question
/// (`qs[0]`). Later questions are read from the events that open them.
fn started(players: &[u64], n_questions: usize, t0: Instant) -> (RoundRegistry, Vec<Question>) {
    let mut reg = lobby(players, t0);
    let events = reg
        .start(CH, HOST, pool(n_questions), &mut rng(), t0)
        .unwrap();
    let first = match &events[..] {
        [RoundEvent::PostQuestion { question, .. }] => question.clone(),
        other => panic!("expected a single PostQuestion, got {other:?}"),
    };
    (reg, vec![first])
}

/// The correct reply for a question under the stand-in grader.
fn right(question: &Question) -> String {
    question.acceptable_answers[0].clone()
}

fn reveal(events: &[RoundEvent]) -> &Reveal {
    events
        .iter()
        .find_map(|e| match e {
            RoundEvent::PostReveal(r) => Some(r),
            _ => None,
        })
        .expect("expected a PostReveal")
}

fn next_question(events: &[RoundEvent]) -> Option<(usize, Question)> {
    events.iter().find_map(|e| match e {
        RoundEvent::PostQuestion {
            index, question, ..
        } => Some((*index, question.clone())),
        _ => None,
    })
}

fn tally(events: &[RoundEvent]) -> Option<&[Standing]> {
    events.iter().find_map(|e| match e {
        RoundEvent::PostTally { standings, .. } => Some(standings.as_slice()),
        _ => None,
    })
}

fn cleanup(events: &[RoundEvent]) -> Option<CleanupReason> {
    events.iter().find_map(|e| match e {
        RoundEvent::Cleanup { reason, .. } => Some(*reason),
        _ => None,
    })
}

// ── registry: one round per channel ─────────────────────────────────────────

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn create_opens_lobby_with_host() {
    let t0 = Instant::now();
    let mut reg = registry();
    reg.create(CH, HOST, t0).unwrap();
    assert_eq!(reg.phase(CH), Some(Phase::Lobby));
    assert_eq!(reg.host(CH), Some(HOST));
    assert_eq!(reg.players(CH), Some(&[][..]), "host is not auto-joined");
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn second_round_in_same_channel_is_rejected() {
    let t0 = Instant::now();
    let mut reg = registry();
    reg.create(CH, HOST, t0).unwrap();
    assert_eq!(
        reg.create(CH, ALICE, t0),
        Err(RoundError::RoundAlreadyActive)
    );
    assert_eq!(reg.host(CH), Some(HOST), "original round untouched");
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn different_channels_are_independent() {
    let t0 = Instant::now();
    let mut reg = registry();
    reg.create(CH, HOST, t0).unwrap();
    reg.create(CH + 1, ALICE, t0).unwrap();
    assert_eq!(reg.host(CH + 1), Some(ALICE));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn channel_is_free_again_after_cleanup() {
    let t0 = Instant::now();
    let mut reg = lobby(&[], t0);
    let events = reg.start(CH, HOST, pool(3), &mut rng(), t0).unwrap();
    assert_eq!(cleanup(&events), Some(CleanupReason::NoPlayers));
    reg.create(CH, ALICE, t0).unwrap();
    assert_eq!(reg.host(CH), Some(ALICE));
}

// ── joining ─────────────────────────────────────────────────────────────────

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn join_is_idempotent_and_keeps_join_order() {
    let t0 = Instant::now();
    let mut reg = lobby(&[], t0);
    assert_eq!(reg.join(CH, BOB), Ok(JoinOutcome::Joined));
    assert_eq!(reg.join(CH, ALICE), Ok(JoinOutcome::Joined));
    assert_eq!(reg.join(CH, BOB), Ok(JoinOutcome::AlreadyJoined));
    assert_eq!(reg.players(CH), Some(&[BOB, ALICE][..]));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn host_can_join_as_a_player() {
    let t0 = Instant::now();
    let mut reg = lobby(&[], t0);
    assert_eq!(reg.join(CH, HOST), Ok(JoinOutcome::Joined));
    assert_eq!(reg.players(CH), Some(&[HOST][..]));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn join_without_round_is_rejected() {
    let mut reg = registry();
    assert_eq!(reg.join(CH, ALICE), Err(RoundError::NoActiveRound));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn late_join_after_start_is_rejected() {
    let t0 = Instant::now();
    let (mut reg, _) = started(&[ALICE], 3, t0);
    assert_eq!(reg.join(CH, BOB), Err(RoundError::AlreadyStarted));
    assert_eq!(reg.players(CH), Some(&[ALICE][..]));
}

// ── starting ────────────────────────────────────────────────────────────────

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn only_host_can_start() {
    let t0 = Instant::now();
    let mut reg = lobby(&[ALICE], t0);
    assert_eq!(reg.ensure_can_start(CH, ALICE), Err(RoundError::NotHost));
    assert_eq!(
        reg.start(CH, ALICE, pool(3), &mut rng(), t0),
        Err(RoundError::NotHost)
    );
    assert_eq!(
        reg.phase(CH),
        Some(Phase::Lobby),
        "non-host start changes nothing"
    );

    assert_eq!(reg.ensure_can_start(CH, HOST), Ok(()));
    assert!(reg.start(CH, HOST, pool(3), &mut rng(), t0).is_ok());
    assert_eq!(reg.phase(CH), Some(Phase::InProgress { index: 0 }));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn start_without_round_is_rejected() {
    let mut reg = registry();
    let t0 = Instant::now();
    assert_eq!(
        reg.ensure_can_start(CH, HOST),
        Err(RoundError::NoActiveRound)
    );
    assert_eq!(
        reg.start(CH, HOST, pool(3), &mut rng(), t0),
        Err(RoundError::NoActiveRound)
    );
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn start_twice_is_rejected() {
    let t0 = Instant::now();
    let (mut reg, _) = started(&[ALICE], 3, t0);
    assert_eq!(
        reg.ensure_can_start(CH, HOST),
        Err(RoundError::AlreadyStarted)
    );
    assert_eq!(
        reg.start(CH, HOST, pool(3), &mut rng(), t0),
        Err(RoundError::AlreadyStarted)
    );
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn start_posts_first_question_with_deadline() {
    let t0 = Instant::now();
    let mut reg = lobby(&[ALICE], t0);
    let events = reg.start(CH, HOST, pool(5), &mut rng(), t0).unwrap();
    match &events[..] {
        [RoundEvent::PostQuestion {
            channel,
            index,
            total,
            deadline,
            ..
        }] => {
            assert_eq!(*channel, CH);
            assert_eq!(*index, 0);
            assert_eq!(*total, 5);
            assert_eq!(*deadline, t0 + QUESTION_TIMEOUT);
        }
        other => panic!("expected one PostQuestion, got {other:?}"),
    }
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn start_uses_at_most_twenty_questions() {
    let t0 = Instant::now();
    let mut reg = lobby(&[ALICE], t0);
    let events = reg.start(CH, HOST, pool(50), &mut rng(), t0).unwrap();
    match &events[0] {
        RoundEvent::PostQuestion { total, .. } => assert_eq!(*total, QUESTIONS_PER_ROUND),
        other => panic!("expected PostQuestion, got {other:?}"),
    }
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn fewer_than_twenty_questions_uses_all_of_them() {
    let t0 = Instant::now();
    let mut reg = lobby(&[ALICE], t0);
    let mut events = reg.start(CH, HOST, pool(3), &mut rng(), t0).unwrap();
    let mut seen = Vec::new();
    loop {
        let (index, question) = next_question(&events).expect("question");
        assert_eq!(index, seen.len());
        seen.push(question.id);
        events = reg.submit_answer(CH, ALICE, index, "whatever", t0).unwrap();
        if tally(&events).is_some() {
            break;
        }
    }
    let distinct: HashSet<_> = seen.iter().copied().collect();
    assert_eq!(seen.len(), 3);
    assert_eq!(distinct, HashSet::from([1, 2, 3]));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn zero_joiners_ends_silently_and_removes_round() {
    let t0 = Instant::now();
    let mut reg = lobby(&[], t0);
    let events = reg.start(CH, HOST, pool(5), &mut rng(), t0).unwrap();
    assert_eq!(
        events,
        vec![RoundEvent::Cleanup {
            channel: CH,
            reason: CleanupReason::NoPlayers
        }],
        "no question, reveal or tally events"
    );
    assert_eq!(reg.phase(CH), None);
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn empty_question_pool_cleans_up() {
    let t0 = Instant::now();
    let mut reg = lobby(&[ALICE], t0);
    let events = reg.start(CH, HOST, Vec::new(), &mut rng(), t0).unwrap();
    assert_eq!(
        events,
        vec![RoundEvent::Cleanup {
            channel: CH,
            reason: CleanupReason::NoQuestions
        }]
    );
    assert_eq!(reg.phase(CH), None);
}

// ── question selection ──────────────────────────────────────────────────────

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn select_questions_is_distinct_and_capped() {
    let picked = select_questions(pool(50), 20, &mut rng());
    assert_eq!(picked.len(), 20);
    let ids: HashSet<_> = picked.iter().map(|q| q.id).collect();
    assert_eq!(ids.len(), 20, "no duplicates");
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn select_questions_small_pool_returns_all() {
    let picked = select_questions(pool(3), 20, &mut rng());
    let ids: HashSet<_> = picked.iter().map(|q| q.id).collect();
    assert_eq!(ids, HashSet::from([1, 2, 3]));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn select_questions_is_deterministic_for_a_seed_and_random_across_seeds() {
    let a = select_questions(pool(50), 20, &mut StdRng::seed_from_u64(1));
    let b = select_questions(pool(50), 20, &mut StdRng::seed_from_u64(1));
    let c = select_questions(pool(50), 20, &mut StdRng::seed_from_u64(2));
    let ids = |v: &[Question]| v.iter().map(|q| q.id).collect::<Vec<_>>();
    assert_eq!(ids(&a), ids(&b));
    assert_ne!(ids(&a), ids(&c));
    assert_ne!(
        ids(&a),
        (1..=20).collect::<Vec<_>>(),
        "not just the first 20"
    );
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn select_questions_zero_is_empty() {
    assert!(select_questions(pool(5), 0, &mut rng()).is_empty());
}

// ── answering ───────────────────────────────────────────────────────────────

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn can_answer_reports_not_in_round_already_answered_and_index() {
    let t0 = Instant::now();
    let (mut reg, qs) = started(&[ALICE, BOB], 3, t0);
    assert_eq!(reg.can_answer(CH, CAROL), Err(RoundError::NotInRound));
    assert_eq!(reg.can_answer(CH, ALICE), Ok(0));
    reg.submit_answer(CH, ALICE, 0, &right(&qs[0]), t0).unwrap();
    assert_eq!(reg.can_answer(CH, ALICE), Err(RoundError::AlreadyAnswered));
    assert_eq!(reg.can_answer(CH, BOB), Ok(0));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn can_answer_in_lobby_or_without_round() {
    let t0 = Instant::now();
    let reg = lobby(&[ALICE], t0);
    assert_eq!(reg.can_answer(CH, ALICE), Err(RoundError::NotInProgress));
    assert_eq!(
        reg.can_answer(CH + 1, ALICE),
        Err(RoundError::NoActiveRound)
    );
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn duplicate_answer_is_rejected() {
    let t0 = Instant::now();
    let (mut reg, qs) = started(&[ALICE, BOB], 3, t0);
    assert_eq!(reg.submit_answer(CH, ALICE, 0, "wrong", t0), Ok(vec![]));
    assert_eq!(
        reg.submit_answer(CH, ALICE, 0, &right(&qs[0]), t0),
        Err(RoundError::AlreadyAnswered)
    );
    // The first (wrong) answer stands.
    let events = reg.submit_answer(CH, BOB, 0, "wrong", t0).unwrap();
    assert!(reveal(&events).correct.is_empty());
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn answer_from_non_joined_player_is_not_in_round() {
    let t0 = Instant::now();
    let (mut reg, qs) = started(&[ALICE], 3, t0);
    assert_eq!(
        reg.submit_answer(CH, CAROL, 0, &right(&qs[0]), t0),
        Err(RoundError::NotInRound)
    );
    assert_eq!(reg.phase(CH), Some(Phase::InProgress { index: 0 }));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn answer_before_start_is_not_in_progress() {
    let t0 = Instant::now();
    let mut reg = lobby(&[ALICE], t0);
    assert_eq!(
        reg.submit_answer(CH, ALICE, 0, "x", t0),
        Err(RoundError::NotInProgress)
    );
    assert_eq!(
        reg.submit_answer(CH + 1, ALICE, 0, "x", t0),
        Err(RoundError::NoActiveRound)
    );
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn answer_for_closed_question_is_stale() {
    let t0 = Instant::now();
    let (mut reg, _) = started(&[ALICE, BOB], 3, t0);
    reg.tick(t0 + QUESTION_TIMEOUT); // closes question 0
    assert_eq!(reg.phase(CH), Some(Phase::InProgress { index: 1 }));
    assert_eq!(
        reg.submit_answer(CH, ALICE, 0, "late", t0 + QUESTION_TIMEOUT),
        Err(RoundError::StaleQuestion)
    );
}

// ── closing a question ──────────────────────────────────────────────────────

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn question_stays_open_until_everyone_answered() {
    let t0 = Instant::now();
    let (mut reg, qs) = started(&[ALICE, BOB, CAROL], 3, t0);
    assert_eq!(
        reg.submit_answer(CH, ALICE, 0, &right(&qs[0]), t0),
        Ok(vec![])
    );
    assert_eq!(reg.submit_answer(CH, BOB, 0, "nope", t0), Ok(vec![]));
    assert_eq!(reg.phase(CH), Some(Phase::InProgress { index: 0 }));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn early_close_when_everyone_answered() {
    let t0 = Instant::now();
    let t1 = t0 + Duration::from_secs(10);
    let (mut reg, qs) = started(&[ALICE, BOB], 3, t0);
    reg.submit_answer(CH, ALICE, 0, &right(&qs[0]), t0).unwrap();
    let events = reg.submit_answer(CH, BOB, 0, "nope", t1).unwrap();

    assert_eq!(events.len(), 2, "reveal then next question: {events:?}");
    assert!(matches!(events[0], RoundEvent::PostReveal(_)));
    match &events[1] {
        RoundEvent::PostQuestion {
            index, deadline, ..
        } => {
            assert_eq!(*index, 1);
            assert_eq!(
                *deadline,
                t1 + QUESTION_TIMEOUT,
                "fresh timer for the next question"
            );
        }
        other => panic!("expected PostQuestion, got {other:?}"),
    }
    assert_eq!(reg.phase(CH), Some(Phase::InProgress { index: 1 }));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn reveal_has_question_display_answer_and_correct_users_in_join_order() {
    let t0 = Instant::now();
    let (mut reg, qs) = started(&[CAROL, ALICE, BOB], 3, t0);
    reg.submit_answer(CH, ALICE, 0, &right(&qs[0]), t0).unwrap();
    reg.submit_answer(CH, BOB, 0, "wrong", t0).unwrap();
    let events = reg
        .submit_answer(CH, CAROL, 0, &right(&qs[0]).to_uppercase(), t0)
        .unwrap();
    let r = reveal(&events);
    assert_eq!(r.channel, CH);
    assert_eq!(r.index, 0);
    assert_eq!(r.question, qs[0]);
    assert_eq!(r.display_answer, qs[0].display_answer);
    assert_eq!(
        r.correct,
        vec![CAROL, ALICE],
        "graded by the injected grader, join order"
    );
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn timeout_closes_question_with_partial_answers() {
    let t0 = Instant::now();
    let (mut reg, qs) = started(&[ALICE, BOB], 3, t0);
    reg.submit_answer(CH, ALICE, 0, &right(&qs[0]), t0).unwrap();

    assert!(
        reg.tick(t0 + QUESTION_TIMEOUT - Duration::from_millis(1))
            .is_empty(),
        "nothing happens before the deadline"
    );
    let events = reg.tick(t0 + QUESTION_TIMEOUT);
    assert_eq!(reveal(&events).correct, vec![ALICE]);
    assert_eq!(next_question(&events).map(|(i, _)| i), Some(1));
    assert_eq!(reg.phase(CH), Some(Phase::InProgress { index: 1 }));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn stale_timer_after_early_close_is_a_no_op() {
    let t0 = Instant::now();
    let t1 = t0 + Duration::from_secs(30);
    let (mut reg, qs) = started(&[ALICE], 3, t0);
    reg.submit_answer(CH, ALICE, 0, &right(&qs[0]), t1).unwrap(); // early close at t1
                                                                  // The timer scheduled for question 0 fires at t0 + 2min; question 1 runs until t1 + 2min.
    assert!(reg.tick(t0 + QUESTION_TIMEOUT).is_empty());
    assert_eq!(reg.phase(CH), Some(Phase::InProgress { index: 1 }));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn next_deadline_tracks_lobby_and_question_timers() {
    let t0 = Instant::now();
    let mut reg = lobby(&[ALICE], t0);
    assert_eq!(reg.next_deadline(CH), Some(t0 + LOBBY_TIMEOUT));
    let t1 = t0 + Duration::from_secs(5);
    reg.start(CH, HOST, pool(3), &mut rng(), t1).unwrap();
    assert_eq!(reg.next_deadline(CH), Some(t1 + QUESTION_TIMEOUT));
    assert_eq!(reg.next_deadline(CH + 1), None);
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn last_question_close_posts_reveal_then_tally_then_cleanup() {
    let t0 = Instant::now();
    let (mut reg, _) = started(&[ALICE], 1, t0);
    let events = reg.submit_answer(CH, ALICE, 0, "x", t0).unwrap();
    assert_eq!(events.len(), 3, "{events:?}");
    assert!(matches!(events[0], RoundEvent::PostReveal(_)));
    assert!(matches!(
        events[1],
        RoundEvent::PostTally { channel: CH, .. }
    ));
    assert_eq!(
        events[2],
        RoundEvent::Cleanup {
            channel: CH,
            reason: CleanupReason::Finished
        }
    );
    assert_eq!(reg.phase(CH), None, "finished round removed from registry");
}

// ── tally ───────────────────────────────────────────────────────────────────

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn full_round_tally_counts_one_point_per_correct_answer() {
    let t0 = Instant::now();
    let mut reg = lobby(&[ALICE, BOB, CAROL], t0);
    let mut events = reg.start(CH, HOST, pool(3), &mut rng(), t0).unwrap();
    // ALICE: 3/3, BOB: 1/3 (first only), CAROL: 0/3 (times out on the last question).
    for i in 0..3 {
        let (index, question) = next_question(&events).unwrap();
        assert_eq!(index, i);
        reg.submit_answer(CH, ALICE, index, &right(&question), t0)
            .unwrap();
        let bob = if i == 0 {
            right(&question)
        } else {
            "no".into()
        };
        events = reg.submit_answer(CH, BOB, index, &bob, t0).unwrap();
        assert!(events.is_empty(), "carol hasn't answered");
        if i < 2 {
            events = reg.submit_answer(CH, CAROL, index, "no", t0).unwrap();
        } else {
            events = reg.tick(t0 + QUESTION_TIMEOUT);
        }
    }
    assert_eq!(
        tally(&events).unwrap(),
        &[
            Standing {
                rank: 1,
                user: ALICE,
                score: 3
            },
            Standing {
                rank: 2,
                user: BOB,
                score: 1
            },
            Standing {
                rank: 3,
                user: CAROL,
                score: 0
            },
        ]
    );
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn rank_orders_by_score_descending() {
    assert_eq!(
        rank(&[(ALICE, 1), (BOB, 5), (CAROL, 3)]),
        vec![
            Standing {
                rank: 1,
                user: BOB,
                score: 5
            },
            Standing {
                rank: 2,
                user: CAROL,
                score: 3
            },
            Standing {
                rank: 3,
                user: ALICE,
                score: 1
            },
        ]
    );
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn rank_ties_share_rank_keep_join_order_and_skip_next_rank() {
    // Join order: CAROL, ALICE, BOB, 40. CAROL and BOB tie on 4.
    assert_eq!(
        rank(&[(CAROL, 4), (ALICE, 7), (BOB, 4), (40, 1)]),
        vec![
            Standing {
                rank: 1,
                user: ALICE,
                score: 7
            },
            Standing {
                rank: 2,
                user: CAROL,
                score: 4
            },
            Standing {
                rank: 2,
                user: BOB,
                score: 4
            },
            Standing {
                rank: 4,
                user: 40,
                score: 1
            },
        ]
    );
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn rank_all_zero_is_a_shared_first_place() {
    let standings = rank(&[(BOB, 0), (ALICE, 0)]);
    assert_eq!(
        standings,
        vec![
            Standing {
                rank: 1,
                user: BOB,
                score: 0
            },
            Standing {
                rank: 1,
                user: ALICE,
                score: 0
            },
        ]
    );
    assert!(rank(&[]).is_empty());
}

// ── abandonment ─────────────────────────────────────────────────────────────

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn unstarted_lobby_is_cleaned_up_after_lobby_timeout() {
    let t0 = Instant::now();
    let mut reg = lobby(&[ALICE], t0);
    assert!(reg
        .tick(t0 + LOBBY_TIMEOUT - Duration::from_secs(1))
        .is_empty());
    assert_eq!(reg.phase(CH), Some(Phase::Lobby));
    assert_eq!(
        reg.tick(t0 + LOBBY_TIMEOUT),
        vec![RoundEvent::Cleanup {
            channel: CH,
            reason: CleanupReason::LobbyTimeout
        }]
    );
    assert_eq!(reg.phase(CH), None);
    assert!(
        reg.tick(t0 + LOBBY_TIMEOUT * 2).is_empty(),
        "cleanup happens once"
    );
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn round_with_no_answers_is_abandoned() {
    let t0 = Instant::now();
    let (mut reg, _) = started(&[ALICE, BOB], 20, t0);
    // Questions keep timing out with no answers until HOST_ABANDON_TIMEOUT.
    let mut now = t0;
    let mut reason = None;
    while reason.is_none() {
        now += QUESTION_TIMEOUT;
        assert!(now <= t0 + HOST_ABANDON_TIMEOUT, "abandon by the timeout");
        let events = reg.tick(now);
        reason = cleanup(&events);
        if reason.is_some() {
            assert!(
                tally(&events).is_none() && next_question(&events).is_none(),
                "no tally or new question for an abandoned round: {events:?}"
            );
        }
    }
    assert_eq!(reason, Some(CleanupReason::Abandoned));
    assert_eq!(reg.phase(CH), None);
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn answers_keep_round_alive_past_abandon_timeout() {
    let t0 = Instant::now();
    let (mut reg, _) = started(&[ALICE, BOB], 20, t0);
    // ALICE answers every question; BOB never does, so each question times out.
    let mut now = t0;
    for i in 0..5 {
        reg.submit_answer(CH, ALICE, i, "x", now).unwrap();
        now += QUESTION_TIMEOUT;
        let events = reg.tick(now);
        assert_eq!(cleanup(&events), None, "question {i}");
    }
    assert!(now > t0 + HOST_ABANDON_TIMEOUT);
    assert_eq!(reg.phase(CH), Some(Phase::InProgress { index: 5 }));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn host_can_cancel_and_others_cannot() {
    let t0 = Instant::now();
    let (mut reg, _) = started(&[ALICE], 3, t0);
    assert_eq!(reg.cancel(CH, ALICE), Err(RoundError::NotHost));
    assert_eq!(
        reg.cancel(CH, HOST),
        Ok(vec![RoundEvent::Cleanup {
            channel: CH,
            reason: CleanupReason::Cancelled
        }])
    );
    assert_eq!(reg.phase(CH), None);
    assert_eq!(reg.cancel(CH, HOST), Err(RoundError::NoActiveRound));
}

#[test]
#[ignore = "red: enable in PR 2 when ragtime::round is implemented"]
fn tick_only_affects_expired_rounds() {
    let t0 = Instant::now();
    let t1 = t0 + Duration::from_secs(5 * 60);
    let mut reg = registry();
    reg.create(CH, HOST, t0).unwrap();
    reg.create(CH + 1, HOST, t1).unwrap();
    let events = reg.tick(t0 + LOBBY_TIMEOUT);
    assert_eq!(
        events,
        vec![RoundEvent::Cleanup {
            channel: CH,
            reason: CleanupReason::LobbyTimeout
        }]
    );
    assert_eq!(reg.phase(CH + 1), Some(Phase::Lobby));
}

#[test]
fn timeout_constants_match_the_spec() {
    assert_eq!(QUESTIONS_PER_ROUND, 20);
    assert_eq!(QUESTION_TIMEOUT, Duration::from_secs(120));
    assert_eq!(LOBBY_TIMEOUT, Duration::from_secs(600));
    assert!(
        HOST_ABANDON_TIMEOUT > QUESTION_TIMEOUT,
        "a single quiet question must not abandon"
    );
}
