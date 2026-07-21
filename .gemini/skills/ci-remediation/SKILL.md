---
name: ci-remediation
description: Remediation workflow for fixing CI/CD pipeline failures and resolving git merge conflicts.
---

# CI/CD Remediation & Conflict Resolution Protocol

This skill guides the cartel when a branch's pipeline fails or when merge conflicts block a pull request.

---

## Remediation Workflow

### 1. Diagnosis
*   **Active Agents:** **The Mechanic**
*   **Action:**
    *   Examine pipeline failure logs (GitHub Actions console, compiler logs, test outputs).
    *   Categorize the error: compilation error, lint/format check failure, failing unit/E2E tests, or validation script failures.

### 2. Conflict Resolution (if applicable)
*   **Active Agents:** **The Inspector** & **The Artist**
*   **Action:**
    *   Pull latest changes from origin main: `git fetch origin` and `git merge origin/main` (or rebase).
    *   Locate conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`).
    *   **The Artist** resolves logical code conflicts; **The Mechanic** resolves test/config conflicts.

### 3. Local Verification
*   **Active Agents:** **The Mechanic** & **The Artist**
*   **Action:**
    *   Format and lint the code.
    *   Run tests: `cargo test`.
    *   Run validation: `bash scripts/devops-validate.sh`.

### 4. Push & Pipeline Monitor
*   **Active Agent:** **The Inspector**
*   **Action:**
    *   Stage and commit changes using conventional commit formats.
    *   Push to branch and monitor GitHub workflow checks until they pass green.
