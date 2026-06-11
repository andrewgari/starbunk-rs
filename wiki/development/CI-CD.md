# CI/CD

## Workflows

### `ci.yml` — Pull Request Checks

Triggered on all PRs to `main`. Jobs:

1. **Lint** — runs `cargo fmt --check` and `cargo clippy -- -D warnings`; skipped if no Rust files or Rust config files changed.
2. **Test** — runs `cargo test` for all affected packages; skipped if no bot code changed.
3. **Validation Success** — gate job that succeeds only if all builds (compilation), tests, and lints succeed. It waits for those and also runs the DevOps validation checks (`scripts/devops-validate.sh`), but does not wait for the docker/E2E test jobs.
4. **Docker Test** — matrix build; builds and smoke-tests each affected bot Docker image; skipped if no bot code changed.
5. **E2E Tests** — builds and runs end-to-end integration tests using the live Discord test suite; skipped if no bot code changed.
6. **Validation Complete** — final check that waits for all jobs, including docker/E2E test jobs.

The `Validation Success` job serves as the primary required status check for PR merging, ensuring builds, tests, and lints pass quickly without waiting for container builds and E2E runs.

#### Selective Validation (Change Detection)

The `ci.yml` workflow optimizes execution times by gating jobs based on changed paths and PR labels:
- **No Rust / DevOps changes (e.g. pure documentation/wiki PRs)**: All CI check jobs are skipped.
- **Global / Core Changes**: Modifying `Cargo.toml`, `Cargo.lock`, `docker/Dockerfile`, or `src/shared/` triggers builds, tests, and docker checks for **all** bots.
- **Specific Shared Libraries**: Changes under `src/shared/replybot` only trigger checks for `bluebot`. Changes under `src/shared/llm` or `src/shared/memory` only trigger checks for `covabot`.
- **Bot-Specific Code**: Modifying code in a specific bot's directory only triggers checks for that bot.
- **E2E Integration Tests**: The E2E container build and integration tests are only executed if `bluebot`, `bunkbot`, or global/shared dependencies changed (since the E2E suite primarily targets those bots).
- **Quick PRs / Docker Skipping**: If the PR has the `quick-pr` or `skip-docker` label, all Docker container builds, Docker smoke tests, and E2E integration test runs are skipped entirely, allowing for fast verification.


### `main.yml` — Merge to Main (auto-release)

Triggered on every push to `main`. This is the only workflow that creates releases
and deploys to Tower — **every merge automatically ships**. Jobs:

1. **Validate DevOps Consistency**
2. **Lint** — `cargo fmt --check` + `cargo clippy`
3. **Test** — `cargo test --all`
4. **Determine Version** — reads the last `v*` git tag and the merge commit title
   to compute the next semver (major/minor/patch via conventional commits).
5. **Docker Publish** — builds all five bots in parallel; pushes `:vX.Y.Z`,
   `:latest`, and `:sha-<short-sha>` to GHCR.
6. **Create Release** — creates a `vX.Y.Z` git tag and GitHub Release, which
   triggers `deploy.yml` automatically.

### `deploy.yml` — Deploy to Tower

Triggered automatically when a GitHub Release is published (i.e., after `main.yml`
completes). Tower deploys `:vX.Y.Z` (the specific version that was just released).
See [[../infrastructure/Deployment|Deployment]].

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
