# BunkBot

> Administrative backbone and general reply bot.

## Goals & Purpose

BunkBot is the primary administrative bot for the StarBunk system. It handles
high message volume with fast reaction times and can post via webhooks as custom
identities using `src/shared/discord::MessageService`.

## Major Features

- General reply bot handlers using the Strategy pattern.
- **Startup DM notification:** On each `ready` event, BunkBot compares the `APP_VERSION` environment variable against a persisted `last_version` file at `$STARTUP_DM_DATA_DIR/last_version`. If the version has changed (or the file is missing), it sends a Discord DM to the user identified by `DISCORD_NOTIFY_USER_ID`. The version file is then updated. The feature is skipped when `APP_VERSION` is unset or set to `"dev"`, or when `DISCORD_NOTIFY_USER_ID` is unset. DM-send failures are logged as warnings and do not prevent the version file from being written.
  - `APP_VERSION` — the current deployment version string.
  - `DISCORD_NOTIFY_USER_ID` — Discord user ID (u64) to DM.
  - `STARTUP_DM_DATA_DIR` — directory holding `last_version` (default: `/app/data`).
- Admin slash commands:
  - `/bot` (subcommands: `enable`, `disable`, `override`, `reset`, `list`) to toggle individual bots and override trigger frequencies. The `bot_name` argument on `enable`, `disable`, `override`, and `reset` supports Discord autocomplete — as the user types, Discord sends an autocomplete interaction and the bot responds with matching loaded bot names (case-insensitive substring filter, capped at 25 suggestions).
  - `/comments` (subcommands: `set`, `append`, `get`, `clear`, `list`) to override a bot's response pool at runtime without editing YAML. The `bot_name` argument supports autocomplete from the loaded bot list. All replies are ephemeral. Only `list` is available to non-admins; all other subcommands require administrator permissions.
  - `/clearwebhooks` to fetch and clear active Starbunk webhooks.
  - `/ping` to verify bot responsiveness.
- Dynamic bot state manager (`BotStateService` / `InMemoryBotStateManager`) to enable/disable bots and apply frequency overrides at runtime.
- Local HTTP API (`127.0.0.1:9082/config`) to view and overwrite the active `bots.yml` configuration, automatically hot-reloading bot strategies.
- Health and liveness endpoints on the same API port:
  - `GET /live` — always returns `200 OK`; used as a Kubernetes liveness probe.
  - `GET /health` — returns `200 OK` once the Discord engine is initialised, or `503 Service Unavailable` while still starting up; used as a readiness probe.
- Filesystem Hot Reloading: BunkBot utilizes `notify` to watch its configuration directory. Changes to `*.yml` or `*.yaml` files (e.g. via Kubernetes ConfigMap updates or manual edits) instantly trigger a bot configuration reload without requiring a restart. Only `.yml`/`.yaml` extension events are processed — editor temp files, swap files, and other filesystem noise are ignored. If a reload returns a non-2xx status, a `WARN` log entry is emitted with the status code rather than silently discarding the failure.
- Config saves via `starbunk-ui` follow a two-phase write: the API must accept the config (HTTP 2xx) before it is persisted to the Kubernetes Secret, preventing corrupted or rejected configs from overwriting the stored state.
- Webhook-based responses using `send_message_with_identity`.

## Dependencies & Architecture

- **Entry point:** `src/bin/bunkbot.rs` → `src/bots/bunkbot::run()`
- **Framework:** `starbunk::run_bot` + `src/shared/discord::MessageService`
- **Identity/webhook:** `src/shared/discord::Identity` + `DiscordIdentityProvider`
- Scaled for high message volume — handlers must remain lightweight and non-blocking.

## Configuration

BunkBot dynamically loads reply bot strategies from `config/bots.yml` at startup. See the [[../infrastructure/Configuration|Configuration]] wiki page for detailed instructions on managing this configuration file in development and production GKE environments.

> **Note on `identity` fields:** The YAML parser accepts both `snake_case` (canonical) and `camelCase` for identity properties. For example, `bot_name` or `botName`, `avatar_url` or `avatarUrl`, and `user_id` or `as_member` (for `mimic` bots).

## Config Write Error Handling

The HTTP API endpoints `POST /config` and `PUT /api/bots` attempt to persist the
new configuration to `botbot.yml` before hot-reloading the bot strategies.

In a Kubernetes environment the config directory is typically a read-only
ConfigMap/Secret mount. Two `std::io::ErrorKind` values are therefore treated as
**expected and non-fatal**:

| `ErrorKind` | Scenario |
|---|---|
| `ReadOnlyFilesystem` | K8s read-only volume mount |
| `PermissionDenied` | Restrictive container permissions |

For these errors a `WARN` log entry is emitted and the request proceeds to
`reload_all_bots` as normal (the in-memory state is still updated).

Any other write failure (e.g. `StorageFull`, `NotFound`) is treated as an
**unexpected error** — an `ERROR` log entry is emitted and the endpoint returns
`500 Internal Server Error` instead of silently succeeding.

## Edge Cases

- Webhook permission errors or timeouts.
- Race conditions on simultaneous admin commands.
- Graceful degradation when Discord API is unreachable.
- Config write failures on non-K8s deployments now surface as HTTP 500 rather
  than being silently swallowed.

## Prometheus /metrics Endpoint

BunkBot exposes a `GET /metrics` endpoint on the same port as the health/config
API (default `0.0.0.0:9082`).  The response uses the standard Prometheus text
format (`text/plain; version=0.0.4; charset=utf-8`) and can be scraped directly
by a Prometheus server or Grafana Agent.

### Exposed metrics

| Metric | Type | Labels | Description |
|---|---|---|---|
| `bunkbot_messages_received_total` | Counter | — | Discord messages that passed the content filter |
| `bunkbot_bot_triggers_total` | Counter | `bot` | Times each named reply bot fired |
| `bunkbot_active_bots` | Gauge | — | Number of currently enabled reply bots |
| `bunkbot_response_latency_seconds` | Histogram | — | Per-dispatch latency (buckets: 5 ms … 2.5 s) |
| `bunkbot_errors_total` | Counter | `kind` | Errors by kind (e.g. `send`, `db`) |

All metrics live in a private `prometheus::Registry` (never the global default
registry) created once at startup by `BunkBotMetrics::new()`.  The `Arc` is
shared between the Axum API state and the Discord event handler.

### Prometheus scrape config example

```yaml
scrape_configs:
  - job_name: bunkbot
    static_configs:
      - targets: ['bunkbot:9082']
    metrics_path: /metrics
```

## See Also

- [[../infrastructure/Architecture|Architecture]]

