---
description: Run all CI checks — build, clippy, tests, and fmt
allowed-tools: [Bash]
---

# Full Check

Run all CI checks: build, clippy lints, tests, and format verification.

## Instructions

Run the full check suite used in CI:

```bash
cargo build --all && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test && \
cargo fmt --all -- --check
```

This runs:
1. **Build** — ensure all packages compile
2. **Clippy** — catch common Rust mistakes (warnings treated as errors)
3. **Tests** — run the full test suite
4. **Format** — verify code is properly formatted

Report the results of each step clearly. If any step fails:
- For build errors: show the specific errors and affected files
- For clippy errors: show the violations and offer to fix them (never use `#[allow(...)]` to suppress)
- For test failures: show which tests failed and analyse the errors
- For fmt errors: run `cargo fmt --all` to auto-fix and show what changed

After all checks pass, also validate DevOps file consistency:
```bash
bash scripts/devops-validate.sh
```
