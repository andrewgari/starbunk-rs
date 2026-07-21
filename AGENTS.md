# AGENTS.md

Canonical agent guide for `starbunk-rs`. All AI coding tools should read this file.
Claude Code loads it automatically via `CLAUDE.md`; other tools should read it directly.

---

## !! MANDATORY: KEEP THE WIKI UP TO DATE !!

> **Every agent, every task, without exception.**

1. **Before starting** — read the relevant `wiki/` page(s) for the area you will touch.
2. **After completing** — update those pages to reflect any changes.
3. **Every meaningful PR** — add an entry to `wiki/Changelog.md`.
4. No wiki page for the area? **Create it.**

The wiki lives at `wiki/`. Start at `wiki/Home.md`. "Meaningful" = anything that affects
architecture, configuration, bot behavior, infrastructure, deployment, testing, or a non-obvious bug fix.

**The `wiki/` folder syncs automatically to the hosted wiki at `wiki.starbunk.net`** via a Gitea
mirror (`StarbunkCrusaders/starbunk-wiki`) that Wiki.js pulls from every 5 minutes. Any `.md` file
you commit under `wiki/` (except `wiki/raw/`) will appear on the live wiki. Do not put draft or
scratch content directly in `wiki/` — use `wiki/raw/` for that (it is excluded from the live wiki).

---

## CHANGELOG Workflow

While on a branch: maintain `wiki/raw/CHANGELOG-<branch>.md` (excluded from live wiki).
Format: `## [Unreleased] — <branch>` with `### Added / Changed / Fixed` sections.
On merge: prepend to `wiki/Changelog.md`, delete the raw file.

---

## !! MANDATORY: TDD WORKFLOW !!

> **Every task involving logic, behavior, or ports. No exceptions.**

- **PR 1 — Tests only:** Add failing Rust tests that define the expected behavior. No implementation. Minimal stubs only to allow compilation. For bot logic changes, also add corresponding E2E integration test cases to the JSON test suite (e.g., `crates/e2e/suites/bunkbot_bluebot.json`).
- **PR 2 — Implementation:** Write code to make the unit and E2E tests pass. Only after PR 1 is approved or merged.

Reference behavior: check `../starbunk-go/cmd/<bot>/` or `../starbunk-js/src/<bot>/` first.
Full details: `wiki/development/TDD.md`.

---

## Before Every Change

1. Confirm you are in the correct worktree for this task — see `/worktree`.
2. Read every file you are about to change. Never edit from memory.
3. If touching `crates/starbunk-shared/`, grep all callers first (`grep -rn "Symbol" src/`).
4. After changes: run `/check`. If DevOps files touched: run `bash scripts/devops-validate.sh`.
5. Update the relevant `wiki/` page — see the table below.

| Changed area | Wiki page |
|---|---|
| Bot behaviour / config | `wiki/bots/<Bot>.md` |
| `crates/starbunk-shared/` | `wiki/infrastructure/Architecture.md` |
| CI/CD workflows | `wiki/development/CI-CD.md` |
| Testing patterns | `wiki/development/Testing.md` |
| Deployment / Docker | `wiki/infrastructure/Deployment.md` |
| Logging / tracing / OTEL | `wiki/development/Observability.md` |

---

## Definition of Done

- [ ] All CI/CD checks in the GitHub repo pass (`Validation Success`)
- [ ] PR has at least one approval and all checks are green
- [ ] `bash scripts/devops-validate.sh` exits cleanly (if any bot or CI/CD file was touched)
- [ ] `cargo test` passes locally
- [ ] Relevant `wiki/` pages updated
- [ ] Entry added to `wiki/Changelog.md` (or `wiki/raw/CHANGELOG-<branch>.md` if PR still open)
- [ ] Two-PR TDD sequence followed (PR 1: Tests, PR 2: Implementation)
- [ ] Tests added or improved to cover the behavior changes

---

## Development Constraints

- **No secrets** — never commit `.env`, tokens, or anything under `config/`, `local/`, `data/`.
- **Bot isolation** — bots never import from each other. Shared logic lives in `crates/starbunk-shared/`.
- **Self-message guard** — every handler must check `msg.author.id != ctx.cache.current_user().id`.
- **Non-blocking** — slow work (LLM, HTTP, audio) must be spawned with `tokio::spawn`.
- **No direct push to `main`** — all changes go through a PR. Never use `--no-verify`.

---

## Worktree Lifecycle

Every task lives in its own worktree under `.claude/worktrees/`. Main stays on `main`, clean,
and always synced to `origin/main`. **Context-check before every task** — if the task doesn't
belong to the current worktree, switch first. See `/worktree` for the full protocol.

---

## Module File Layout

Use the **Rust 2018 sibling-file pattern** for every module that has submodules:

```
src/
  foo.rs          ← module root: declares submodules, re-exports, top-level logic
  foo/
    bar.rs        ← submodule
    baz.rs        ← submodule
```

**Rules:**
- **Never use `mod.rs`** — always prefer `foo.rs` as the module root alongside a `foo/` directory.
- The root file (`foo.rs`) owns: `mod` declarations, `pub use` re-exports, and any logic that belongs to the module as a whole.
- Each submodule file contains exactly one concept (one struct, one trait, one command, etc.).
- When a file grows beyond ~150 lines, split it — one concept per file.

**Correct** (`djcova` commands, `starbunk-shared` top-level modules):
```
commands.rs       ← mod play; mod skip; pub use ...; pub fn all_commands()
commands/play.rs  ← pub fn play_command()
commands/skip.rs  ← pub fn skip_command()
```

