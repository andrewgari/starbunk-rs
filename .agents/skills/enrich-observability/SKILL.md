---
name: enrich-observability
description: Analyzes the current branch/PR and adds comprehensive, well-formatted structured logging and spans for observability.
---

# Enrich Observability

**Role:** You act as "The Painter" focusing on observability. Your goal is to review the code modified in the current worktree (or PR) and inject clear, structured, and consistent logging using the `tracing` crate.

## Steps

1. **Identify Modified Code:** Use `git diff main --name-only` or `git status` to identify which files have been modified in the current branch.
2. **Review for Observability Gaps:** Look for:
   - Public async functions that do meaningful work but lack `#[tracing::instrument]`.
   - Complex logic branches, errors, or significant lifecycle events that lack inline logging (`tracing::info!`, `tracing::warn!`, `tracing::error!`, etc.).
   - Code that uses `println!`, `eprintln!`, or the `log` crate (which must be replaced with `tracing`).
3. **Inject Spans:**
   - Add `#[tracing::instrument(skip(ctx, ...), fields(...))]` to important public async functions (e.g., Discord event handlers, LLM calls, DB queries, message sends).
   - Ensure you `skip` large objects or secrets (like Context, large structs, user tokens).
   - Add searchable `fields()` for important context (e.g., `channel = %msg.channel_id`, `bot = "bluebot"`).
4. **Inject Structured Logs:**
   - Add inline `tracing` macros where appropriate:
     - `error!`: Unrecoverable conditions or external failures (e.g., Discord send fail, DB error).
     - `warn!`: Degraded behaviour (e.g., unexpected data but can continue).
     - `info!`: Lifecycle events, successful operations.
     - `debug!`: Internal decision-making.
   - **MANDATORY:** Always include structured fields. Never use unstructured strings.
     - *Good:* `tracing::info!(bot = "bunkbot", user = %msg.author.id, "processed command");`
     - *Bad:* `tracing::info!("processed command for user {}", msg.author.id);`
5. **Verify Telemetry Pipeline:** Ensure the code doesn't try to initialize `tracing_subscriber::fmt::init()` directly. It must rely on `starbunk_shared::telemetry::init`.
6. **Commit Changes:** Once you have injected and formatted the logs (respecting the existing style), compile to ensure correctness and commit the changes with an appropriate commit message.

## Strict Observability Standards
- You must strictly adhere to the Observability Standards defined in `AGENTS.md` and `wiki/development/Observability.md`.
