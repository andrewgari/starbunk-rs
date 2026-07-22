---
description: Run Clippy and rustfmt on the codebase
argument-hint: [fix|check]
allowed-tools: [Bash]
---

# Lint

Run Clippy and rustfmt on the codebase.

## Arguments

The user invoked this with: $ARGUMENTS

- No argument or `fix`: auto-format with rustfmt and report any clippy issues
- `check`: check format and lints without modifying files

## Instructions

1. If `fix` is provided (or no argument), auto-format the code:
   ```bash
   cargo fmt --all
   ```
   Then run clippy and report any violations:
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

2. If `check` is provided, check without modifying:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. Report any clippy violations that couldn't be auto-fixed.

4. For remaining clippy issues, offer to fix them manually if they are straightforward.
   - Never suggest adding `#[allow(...)]` to suppress warnings — fix the underlying issue.
