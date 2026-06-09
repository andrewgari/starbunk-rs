# AGENTS.md

Canonical agent guide for `starbunk-rs`. All AI coding tools should read this
file. Claude Code loads it automatically via `CLAUDE.md`; other tools should
read it directly.

---

## !! MANDATORY: KEEP THE WIKI UP TO DATE !!

> **This rule applies to every agent, every task, without exception.**

1. **Before starting any task** — read the relevant `wiki/` page(s) for the area you will touch.
2. **After completing any task** — update the relevant wiki page(s) to reflect any changes to architecture, behavior, configuration, or patterns.
3. **For every significant change or PR** — add an entry to `wiki/Changelog.md` under today's date.
4. If a wiki page does not exist for the area you are working in, **create it**.

The wiki lives at `wiki/` in the repo root. Start at `wiki/Home.md`.

Any change that affects: architecture, configuration, a bot's behavior, infrastructure, deployment,
testing strategy, a new feature, or a bug fix with a non-obvious root cause counts as "meaningful".

---

## CHANGELOG Workflow

Every branch that introduces meaningful changes maintains its own changelog entry in `wiki/raw/`.

### While working on a branch

Create or update `wiki/raw/CHANGELOG-<branch-name>.md` as you go:

```markdown
## [Unreleased] — <branch-name>

### Added
- ...

### Changed
- ...

### Fixed
- ...
```

Keep this file local-only (do not commit it to Git; it is ignored by `.gitignore`).

### On merge (PR completion)

Prepend the branch entry into `wiki/Changelog.md` under the current date, then
delete `wiki/raw/CHANGELOG-<branch-name>.md`.

---

## !! MANDATORY: TEST-DRIVEN DEVELOPMENT (TDD) WORKFLOW !!

> **This rule applies to every agent, every task involving logic, behavior, or ports, without exception.**

1. **Two-PR Sequence constraint**:
   - **PR 1: Test-Only / Behavior Definition**: Add/improve Rust tests (`#[test]`, `#[tokio::test]`) defining expected behavior. No implementation changes. This PR must fail the new tests (Red phase). Minimal stubs/traits may be added only to allow compilation.
   - **PR 2: Implementation**: Only after the test-only PR is approved or merged may you write the Rust code to satisfy those tests (Green/Refactor phase).
2. **Finding Reference Behavior**: Look up legacy Go behavior in `../starbunk-go/cmd/<bot>/` first, or JS in `../starbunk-js/src/<bot>/`.
3. For full details and step-by-step examples, read [[wiki/development/TDD.md]].

---

## Before Every Change

1. Confirm you are on the correct branch and it is up to date with `main`.
2. **Read every file you are about to change.** Never edit from memory.
3. If your change touches `crates/starbunk-shared/`, grep all callers first to know the blast radius.

After code changes, run `/check` (build + clippy + test + fmt). If DevOps files were touched, also run:
```bash
bash scripts/devops-validate.sh
```

When CI fails: read the full log, identify the exact root cause, fix only what is broken.
When unsure: run `/check` and read the relevant `wiki/` page before guessing.

**Wiki / code consistency** — for each file changed:

| Changed area | Wiki page to verify |
|---|---|
| A bot's behaviour, commands, config | `wiki/bots/<Bot>.md` |
| `crates/starbunk-shared/` | `wiki/infrastructure/Architecture.md` |
| CI/CD workflows | `wiki/development/CI-CD.md` |
| Testing patterns | `wiki/development/Testing.md` |
| Deployment, Docker, health checks | `wiki/infrastructure/Deployment.md` |

---

## Definition of Done

A task is **not complete** until:

