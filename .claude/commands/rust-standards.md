---
name: rust-standards
description: Full Rust code quality standards for starbunk-rs. Read this before writing or reviewing any Rust code.
---

# Rust Code Standards

These rules apply to every file written or changed in this project.

## File discipline

- **Target ≤ 150 lines per file.** Split by responsibility when exceeded.
- **One concept per file.** A file owns one thing: a trait, a filter primitive, one strategy, one client.
- **`mod.rs` is wiring only.** Declares submodules, exposes `pub async fn run()`. No business logic.
- **No copy-paste between bots.** Shared logic belongs in `crates/starbunk-shared/`.

## Bot isolation

- Each bot crate may import from `crates/starbunk-shared/` but **never** from another bot's crate.
- Shared mutable state lives in `crates/starbunk-shared/` with `Arc<Mutex<_>>` or `DashMap`.

## Types and traits

- **`Arc<dyn Trait>`** for all injected dependencies — never concrete types. Mandatory for testability.
- **`#[derive(Debug)]`** on every public `struct` and `enum`.
- **`LazyLock<Regex>`** for all compiled regexes. Never call `Regex::new(...)` in a hot path:
  ```rust
  static PAT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"...").expect("pattern name"));
  ```
- **No unnecessary clones.** Pass references where ownership is not required.

## Error handling

- Application code uses `anyhow::Result`. `run()` returns `anyhow::Result<()>`.
- **Never `.unwrap()` in production code.** Use `?` or `.expect("reason")` on programmer-error panics only.
- In Serenity event handlers, log errors rather than propagating:
  ```rust
  if let Err(e) = sender.send(channel_id, &resp).await {
      tracing::error!(strategy = name, "failed to send: {}", e);
  }
  ```

## Async

- All slow work (LLM, HTTP, DB, audio) must be spawned with `tokio::spawn`. Blocking handlers stall the executor.
- Declare `async fn` only where the function actually awaits something.

## Testing

- **Test names describe behavior:** `triggers_on_blue_variants` not `test_blue_regex`.
- **`build_msg` / `fake_ctx` helpers** live at the top of `mod tests`. Build `serenity::Message` with `serde_json::json!`.
- **Mock types** belong in the `mod tests` block of the file that uses them.
- Every `Strategy` must test: triggers on canonical input, does not trigger on non-matching input, returns the expected response.
- Every `MessageFilter` must test: pass case, fail case, and composition inside `all_of` / `any_of`.
