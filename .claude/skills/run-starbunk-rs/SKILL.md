---
name: run-starbunk-rs
description: Run, start, build, test, or screenshot the starbunk-rs Discord bot system. Use when asked to run any bot, check health, smoke-test the API, or verify the build.
---

starbunk-rs is a Cargo workspace of 5 independent Discord bots (bluebot, bunkbot, covabot, djcova, ratbot) plus a shared library crate. The interaction surface is an HTTP health endpoint on each bot and a REST API server on bunkbot. The primary agent path is `smoke.sh` — it builds, runs unit tests, and probes live endpoints via `curl`.

Paths below are relative to the repo root.

## Prerequisites

Postgres is required for bunkbot and ratbot. A running local instance is expected:

```bash
psql postgres://starbunk:starbunk@localhost/starbunk_memory -c '\l'
```

If Postgres is unavailable, run only `bluebot`, `covabot`, or `djcova` — those don't need a DB.

## Build

```bash
cargo build --workspace
```

All 5 bot binaries end up in `target/debug/`.

## Run (agent path) — smoke.sh

The driver is `.claude/skills/run-starbunk-rs/smoke.sh`. It builds, runs unit tests, starts each bot, polls its health endpoint, and (for bunkbot) probes the REST API.

```bash
# Run all bots
DATABASE_URL=postgres://starbunk:starbunk@localhost/starbunk_memory \
  bash .claude/skills/run-starbunk-rs/smoke.sh

# Run a single bot
bash .claude/skills/run-starbunk-rs/smoke.sh bluebot

# Available: bluebot | bunkbot | covabot | djcova | ratbot | all
```

Required env vars (set or export before running):

| Var | Required for | Default |
|-----|-------------|---------|
| `DISCORD_TOKEN` | All bots (to make them start) | `"fake"` (smoke uses this to quickly get the health response before auth fails) |
| `DATABASE_URL` | bunkbot, ratbot | `postgres://starbunk:starbunk@localhost/starbunk_memory` |
| `BUNKBOT_ADMIN_TOKEN` | bunkbot API auth | `"testtoken"` |

Expected output:
```
--- Build ---
PASS: cargo build --workspace
--- Unit tests ---
PASS: cargo test --workspace (all suites)
--- Health endpoints ---
PASS: bluebot health endpoint (port 8081)
...
===============================
PASS: 6  FAIL: 0
===============================
```

### BunkBot REST API (port 9082)

Bunkbot runs an Axum REST server on port 9082. With DB available, these all work:

```bash
curl http://127.0.0.1:9082/config           # Returns bots.yml YAML
curl http://127.0.0.1:9082/api/bots         # Returns bot configs as JSON
curl http://127.0.0.1:9082/api/bots/status  # Returns enabled/frequency/triggers_today
```

Admin endpoints require `Authorization: Bearer <BUNKBOT_ADMIN_TOKEN>`:

```bash
curl -X POST http://127.0.0.1:9082/api/bots/nice-bot/enable  \
     -H "Authorization: Bearer testtoken"
curl -X POST http://127.0.0.1:9082/api/bots/nice-bot/frequency \
     -H "Authorization: Bearer testtoken" \
     -H "Content-Type: application/json" \
     -d '{"frequency": 50}'
```

### Direct invocation (unit tests without running the app)

Most bots have extensive unit tests that run without any Discord token or DB:

```bash
cargo test --lib -p bluebot    # 17 tests — strategy pattern for message matching
cargo test --lib -p bunkbot    # 171 tests — config parsing, API handlers, engine
cargo test --lib -p starbunk   # 79 tests — shared middleware, replybot, tracking
```

PRs touching bot logic, config parsing, or middleware usually only need this — no live process required.

## Run (human path)

Start a bot with a real token:

```bash
export DISCORD_TOKEN=<your-token>
export DATABASE_URL=postgres://...
cargo run --bin bunkbot
```

Each bot listens on a fixed health port and exits with an error if `DISCORD_TOKEN` is not set.

| Bot | Health port | API port |
|-----|-------------|----------|
| bluebot | 8081 | — |
| bunkbot | 8082 | 9082 |
| covabot | 8083 | — |
| djcova  | 8084 | — |
| ratbot  | 8085 | — |

## Gotchas

- **Health server responds before Discord auth.** Bots start the health HTTP server before connecting to Discord, so `/health` returns `{"status":"ok"}` even with a fake token, then the process exits ~300ms later when Discord rejects it. The smoke script polls in a tight loop to catch this window. With a real token the health server stays up.

- **OTEL export errors at startup are normal.** Every bot logs `ExportFailed` on startup because there's no OTLP collector running locally. This is expected and harmless — the bots work fine.

- **BunkBot needs DB twice.** `run()` connects to Postgres at startup (before Discord connects) to initialize the API state. A second connection happens inside the `ready` event handler. Both must succeed. If DB is down, bunkbot panics at startup.

- **Health path is `/health`, API path is at the root.** The health server at port 808x only handles `/health`. The Axum REST server at port 9082 (bunkbot only) handles `/config`, `/api/bots`, etc. Hitting port 8082 for `/config` returns the health JSON response, not the config.

- **`bunkbot --lib` runs 171 tests but `bunkbot` bin runs 0.** Test functions live in the lib crate. `cargo test -p bunkbot` runs both but shows them separately — don't mistake the bin's "0 tests" for no coverage.

## Troubleshooting

**`DISCORD_TOKEN not set` at startup**
→ Set `DISCORD_TOKEN=fake` (or a real token). The smoke script does this automatically.

**`Failed to connect to DB` / `connect error`**
→ Postgres isn't running. Start it or skip bunkbot: `bash smoke.sh bluebot`.

**Health endpoint returns nothing / connection refused**
→ The bot crashed before the health server accepted a connection. Check stderr for `error\[` lines.

**`cargo build` fails**
→ Run `cargo build --workspace 2>&1 | grep "error\["` to find the failing file. Usually a missing dependency after a `git pull`.
