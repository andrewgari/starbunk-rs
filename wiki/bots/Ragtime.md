# Ragtime

> Trivia bot for the guild. **Status: foundation only** — this page documents the question
> bank (model, Postgres repository, migration) and the bot runtime. Game logic, answer grading
> and the question editor are future work.

## Goals & Purpose

Ragtime will run trivia rounds in Discord channels. Questions live in a Postgres question bank
(`ragtime_questions`) shared with the existing compose Postgres. This first stage only defines
how questions are stored and accessed.

## Status

| Stage | PR | Contents |
|---|---|---|
| 1 — tests first | #196 `test/ragtime-question-repository` | Crate skeleton, `Question` model, `QuestionRepository` trait, `PgQuestionRepository` stubs (`todo!()`), migration, contract tests |
| 2 — implementation | `feat/ragtime-question-repository` | sqlx implementation of `PgQuestionRepository`, bot runtime (telemetry, migrations at startup, serenity client, `ready` log) |
| Future | — | Game logic, grading, editor UI |

## Runtime

`crates/ragtime/src/main.rs` → `ragtime::run(DATABASE_URL)`:

1. `starbunk::telemetry::init("ragtime")` — unified OTEL pipeline.
2. `PgQuestionRepository::connect` — opens a `PgPool` and runs `sqlx::migrate!("./migrations")`
   (embedded at compile time, tracked in `_sqlx_migrations`, idempotent on every boot).
3. `starbunk::utils::run_bot("Ragtime", default_intents(), RagtimeHandler)` — serenity 0.12 client.
   The `ready` handler logs `Ragtime connected and ready` with `user` and `guilds` fields.

There are no commands, message handlers or game logic yet.

## Environment Variables

| Variable | Required | Description |
|---|---|---|
| `DISCORD_TOKEN` | yes | Bot token. Compose maps it from `RAGTIME_TOKEN`, falling back to `STARBUNK_TOKEN`. |
| `DATABASE_URL` | yes | Postgres connection string. Compose points it at the `postgres` service. |
| `RUST_LOG` / `VERBOSE` / `OTEL_EXPORTER_OTLP_ENDPOINT` | no | Standard telemetry settings — see [[development/Observability|Observability]]. |

## Schema — `ragtime_questions`

Migration: `crates/ragtime/migrations/20261011000000_create_ragtime_questions.sql`
(standard sqlx `<timestamp>_<name>.sql` naming; ragtime is the first crate to use a sqlx
migrations directory — other bots create tables inline in `init_schema`).

| Column | Type | Notes |
|---|---|---|
| `id` | `BIGSERIAL` | Primary key |
| `question` | `TEXT NOT NULL` | Prompt shown to players |
| `display_answer` | `TEXT NOT NULL` | Canonical answer revealed after a round |
| `acceptable_answers` | `TEXT[] NOT NULL` | Every answer graded as correct |
| `created_at` | `TIMESTAMPTZ NOT NULL DEFAULT now()` | |
| `updated_at` | `TIMESTAMPTZ NOT NULL DEFAULT now()` | Bumped on every update |

## Repository API

`ragtime::repository::QuestionRepository` (async trait, inject as `Arc<dyn QuestionRepository>`):

| Method | Returns | Missing id |
|---|---|---|
| `list()` | all questions ordered by `id` | — |
| `get(id)` | `Option<Question>` | `None` |
| `create(NewQuestion)` | stored `Question` | — |
| `update(id, QuestionUpdate)` | `Option<Question>`; `None` fields are left unchanged, `updated_at` is bumped | `None` |
| `delete(id)` | `bool` — whether a row was removed | `false` |
| `random(n)` | up to `n` distinct questions in random order (`ORDER BY random() LIMIT n`); `n == 0` → empty | — |

`update` uses `COALESCE` per field and sets `updated_at = clock_timestamp()` (not `now()`, which
is frozen for the whole transaction).

Missing rows return `None`/`false` rather than an error, matching `ratbot::store::Store`.

## Testing

Contract tests: `crates/ragtime/tests/question_repository.rs` and `tests/migrations.rs`
(`#[sqlx::test]`, `#[ignore]`d in CI because they need Postgres). See [[development/Testing|Testing]] for how to run them.
