# CI/CD

> **Note:** The GCP pipelines (`release.yml`, `health-check.yml`, `main.yml`,
> `deploy.yml`, `deploy-pr.yml`) are retired — production is the Tower/Portainer
> stack. They are manual-dispatch only and every job is gated on the repository
> variable `GCP_PIPELINES_ENABLED` (unset ⇒ a dispatch does nothing). Only
> `ci.yml` runs automatically. See
> [[../infrastructure/Deployment|Deployment]].

## Workflows

### `ci.yml` — Pull Request Checks

Triggered on all PRs to `main`. The pipeline runs in two sequential steps:

#### Step 1: Compilation & Verification
- **1a. Cargo Build (Release)** — Compiles the changed bots/packages in production/release mode (`cargo build --release -p <package>`).
- **1b. Unit Tests** — Runs unit tests only for the changed bots/packages (`cargo test -p <package>`).
- **1c. Lint & DevOps** — Verifies formatting (`cargo fmt`), runs Clippy (`cargo clippy`), and runs DevOps consistency validation (`bash scripts/devops-validate.sh`).
- **1d. Build E2E Container** — Builds the E2E test runner docker image (`starbunk-e2e:ci-test`).

#### Step 2: Integration & E2E Validation (Only runs if Step 1 succeeds)
- **2a. E2E Tests** — Runs the end-to-end integration test suites against live Discord.
- **2b. Health Checks** — Verifies that all spawned bots successfully bind and respond to the `/health` endpoints.

#### Final Gate
- **Validation Success** — Replaces the old two-check gate. It runs at the very end of the pipeline. If both Step 1 and Step 2 pass, the `Validation Success` status check is green, serving as the required check for merging.

#### Selective Validation (Change Detection)

The `ci.yml` workflow optimizes execution times by gating jobs based on changed paths and PR labels:
- **No Rust / DevOps changes (e.g. pure documentation/wiki PRs)**: All CI check jobs are skipped.
- **Global / Core Changes**: Modifying `Cargo.toml`, `Cargo.lock`, `docker/Dockerfile`, or `src/shared/` triggers builds, tests, and docker checks for **all** bots.
- **Specific Shared Libraries**: Changes under `src/shared/replybot` only trigger checks for `bluebot`. Changes under `src/shared/llm` or `src/shared/memory` only trigger checks for `covabot`.
- **Bot-Specific Code**: Modifying code in a specific bot's directory only triggers checks for that bot.
- **E2E Integration Tests**: The E2E container build and integration tests are only executed if `bluebot`, `bunkbot`, or global/shared dependencies changed (since the E2E suite primarily targets those bots).
- **Quick PRs / Docker Skipping**: If the PR has the `quick-pr` or `skip-docker` label, all Docker container builds, Docker smoke tests, and E2E integration test runs are skipped entirely, allowing for fast verification.


### `main.yml` — Merge to Main (retired)

**Retired.** Previously triggered on every push to `main` to validate, lint,
test, compute the next semver, publish images to GCP Artifact Registry, and
create a GitHub Release. The `push` trigger and the `confirm` input are gone; the
workflow is manual-dispatch only and every job is gated on
`GCP_PIPELINES_ENABLED`, so it no longer publishes anything by default.

### `deploy.yml` — Deploy to GKE (retired)

**Retired.** Previously fired on a published GitHub Release and deployed to GKE
via `kubectl`. It is now manual-dispatch only and gated on
`GCP_PIPELINES_ENABLED`. Production deploys happen on Tower through Portainer —
see [[../infrastructure/Deployment|Deployment]].

---

## Version bump rules

The `version` job reads the merge commit title (conventional commits):

| Commit title | Bump |
|---|---|
| `feat!:` or body contains `BREAKING CHANGE` | major |
| `feat:` | minor |
| `fix:`, `chore:`, `refactor:`, anything else | patch |

---

## Definition of Done

A task is **not complete** until:

1. All CI checks pass.
2. The PR has at least one approval and all checks are green.
3. `scripts/devops-validate.sh` exits cleanly (if any DevOps file was touched).
4. The relevant `wiki/` page(s) have been updated.
5. An entry has been added to `wiki/Changelog.md`.

## See Also

- `.github/workflows/`
- [[../infrastructure/Deployment|Deployment]]
- [[../Versioning|Versioning]]
