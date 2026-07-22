---
name: pr-start
description: Facilitate the implementation of a PR from an issue or text description, from start until it is ready to merge, leveraging The Bois for specialized tasks.
---

# PR Start Skill

Usage: `/pr-start <issue_number_or_text>`

You MUST coordinate with **The Bois** to execute this workflow:

1. **Analysis & Branch Creation**: The Face coordinates the work. If provided an issue number, read the issue details (e.g., using `gh issue view <issue_number>`). If provided a text description, use that as the implementation requirement. Create and check out a new branch for the task.
2. **Implementation**: Delegate the core implementation of the requirements to **The Artist** (for core Rust features) and **The Painter** (for UI/Observability).
3. **PR Creation**: Once the initial implementation is ready and tests are written, open a PR for the branch (e.g., using the `/pr-create` skill).
4. **Definition of Done**: Ensure the following criteria are met before the PR is considered ready to merge:
   - **All CI/CD checks pass**: Have **The Mechanic** diagnose and fix any test/CI failures (check using `/ci-diagnose` or `gh pr checks`).
   - **No conflicts**: Verify there are no merge conflicts with the target branch.
   - **No outstanding reviews**: You must defer to the `review` skill or `/pr-comment-review` to address existing comments. Have **The Critic** perform a final pass or address unresolved review feedback.
5. **Iterate**: If any criteria fail, dispatch the appropriate member of The Bois to fix the issues, push changes, and iterate until the PR is fully ready to merge.
