# AGENTS.md

Canonical agent guide for `starbunk-rs`. All AI coding tools should read this
file. Claude Code loads it automatically via `CLAUDE.md`; other tools should
read it directly.

---

## !! MANDATORY: KEEP THE WIKI UP TO DATE !!

> **This rule applies to every agent, every task, without exception.**
> There is no situation where skipping a wiki update is acceptable.

1. **Before starting any task** — read the relevant `wiki/` page(s) for the area you will touch.
2. **After completing any task** — update the relevant wiki page(s) to reflect any changes to architecture, behavior, configuration, or patterns.
3. **For every significant change or PR** — add an entry to `wiki/Changelog.md` under today's date.
4. If a wiki page does not exist for the area you are working in, **create it**.
5. Create new pages freely. Use plain relative links to connect related pages.
6. Keep `wiki/raw/` for local drafts and staged content (do not commit to Git). Promote to `wiki/` on merge.

The wiki lives at `wiki/` in the repo root. Start at `wiki/Home.md`.

### What counts as "meaningful"

Any change that affects: architecture, configuration, a bot's behavior,
infrastructure, deployment, testing strategy, a new feature, or a bug fix with
a non-obvious root cause.

---

## CHANGELOG Workflow

Every branch that introduces meaningful changes maintains its own changelog
entry in `wiki/raw/`.

### While working on a branch

Create or update `wiki/raw/CHANGELOG-<branch-name>.md` as you go. Format:

```markdown
## [Unreleased] — <branch-name>

### Added
- ...

### Changed
- ...

### Fixed
- ...
```

Keep this file local-only (do not commit it to Git; it is ignored by `.gitignore`). It is the local staging area for the real changelog.

### On merge (PR completion)

Prepend the branch entry into `wiki/Changelog.md` under the current date, then
delete `wiki/raw/CHANGELOG-<branch-name>.md`. The permanent changelog lives at
`wiki/Changelog.md`.

---

## !! MANDATORY: USE AVAILABLE SKILLS AND TOOLS PROACTIVELY !!

> **Do not wait to be told.** When a situation matches an available skill or
> tool capability, use it immediately without prompting.

Examples:
- Starting a non-trivial implementation → use a plan / task breakdown first
- Code has been written or changed → review it for simplicity and quality
- A PR needs deployment → use the deploy skill/tool
- Tests are available → run them before declaring a task done

---

## !! MANDATORY: TEST-DRIVEN DEVELOPMENT (TDD) WORKFLOW !!

> **This rule applies to every agent, every task involving logic, behavior, or ports, without exception.**
> There is no situation where implementing code without first writing failing tests is acceptable.

1. **Two-PR Sequence constraint**:
   - **PR 1: Test-Only / Behavior Definition**: You MUST only add/improve Rust tests (`#[test]`, `#[tokio::test]`) defining the expected behavior. No implementation changes are allowed. This PR must fail the newly added tests (Red phase). Minimal stubs/traits may be added only to allow compilation.
   - **PR 2: Implementation**: Only after the test-only PR is approved or merged may you write the Rust code to satisfy those tests (Green/Refactor phase).
2. **Finding Reference Behavior**: Look up legacy Go behavior in the sibling directory `../starbunk-go/cmd/<bot>/` first, or JS in `../starbunk-js/src/<bot>/`.
3. For full details and step-by-step examples, read [[wiki/development/TDD.md]].

---

## Self-Correction Protocol

When something doesn't work as expected, work through this sequence before
retrying or escalating.

### 0. Before touching any file — sync and read

1. Confirm you are on the correct branch and it is up to date with `main`.
2. **Read every file you are about to change.** Never edit from memory or from
   a previous read earlier in the conversation.
3. If your change touches `src/shared/`, grep all callers first:
   ```bash
   grep -rn "SymbolYouAreChanging" src/
   ```
   You need to know the full blast radius before writing a single line.

### 1. After any code change — verify in this order

```bash
cargo build --all         # all packages compile (catches import errors fast)
cargo clippy -- -D warnings  # catches common Rust mistakes
cargo test                # all tests pass
```

