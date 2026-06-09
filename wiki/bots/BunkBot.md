# BunkBot

> Administrative backbone and general reply bot.

## Goals & Purpose

BunkBot is the primary administrative bot for the StarBunk system. It handles
high message volume with fast reaction times and can post via webhooks as custom
identities using `src/shared/discord::MessageService`.

## Major Features

- General reply bot handlers using the Strategy pattern.
- Admin slash commands.
- Webhook-based responses using `send_message_with_identity`.

## Dependencies & Architecture

- **Entry point:** `src/bin/bunkbot.rs` → `src/bots/bunkbot::run()`
- **Framework:** `starbunk::run_bot` + `src/shared/discord::MessageService`
- **Identity/webhook:** `src/shared/discord::Identity` + `DiscordIdentityProvider`
- Scaled for high message volume — handlers must remain lightweight and non-blocking.

## Edge Cases

- Webhook permission errors or timeouts.
- Race conditions on simultaneous admin commands.
- Graceful degradation when Discord API is unreachable.

## See Also

- [[../infrastructure/Architecture|Architecture]]
