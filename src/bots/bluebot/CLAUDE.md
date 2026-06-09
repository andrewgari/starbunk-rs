# BlueBot — Development Instructions

> See also: `wiki/bots/BlueBot.md`

## Goals & Purpose

Detect any message that references "blue" or Blue Mage and reply with a witty,
character-themed response. Ported from starbunk-js BlueBot.

## Major Features

- Regex / string pattern matching across all guild messages.
- Contextual, character-specific replies.
- Optional LLM-enhanced validation to reduce false positives (not yet wired).

## Dependencies & Architecture

- `src/bin/bluebot.rs` — entry point, sets up serenity client and event handlers.
- `src/shared/discord/` — `MessageService` sends replies.
- `src/bots/bluebot/` — pattern matching logic and reply strategies.
- No external services required for basic pattern matching.

## Edge Cases

- Keep regex patterns simple and bounded to avoid ReDoS (Regex Denial of
  Service). Never use unbounded quantifiers on large character classes.
- **Rate-limit replies** — use separate windows per reply type:
  - Standard pattern matches: ~5-minute cooldown per channel.
  - Rare / enemy-themed responses: ~24-hour cooldown per channel.
- Distinguish colloquial "blue" (e.g. colour, mood) from intentional Blue Mage
  references. Optional LLM validation can reduce false positives once wired.
- Avoid triggering on messages sent by other bots (check `msg.author.bot`).
