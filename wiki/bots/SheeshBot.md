# SheeshBot

SheeshBot listens for "sheesh"-like expressions and replies with an increasingly drawn-out `sh{N}sh 😤`.

## Trigger

Any guild message (not from a bot) that contains a word matching `sh(e{2,})sh` — case-insensitive.

The token must contain at least two consecutive 'e' characters between the opening and closing `sh`.

| Message | Triggers? |
|---|---|
| `sheesh` | Yes (4 e's) |
| `sheeeesh` | Yes (6 e's) |
| `SHEESH` | Yes (case-insensitive) |
| `oh sheesh that's wild` | Yes (embedded) |
| `she` | No (no closing sh) |
| `shed` | No (no closing sh) |
| `shell` | No (no closing sh) |
| `shesh` | No (only 1 e) |

## Reply

A random string of the form `sh{N}sh 😤` where N is chosen uniformly at random from [2, 20] (inclusive).

Examples: `sheesh 😤`, `sheeeeesh 😤`, `sheeeeeeeeeeeeeeeeesh 😤`.

## Configuration

No configuration files or environment variables beyond `DISCORD_TOKEN`.

Uses `SHEESHBOT_TOKEN` if set, otherwise falls back to `STARBUNK_TOKEN`.

## Crate layout

```
crates/sheeshbot/
  Cargo.toml
  src/
    lib.rs          — event handler wiring
    main.rs         — entrypoint, telemetry init
    strategy.rs     — module root (re-exports SheeshStrategy)
    strategy/
      sheesh.rs     — trigger regex, reply generation, unit tests
```

## Health

Health endpoint: `http://localhost:8086/health` (default port `8086`).

Returns `{"status":"ok"}` once connected to the Discord gateway.

## DevOps registration

| File | Entry |
|---|---|
| `docker/docker-compose.yml` | `sheeshbot` service, `BOT_NAME: sheeshbot` |
| `.github/workflows/ci.yml` | `crates/sheeshbot/**` path filter |
| `.github/workflows/main.yml` | `sheeshbot` in docker build matrix |
| `AGENTS.md` | Bots table and Architecture crate list |
