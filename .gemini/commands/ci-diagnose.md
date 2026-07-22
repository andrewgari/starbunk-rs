---
name: ci-diagnose
description: Autonomously diagnose and fix a failing CI pipeline on the current branch.
---

# CI Pipeline Diagnose & Fix Workflow

Follow these steps to autonomously diagnose and resolve CI failures on the current branch.

## Instructions

1. **Find Latest Failed Run**:
   Run the following command to retrieve the list of recent runs for the current branch and locate the latest failure:
   ```bash
   gh run list --branch $(git branch --show-current)
   ```

2. **Retrieve Failure Logs**:
   Query the detailed failure logs for the specific failed Run ID identified:
   ```bash
   gh run view [RUN_ID] --log-failed
   ```

3. **Reproduce Locally**:
   Run the exact linter command, test command, or build step reported in the failure log on your local environment (e.g. running specific linters, `cargo test --all`, or container builds).

4. **Fix the Root Cause**:
   Resolve the root issue, checking for configuration schema mismatches, version alignment, or code bugs. Do not just address the symptom.

5. **Local CI Check**:
   Before committing, run the full suite of local validations:
   ```bash
   cargo clippy -- -D warnings
   cargo test --all
   ```
   Ensure any pre-commit hooks also pass cleanly.

6. **Commit & Push**:
   Commit your changes using a conventional commit message:
   ```bash
   git commit -m "fix(ci): [description of root cause]"
   git push origin $(git branch --show-current)
   ```

7. **Verify Pipeline Execution**:
   Wait 30 seconds, then check the run list again to verify that the CI pipeline is running and progressing:
   ```bash
   gh run list
   ```

8. **Iterate**:
   If CI fails again with a DIFFERENT error, repeat starting from step 2. If it fails with the SAME error, your local reproduction was incorrect; investigate the environment or reproduction command deeper. Do up to 3 iterations. Report what you fixed and why it broke.