**Wrong** — never do this:
```
commands/mod.rs   ← ✗ use foo.rs instead
```

---

## Observability Standards — MANDATORY

**Full details: `wiki/development/Observability.md`**

All bots use the unified OTEL pipeline via `starbunk_shared::telemetry::init`.
Never call `tracing_subscriber::fmt::init()` directly — it bypasses OTEL.

### Logging (tracing macros)

Use the `tracing` crate. **Never** use `println!`, `eprintln!`, or the `log` crate.

| Level | When to use |
|---|---|
| `error!` | Unrecoverable condition or external failure (Discord send fail, DB error) |
| `warn!` | Degraded behaviour — bot continues but something unexpected happened |
| `info!` | Lifecycle events: startup, shutdown, Discord `ready`, successful operations |
| `debug!` | Internal decision-making — only enabled in verbose mode |
| `trace!` | Per-message evaluation, loop iterations — extremely chatty, never in prod |

**Always include structured fields** — fields are indexed in Loki and Tempo.

```rust
// Good — fields are searchable in Grafana
tracing::info!(bot = "bluebot", channel = %msg.channel_id, "message received");
tracing::error!(strategy = strategy.name(), err = %e, "send failed");

// Bad — unstructured string only
tracing::info!("bluebot got a message in channel {}", msg.channel_id);
```

### Spans (`#[tracing::instrument]`)

Add `#[tracing::instrument]` to every public async function that does meaningful
work — Discord event handlers, LLM calls, DB queries, message sends.

```rust
#[tracing::instrument(skip(ctx, msg), fields(channel = %msg.channel_id))]
pub async fn handle(&self, ctx: &Context, msg: &Message) { ... }
```

- `skip` the fields you don't want in every span (large or secret values).
- Add `fields(...)` for high-value searchable attributes.

### Verbose mode

Set `VERBOSE=1` to activate debug-level logging, span enter/close events,
thread IDs, and file+line numbers. Never log at debug/trace in normal mode
unless gated by `tracing::enabled!(Level::DEBUG)`.

### Metrics

Use `opentelemetry::global::meter("botname")` to get a meter and record
custom metrics. Standard counters to add to every bot:

| Metric | Type | Labels |
|---|---|---|
| `bot.messages.received` | Counter | `bot`, `guild` |
| `bot.messages.sent` | Counter | `bot`, `strategy` |
| `bot.errors` | Counter | `bot`, `kind` |

### Environment variables

| Variable | Default | Effect |
|---|---|---|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `http://otel-collector:4317` | OTLP gRPC endpoint |
| `RUST_LOG` | `info` | tracing filter (e.g. `debug,serenity=warn`) |
| `VERBOSE` | `false` | Verbose mode (`1` or `true`) |

---

## Rust Code Standards

Key rules — **see `/rust-standards` for the full ruleset:**

- `Arc<dyn Trait>` for all injected dependencies (never concrete types)
- `#[derive(Debug)]` on every public struct and enum
- `LazyLock<Regex>` for all compiled regexes — never in a hot path
- No `.unwrap()` in production code
- All slow async work spawned with `tokio::spawn`

---

## DevOps File Maintenance

Every bot crate must be registered in all four files. Use `/add-bot` when adding a new bot.

| File | What to update |
|---|---|
| `docker/docker-compose.yml` | Add service with `BOT_NAME: <bot>` build arg |
| `.github/workflows/ci.yml` | Add `crates/<bot>/**` to paths-filter |
| `.github/workflows/main.yml` | Add `<bot>` to docker build matrix |
| `AGENTS.md` | Update bot list in Architecture and Bots sections |

Run `bash scripts/devops-validate.sh` after any change and fix every `FAIL` before opening a PR.

---

## Architecture

**Rust Cargo workspace** — 5 independent Discord bots, each a crate under `crates/<botname>/`.

```
Cargo.toml                  # workspace root
crates/
  starbunk-shared/          # shared lib: discord, llm, memory, middleware, replybot
  bluebot/                  # lib + bin
  bunkbot/                  # lib + bin
  covabot/                  # lib + bin
  djcova/                   # lib + bin
  ratbot/                   # lib + bin
```

Per-bot module layout: `lib.rs` (wiring only) · `strategy.rs` · `<concern>.rs` · `main.rs`

Discord intents: `GUILD_MESSAGES | MESSAGE_CONTENT`. DJCova also needs `GUILD_VOICE_STATES`.

---

## Bots

| Bot | Purpose | Wiki |
|---|---|---|
| `bluebot` | Pattern-matches "blue"/Blue Mage references, replies contextually | `wiki/bots/BlueBot.md` |
| `bunkbot` | Admin backbone and general reply bot | `wiki/bots/BunkBot.md` |
| `covabot` | LLM personality emulator, mimics a specific user's tone | `wiki/bots/CovaBot.md` |
| `djcova` | Voice channel music streaming, YouTube queue per guild | `wiki/bots/DJCova.md` |
| `ratbot` | Secret Santa organiser — sign-ups, matching, DM notifications | `wiki/bots/RatBot.md` |

---

## Branch protection — `main`

| Rule | Setting |
|---|---|
| Required checks | `Validation Success` |
| Up to date | Yes (strict) |
| Required approvals | 1 |
| Force push | Blocked |

---

## Wiki maintenance

`AGENTS.md` is the single source of truth for rules and architecture.
`CLAUDE.md` imports this file and adds Claude Code-specific notes.
`.github/copilot-instructions.md` points GitHub Copilot here.
