---
name: check
description: Run the full CI test suite locally (cargo test, cargo clippy, and devops-validate)
---

# Full Check

Run all local CI checks to ensure code is ready for PR.

## Instructions

Run the following checks sequentially:

1. **DevOps Validation**:
   ```bash
   bash scripts/devops-validate.sh
   ```
2. **Cargo Check**:
   ```bash
   cargo check
   ```
3. **Linting**:
   ```bash
   cargo clippy -- -D warnings
   ```
4. **Testing**:
   ```bash
   cargo test --all
   ```

Report the results of each step clearly. If any step fails, analyze the output and suggest fixes.
