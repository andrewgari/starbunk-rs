# Test-Driven Development (TDD) SDLC Workflow

To maintain code quality, ensure correct behavior replication from the legacy Go and JS codebases,
and prevent regression, `starbunk-rs` implements a strict **Test-Driven Development (TDD)**
Software Development Life Cycle (SDLC).

This workflow is **mandatory** for all developers and AI agents working on this project.

---

## Core Rule: The Two-PR Sequence

Every new feature, bot port, or behavioral adjustment must be completed in **two distinct pull requests**:

```
Start Task
  ↓
Locate Behavior in starbunk-go (or starbunk-js)
  ↓
Write Rust tests (#[test] / #[tokio::test])
  ↓
Verify Tests Fail — Red Phase
  ↓
Submit PR 1: Test-Only / Behavior Definition
  ↓ (Approved / Merged)
Write Minimal Implementation — Green Phase
  ↓
Refactor & Clippy — Refactor Phase
  ↓
Submit PR 2: Implementation
```

### 1. PR 1: Test-Only / Behavior Definition
- **Scope**: Must contain **only** test code (`#[cfg(test)]` blocks, `tests/` files) and the minimal necessary stubs or trait definitions to allow the code to compile.
- **E2E Integration Tests**: If the change affects a bot's message strategy or trigger logic, you **must** also add corresponding E2E integration test cases to the JSON test suite (e.g. `crates/e2e/suites/bunkbot_bluebot.json`).
- **Rule**: Absolutely no functional production implementation changes are allowed in this PR.
- **Goal**: To define the expected behavior through unit and integration tests. The newly added tests must fail. This is the **Red** phase.

### 2. PR 2: Implementation
- **Scope**: Contains the actual production code that implements the feature or bot behavior.
- **Rule**: This PR must make both the unit tests and the live E2E integration tests pass without modifying the tests themselves (unless test bugs are found and corrected).
- **Goal**: To satisfy all test constraints (unit and E2E) with clean, minimal code. This is the **Green & Refactor** phase.

---

## Step-by-Step TDD Guide

### Step 1: Locate the Behavior in `starbunk-go` or `starbunk-js`

Since `starbunk-rs` is a Rust port of `starbunk-go` (which itself was ported from `starbunk-js`),
the desired behavior already exists. Before writing any code:

1. Check `/home/andrewgari/workspace/starbunk-go/cmd/<botname>/` for the Go implementation.
2. Or check `/home/andrewgari/workspace/starbunk-js/src/<botname>/` for the original JS.
3. Identify:
   - What triggers the bot (message patterns, roles, guild permissions).
   - What the bot replies or executes (message text, DM alerts, webhooks, voice channels).
   - Edge cases (caps/lowercase, punctuation, bot self-messages, command arguments).

### Step 2: Write the Rust Tests (Red Phase)

Write tests in the appropriate module under `src/bots/<botname>/` or `src/shared/`.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper to build minimal test messages
    fn make_msg(content: &str) -> String {
        content.to_string()
    }

    #[test]
    fn blue_strategy_triggers_on_blue() {
        let strategy = BlueStrategy::new();
        assert!(strategy.should_trigger(&make_msg("I love blue")));
    }

    #[test]
    fn blue_strategy_does_not_trigger_on_bluetooth() {
        let strategy = BlueStrategy::new();
        assert!(!strategy.should_trigger(&make_msg("connect bluetooth")));
    }

    #[test]
    fn blue_strategy_response_is_correct() {
        let strategy = BlueStrategy::new();
        assert_eq!(strategy.response(), "Did somebody say Blu?");
    }
}
```

Run the tests and confirm they fail:

```bash
cargo test --lib bots::bluebot
```

Submit PR 1 containing only the test code.

---

### Step 3: Implement and Refactor (Green & Refactor Phase)

Once PR 1 is merged/approved, implement the code to pass the tests.

1. Write the minimal Rust code required to satisfy the failing tests.
2. Run tests locally to confirm they pass:
   ```bash
   cargo test
   ```
3. Refactor for Rust idioms: ownership patterns, error handling with `Result`, async where needed.
4. Run static validation:
   ```bash
   cargo build --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   bash scripts/devops-validate.sh
   ```
5. Submit PR 2 containing the implementation.

---

## Benefits of this SDLC
- **Clear Objectives**: Writing tests first forces complete understanding of requirements before coding.
- **Strict Separation**: Separating tests from implementation prevents writing tests that match implementation bugs.
- **Accurate Ports**: Translating Go/JS specs into Rust tests ensures behaviors are not lost during porting.
- **Rust Safety**: The test-first approach surfaces lifetime and ownership issues before they become complex bugs.