If you changed anything in `src/shared/` (shared code), also build every bot
individually to confirm nothing silently broke:

```bash
for bot in bluebot bunkbot covabot djcova ratbot; do
    cargo build --bin $bot || echo "BROKEN: $bot"
done
```

Run fmt check and clippy before opening a PR:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

### 2. Read errors completely before acting

- **Never retry a failed command with identical input.** If it failed once, it
  will fail again.
- Read the full error output. The root cause is usually in the first or last
  few lines, not buried in the middle.
- If a clippy error appears, fix exactly what is reported. Do not add `#[allow(...)]`
  suppressions to make it disappear — that hides a real problem.
- If a test fails, decide: **is the test wrong, or is the code wrong?** Fix
  the right thing.

### 3. DevOps drift check

If you touched any of: `src/bin/`, `docker-compose.yml`, `docker/docker-compose.yml`,
`.github/workflows/`, `scripts/deployment/health-check.sh`:

```bash
bash scripts/devops-validate.sh
```

A `FAIL` here blocks CI. Fix it before opening or updating a PR.

### 4. Wiki / code consistency check

Before closing a task, ask for each file you changed:

| Changed area | Wiki page to verify |
|---|---|
| A bot's behaviour, commands, config | `wiki/bots/<Bot>.md` |
| `src/shared/` contracts or architecture | `wiki/infrastructure/Architecture.md` |
| CI/CD workflows | `wiki/development/CI-CD.md` |
| Testing patterns | `wiki/development/Testing.md` |
| Deployment, Docker, health checks | `wiki/infrastructure/Deployment.md` |

Mismatch between code and docs is a bug. Update the docs.

### 5. When CI fails on a PR

1. Read the failing job log in full — don't skim.
2. Identify the exact root cause before writing any fix.
3. Fix only what is broken; don't refactor unrelated code in the same commit.
4. Push the fix and wait for the check to re-run.
5. If the same check fails twice with the same message, stop and investigate
   the environment or assumptions before trying again.

### 6. When you are unsure whether a change is correct

- Run `cargo build --all` and `cargo test` first.
- Check the relevant `wiki/` page.
- Check git log for the file (`git log --oneline -10 <file>`).
- If still unsure, ask. Do not guess and ship.

---

## Definition of Done

A task is **not complete** until:

- [ ] All CI checks pass (`Validate DevOps Consistency`, `Lint`, `Test`)
- [ ] If a PR was opened — it has at least one approval and all checks are green
- [ ] `bash scripts/devops-validate.sh` exits cleanly (if any bot or CI/CD file was touched)
- [ ] `cargo test` passes locally
- [ ] The relevant `wiki/` page(s) have been updated
- [ ] An entry has been added to `wiki/Changelog.md` (or to a local draft `wiki/raw/CHANGELOG-<branch>.md` if the PR is still open)
- [ ] The change follows the Two-PR TDD sequence constraint (PR 1: Test-Only, PR 2: Implementation)
- [ ] Rust tests are added or improved to fully cover the behavior changes

"The code works locally" is not done. "The PR is open" is not done.

---

## Development Constraints

- **Never commit secrets or local config** — `.env` files, tokens, and anything
  under `config/`, `local/`, or `data/` directories must not be committed.
- **Maintain container isolation** — each bot binary under `src/bin/` is its
  own container. Cross-bot shared logic belongs in `src/shared/`, not copied
  between bots.
- **Self-message guard** — every message handler must check `msg.author.id != ctx.cache.current_user().id`
  (or use the `NOT_SELF` filter) to prevent bot reply loops.
- **Non-blocking handlers** — Serenity event handlers run on a shared async executor.
  Any slow operation (LLM calls, HTTP, audio processing) must be spawned with `tokio::spawn`
  so it cannot stall other handlers.
- **Use correct Docker service names** — internal service communication uses the
  service names defined in `docker-compose.yml` (e.g. `starbunk-rs-bunkbot`).

---

## Rust Code Standards

These rules apply to every file written or changed in this project, without exception.

### File discipline

