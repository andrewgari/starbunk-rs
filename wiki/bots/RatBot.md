# RatBot

> Rat-themed Secret Santa bot — organises the guild's "Ratmas" gift exchange.

## Goals & Purpose

RatBot manages **Ratmas**, a rat-themed Secret Santa event for the guild.
It handles sign-ups, randomly pairs gifters with recipients, notifies
participants, and keeps the guild informed with themed announcements.

This bot is not a generic trigger/response bot. Its sole purpose is running
the Ratmas Secret Santa exchange with maximum rat energy.

## Major Features

- **Sign-up management** — guild members opt in to Ratmas through a command or reaction.
- **Secret assignment** — once sign-ups close, each participant is randomly
  assigned a recipient (no self-assignments) and notified via DM.
- **Announcements** — Ratmas open/close and reminder messages are posted to a
  configured guild channel.
- **Rat-themed UX** — all copy, embeds, and imagery use rat / Ratmas branding.

## Dependencies & Architecture

- **Entry point:** `src/bin/ratbot.rs` → `src/bots/ratbot::run()`
- **Framework:** `starbunk::run_bot` + `src/shared/discord::MessageService`
- Assignment logic is self-contained; no external services required.

## Edge Cases

- A participant must never be assigned themselves.
- Odd participant counts need a graceful fallback (e.g. a three-way cycle).
- Duplicate sign-ups from the same user must be deduplicated.
- Sign-up and admin commands should be gated to appropriate roles/channels.

## See Also

- [[BlueBot|BlueBot]] — another pattern-based bot in the same monorepo
