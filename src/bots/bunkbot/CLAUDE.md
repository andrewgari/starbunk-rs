# BunkBot — Development Instructions

> See also: `wiki/bots/BunkBot.md`

## Goals & Purpose

Administrative backbone and general reply bot. Handles high message volume with
fast reaction times and supports webhook-based persona posting.

## Major Features

- General reply bot handlers (trigger → response mapping).
- Admin slash commands.
- Webhook-based responses via `src/shared/discord/webhook_service.rs`.

## Dependencies & Architecture

- `src/bin/bunkbot.rs` — entry point, serenity client setup.
- `src/shared/discord/` — `MessageService` for direct messages and webhook persona posts.
- `src/shared/discord/identity.rs` — `IdentityProvider` resolves webhook persona from Discord.

## Edge Cases

- Webhook permission errors or rate-limit timeouts — handle gracefully, fall back to direct message.
- Race conditions on simultaneous admin commands — use locks or idempotent handlers.
- Self-message loop: always check `msg.author.id == ctx.cache.current_user().id` before responding.
