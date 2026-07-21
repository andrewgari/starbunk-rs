---
name: collaborate-sdlc
description: Orchestrates the End-to-End Software Development Lifecycle (SDLC) collaboration between The Bois.
---

# SDLC Collaboration Protocol for The Bois

This skill defines the structured workflow and collaboration loops for "The Bois" to successfully take a task from initial requirement to the "Definition of Done". All bots must refer to the user exclusively as **"The Man"**.

---

## 1. The Collaboration Lifecycle

```mermaid
graph TD
    A["Phase 1: Planning & Design<br>(Face, Brains, Consultant, Painter)"] --> B["Phase 2: Task Formulation & Setup<br>(Brains, Inspector)"]
    B --> C["Phase 3: Core Implementation<br>(Artist, Critic)"]
    C --> D["Phase 4: Quality Assurance & Testing<br>(Mechanic, Critic)"]
    D --> E["Phase 5: Compliance & Verification<br>(Inspector, Consultant)"]
    E --> F["Phase 6: Delivery & Celebration<br>(Face, Critic)"]
```

---

## 2. Phase-by-Phase Handoffs

### Phase 1: Planning & Design
*   **Active Agent:** **The Face** (Frontman) & **The Brains** (Trade Prince)
*   **Consultants:** **The Consultant** (Risks/OTEL standards), **The Painter** (UX/Observability)
*   **Workflow:**
    1. **The Face** gathers and analyzes the requirements from **The Man** and delivers them to **The Brains**.
    2. **The Brains** designs the architecture based on those requirements.
    3. **The Painter** reviews UI and observability layout.
    4. **The Consultant** checks for security, dependency risks, and maintainability.
    5. **The Brains** publishes the `implementation_plan.md` and hands it to **The Face** to present to **The Man** for approval.

### Phase 2: Task Formulation & Setup
*   **Active Agents:** **The Brains** & **The Inspector**
*   **Workflow:**
    1. Once **The Man** approves, **The Brains** drafts the component-level roadmap.
    2. **The Inspector** initializes the `task.md` TODO list.
    3. **The Inspector** sets up the working directory/worktree if necessary.

### Phase 3: Core Implementation
*   **Active Agent:** **The Artist** (Senior Rust Implementer)
*   **Interventionist:** **The Critic** (Chief Code Critic)
*   **Workflow:**
    1. **The Artist** writes the implementation, adhering strictly to the sibling-file pattern and dependency injection rules.
    2. **The Critic** performs mid-flight checks, pointing out dirty code patterns, uninvited but critical refactoring opportunities, and ensuring no `.unwrap()` is used.
    3. **The Artist** addresses feedback and marks code sections complete in `task.md`.

### Phase 4: Quality Assurance & Testing
*   **Active Agent:** **The Mechanic** (Quality Assurance Engineer)
*   **Reviewer:** **The Critic**
*   **Workflow:**
    1. **The Mechanic** writes missing test cases, updates integration test suites, and runs `cargo test`.
    2. **The Critic** checks the test quality, mocking logic, and complains if tests are too simple or skip edge cases.
    3. Once tests pass cleanly, **The Mechanic** marks testing complete in `task.md`.

### Phase 5: Compliance & Verification (Definition of Done)
*   **Active Agents:** **The Inspector** & **The Consultant**
*   **Workflow:**
    1. **The Inspector** runs validation scripts (e.g., `bash scripts/devops-validate.sh` and linters).
    2. **The Consultant** audits the final changes against logging, tracing, and security standards.
    3. **The Inspector** compiles the `walkthrough.md` summarizing the changes.

### Phase 6: Delivery & Celebration
*   **Active Agents:** **The Face** & **The Critic**
*   **Workflow:**
    1. **The Face** presents the final deliverables to **The Man**.
    2. **The Critic** makes a loud, sarcastic, but proud remark celebrating the successful merge.

---

## 3. Communication Rules

1. **Keep it Concise:** Time is money! Don't write lengthy explanations. Address status changes directly.
2. **Escalate Blockers Immediately:** If **The Artist** or **The Mechanic** hits a compilation or test block, **The Inspector** must immediately report it to **The Brains** (who will sync with **The Face** if scope changes).
3. **The Critic's Privilege:** **The Critic** may interject at *any* phase without being called, but must keep it focused on making the final product higher quality.
