## [Unreleased] — test/ragtime-round-state-machine

### Added
- `ragtime::round`: Discord-free round state machine API (`RoundRegistry`, `RoundEvent`, `RoundError`, `Phase`, `Reveal`, `Standing`, `rank`, `select_questions`, `Grader`) with `todo!()` stubs and documented constants (`QUESTIONS_PER_ROUND`, `QUESTION_TIMEOUT`, `LOBBY_TIMEOUT`, `HOST_ABANDON_TIMEOUT`) — TDD PR 1.
- `crates/ragtime/tests/round_state_machine.rs`: 43 unit tests (host-only start, one round per channel, <20 questions, zero joiners, duplicate/non-joined/stale answers, early and timeout close, reveal list, tally ranking and ties, lobby/abandon/cancel cleanup).
- `rand` dependency for injectable question selection.
- `wiki/bots/Ragtime.md` § Round State Machine.
