# Changelog

Running log of all significant work done on starbunk-rs.
Add an entry under today's date for every PR or significant change.

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
