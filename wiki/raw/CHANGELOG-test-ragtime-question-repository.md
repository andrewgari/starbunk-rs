## [Unreleased] — test/ragtime-question-repository

### Added
- `crates/ragtime`: new lib + bin crate for the Ragtime trivia bot (TDD PR 1 of 2).
- `Question` model plus `NewQuestion` / `QuestionUpdate` inputs.
- `QuestionRepository` async trait (`list`, `get`, `create`, `update`, `delete`, `random`) and `PgQuestionRepository` with `todo!()` stubs.
- sqlx migration `crates/ragtime/migrations/20261011000000_create_ragtime_questions.sql` creating `ragtime_questions`.
- `#[sqlx::test]` contract tests in `crates/ragtime/tests/question_repository.rs` (ignored in CI; need Postgres).
- Ragtime registered in `docker/docker-compose.yml`, `ci.yml`, `main.yml`, `AGENTS.md`; `wiki/bots/Ragtime.md`; DB-testing notes in `wiki/development/Testing.md`.
