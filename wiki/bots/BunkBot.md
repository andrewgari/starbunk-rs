# BunkBot

> Administrative backbone and general reply bot.

## Goals & Purpose

BunkBot is the primary administrative bot for the StarBunk system. It handles
high message volume with fast reaction times and can post via webhooks as custom
identities using `src/shared/discord::MessageService`.

## Major Features

- General reply bot handlers using the Strategy pattern.
- Admin slash commands:
  - `/bot` (subcommands: `enable`, `disable`, `override`, `reset`, `list`) to toggle individual bots and override trigger frequencies.
  - `/clearwebhooks` to fetch and clear active Starbunk webhooks.
  - `/ping` to verify bot responsiveness.
- Dynamic bot state manager (`BotStateService` / `InMemoryBotStateManager`) to enable/disable bots and apply frequency overrides at runtime.
- Local HTTP API (`127.0.0.1:9082/config`) to view and overwrite the active `bots.yml` configuration, automatically hot-reloading bot strategies.
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

## See Also

- [[../infrastructure/Architecture|Architecture]]