- [ ] All CI checks pass (`Validate DevOps Consistency`, `Lint`, `Test`)
- [ ] If a PR was opened — it has at least one approval and all checks are green
- [ ] `bash scripts/devops-validate.sh` exits cleanly (if any bot or CI/CD file was touched)
- [ ] `cargo test` passes locally
- [ ] The relevant `wiki/` page(s) have been updated
- [ ] An entry has been added to `wiki/Changelog.md` (or `wiki/raw/CHANGELOG-<branch>.md` if the PR is still open)
- [ ] The change follows the Two-PR TDD sequence (PR 1: Test-Only, PR 2: Implementation)
- [ ] Rust tests are added or improved to fully cover the behavior changes

---

## Development Constraints

- **Never commit secrets or local config** — `.env` files, tokens, and anything under `config/`, `local/`, or `data/` directories must not be committed.
- **Maintain container isolation** — each bot is its own Cargo crate under `crates/<bot>/` and its own container. Cross-bot shared logic belongs in `crates/starbunk-shared/`, not copied between bots.
- **Self-message guard** — every message handler must check `msg.author.id != ctx.cache.current_user().id` (or use the `NOT_SELF` filter) to prevent bot reply loops.
- **Non-blocking handlers** — Serenity event handlers run on a shared async executor. Any slow operation (LLM calls, HTTP, audio processing) must be spawned with `tokio::spawn`.
- **Use correct Docker service names** — internal service communication uses service names defined in `docker-compose.yml` (e.g. `starbunk-rs-bunkbot`).
- **Never push directly to `main`** — all changes go through a PR. Never use `--no-verify`.

---

## Rust Code Standards

These rules apply to every file written or changed in this project, without exception.

### File discipline

- **Target ≤ 150 lines per file.** When a file grows beyond that, split by responsibility.
- **One concept per file.** A file owns one thing: a trait, a set of related filter primitives, one strategy, one client implementation, etc.
- **`mod.rs` files are wiring only.** They declare submodules and expose `pub async fn run()`. No business logic.
- **No copy-paste between bots.** If two bots need the same logic, it belongs in `crates/starbunk-shared/`.

### Bot isolation

- Each bot crate may import from `crates/starbunk-shared/` but **never** from another bot's crate.
- Bots must not share mutable state. Any shared state belongs in `crates/starbunk-shared/` with `Arc<Mutex<_>>` or `DashMap`.

### Types and traits

- **Expose dependencies as `Arc<dyn Trait>`**, not concrete types. This is mandatory for testability.
- **`#[derive(Debug)]`** on every public `struct` and `enum`.
- **`LazyLock<Regex>`** for all compiled regular expressions. Never call `Regex::new(...)` inside a hot path:
  ```rust
  static PAT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"...").expect("pattern name"));
  ```
- **No unnecessary clones.** Pass references where ownership is not required.

### Error handling

- Application code uses `anyhow::Result`. `run()` returns `anyhow::Result<()>`.
- **Never `.unwrap()` in production code.** Use `?` or `.expect("reason")` only on programmer-error panics.
- In Serenity event handlers, log errors instead of propagating:
  ```rust
  if let Err(e) = sender.send(channel_id, &resp).await {
      tracing::error!(strategy = name, "failed to send: {}", e);
  }
  ```

### Async

- All slow work (LLM, HTTP, DB, audio) must be spawned with `tokio::spawn`.
- Declare `async fn` only where the function actually awaits something.

### Testing

- **Test names describe behavior:** `triggers_on_blue_variants` not `test_blue_regex`.
- **`build_msg` / `fake_ctx` helpers** live at the top of `mod tests`. Build `serenity::Message` values with `serde_json::json!`.
- **Mock types** belong in the `mod tests` block of the file that uses them.
- Every `Strategy` must test: triggers on canonical input, does not trigger on non-matching input, returns the expected response.
- Every `MessageFilter` must test: pass case, fail case, and composition inside `all_of` / `any_of`.

---

## !! DevOps File Maintenance — MANDATORY !!

Every bot crate under `crates/<botname>/` **must** be registered in **all six** of the following files:

