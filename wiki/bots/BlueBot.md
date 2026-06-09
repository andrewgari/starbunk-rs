# BlueBot

> Pattern-matching bot for "blue" / Blue Mage references.

## Goals & Purpose

BlueBot detects any message that references "blue" or Blue Mage and replies with
the classic catchphrase: **"Did somebody say Blu?"**

It is inspired by the starbunk-js BlueBot. The Rust implementation prioritises a
clean, extensible architecture so that the trigger mechanism can be swapped or
augmented (e.g. with an LLM) without restructuring the bot.

## Architecture

### Strategy pattern

Bot-specific detection logic lives in `src/bots/bluebot/` alongside the module entry.
The shared dispatcher and interface live in `src/shared/replybot/` — every
reply-style bot uses them. The central abstraction is the `Strategy` trait:

```rust
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    fn should_trigger(&self, ctx: &Context, msg: &Message) -> bool;
    fn response(&self, ctx: &Context, msg: &Message) -> String;
}
```

`ReplyBot` holds an ordered `Vec<Arc<dyn Strategy>>` and dispatches to the first that
triggers. Adding behaviour means adding a `Strategy` — nothing else changes.

### Current strategies

| Strategy | Trigger | Response |
|---|---|---|
| `BlueStrategy` | Regex match on "blue" variants | `"Did somebody say Blu?"` |

### Extensibility roadmap

- **LLM trigger** — swap or layer an LLM call alongside the regex by
  implementing a new `Strategy` and passing it to `ReplyBot::new`.
- **Reply window** — a stateful strategy that opens a follow-up window after
  a blue detection, triggering on short confirmations within N minutes.
- **Enemy user** — a strategy that returns a hostile response for a specific
  user ID configured via environment variable.
- **Response variation** — extend `BlueStrategy::response` to pick randomly
  from a list, or delegate to an LLM for generated replies.

### Pattern coverage

The regex (`BLUE_PATTERN` in `src/bots/bluebot/strategy.rs`) matches:

| Variant | Example |
|---|---|
| Plain colour | `blue`, `Blue`, `BLUE` |
| Extended vowels | `bloo`, `bluu`, `blooo` |
| Archaic spelling | `blew` |
| French | `bleu` |
| Spanish | `azul` |
| German | `blau` |
| Bot name | `bluebot` |

Word-bounded to prevent false positives on compound words:

| Non-matching | Why |
|---|---|
| `bluetooth` | compound word |
| `blueprint` | compound word |
| `blueberry` | compound word |

### Rate limiting

- Standard blue replies: 5-minute cooldown per channel.
- Rare/special replies: 24-hour cooldown.

## Environment Variables

None — BlueBot has no external dependencies.

## Entry Point

`src/bin/bluebot.rs` → `src/bots/bluebot::run()`.

## See Also

- [[../infrastructure/Architecture|Architecture]] — strategy pattern details
- [[../development/TDD|TDD SDLC]] — mandatory test-first workflow
