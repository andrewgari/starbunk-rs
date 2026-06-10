---
name: worktree
description: Worktree lifecycle protocol — context switching, creating, and cleaning up git worktrees for starbunk-rs.
---

# Worktree Protocol

Every task lives in its own isolated git worktree under `.claude/worktrees/`. The main worktree
stays on `main` and is kept functionally empty — it is a launchpad, not a workspace.

## Context check — run this before every task

```bash
git worktree list
```

Ask: *does this task belong to the current worktree's branch?*
- **Yes** → proceed.
- **No / different feature / new task** → perform a context switch before touching anything.

## Context switching

```bash
# 1. Sync main to origin — always start from a clean base
git -C /mnt/data/tank/workspace/starbunk-rs fetch origin main
git -C /mnt/data/tank/workspace/starbunk-rs reset --hard origin/main

# 2a. Worktree for the target branch already exists — go there
cd /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/<branch-slug>

# 2b. No worktree yet — create one from the freshly synced main
BRANCH=feat/my-feature
mkdir -p /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees
git -C /mnt/data/tank/workspace/starbunk-rs branch $BRANCH main
git -C /mnt/data/tank/workspace/starbunk-rs checkout -b $BRANCH
mkdir -p /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees
git -C /mnt/data/tank/workspace/starbunk-rs worktree add \
    /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/${BRANCH//\//-} $BRANCH
cd /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/${BRANCH//\//-}
```

Never carry work from one branch's worktree into another.

## Keeping main fresh

Use `reset --hard` rather than `pull` to eliminate local drift:

```bash
git -C /mnt/data/tank/workspace/starbunk-rs fetch origin main
git -C /mnt/data/tank/workspace/starbunk-rs reset --hard origin/main
```

Run this every time you start a new task or switch contexts.

## Cleanup

A cron job runs `scripts/cleanup-worktrees.sh --apply` hourly. It removes any worktree whose
working tree is completely clean (no staged, unstaged, or untracked files).

Manual cleanup:

```bash
bash scripts/cleanup-worktrees.sh          # dry-run
bash scripts/cleanup-worktrees.sh --apply  # remove clean worktrees
```

## Rules

- **Main is a launchpad.** Stays on `main`, always synced to `origin/main`, never edited directly.
- **Context-check every request.** Wrong worktree = switch before any work.
- **Sync main before every branch creation** — `reset --hard origin/main`, not `pull`.
- **One worktree = one branch = one PR.** Never reuse a worktree for a second PR.
- **Worktrees live under `.claude/worktrees/`** — gitignored, auto-cleaned when idle.