- **Target ≤ 150 lines per file.** When a file grows beyond that, split by responsibility.
- **One concept per file.** A file owns one thing: a trait, a set of related filter primitives,
  one strategy, one client implementation, etc.
- **`mod.rs` files are wiring only.** They declare submodules, import the `Handler` struct, and
  expose `pub async fn run()`. No business logic belongs there.
- **Business logic lives in named submodules.** Name them for what they do:
  `strategy.rs`, `conversation.rs`, `engagement.rs`, `tagger.rs`.
- **No copy-paste between bots.** If two bots need the same logic, it belongs in `src/shared/`.

### Bot isolation

- Each bot in `src/bots/<bot>/` is fully self-contained. It may import from `src/shared/`
  but **never** from another bot's submodule (`src/bots/<other>/`).
- The dependency graph is strictly one-directional: `bin/` → `bots/` → `shared/`.
- Bots must not share mutable state. Any state that looks shared (e.g. webhook caches)
  belongs in `src/shared/` with proper `Arc<Mutex<_>>` or `DashMap` protection.

### Types and traits

- **Expose dependencies as `Arc<dyn Trait>`**, not concrete types. This is mandatory for
  testability. A handler that holds `Arc<dyn MessageService>` can be tested with a mock;
  one that holds `DiscordMessageService` cannot.
- **`#[derive(Debug)]`** on every public `struct` and `enum`. Missing `Debug` prevents
  use in `?` chains and `tracing` macro format strings.
- **`LazyLock<Regex>`** (from `std::sync`) for all compiled regular expressions. Never
  call `Regex::new(...)` inside a hot path. Pattern: `static PAT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"...").expect("pattern name"));`
- Prefer `impl Trait` in return position for free functions returning a single concrete type.
  Use `Box<dyn Trait>` or `Arc<dyn Trait>` when the type is stored in a struct or collection.
- **No unnecessary clones.** Pass references where ownership is not required. Clone only
  when you genuinely need a new owner (e.g. crossing a `tokio::spawn` boundary).

### Error handling

- Application code (`src/bin/`, `src/bots/`) uses `anyhow::Result`. `run()` returns
  `anyhow::Result<()>`.
- **Never `.unwrap()` in production code.** Use `?` or `.map_err(|e| anyhow::anyhow!("{}", e))`.
  The only exceptions are: `expect()` on values that are programmer errors (e.g. a static
  `Regex::new(...)` that cannot fail), and test code.
- In Serenity event handlers where an error must not crash the bot, use the pattern:
  ```rust
  if let Err(e) = sender.send(channel_id, &resp).await {
      tracing::error!(strategy = name, "failed to send: {}", e);
  }
  ```

### Async

- All slow work (LLM calls, HTTP requests, DB queries, audio processing) must be spawned
  with `tokio::spawn`. Event handlers that block will stall the entire Serenity executor.
- Use `tokio::sync::OnceCell` for handler-level values that are initialized once at
  connection time (e.g. `ReplyBot`, `WebhookService` inside a handler struct).
- Declare `async fn` only where the function actually awaits something. Synchronous
  helpers should be plain `fn`.

### Testing

- **Test names describe behavior, not implementation.** Prefer `triggers_on_blue_variants`
  over `test_blue_regex`. Prefer `first_match_wins_second_strategy_not_called` over
  `test_reply_bot`.
- **`build_msg` / `fake_ctx` helpers** live at the top of `mod tests`. Never add
  test-only constructors to production structs; build `serenity::Message` values with
  `serde_json::json!` and `.expect("description")`.
- **Mock types** (`MockSender`, `MockStrategy`, etc.) belong in the `mod tests` block of
  the file that uses them. Move to `tests/` only when the mock must be shared across
  multiple test files.
- **Dangling pointer pattern** is acceptable in tests whose strategies/filters declare
  `_ctx` and never dereference it:
  ```rust
  // SAFETY: this filter never dereferences ctx.
  let ctx_ptr = std::ptr::NonNull::<Context>::dangling();
  filter.check(unsafe { ctx_ptr.as_ref() }, &msg);
  ```
  Always include the safety comment.
