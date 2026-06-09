---
name: git-workflow
description: Mandatory rules for general git use in this repository. Follow these constraints before starting work, committing, or pushing.
---

# General Git Workflow

These are the strict rules for all Git usage in this repository. You MUST adhere to this workflow whenever tasked with writing code, fixing a bug, or implementing a feature.

## 1. Branching & Worktree Isolation
**Always sync `main` first**: Before starting any task, you must fetch and pull the latest changes from `main` to avoid drift.
```bash
git checkout main
git pull origin main
```

**Always branch out**: Never work directly on `main`. Create a descriptive branch (e.g., `feat/...`, `fix/...`, `chore/...`).
```bash
git checkout -b <branch-name>
```

**Always use worktrees**: You must isolate your work by adding a git worktree. This prevents state conflicts and keeps your workspace clean.
```bash
mkdir -p "$(dirname ".claude/worktrees/<branch-name>")"
git worktree add .claude/worktrees/<branch-name> <branch-name>
```
*(Note: All subsequent code modifications must be performed inside this worktree directory.)*

## 2. Pre-Commit Validation
Before you are allowed to execute a `git commit`, you MUST verify that your changes are sound locally. Run the following checks from within your worktree:
1. **Build**: Ensure the code compiles successfully (`cargo build --bins`).
2. **Test**: Ensure all existing tests pass (`cargo test --all`).
3. **Lint**: Ensure the linter is clean (`cargo clippy -- -D warnings`).

Only after these three checks pass successfully are you allowed to stage and commit your code.

## 3. Commit Message Rules
All commits must conform to the **Conventional Commits** specification. A git `commit-msg` hook is installed at `.git/hooks/commit-msg` to validate and enforce this rule.

- **Format**: `<type>(<scope>): <subject>`
  - `<type>`: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`
  - `<scope>`: optional (e.g. `bluebot`, `bunkbot`, `covabot`, `djcova`, `ratbot`, `shared`, `wiki`, `agents`, `git`)
  - `<subject>`: present tense, lowercase, no trailing period
- **Header Length**: Maximum of 72 characters.
- **Template**: Use the `.gitmessage` template file in the repository root.

## 4. Pushing Rules
**NEVER `git push` without express permission.**
Even if your tests pass and you have successfully committed your changes to your isolated branch, you must STOP and ask the user for explicit permission before attempting to push your code or open a PR.
