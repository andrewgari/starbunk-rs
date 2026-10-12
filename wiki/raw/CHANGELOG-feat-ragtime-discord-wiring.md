## [Unreleased] — feat/ragtime-discord-wiring

### Added
- `ragtime::bot`: live Discord wiring for the Ragtime contest — @mention opens a lobby, 🙋 reaction
  joins, `/ragtime start|cancel` (host only), \[Answer\] button → modal, ephemeral acknowledgement
  that never reveals correctness, reveal → next question → final tally.
- tokio timers for the 2-minute question timeout, 10-minute lobby timeout and 6-minute abandon
  timeout, with per-channel timer replacement so stale timers never close a later question.
- `RagtimeEngine` + `Outbox` trait so the whole game loop is tested without Discord;
  `tests/discord_wiring.rs` (17 tests, paused tokio time) covers full 20-question rounds,
  timeouts, abandonment, cancel, edge-case ephemerals and concurrent submits.
- `crates/ragtime/README.md` with commands and timeout values.
- Interim exact-match grader until the fuzzy `grade()` lands.

### Changed
- `PgQuestionRepository` implemented with sqlx (all 11 contract tests pass against Postgres 16).
- `ragtime::run` now connects, runs migrations and starts the bot with the reaction intent.
