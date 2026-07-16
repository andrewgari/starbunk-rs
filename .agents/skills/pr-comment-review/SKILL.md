---
name: pr-comment-review
description: Review PR comments, categorize them into critical/nitpicks, fix user-selected ones, and close resolved comments.
---
# PR Comment Review Workflow

When triggered, perform the following steps in order:

1. **Fetch Comments**: Fetch the pull request review comments for the current branch using the GitHub MCP or `gh pr review`.
2. **Analyze and Categorize**: Read each comment and categorize them into:
   - **Critical:** Bugs, architectural flaws, failing tests, or blockers.
   - **Nitpicks:** Style suggestions, minor naming conventions, or optional improvements.
3. **Present to User**: Show the categorized list to the user and ask which ones they want implemented. Recommend fixing the critical ones. Wait for the user's response.
4. **Implement Fixes**: For each approved fix, make the necessary codebase changes. Run tests/linters to verify.
5. **Commit and Push**: Stage, commit (`git commit -m "fix(pr): address review comments"`), and push the changes.
6. **Close Comments**: Mark the addressed comments as resolved on GitHub using the MCP or `gh` CLI.
7. **Prompt for Unfixed**: Ask the user if you should automatically close any comments that were categorized as nitpicks and deliberately not fixed, leaving an explanatory note if desired.
