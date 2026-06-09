# Message Filter Abstraction

> **Status:** Implemented — `src/shared/middleware`
> **Relates to:** [[../infrastructure/Architecture|Architecture]], [[../bots/BunkBot|BunkBot]], [[../bots/BlueBot|BlueBot]], [[../bots/CovaBot|CovaBot]]

---

## Problem

Without a shared abstraction, every bot implements its own self-guard inline:

```rust
async fn message(&self, ctx: Context, msg: Message) {
    if msg.author.bot {
        return;
    }
    // bot logic
}
```

This is insufficient for two reasons:

1. **It is opt-in.** A bot author can forget to write it. Nothing in the framework enforces it.
2. **It does not scale.** Each bot will need a different set of acceptance rules. Without a shared abstraction, every bot reimplements the same logic differently, making it untestable in isolation and impossible to compose.

The fix is **making the filter the bot's declared policy**, checked at the top of every event handler.

---

## Design Principle

> Every `message` event is checked against the bot's `MessageFilter` before any handler logic runs. This is enforced by the bot's `EventHandler` implementation calling `self.filter.check(&ctx, &msg)` at the top of every handler.

Bots declare **what they accept** (their filter). The framework handles **ensuring it runs** on every message.

---

## Core Trait: `MessageFilter`

```rust
// src/shared/middleware/mod.rs

/// MessageFilter is the mandatory evaluation gate for incoming Discord messages.
/// Every bot's EventHandler should call filter.check() at the top of message().
///
/// Returns true if the message should be processed, false if it should be dropped silently.
pub trait MessageFilter: Send + Sync {
    fn check(&self, ctx: &Context, msg: &Message) -> bool;
}
```

---

## Primitives

Atomic building blocks in `src/shared/middleware`. Each is a `lazy_static!` or `once_cell` singleton
implementing `MessageFilter`.

| Var | Drops when… |
|---|---|
| `NOT_SELF` | `msg.author.id == ctx.cache.current_user().id` |
| `NOT_BOT` | `msg.author.bot == true` |
| `IS_BOT` | `msg.author.bot == false` |
| `HAS_CONTENT` | `msg.content.trim().is_empty()` |
| `GUILD_ONLY` | `msg.guild_id.is_none()` (i.e. a DM) |
| `DM_ONLY` | `msg.guild_id.is_some()` (i.e. a guild message) |

---

## Combinators

Build composite filters from primitives. All return `Arc<dyn MessageFilter>`.

```rust
/// Passes only when every child filter passes. Short-circuits on first failure.
pub fn all_of(filters: Vec<Arc<dyn MessageFilter>>) -> Arc<dyn MessageFilter>

/// Passes when at least one child filter passes. Short-circuits on first success.
pub fn any_of(filters: Vec<Arc<dyn MessageFilter>>) -> Arc<dyn MessageFilter>

/// Inverts a filter.
pub fn not(f: Arc<dyn MessageFilter>) -> Arc<dyn MessageFilter>
```

---

## Per-Bot Filters

Each bot declares its filter inline in its entry point or `run()` function.

### BlueBot

Never triggers off itself or any other bot. Guild messages only. Must have content.

```rust
let filter = middleware::all_of(vec![
    middleware::NOT_SELF.clone(),
    middleware::NOT_BOT.clone(),
    middleware::GUILD_ONLY.clone(),
    middleware::HAS_CONTENT.clone(),
]);
```

### CovaBot

Same base policy as BlueBot.

```rust
let filter = middleware::all_of(vec![
    middleware::NOT_SELF.clone(),
    middleware::NOT_BOT.clone(),
    middleware::GUILD_ONLY.clone(),
    middleware::HAS_CONTENT.clone(),
]);
```

### BunkBot

More permissive. Does not exclude bot messages.

```rust
let filter = middleware::all_of(vec![
    middleware::NOT_SELF.clone(),
    middleware::HAS_CONTENT.clone(),
]);
```

---

## Tier 2: Strategy-Level Conditions (`src/shared/replybot`)

For bots that use the `ReplyBot` dispatcher (e.g. BlueBot), a second tier of
filtering exists at the **individual strategy level**. This enables different
strategies within the same bot to have conflicting filter policies.

### How it works

`ReplyBot::handle()` checks whether a strategy implements the optional
`ConditionedStrategy` trait before calling `should_trigger`. If the condition
fails, evaluation continues with the next strategy.

```rust
// src/shared/replybot/strategy.rs

/// ConditionedStrategy is an optional extension of Strategy.
pub trait ConditionedStrategy: Strategy {
    fn condition(&self) -> Arc<dyn MessageFilter>;
}
```

`with_condition` wraps any existing `Strategy` with a condition:

```rust
pub fn with_condition(cond: Arc<dyn MessageFilter>, strategy: Arc<dyn Strategy>) -> Arc<dyn Strategy>
```

### Tier summary

| Tier | Mechanism | Where declared |
|------|-----------|----------------|
| 1 | `MessageFilter` in EventHandler | Bot entry point (`src/bin/<bot>.rs` or `src/bots/<bot>/mod.rs`) |
| 2 | `ConditionedStrategy` / `with_condition` | Strategy construction |

---

## See Also

- [[../infrastructure/Architecture|Architecture]] — shared library overview and bot pattern
- [[../bots/BunkBot|BunkBot]] — most complex filter policy
- [[../bots/BlueBot|BlueBot]] — standard strict filter policy
- [[../bots/CovaBot|CovaBot]] — standard strict filter policy
