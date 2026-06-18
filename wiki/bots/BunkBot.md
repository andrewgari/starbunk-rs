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
- Webhook-based responses using `send_message_with_identity`.

## Dependencies & Architecture

- **Entry point:** `src/bin/bunkbot.rs` → `src/bots/bunkbot::run()`
- **Framework:** `starbunk::run_bot` + `src/shared/discord::MessageService`
- **Identity/webhook:** `src/shared/discord::Identity` + `DiscordIdentityProvider`
- Scaled for high message volume — handlers must remain lightweight and non-blocking.

## Configuration

BunkBot dynamically loads reply bot strategies from `config/bots.yml` at startup. See the [[../infrastructure/Configuration|Configuration]] wiki page for detailed instructions on managing this configuration file in development and production GKE environments.

## Edge Cases

- Webhook permission errors or timeouts.
- Race conditions on simultaneous admin commands.
- Graceful degradation when Discord API is unreachable.

## See Also

- [[../infrastructure/Architecture|Architecture]]

