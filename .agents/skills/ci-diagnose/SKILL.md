---
name: ci-diagnose
description: Evaluate the CI/CD pipeline on the PR, check for failures, and fix any issues.
---
# CI/CD Diagnose Workflow

When triggered, perform the following steps in order:

1. **Check Pipeline Status**: Fetch the latest CI pipeline status for the current branch or PR. You can use `gh pr checks` or the GitHub MCP.
2. **Fetch Logs**: If the pipeline failed, extract the error logs from the failed jobs. Do not guess the failure reason without reading the logs.
3. **Analyze**: Identify the root cause of the failure (e.g., linting error, failing test, build error, Docker issue).
4. **Fix**: Make the necessary edits to the codebase to resolve the issue. If it's a test failure, ensure the tests pass locally before proceeding.
5. **Commit and Push**: Stage, commit, and push the fix (`git push`).
6. **Re-check**: Verify the pipeline starts running again and monitor it until it passes, or instruct the user to check the pipeline status once it's triggered.
