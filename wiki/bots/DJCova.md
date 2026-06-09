# DJCova

> Voice channel music streaming service.

## Goals & Purpose

DJCova joins Discord voice channels on demand and streams YouTube audio. It
manages a per-guild queue so each server has independent playback state. Ported
from starbunk-js DJCova.

## Major Features

- `/play <youtube-url>` command — joins voice and streams audio.
- Per-guild queue management (add, skip, clear).
- Voice channel state management (join, leave, reconnect).

## Dependencies & Architecture

- **Entry point:** `src/bin/djcova.rs` → `src/bots/djcova::run()`
- **Framework:** `starbunk::run_bot` — **requires voice intents**: `GatewayIntents::GUILD_VOICE_STATES`.
- **Audio:** Uses `songbird` (Serenity voice library) + `yt-dlp` for audio extraction.
- CPU-intensive; audio decoding and streaming must not block the async executor.

## Edge Cases

- Voice connection health monitoring and reconnection.
- Concurrent `/play` requests and queue races.
- YouTube playback errors or geo-restricted videos.
- Proper cleanup of `songbird` players and yt-dlp processes on disconnect or crash.

## See Also

- [[../infrastructure/Architecture|Architecture]] — note on extending intents for voice
