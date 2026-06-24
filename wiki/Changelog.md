# Changelog

Running log of all significant work done on starbunk-rs.
Add an entry under today's date for every PR or significant change.

## 2026-06-24 — Fix Postgres GKE CrashLoopBackOff

### Fixed
- Fixed an issue where the GKE Postgres StatefulSet would get stuck in a `CrashLoopBackOff` state due to `initdb` failing when a `lost+found` directory is present in the persistent volume mount. This was resolved by setting the `PGDATA` environment variable to a subdirectory.

---

## 2026-06-18 — CovaBot Personality and Engagement System

### Added
- Created `personality.rs` to load and parse CovaBot's personality profile (YAML) which includes `topic_affinities` and `social_battery_config`.
- Implemented `GateReason::TopicAffinity` in `engagement.rs` to pull CovaBot into conversations involving topics he cares about (e.g., "Cheeseburgers").
- Implemented Social Battery logic in `engagement.rs` to suppress ambient conversational engagement when the battery drops below 20%.

### Changed
- Updated `crates/covabot/src/lib.rs` to dynamically load `config/bots/covabot.yml` on startup and pass topic affinities to the `EngagementManager`.
- Updated `engagement.rs` unit tests to ensure that direct mentions and direct replies override battery constraints, while ambient responses respect dampening.

---

## 2026-06-18 — Secure BunkBot Configuration Deployment

### Added
- Configured standard secret volume in `kubernetes/bunkbot.yaml` to mount `bots.yml` from `starbunk-secrets` under key `BOTS_CONFIG_YAML` at `/app/config/bots.yml`.
- Updated `.gitignore` to prevent committing local `config/bots.yml` configurations to GitHub.
- Updated `scripts/kube_secrets.sh` to package and upload `config/bots.yml` to GKE's `starbunk-secrets` dynamically from the local workspace (moved from root).
- Added `scripts/deploy_config.sh` script to automate base64 encoding of `bots.yml`, GKE secret patching, and zero-downtime rolling restart of BunkBot.
- Added `scripts/deploy_k8s.sh` script to automate GKE Kubernetes manifest deployment, including optional image tag pinning and rollout status monitoring.
- Added `scripts/restart_bots.sh` script to perform zero-downtime rolling restarts of specific bots or all bots in GKE.

### Changed
- Updated `wiki/infrastructure/Configuration.md` and `wiki/bots/BunkBot.md` to document the secret-based deployment strategy and scripts usage.

---

## 2026-06-18 — BunkBot State Manager and Slash Commands Implementation

### Added
- Implemented `BotStateService` and `InMemoryBotStateManager` in `crates/bunkbot/src/state.rs` to allow dynamic toggling of bots and frequency overrides.
- Added `/bot` slash command with `enable`, `disable`, `override`, `reset`, and `list` subcommands in `crates/bunkbot/src/commands/bot.rs` to control bot behavior at runtime.
- Integrated the state manager into `Engine` (`crates/bunkbot/src/engine.rs`) to evaluate bot enablement and overrides during message processing.

### Changed
- Updated `crates/bunkbot/src/commands.rs` to route the `/bot` command to the new handlers.
- Updated `wiki/bots/BunkBot.md` to document the new state manager and administrative slash commands.

---

## 2026-06-15 — Logging and Observability Improvements for DJCova

### Added
- Added `opentelemetry` dependency in `crates/djcova/Cargo.toml`.
- Implemented `record_error` metric helper function in `crates/djcova/src/lib.rs` to report bot errors to Prometheus via OpenTelemetry's global meter.

### Changed
- Refactored `ready`, `interaction_create`, and `voice_state_update` in `crates/djcova/src/lib.rs` to use structured logging with standard fields (`bot = "djcova"`, `guild`, `user_id`, etc.) and log failures at the appropriate levels.
- Improved logging throughout `GuildAudioManager` in `crates/djcova/src/manager.rs` to trace healthy playback events (play, queue, skip, stop, pause, resume, volume adjustment, shuffle, repeat mode, timer expiration, gif loop activity) and failures.
- Updated button interaction handling in `crates/djcova/src/commands/buttons.rs` and the play command in `crates/djcova/src/commands/play.rs` to capture and log errors, incrementing the error metrics appropriately.

