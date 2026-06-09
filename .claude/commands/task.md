---
description: Start a new task on a fresh branch and open a PR when done
argument-hint: [type/description]
allowed-tools: [Bash, Read, Write, Edit, MultiEdit, Glob, Grep]
---

# Task: Branch → Worktree → Work → PR

Full development workflow: sync main, branch, create an isolated worktree, do the work, open a PR, clean up.

Using a worktree gives each task its own working directory so multiple agents can run in parallel without conflicting.

## Arguments

The user invoked this with: $ARGUMENTS

Parse the arguments to determine:
- **type**: one of `feat`, `fix`, `chore`, `refactor`, `docs`, `test` — infer from context if not given
- **description**: short kebab-case summary of the work (max 50 chars, lowercase, hyphens only)

If no arguments were provided, ask the user: "What are we working on? (e.g. `feat/add-ratmas-signup` or just describe the task)"

## Step 1 — Sync main

From the **main repo root** (`/mnt/data/tank/workspace/starbunk-rs`):

```bash
git checkout main
git pull origin main
```

If `git checkout main` fails (dirty working tree), stop and tell the user: "Working tree has uncommitted changes. Please stash or commit them first."

## Step 2 — Create branch

Derive the branch name as `<type>/<description>` (e.g. `feat/add-ratmas-signup`, `fix/covabot-crash`).

```bash
git checkout -b <branch-name>
```

## Step 3 — Create worktree

Create an isolated working directory for this task. All file work happens here from this point on.

```bash
mkdir -p .claude/worktrees
git worktree add .claude/worktrees/<branch-name> <branch-name>
```

The worktree path is: `.claude/worktrees/<branch-name>` (gitignored, safe to create freely).

Tell the user: "Worktree ready at `.claude/worktrees/<branch-name>`. Working in isolation."

## Step 4 — Do the work

**All file reads, edits, and writes must use the worktree path as root:**
`/mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/<branch-name>/`

Follow project conventions from CLAUDE.md and AGENTS.md:
- Shared code goes in `crates/starbunk-shared/`
- Each bot is isolated under `crates/<bot>/` — never import between bots
- Do not commit anything under `config/`, `local/`, or `data/` directories
- Follow TDD: write failing tests first, then implement

Run checks from the worktree root:
```bash
cd /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/<branch-name>
cargo build --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

If any DevOps files were touched (docker-compose, CI workflows, health-check):
```bash
bash scripts/devops-validate.sh
```

## Step 5 — Commit

Stage only the files changed for this task. Write a conventional commit message:

```
<type>(<scope>): <short description>

<optional body if non-obvious>

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
```

Examples:
- `feat(ratbot): add Ratmas gift assignment logic`
- `fix(covabot): handle empty LLM response gracefully`
- `chore(ci): update docker build matrix`

```bash
cd /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/<branch-name>
git add <specific files>
git commit -m "..."
```

## Step 6 — Push and open PR

```bash
git push -u origin <branch-name>
```

Then create the PR:

```bash
gh pr create \
  --title "<type>(<scope>): <short description>" \
  --body "$(cat <<'EOF'
## Summary
- <bullet points describing what changed and why>

## Test plan
- [ ] <manual or automated test steps>

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

## Step 7 — Clean up worktree

After the PR is open, remove the worktree to keep the repo tidy:

```bash
cd /mnt/data/tank/workspace/starbunk-rs
git worktree remove .claude/worktrees/<branch-name>
```

Return the PR URL to the user.

## Rules

- Never skip Step 1 — always start from an up-to-date main.
- Never commit to main directly.
- Never use `git add .` or `git add -A` — always stage specific files.
- Never use `--no-verify` on commits.
- All file operations after Step 3 must use the worktree path, not the main repo.
- Write failing tests before implementing (TDD: PR 1 = tests, PR 2 = implementation).
- If the task spans multiple logical changes, use multiple commits on the same branch.
- Update the relevant `wiki/` page(s) and add a `wiki/Changelog.md` entry before opening the PR.
