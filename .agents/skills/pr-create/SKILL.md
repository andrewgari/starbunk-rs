---
name: pr-create
description: Create a PR using the current branch. Generates a PR description, commits changes, and pushes.
---
# PR Create Workflow

When triggered, perform the following steps in order:

1. **Check Status**: Use `git status` or the GitHub MCP to confirm there are uncommitted changes on the current branch.
2. **Draft Description**: Analyze the `git diff` and write a comprehensive PR description summarizing the goals, changes, and any technical decisions.
3. **Commit**: Stage and commit the changes following Conventional Commits (e.g., `feat: ...`, `fix: ...`). You may use `run_command` with `git add .` and `git commit -m "..."`. 
4. **Sync and Check Conflicts**: Run `git fetch origin main` and `git merge origin/main` to ensure your branch is up to date and there are no merge conflicts. If conflicts arise, resolve them, commit the resolution, and verify tests pass before proceeding.
5. **Push**: Push the current branch to the remote repository (`git push origin <branch_name>`).
6. **Create PR**: Create the pull request on GitHub. You can use the GitHub MCP `create_pull_request` tool or run `gh pr create --title "..." --body "..."`.
7. **Report**: Provide the PR link to the user and mark the task as complete.
