# RatBot — Development Instructions

> See also: `wiki/bots/RatBot.md`

## Goals & Purpose

RatBot is a **rat-themed Secret Santa bot** that organises "Ratmas" — a
Secret Santa exchange for the guild. It is not a general-purpose trigger bot;
its entire purpose is managing the Ratmas gift exchange event.

## Major Features

- Accepting sign-ups for the Ratmas Secret Santa event.
- Randomly assigning gifters to recipients and notifying each participant via DM.
- Announcing Ratmas events and reminders in the guild.
- Rat-themed copy and imagery throughout (names, messages, embeds, etc.).

## Dependencies & Architecture

- `src/bin/ratbot.rs` — entry point with serenity client.
- `src/shared/discord/` — `MessageService` for guild messages and DMs.
- No external services required for basic assignment logic.

## Edge Cases

- Prevent a participant from being assigned themselves.
- Handle an odd number of participants gracefully.
- Guard against duplicate sign-ups from the same user.
- Only guild members with the appropriate role / channel access should be able
  to interact with Ratmas commands.
