## [Unreleased] — feat/ragtime-question-repository

### Added
- `PgQuestionRepository` sqlx implementation (`list`, `get`, `create`, `update`, `delete`, `random` via `ORDER BY random() LIMIT $1`); TDD PR 2 of 2 for #196.
- `PgQuestionRepository::connect` / `migrate` — runs `crates/ragtime/migrations/` via `sqlx::migrate!` at startup.
- `ragtime::run` bot runtime and `RagtimeHandler` (`ready` log only; no commands or game logic).
- `crates/ragtime/tests/migrations.rs` — migrations are idempotent across restarts.
- Ragtime added to `docker/Dockerfile` manifest/stub list and `docker/docker-compose.tower.yml` (`RAGTIME_TOKEN`).

### Changed
- `wiki/bots/Ragtime.md` documents the runtime and implementation details.
- `.env.example` and `wiki/infrastructure/Deployment.md` list `RAGTIME_TOKEN`.
