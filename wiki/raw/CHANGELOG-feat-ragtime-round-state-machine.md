## [Unreleased] — feat/ragtime-round-state-machine

### Added
- `ragtime::round` implementation: `RoundRegistry` (one round per channel, host-only start/cancel, idempotent lobby joins, late joins rejected), random question selection (`partial_shuffle`, up to 20), injected `Grader`, early close when every player answered, 2-minute timeout close via `tick(now)`, reveal with correct players in join order, ranked tally (shared rank, join order within ties), lobby (10 min) and abandon (6 min without answers) cleanup.
- Enabled the 42 round state machine tests from PR 1 (`#[ignore]` removed; tests otherwise unchanged).