---

## 2026-06-15 — Asynchronous YouTube Metadata Resolution for DJCova

### Changed
- Modified `VoiceService::play` signature and `DiscordVoiceService::play` implementation to no longer block on `yt-dlp` metadata resolution synchronously. Audio playback now starts immediately after joining.
- Changed `GuildAudioManager::play` to return immediately with a "Loading..." title placeholder and a unique `id` for each `QueueItem`.
- Updated `/play` command handler in `crates/djcova/src/commands/play.rs` to spawn a background tokio task that resolves metadata asynchronously using `VoiceService::resolve_metadata` and edits the original interaction response via `cmd.edit_response` when resolved.
- Scoped the `GuildAudioManager` mutex lock within the `/play` background task so it drops before calling `cmd.edit_response`, preventing voice channel command deadlocks and responsiveness stalls.
- Updated `GuildAudioManager::play` to search the current track, queue, and history for any duplicate URL, reusing existing resolved metadata immediately.

### Added
- Added `id` field to `QueueItem` and a `next_item_id` counter to `GuildAudioManager` to track and update individual queued tracks.
- Added a `get_voice_service` getter on `GuildAudioManager` to retrieve a clone of the inner `VoiceService` reference without holding the manager's lock during background resolution.
- Added `update_track_metadata` to `GuildAudioManager` to safely update track title, duration, and thumbnail of a queue item (current or queued) by its unique ID.
- Added a unit test `test_update_track_metadata` in `manager.rs` to verify that immediate playback starts with a placeholder and metadata is correctly updated when resolved.
- Added a unit test `test_metadata_caching` in `manager.rs` to verify duplicate URLs immediately reuse resolved metadata.

---

## 2026-06-11 — E2E Testing Framework

### Added
- Created a new workspace crate `crates/e2e` containing the E2E test runner.
- Added `crates/e2e/suites/bunkbot_bluebot.json` with sample E2E test cases for BlueBot and BunkBot.
- Added `E2eDebugHandler` wrapper in `crates/starbunk/src/discord/e2e.rs` to filter events and mock simulated user/bot authors for E2E validation.
- Documented the E2E framework in the wiki `wiki/development/Testing.md`.

### Changed
- Registered `crates/e2e` in the workspace `Cargo.toml`.
- Integrated `E2eDebugHandler` into `starbunk::utils::run_bot` conditionally activated via `E2E_MODE` environment variable.

---

## 2026-06-11 — Crate and container rename

### Changed
- Renamed the `starbunk-shared` workspace package and lib target to `starbunk`.
- Refactored `run_bot` and `default_intents` helpers into a new `starbunk::utils` module.
- Renamed all Docker containers and image references to use the simpler `starbunk-<bot>` prefix instead of `starbunk-rs-<bot>`.
- Updated all import paths, compose files, workflows, validation scripts, and wiki pages.

---

## 2026-06-10 — DJCova implementation and integration

### Added
- Completed implementation of the `djcova` music bot in Rust.
- Decoupled `VoiceService` and `GifService` traits with test mocks, implementing 9 comprehensive unit tests covering playback, queueing, repeat modes, skip logic, and disconnect/idle timers.
- Integrated slash commands (`/play`, `/skip`, `/stop`, `/queue`, `/nowplaying`, `/history`, `/shuffle`, `/help`, `/volume`, `/clear`, `/repeat`) and interactive button controls (Stop, Skip, Restart, Re-queue).
- Configured Docker build steps to download `yt-dlp` and `ffmpeg` when compiling the `djcova` bot.

---

## 2026-06-09 — Clean git history and conventional commit enforcement

### Added
- `scripts/git/hooks/commit-msg` python script to validate commit messages.
- `.gitmessage` template file in the repository root.

### Changed
- Rewrote the git history of the repository to be linear, flat, and strictly conform to Conventional Commits format.
- Updated `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.claude/commands/git-workflow.md`, and `.claude/agents/task-runner.md` to document and enforce conventional commits and hook validation.

---

## 2026-06-09 — Test parity with starbunk-go