- Every `Strategy` implementation must have tests for: (1) `should_trigger` returns true
  on canonical inputs, (2) `should_trigger` returns false on non-matching inputs, and
  (3) `response` returns the expected string.
- Every new `MessageFilter` primitive must have tests for the pass case, the fail case,
  and its composition inside `all_of` / `any_of`.

---

## Commands

```bash
# Run all tests
cargo test

# Run tests in a specific module
cargo test --lib shared::middleware

# Run a single test by name
cargo test test_blue_strategy

# Build a specific bot
cargo build --bin bunkbot

# Build all bots
cargo build --bins

# Run a bot locally (requires DISCORD_TOKEN env var)
DISCORD_TOKEN=<token> cargo run --bin bunkbot

# Build and run all containers (local dev — builds from source)
docker compose -f docker/docker-compose.yml up -d --build

# Build a single container (local dev)
docker compose -f docker/docker-compose.yml up -d --build bunkbot

# Validate DevOps file consistency (REQUIRED after any bot or CI/CD change)
bash scripts/devops-validate.sh
```

---

## !! DevOps File Maintenance — MANDATORY !!

> **This section applies to every agent and every task. Skipping it is not
> acceptable.** The CI pipeline enforces this check — drift will cause the
> `validate_devops` job to fail and block the entire pipeline.

### The rule

Every bot that lives under `src/bin/<botname>.rs` **must** be registered in **all
six** of the following files. They must always be kept in sync with each other:

| File | What to update |
|---|---|
| `docker-compose.yml` | Add a service with `image: ghcr.io/andrewgari/starbunk-rs-<bot>:${IMAGE_TAG:-latest}` |
| `docker/docker-compose.yml` | Add a service with `BOT_NAME: <bot>` build arg |
| `.github/workflows/ci.yml` | Add `src/bin/<bot>.rs` to the paths-filter block |
| `.github/workflows/main.yml` | Add `<bot>` to the docker build matrix |
| `scripts/deployment/health-check.sh` | Add `"<bot>"` to the `EXPECTED_SERVICES` array |
| `AGENTS.md` | Update the bot list everywhere it appears in this file |

### Validation step — run this after every relevant change

```bash
bash scripts/devops-validate.sh
```

Fix every `FAIL` line before marking the task complete.

---

## Architecture

This is a **Rust monorepo** housing 5 independent Discord bots (`bluebot`, `bunkbot`, `covabot`, `djcova`, `ratbot`), each with its own binary entry point in `src/bin/<botname>.rs` and its own Discord token.

### Shared libraries (`src/shared/`)

- **`src/shared/discord`** — Messaging abstraction. `MessageService` trait wraps Serenity for sending, replying, editing, and deleting messages. `send_message_with_identity` uses a per-channel webhook to post as a custom user/avatar.

- **`src/shared/llm`** — `LlmService` trait + `TieredRegistry` for High/Medium/Low tier routing across Anthropic, Google, Ollama, and OpenAI providers.

- **`src/shared/memory`** — Semantic memory with pgvector. Async fact extraction and recall for context injection.

- **`src/shared/middleware`** — Composable `MessageFilter` trait with primitives and combinators.

- **`src/shared/replybot`** — Strategy-pattern dispatcher for reply-style bots.

### Bot pattern

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    starbunk::bots::bunkbot::run().await
}
```

### Discord intents

`run_bot` uses `GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT`. DJCova additionally needs `GatewayIntents::GUILD_VOICE_STATES`.

### Module structure per bot

```
src/bots/<bot>/
  mod.rs          # Handler struct, EventHandler impl, pub fn run() — wiring only
  strategy.rs     # Strategy implementations (for reply-style bots)
  <concern>.rs    # One file per additional domain concern (conversation, tagger, etc.)
```

### Testing

Tests use Rust's built-in `#[test]` / `#[tokio::test]`. Unit tests live in `#[cfg(test)]`
blocks inside the file under test. Integration tests live in `tests/`. Test helpers and
mock types are co-located with the tests that use them — see the Rust Code Standards
section for naming and placement rules.

---

## Bots

