# Changelog

Running log of all significant work done on starbunk-rs.
Add an entry under today's date for every PR or significant change.

---

## 2026-06-09 — Initial Rust port and project infrastructure

### Added
- Full Rust monorepo scaffold ported from starbunk-go.
- `src/bin/` entry points for all 5 bots (bluebot, bunkbot, covabot, djcova, ratbot).
- `src/shared/` — shared libraries: discord, llm, memory, middleware, replybot.
- `src/bots/` — per-bot implementation modules.
- `Cargo.toml` with all dependencies: serenity, tokio, async-trait, serde, reqwest, regex, tracing, sqlx, pgvector.
- Multi-stage `docker/Dockerfile` with dependency caching and unprivileged user.
- Production `docker-compose.yml` with all 5 bots + pgvector postgres.
- Dev `docker/docker-compose.yml` for local builds from source.
- GitHub Actions workflows: `ci.yml`, `main.yml`, `deploy.yml`.
- DevOps validation script `scripts/devops-validate.sh` adapted for `src/bin/` layout.
- Deployment scripts: `scripts/deployment/deploy.sh`, `scripts/deployment/health-check.sh`.
- Full wiki documentation ported and translated from starbunk-go.
- `AGENTS.md` and `CLAUDE.md` with Rust-specific guidance.
- `.env.example` with all environment variable documentation.
