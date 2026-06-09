---
description: Run tests for Starbunk-rs. Specify a bot/module or run all tests.
argument-hint: [bot-name|module|all]
allowed-tools: [Bash]
---

# Test Runner

Run tests for Starbunk-rs. Specify a bot name, module path, or run the full suite.

## Arguments

The user invoked this with: $ARGUMENTS

- No argument or `all`: run the full test suite
- A bot name (`bluebot`, `bunkbot`, `covabot`, `djcova`, `ratbot`): run tests for that crate
- A module path (e.g. `shared::middleware`): run tests matching that path
- A test name: run the specific test

## Instructions

Run tests based on the argument provided:

1. If no argument or `all` is provided, run the full test suite:
   ```bash
   cargo test
   ```

2. If a specific bot is provided (bluebot, bunkbot, covabot, djcova, ratbot):
   ```bash
   cargo test -p <bot>
   ```

3. If a module path is provided (e.g. `shared::middleware`):
   ```bash
   cargo test --lib <module>
   ```

4. If a test name is provided:
   ```bash
   cargo test <test_name>
   ```

Report the test results clearly, highlighting any failures.

If tests fail, analyse the output and suggest fixes. Remember:
- If the test is wrong, fix the test
- If the code is wrong, fix the code
- Never add `#[allow(...)]` to make tests pass
