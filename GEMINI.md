# GEMINI.md

Agent guide for Gemini. All rules, architecture notes, and the DevOps
maintenance checklist live in [AGENTS.md](AGENTS.md).

## Gemini: Proactive Skill Use

Use available skills **without being told**. When the situation matches, invoke
the skill immediately — don't describe what you'd do, just do it.

| Situation | Skill / Rule |
|---|---|
| Any coding, fixing, or refactoring task | Use `task-runner` or `rust-craftsman` skill |
| Developing features, bugfixes, or ports | **Mandatory TDD**: Write Rust tests first (Test-Only PR 1), then implement (PR 2) |
| Code has been written or changed | `rust-craftsman` — review for quality and reuse |
| Deploying or updating containers on Tower | `devops` skill |
| PR is open — review comments to address | run `cargo test` locally, then address each comment |
| CI pipeline is failing on current branch | Use `ci-diagnose` skill to autonomously fix it |
| Making any commits to the repo | Git Commit Standards — follow conventional commits & commit-msg hook validation |

Before declaring any task done, follow the TDD SDLC workflow and run `cargo test` locally. If tests fail,
fixing them is part of the task.

Please read `AGENTS.md` for the core rules of this codebase.
