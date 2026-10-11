# Ragtime

> Trivia bot for the guild. **Status: playable** — question repository (Postgres), the
> Discord-free round state machine, presentation builders and the live Discord wiring are done.
> The fuzzy grader (interim exact-match in use) and the question editor are separate cards.

## Goals & Purpose

Ragtime will run trivia rounds in Discord channels. Questions live in a Postgres question bank
(`ragtime_questions`) shared with the existing compose Postgres. This first stage only defines
how questions are stored and accessed.

## Status

| Stage | PR | Contents |
|---|---|---|
| 1 — tests first | `test/ragtime-question-repository` | Crate skeleton, `Question` model, `QuestionRepository` trait, `PgQuestionRepository` stubs (`todo!()`), migration, contract tests |
| 2 — implementation | follows | sqlx implementation of `PgQuestionRepository`, bot runtime (telemetry, serenity client, `ready` log) |
| 3 — round state machine | `feat/ragtime-round-state-machine` | Discord-free `ragtime::round`: lobby, join, start, question rotation, grading, reveal, tally, abandonment |
| 4 — presentation builders | `test/ragtime-presentation-builders` | `ragtime::presentation`: pure embed/component/modal builders — lobby, question + \[Answer\] button, answer modal, reveal, final tally, ephemeral responses |
| 5 — Discord wiring | `feat/ragtime-discord-wiring` | `ragtime::bot`: mention/reaction/slash-command/button/modal handlers, tokio timers, `PgQuestionRepository` implementation, bot runtime |
| Future | — | Fuzzy `grade()` (replaces interim grader), question editor |

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

## Presentation builders (`ragtime::presentation`)

Pure functions mapping plain data (`u64` ids, strings, scores) to `serenity` builder types
(`CreateEmbed`, `CreateButton`, `CreateActionRow`, `CreateModal`). No network I/O and no
dependency on `ragtime::round` — the module has its own small structs (`RevealInfo`, `TallyRow`)
that mirror `round::Reveal` / `round::Standing` so the two can evolve independently. See the
module's rustdoc (`cargo doc -p ragtime --open`) for the full wiring table mapping each
`RoundEvent` to a builder call.

| Builder | Produces |
|---|---|
| `lobby::lobby_embed(host)` | "Ragtime contest" embed; join via the `lobby::JOIN_EMOJI` reaction |
| `question::question_embed` + `question::question_action_row` | "Question N/total" embed + \[Answer\] button |
| `modal::answer_modal` | single-text-input modal for submitting an answer |
| `reveal::reveal_embed` | shows the display answer and mentions every correct user, or says nobody got it |
| `tally::tally_embed` | ranked leaderboard; truncates with an "...and N more." line if it would exceed Discord's embed description limit |
| `response::not_in_round_text` / `response::already_answered_text` | ephemeral copy for the two \[Answer\]-button edge cases |

Routing: `custom_id::AnswerRoute { channel, index }` is encoded into both the \[Answer\] button's
and the answer modal's `custom_id` (different prefixes, so a button id can never be mistaken for
a modal id) and parsed back by `custom_id::parse_button` / `custom_id::parse_modal`.

## Discord wiring — `ragtime::bot`

| Input | Effect |
|---|---|
| @mention the bot (direct `<@id>` only) | Lobby embed + 🙋 reaction, mentioner is host. Busy channel → "already running" reply |
| 🙋 reaction on the lobby message | Join (bots and the bot itself ignored; reactions on other messages ignored) |
| `/ragtime start` (host only) | Loads up to 20 questions via `QuestionRepository::random`, posts question 1. No joiners → closed silently, no DB hit. Empty bank → notice |
| `/ragtime cancel` (host only) | Ends the round with a notice |
| \[Answer\] button | Ephemeral "not in this round" / "already answered" / "question closed", otherwise opens the modal |
| Modal submit | Graded by the state machine; ephemeral "Answer received!" for right and wrong alike |

Slash command follows ratbot's `/ratmas` convention (one global command, subcommands). Non-host
start/cancel get an ephemeral "Only the host can do that."

Gateway intents: `GUILDS | GUILD_MESSAGES | MESSAGE_CONTENT | GUILD_MESSAGE_REACTIONS`.

**Structure.** `RagtimeEngine` is Discord-free orchestration (plain `u64` ids) over an `Outbox`
trait; `DiscordOutbox` implements it with serenity and `RagtimeHandler` is the `EventHandler`.
`crates/ragtime/tests/discord_wiring.rs` drives the engine through full rounds with a fake outbox,
an in-memory repository and paused tokio time.

**Concurrency.** Two `tokio::sync::Mutex`es, always in the order `seq` → `state`. `seq` serialises a
state change with the posts it causes, so a channel's reveal always precedes the next question.
`state` (registry + lobby message ids + timer handles) is never held across an `.await`. The
\[Answer\] button path only takes `state` briefly so the modal opens inside Discord's 3 s window;
commands and modal submits are deferred ephemerally first.

**Timers.** One tokio task per channel sleeps until `RoundRegistry::next_deadline` and then calls
`tick`. Every state change aborts and replaces the channel's timer; a stale timer that fires anyway
is a no-op because `tick` only acts on deadlines that have passed. Timeouts: question 2 min, lobby
10 min, abandon 6 min without any answer (see the constants table above).

**Grading.** `bot::interim_grader()` is a case/whitespace-insensitive exact match on the display
answer and acceptable answers, used until the fuzzy `grade()` (card t_a1639804) lands.

## Testing

Contract tests: `crates/ragtime/tests/question_repository.rs` (`#[sqlx::test]`, `#[ignore]`d in
CI because they need Postgres). See [[development/Testing|Testing]] for how to run them.
