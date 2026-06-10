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

**Always branch out & use worktrees**: Never work directly on `main`. Create a descriptive branch (e.g., `feat/...`, `fix/...`, `chore/...`) and isolate your work by adding a git worktree. This prevents state conflicts and keeps your workspace clean. Create the branch without checking it out in the main repository first:
```bash
BRANCH=<branch-name>
mkdir -p /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees
git -C /mnt/data/tank/workspace/starbunk-rs branch $BRANCH main
git -C /mnt/data/tank/workspace/starbunk-rs worktree add \
    /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/${BRANCH//\//-} $BRANCH
cd /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/${BRANCH//\//-}
```
*(Note: All subsequent code modifications must be performed inside this worktree directory.)*

## 2. Pre-Commit Validation
Before you are allowed to execute a `git commit`, you MUST verify that your changes are sound locally. Run the following checks from within your worktree:
1. **Build**: Ensure the code compiles successfully (`cargo build --bins`).
2. **Test**: Ensure all existing tests pass (`cargo test --all`).
3. **Lint**: Ensure the linter is clean (`cargo clippy -- -D warnings`).

Only after these three checks pass successfully are you allowed to stage and commit your code.

## 3. Pushing Rules
**NEVER `git push` without express permission.**
Even if your tests pass and you have successfully committed your changes to your isolated branch, you must STOP and ask the user for explicit permission before attempting to push your code or open a PR.