### Added
- `middleware/author`: `author_named_is_case_sensitive` test.
- `middleware/content`: `has_attachment_passes_with_attachment` test.
- `middleware/mod`: BunkBot policy composition, Jeff-bot weekday-rejection, and Scenario 2 (`AnyOf`/`AllOf`/`Chance`) complex composition tests — all matching Go's `auditor_test.go`.
- `replybot/bot`: `SpyStrategy` helper with `Arc<AtomicUsize>` trigger counter shareable across box boundary; 9 new tests covering condition-pass, condition-fallthrough, mixed conditioned/unconditioned, AllOf multi-criteria, and NotSelf-via-bot-id — matching Go's `bot_test.go`.
- Total starbunk-shared tests: 65 (up from ~47).

---

## 2026-06-09 — Code parity, TDD, and idiomatic Rust refinements

### Added
- 75 unit tests across all modules (middleware, replybot, bluebot, covabot engagement, tagger, conversation tracker).
- `src/bots/covabot/conversation.rs` — 5 tests covering: new-conversation seeding, high-similarity join, low-similarity fork, empty tags, no channel crossover.
- `src/shared/middleware/` — 40+ tests covering all filters and combinators including complex composition scenarios (BlueBot policy, BunkBot policy, scenario-1/2 from Go parity).
- `src/shared/replybot/bot.rs` — 5 async tests via `MockSender` + `MockStrategy` with condition, identity, and first-match-wins coverage.
- `src/bots/bluebot/strategy.rs` — `name()` and `response()` tests; extended pattern matching test table with 16 positive and 7 negative cases.
- `src/shared/discord/identity.rs` — `Identity::resolve()` async method for resolving identity from Discord API.

### Changed
- `IdentityProvider` converted from concrete struct to `trait` for testability; `DiscordIdentityProvider` is the implementing type.
- `not_self_with_bot_id(bot_id: UserId)` filter added for test contexts where serenity `Context` is unavailable.
- Middleware `mod.rs` re-exports updated to include `not_author_id` and `not_self_with_bot_id`.
- All filter test helpers use `NonNull::<Context>::dangling()` (never dereferenced) instead of zeroed memory.

### Fixed
- `not_author_id` was defined but not re-exported from `middleware/mod.rs`.
- `FnFilter` type alias added to satisfy `clippy::type_complexity` lint.

---

## 2026-06-09 — Initial Rust port and project infrastructure

### Added
- Full Rust monorepo scaffold ported from starbunk-go.
- `src/bin/` entry points for all 5 bots (bluebot, bunkbot, covabot, djcova, ratbot).
- `src/shared/` — shared libraries: discord, llm, memory, middleware, replybot.
- `src/bots/` — per-bot implementation modules.
- `Cargo.toml` with all dependencies: serenity, tokio, async-trait, serde, reqwest, regex, tracing, sqlx, pgvector.
- Multi-stage `docker/Dockerfile` with dependency caching and unprivileged user.
- Production `docker-compose.yml` with all 5 bots + pgvector postgres.
- Dev `docker/docker-compose.yml` for local builds from source.
- GitHub Actions workflows: `ci.yml`, `main.yml`, `deploy.yml`.
- DevOps validation script `scripts/devops-validate.sh` adapted for `src/bin/` layout.
- Deployment scripts: `scripts/deployment/deploy.sh`, `scripts/deployment/health-check.sh`.
- Full wiki documentation ported and translated from starbunk-go.
- `AGENTS.md` and `CLAUDE.md` with Rust-specific guidance.
- `.env.example` with all environment variable documentation.

## 2026-06-09

### Changed
- **Workspace refactor**: migrated from a single Cargo package to a multi-crate
  workspace (`crates/starbunk-shared`, `crates/bluebot`, `crates/bunkbot`,
  `crates/covabot`, `crates/djcova`, `crates/ratbot`).
- CI `test` job now runs `cargo test -p <package>` per changed crate instead of
  `cargo test --all`, enabling true per-bot test isolation.
- Dockerfile updated for workspace layout (copies all crate manifests, uses
  `cargo build -p <bot>`).
- `devops-validate.sh` discovers bots from `crates/` instead of `src/bin/`.
- `AGENTS.md` and `wiki/infrastructure/Architecture.md` updated to reflect new
  workspace structure.
