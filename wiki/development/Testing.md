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

## See Also

- [[development/TDD|TDD SDLC Workflow]] — mandatory test-first development lifecycle
- [[CI-CD|CI/CD]] — tests run as a required check on every PR
