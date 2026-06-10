# GEMINI.md

Agent guide for Gemini. All rules, architecture notes, and the DevOps
maintenance checklist live in [AGENTS.md](AGENTS.md).

## Permissions & Autonomy

You have broad autonomy to work without asking for confirmation. Default to doing, not asking.

### Always allowed without asking
- Read any file in this repo, including dotfiles and config directories (.claude, .gemini, .git, .env*, etc.)
- Edit, create, or delete files within the project directory
- Run read-only shell commands (ls, cat, grep, find, git status, git log, git diff, etc.)
- Install dependencies (npm install, go mod tidy, pip install, etc.)
- Run builds, tests, and linters
- Create branches, stage changes, and commit (but not push)

### Ask before doing
- `git push` or opening PRs
- Deleting files that aren't obviously temporary or generated
- Making changes outside the project directory
- Installing global packages
- Any destructive operation that can't be undone

### Never do
- Force push
- Modify CI/CD secrets or credentials
- Push directly to main/master

## How to work

- Don't ask clarifying questions mid-task unless you are genuinely blocked
- If something is ambiguous, make a reasonable assumption, state it, and proceed
- Prefer editing existing files over creating new ones unless structure demands it
- If a task would require a destructive or irreversible action, stop and describe what you need to do and why

## Gemini: Proactive Skill Use

Use available skills **without being told**. When the situation matches, invoke
the skill immediately — don't describe what you'd do, just do it.

| Situation | Skill / Rule |
|---|---|
| Any coding, fixing, or refactoring task | `/task` (or execute `/task` workflow) |
| Developing features, bugfixes, or ports | **Mandatory TDD**: Write Rust tests first (Test-Only PR 1), then implement (PR 2) |
| Code has been written or changed | `rust-craftsman` — review for quality and reuse |
| Deploying or updating containers on Tower | `devops` skill |
| PR is open — review comments to address | run `cargo test` locally, then address each comment |
| CI pipeline is failing on current branch | Use `ci-diagnose` skill to autonomously fix it |
| Making any commits to the repo | Git Commit Standards — follow conventional commits & commit-msg hook validation |

## Task Integration & Discretionary Loading
If the user asks to implement a feature, fix a bug, or perform any refactoring/coding task directly in chat without explicitly typing `/task`:
1. **Recognize the context**: Identify that the request constitutes a "task" (branch -> worktree -> build/test -> PR -> CI watch).
2. **Take action**: Proactively ask or recommend that the user invoke the `/task` command, OR load and execute the `/task` workflow at your own discretion to handle the workflow properly in an isolated worktree. Never work directly on `main` or bypass the worktree setup.

Before declaring any task done, follow the TDD SDLC workflow and run `cargo test` locally. If tests fail,
fixing them is part of the task.

Please read `AGENTS.md` for the core rules of this codebase.