| File | What to update |
|---|---|
| `docker-compose.yml` | Add service with `image: ghcr.io/andrewgari/starbunk-rs-<bot>:${IMAGE_TAG:-latest}` |
| `docker/docker-compose.yml` | Add service with `BOT_NAME: <bot>` build arg |
| `.github/workflows/ci.yml` | Add `crates/<bot>/**` to the paths-filter block |
| `.github/workflows/main.yml` | Add `<bot>` to the docker build matrix |
| `scripts/deployment/health-check.sh` | Add `"<bot>"` to the `EXPECTED_SERVICES` array |
| `AGENTS.md` | Update the bot list in Architecture and Bots sections |

Run `bash scripts/devops-validate.sh` after any change and fix every `FAIL` before opening a PR.

To add a new bot, use the `/add-bot` skill — it walks through all required steps.

---

## Architecture

This is a **Rust Cargo workspace** housing 5 independent Discord bots (`bluebot`, `bunkbot`, `covabot`, `djcova`, `ratbot`), each as its own crate under `crates/<botname>/`.

### Workspace layout

```
Cargo.toml                  # workspace root with [workspace.dependencies]
crates/
  starbunk-shared/          # lib crate — all shared code + run_bot + default_intents
  bluebot/                  # lib + bin crate
  bunkbot/                  # lib + bin crate
  covabot/                  # lib + bin crate
  djcova/                   # lib + bin crate
  ratbot/                   # lib + bin crate
```

### Shared libraries (`crates/starbunk-shared/`)

- **`discord`** — `MessageService` trait wraps Serenity for sending, replying, editing, and deleting. `send_message_with_identity` posts as a custom user/avatar via webhook.
- **`llm`** — `LlmService` trait + `TieredRegistry` for High/Medium/Low tier routing across Anthropic, Google, Ollama, and OpenAI.
- **`memory`** — Semantic memory with pgvector. Async fact extraction and recall.
- **`middleware`** — Composable `MessageFilter` trait with primitives and combinators.
- **`replybot`** — Strategy-pattern dispatcher for reply-style bots.

### Module structure per bot crate

```
crates/<bot>/
  Cargo.toml
  src/
    lib.rs        # Handler struct, EventHandler impl, pub fn run() — wiring only
    strategy.rs   # Strategy implementations (for reply-style bots)
    <concern>.rs  # One file per additional domain concern
    main.rs       # Entry point: calls <bot>::run()
```

Discord intents: `run_bot` uses `GUILD_MESSAGES | MESSAGE_CONTENT`. DJCova additionally needs `GUILD_VOICE_STATES`.

---

## Bots

Current bots: `bluebot`, `bunkbot`, `covabot`, `djcova`, `ratbot`.

### bluebot
Pattern-matching bot. Detects references to "blue" or Blue Mage and replies with contextual responses. See `wiki/bots/BlueBot.md`.

### bunkbot
Administrative backbone and general reply bot. High message volume, fast reaction times. May use webhooks to post as other identities. See `wiki/bots/BunkBot.md`.

### covabot
AI personality emulator. Responds to conversational mentions with LLM-driven replies that mimic a specific user's tone. See `wiki/bots/CovaBot.md`.

### djcova
Voice channel music streaming service. Joins voice on demand, plays YouTube audio, manages a per-guild queue. See `wiki/bots/DJCova.md`.

### ratbot
Rat-themed Secret Santa bot. Handles sign-ups, randomly assigns gifters to recipients, notifies via DM. See `wiki/bots/RatBot.md`.

---

## Branch protection — `main`

| Rule | Setting |
|---|---|
| Required status checks | `Validate DevOps Consistency`, `Lint`, `Test` |
| Branches must be up to date | Yes (strict mode) |
| Required PR approvals | 1 |
| Force pushes | Blocked |

---

## Wiki maintenance

`AGENTS.md` is the single source of truth for rules and architecture.
`CLAUDE.md` imports this file and adds Claude Code-specific notes.
`.github/copilot-instructions.md` points GitHub Copilot here.
