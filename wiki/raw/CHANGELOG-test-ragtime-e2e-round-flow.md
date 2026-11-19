## [Unreleased] — test/ragtime-e2e-round-flow

### Added
- `crates/e2e/suites/ragtime.json`: live E2E suite for the Ragtime round flow's webhook-drivable
  steps (mention opens the "Ragtime contest" lobby; second mention / other user → "already running";
  bot mentions and non-mentions ignored). Its `not_covered` list records the steps the runner cannot
  drive (reaction join, `/ragtime`, buttons, modals, ephemeral replies, timeouts) and which tests
  cover them instead.
- `discord_wiring.rs::e2e_round_flow_two_questions`: one scripted engine-level run of the whole
  flow with a 2-question bank (lobby → join → non-host start rejected → host start → modal answer
  recorded → "not in this round" → early close + reveal → final tally).
- E2E runner: `{bot_mention}` placeholder in suite messages; `expect` now also matches embed
  title/description/fields; `ragtime` can be spawned (`E2E_TEST_BOTS=ragtime`; like ratbot it gets no `/health` probe because
  its health server only starts after the Postgres migrations, so a slow DB would fail the probe);
  unit tests that parse every committed suite.
- CI: "Run E2E — ragtime" step; ragtime changes now trigger the E2E job.

### Fixed
- `docker/docker-compose.e2e.yml` now mounts `crates/e2e/suites` into the runner container. The
  runtime image holds only the binary, so every CI E2E step's `E2E_SUITE_PATH` was unreadable and
  the runner silently fell back to the built-in bunkbot/bluebot suite.
- The runner now fails if an explicitly set `E2E_SUITE_PATH` can't be read, instead of running a
  different suite. `E2E_STARTUP_WAIT_SECS` (default 5) sets the post-spawn wait; the ragtime CI step
  uses 15 because ragtime runs migrations before connecting.

### Changed
- Ragtime's message handler classifies authors with `StarbunkMessage::from_serenity`, so E2E
  webhook posts (`[E2E_HUMAN]`, `[E2E_MOCK_USER:id]`) can open lobbies in E2E mode. Outside E2E
  mode bots and webhooks are still ignored.
- Dockerfile dependency-cache stage now copies/stubs the `ragtime` manifest (the E2E runner depends
  on it).
