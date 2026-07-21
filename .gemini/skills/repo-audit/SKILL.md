---
name: repo-audit
description: Reviewing repository quality, test coverage, CI/CD configuration, and security hotspots.
---

# Repository Audit Protocol

This skill outlines how the cartel inspects the codebase for technical debt, security issues, test coverage gaps, and adherence to engineering standards.

---

## Audit Workflow

### 1. CI/CD & DevOps Compliance
*   **Active Agent:** **The Critic**
*   **Action:**
    *   Verify GitHub Actions workflow configurations (`.github/workflows/`).
    *   Run `bash scripts/devops-validate.sh` and fix/highlight any deviations.
    *   Review Dockerfile configurations for multi-stage optimizations.

### 2. Code Quality & Standards
*   **Active Agent:** **The Critic**
*   **Action:**
    *   Audit codebase for anti-patterns: search for raw `.unwrap()`, lack of `#[derive(Debug)]` on public structs, or missing `Arc<dyn Trait>` dependency injections.
    *   Check directory structures to ensure compliance with the sibling-file pattern.

### 3. Test Coverage & Integration
*   **Active Agent:** **The Mechanic**
*   **Action:**
    *   Verify test counts and E2E integration test suites (e.g. JSON test suites).
    *   Check for fragile tests or mocked boundaries that hide issues.

### 4. Security & Risk Assessment
*   **Active Agent:** **The Consultant**
*   **Action:**
    *   Scan for hardcoded secrets, API tokens, or dotfiles in tracked directories.
    *   Review Cargo lock/dependencies for known CVEs.

### 5. Report & Presentation
*   **Active Agents:** **The Critic** & **The Consultant**
*   **Action:**
    *   Compile findings into a structured summary (`audit_results.md`).
    *   Present recommendations to **The Brains** to decide on refactoring tasks.
