---
name: test
description: Run the Rust test suite for the starbunk-rs monorepo or a specific crate/module
---

# Test Runner

Run tests for the project.

## Arguments
- `$ARGUMENTS` - Optional: specific package or test filter (e.g. `--lib`, `middleware`) or leave blank for all.

## Instructions

1. If no argument is provided, run the full test suite:
   ```bash
   cargo test --all
   ```

2. If an argument is provided, run tests matching the filter:
   ```bash
   cargo test <filter>
   ```

3. Report the test results clearly, highlighting any failures.

4. If tests fail, analyze the output and suggest fixes if the errors are straightforward.
