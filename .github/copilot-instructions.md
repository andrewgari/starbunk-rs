# Copilot Instructions

The full agent guide for this repository is in [AGENTS.md](../AGENTS.md) at the
repo root. Read that file before making any changes — it covers required DevOps
file maintenance, the branch protection workflow, architecture, and the complete
checklist for adding a new bot.

Key non-negotiables (details in AGENTS.md):

- Never push directly to `main` — all changes go through a PR.
- Follow the **Test-Driven Development (TDD) two-PR sequence** (PR 1: Test-Only / Behavior Definition; PR 2: Implementation).
- After any bot or CI/CD change, run `bash scripts/devops-validate.sh` and fix
  every `FAIL` before committing.
- PRs require `Validate DevOps Consistency`, `Lint`, and `Test` checks to pass.
