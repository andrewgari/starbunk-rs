# Testing

> [!IMPORTANT]
> **Mandatory TDD SDLC**: All development in this repository must follow the [[development/TDD|Test-Driven Development (TDD) SDLC Workflow]]. Feature implementation and test writing are separated into a two-PR sequence (Test-Only PR followed by Implementation PR).

## Framework

Tests use Rust's built-in `#[test]` attribute and `#[tokio::test]` for async tests.

- Unit tests live in the same file as the code under test, in a `#[cfg(test)] mod tests { ... }` block.
- Integration tests live in `tests/` at the crate root.
- Async tests require `tokio::test`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_something_async() {
        // ...
    }

    #[test]
    fn test_something_sync() {
        // ...
    }
}
```

## Running Tests

```bash
# All tests
cargo test

# Tests in a specific module
cargo test --lib shared::middleware

# A single test by name
cargo test test_blue_strategy_triggers

# With output (don't capture stdout)
cargo test -- --nocapture
```

## Writing Tests

Use Rust's assertion macros:

```rust
assert!(condition);
assert_eq!(actual, expected);
assert_ne!(actual, unexpected);
// For Results:
assert!(result.is_ok());
assert!(result.is_err());
```

For more expressive assertions, the `assert_matches!` macro (stable since Rust 1.82):

```rust
use std::assert_matches::assert_matches;
assert_matches!(result, Ok(val) if val > 0);
```

### Example — middleware filter test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn make_message(author_id: &str, content: &str, is_bot: bool) -> Message {
        // construct minimal Message for testing
    }

    #[test]
    fn not_bot_passes_human_messages() {
        let msg = make_message("user-123", "hello", false);
        assert!(NOT_BOT.check(&ctx, &msg));
    }

    #[test]
    fn not_bot_drops_bot_messages() {
        let msg = make_message("bot-456", "hello", true);
        assert!(!NOT_BOT.check(&ctx, &msg));
    }
}
```

## End-to-End (E2E) Testing

Starbunk includes an E2E testing framework in the `starbunk-e2e` crate. It allows validating bot responses in a live Discord server using a single Discord bot token.

> [!NOTE]
> **Comparison with `starbunk-js`**
> The mature `starbunk-js` E2E testing architecture requires setting up 6 distinct Discord bot applications and tokens (`E2E_BUNKBOT_TOKEN`, `E2E_BLUEBOT_TOKEN`, `E2E_DISCORD_SENDER_TOKEN`, `E2E_DISCORD_ENEMY_TOKEN`, etc.).
> 
> The Rust E2E framework improves on this by using a **single Discord token** for all bots and the test listener. To bypass self-message filters (`NOT_SELF`) and simulate human vs. bot authors without requiring separate bot tokens, the Rust runner executes test messages via a Discord Webhook and uses content prefixes (`[E2E_HUMAN]` / `[E2E_BOT]`).

### How it Works
1. **Single-Token Webhook Driver**: Since all bots share a single token in debugging, a bot cannot trigger itself because of `NOT_SELF` filters. To bypass this cleanly, the E2E runner posts test messages to the whitelisted channel via a **Discord Webhook** (automatically created by the runner).
2. **Simulating Bot vs. Human Senders**: To test if the bots correctly filter out other bots (`NOT_BOT` rule), the runner prefixes messages with `[E2E_HUMAN]` or `[E2E_BOT]`.
3. **E2E Gate Wrapper**: In E2E mode (`E2E_MODE=true`), the shared `E2eDebugHandler` intercepts gateway events, automatically strips these prefixes, and overrides `msg.author.bot` before passing the event to the bot's actual handler. It also drops any message that is not in the whitelisted `E2E_GUILD_ID`.
4. **Listener & Assertions**: The runner listens to the channel and asserts that the bot responds (or doesn't respond) within a given timeout.

### Required Environment Variables
- `DISCORD_TOKEN`: The Discord bot token.
- `E2E_CHANNEL_ID`: The channel ID where the test messages are sent and monitored.
- `E2E_GUILD_ID`: The whitelisted Guild ID where tests take place (bots ignore all other guilds in E2E mode).

### Optional Environment Variables
- `E2E_START_BOTS`: Set to `true` (default) to start the bots in the background inside the runner, or `false` if they are already running externally (e.g. in Docker).
- `E2E_TEST_BOTS`: Comma-separated list of bots to spawn (default: `"bunkbot,bluebot"`).
- `E2E_SUITE_PATH`: Path to a JSON file containing the test suite. If not specified, uses a built-in suite.

### Running the E2E Suite
```bash
DISCORD_TOKEN="your-bot-token" \
E2E_CHANNEL_ID="1234567890" \
E2E_GUILD_ID="9876543210" \
cargo run -p starbunk-e2e
```

### JSON Test Suite Format
Test suites are defined in simple JSON files:
```json
{
  "tests": [
    {
      "name": "Bunkbot ping response",
      "sender": "human",
      "message": "ping bunkbot",
      "expect": "Pong from bunkbot!",
      "timeout_ms": 2500
    },
    {
      "name": "Bunkbot ignores bot pings",
      "sender": "bot",
      "message": "ping bunkbot",
      "expect_no_response": true,
      "timeout_ms": 2500
    }
  ]
}
```

## See Also

- [[development/TDD|TDD SDLC Workflow]] — mandatory test-first development lifecycle
- [[CI-CD|CI/CD]] — tests run as a required check on every PR
