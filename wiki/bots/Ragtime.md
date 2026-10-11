# Ragtime

> Trivia bot for the guild. **Status: in development** — this page documents the foundation
> (crate skeleton, question model, repository contract). Game logic, answer grading and the
> question editor are future work.

## Goals & Purpose

Ragtime will run trivia rounds in Discord channels. Questions live in a Postgres question bank
(`ragtime_questions`) shared with the existing compose Postgres. This first stage only defines
how questions are stored and accessed.

## Status

| Stage | PR | Contents |
|---|---|---|
| 1 — tests first | `test/ragtime-question-repository` | Crate skeleton, `Question` model, `QuestionRepository` trait, `PgQuestionRepository` stubs (`todo!()`), migration, contract tests |
| 2 — implementation | follows | sqlx implementation of `PgQuestionRepository`, bot runtime (telemetry, serenity client, `ready` log) |
| Future | — | Game logic, grading, editor UI |

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
| `random(n)` | up to `n` distinct questions in random order; `n == 0` → empty | — |

Missing rows return `None`/`false` rather than an error, matching `ratbot::store::Store`.

## Round State Machine — `ragtime::round`

A pure, Discord-free game loop (no serenity types). The Discord layer holds one
`RoundRegistry` behind an async lock (never held across `.await`), feeds it inputs and posts
the returned `RoundEvent`s (`PostQuestion`, `PostReveal`, `PostTally`, `Cleanup`) in order.
Time is passed in explicitly (`now: Instant`, `tick(now)`); randomness is an injected `Rng`;
grading is an injected `Grader` (the crate's `grade()` function). Full API: rustdoc on
`ragtime::round`.

| Rule | Behaviour |
|---|---|
| One round per channel | `create` on a busy channel → `RoundAlreadyActive` |
| Host | The mentioner; **not** auto-joined. Only the host can start/cancel (`NotHost`) |
| Joining | Lobby only, idempotent. **Late joins are rejected** (`AlreadyStarted`) so the "everyone answered" early close stays well-defined |
| Start | Up to 20 random questions; fewer → all of them. No joiners → silent `Cleanup(NoPlayers)`. Empty bank → `Cleanup(NoQuestions)` |
| Answers | One per player per question (`AlreadyAnswered`); non-joined → `NotInRound` ("not in this round"); answer for a closed question → `StaleQuestion` |
| Question close | All joined players answered, or `QUESTION_TIMEOUT` (2 min). Reveal lists correct players in join order |
| Tally | 1 point per correct answer, descending. Ties share a rank (1, 2, 2, 4) and keep join order |

| Constant | Value | Meaning |
|---|---|---|
| `QUESTIONS_PER_ROUND` | 20 | Max questions per round |
| `QUESTION_TIMEOUT` | 2 min | Per-question timer |
| `LOBBY_TIMEOUT` | 10 min | Never-started lobby → `Cleanup(LobbyTimeout)` |
| `HOST_ABANDON_TIMEOUT` | 6 min | In-progress round with **no answers** for 3 question windows → `Cleanup(Abandoned)`. Resets on every answer |

## Testing

Contract tests: `crates/ragtime/tests/question_repository.rs` (`#[sqlx::test]`, `#[ignore]`d in
CI because they need Postgres). See [[development/Testing|Testing]] for how to run them.
