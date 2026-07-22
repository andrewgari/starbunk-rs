---
description: Start a new task on a fresh branch and open a PR when done
argument-hint: [type/description]
allowed-tools: [Bash, Read, Write, Edit, MultiEdit, Glob, Grep]
---

# Task Workflow: Branch → Worktree → Work → PR → CI Watch

This command automates the complete development lifecycle for a task. It ensures work occurs in an isolated git worktree, validates the changes, pushes to origin, creates a pull request, and watches GitHub Actions CI to report any issues.

## Arguments

The user invoked this command with: `$ARGUMENTS`

Parse the arguments to determine:
- **type**: One of `feat`, `fix`, `chore`, `refactor`, `docs`, `test`. Infer from the description or context if not explicitly provided.
- **description**: A short kebab-case summary of the work (max 50 characters, lowercase, alphanumeric, and hyphens only).

If no arguments were provided, ask the user: "What are we working on? (e.g. `feat/add-ratmas-signup` or just describe the task)"

---

## Step 1 — Check Local State for Uncommitted Changes

Before switching branches or gears, check for any uncommitted changes in the current working directory:

```bash
git status --porcelain
```

If there are any staged, unstaged, or untracked changes:
1. List the files with uncommitted changes clearly to the user.
2. Ask the user how they would like to resolve these conflicts to proceed:
   - **Stash**: Run `git stash push -m "Automatic stash before task: <description>"` and continue.
   - **Commit**: Ask the user for a commit message, stage the changes (`git add -A`), commit them (`git commit -m "<message>"`), and continue.
   - **Discard**: Warning: this is destructive. Confirm with the user, then run `git reset --hard` and `git clean -fd` to discard all changes, and continue.
   - **Abort**: Stop execution and tell the user they can resolve the changes manually.

---

## Step 2 — Sync Main from Origin

Once the local state is clean:
1. Switch to the `main` branch on the main repository:
   ```bash
   git checkout main
   ```
2. Fetch and pull the latest changes from origin:
   ```bash
   git fetch origin
   git pull origin main
   ```

---

## Step 3 — Branch and Worktree Setup

Derive the branch name as `<type>/<description>` (e.g., `feat/add-ratbot-commands`).
Derive the worktree directory name by replacing slashes in the branch name with dashes:
`BRANCH_SLUG="${BRANCH_NAME//\//-}"` (e.g. `feat-add-ratbot-commands`).

1. Check if the branch already exists:
   ```bash
   git show-ref --verify refs/heads/<branch-name>
   ```
2. Check if a worktree already exists for this branch:
   ```bash
   git worktree list | grep -F ".claude/worktrees/${BRANCH_SLUG}"
   ```

3. Resolve the workspace configuration:
   - **Case 1: Worktree already exists**
     Tell the user: "Reusing existing worktree for <branch-name>."
     Navigate into the existing worktree path:
     ```bash
     cd /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/${BRANCH_SLUG}
     ```
   - **Case 2: Branch exists, but no worktree exists**
     Create the worktree pointing to the existing branch:
     ```bash
     mkdir -p /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees
     git worktree add /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/${BRANCH_SLUG} <branch-name>
     cd /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/${BRANCH_SLUG}
     ```
   - **Case 3: Neither exists**
     Create the branch from the freshly updated `main` and add the worktree:
     ```bash
     mkdir -p /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees
     git branch <branch-name> main
     git worktree add /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/${BRANCH_SLUG} <branch-name>
     cd /mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/${BRANCH_SLUG}
     ```

---

## Step 4 — Implement Changes

Once in the worktree directory (`/mnt/data/tank/workspace/starbunk-rs/.claude/worktrees/${BRANCH_SLUG}/`), proceed to implement the requested task.

**Rule**: All reads, writes, and edits must be performed inside this worktree directory.

### Pre-Commit Validation
Before staging and committing, you must verify that the changes are sound by running the following validation checks inside the worktree directory:
1. **Build**: `cargo build --all`
2. **Clippy**: `cargo clippy --all-targets --all-features -- -D warnings`
3. **Test**: `cargo test`
4. **DevOps Consistency**: If any bot, Docker, or CI/CD files are modified, you must run:
   ```bash
   bash scripts/devops-validate.sh
   ```

All checks must pass cleanly. If there are any errors or warnings, resolve them before committing.

---

## Step 5 — Commit Changes

Stage only the files changed for this task. Do not use `git add .` or `git add -A` unless you are sure all modified files are relevant.

Write a conventional commit message matching the repository's `.gitmessage` template format:
```
<type>(<scope>): <short description>

<optional detailed description if necessary>

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
```
*Example:* `feat(bluebot): reply to references of blue mage`

Execute the commit inside the worktree directory:
```bash
git add <modified-files>
git commit -m "..."
```

---

## Step 6 — Push, Create PR, and Watch CI

1. Push the branch to origin:
   ```bash
   git push -u origin <branch-name>
   ```

2. Create a Pull Request using the GitHub CLI (`gh`):
   ```bash
   gh pr create \
     --title "<type>(<scope>): <short description>" \
     --body "$(cat <<'EOF'
   ## Summary
   - <bullet points describing what changed and why>

   ## Test plan
   - [ ] <manual or automated verification details>

   🤖 Generated with [Claude Code](https://claude.com/claude-code)
   EOF
   )"
   ```

3. **Watch the CI workflow run**:
   - Wait 15 seconds to allow the GitHub Actions workflow to trigger.
   - Fetch the run ID for the current branch:
     ```bash
     gh run list --branch <branch-name> --limit 1 --json databaseId,status --jq '.[0].databaseId'
     ```
   - If a workflow run is active, watch it to completion:
     ```bash
     gh run watch <run-id>
     ```
   - If the run fails, analyze the logs to report issues immediately.
   - If the run passes, tell the user the PR is green and ready.

---

## Step 7 — Cleanup

After the PR has been created and verified, clean up the local worktree to keep the workspace tidy:
1. Navigate back to the main repository root:
   ```bash
   cd /mnt/data/tank/workspace/starbunk-rs
   ```
2. Remove the worktree:
   ```bash
   git worktree remove .claude/worktrees/${BRANCH_SLUG}
   ```

Report the PR URL and CI build status to the user.