Current bots: `bluebot`, `bunkbot`, `covabot`, `djcova`, `ratbot`.

### bluebot
Pattern-matching bot. Detects references to "blue" or Blue Mage in messages and
replies with contextual or character-themed responses. Ported from starbunk-js via starbunk-go.
See `wiki/bots/BlueBot.md`.

### bunkbot
Administrative backbone and general reply bot. Handles high message volume with
fast reaction times. May use webhooks to post as other identities.
See `wiki/bots/BunkBot.md`.

### covabot
AI personality emulator. Responds to conversational mentions with LLM-driven
replies that mimic a specific user's tone. Depends on an LLM provider (Ollama /
Anthropic / Gemini / OpenAI). See `wiki/bots/CovaBot.md`.

### djcova
Voice channel music streaming service. Joins voice on demand, plays YouTube
audio, manages a per-guild queue. Requires additional voice intents.
See `wiki/bots/DJCova.md`.

### ratbot
Rat-themed **Secret Santa bot** that organises the guild's "Ratmas" gift
exchange. Handles sign-ups, randomly assigns gifters to recipients (no
self-assignment), notifies participants via DM.
See `wiki/bots/RatBot.md`.

---

## Adding a new bot — complete checklist

> After completing every step, run `bash scripts/devops-validate.sh`.
> All checks must pass before the work is done.

1. **Create** `src/bin/<newbot>.rs` calling `starbunk::bots::<newbot>::run()`.
2. **Create** `src/bots/<newbot>/mod.rs` with the bot implementation.
3. **Create** `wiki/bots/<NewBot>.md` documenting the bot.

4. **`docker-compose.yml`** — add a service block:
   ```yaml
   <newbot>:
     image: ghcr.io/andrewgari/starbunk-rs-<newbot>:${IMAGE_TAG:-latest}
     container_name: starbunk-rs-<newbot>
     restart: unless-stopped
     environment:
       - DISCORD_TOKEN=${NEWBOT_TOKEN:-${STARBUNK_TOKEN}}
       - RUST_LOG=${RUST_LOG:-info}
     logging:
       driver: "json-file"
       options:
         max-size: "10m"
         max-file: "3"
     labels:
       - "com.centurylinklabs.watchtower.enable=true"
   ```

5. **`docker/docker-compose.yml`** — add a service block:
   ```yaml
   <newbot>:
     build:
       context: ..
       dockerfile: docker/Dockerfile
       args:
         BOT_NAME: <newbot>
     container_name: starbunk-rs-<newbot>
     restart: unless-stopped
     environment:
       - DISCORD_TOKEN=${NEWBOT_TOKEN:-${STARBUNK_TOKEN}}
       - RUST_LOG=${RUST_LOG:-info}
     logging:
       driver: "json-file"
       options:
         max-size: "10m"
         max-file: "3"
   ```

6. **`.github/workflows/ci.yml`** — add path filter for `src/bin/<newbot>.rs`.

7. **`.github/workflows/main.yml`** — add `<newbot>` to the docker build matrix.

8. **`scripts/deployment/health-check.sh`** — add `"<newbot>"` to `EXPECTED_SERVICES`.

9. **`AGENTS.md`** — update the bot list in the Architecture and Bots sections.

10. **Run validation**:
    ```bash
    bash scripts/devops-validate.sh
    ```

11. **Update `wiki/Home.md`** and add `wiki/bots/<NewBot>.md`.
12. **Add an entry to `wiki/Changelog.md`**.

---

## Branch protection — `main`

| Rule | Setting |
|---|---|
| Required status checks | `Validate DevOps Consistency`, `Lint`, `Test` |
| Branches must be up to date | Yes (strict mode) |
| Required PR approvals | 1 |
| Dismiss stale reviews on new commits | Yes |
| Force pushes | Blocked |
| Branch deletion | Blocked |

- **Never push directly to `main`.** All changes go through a PR.

---

## Wiki maintenance

`AGENTS.md` is the single source of truth for rules and architecture.
`CLAUDE.md` imports this file and adds Claude Code-specific notes.
`.github/copilot-instructions.md` points GitHub Copilot here.
