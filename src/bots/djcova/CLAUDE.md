# DJCova — Development Instructions

> See also: `wiki/bots/DJCova.md`

## Goals & Purpose

Voice channel music streaming service. Joins Discord voice on demand, streams
YouTube audio, and manages a per-guild playback queue. Ported from starbunk-js
DJCova.

## Major Features

- `/play <youtube-url>` — joins voice channel and streams audio.
- Per-guild queue (add, skip, clear).
- Voice channel state management (join, leave, idle timeout, reconnect).

## Dependencies & Architecture

- `src/bin/djcova.rs` — entry point with **voice intents** (`GatewayIntents::GUILD_VOICE_STATES`).
- `src/shared/discord/` — `MessageService` for status replies in text channel.
- Audio pipeline: serenity voice + songbird + ffmpeg (not yet fully wired).
- CPU-intensive audio processing — must not block the tokio event loop; use `tokio::spawn`.

## Edge Cases

- Monitor voice connection health; reconnect on disconnect using songbird's reconnect logic.
- Concurrent `/play` requests: serialize queue writes with `Arc<Mutex<Queue>>`.
- YouTube geo-restrictions and playback errors — report to text channel, skip to next.
- Clean up ffmpeg processes on bot disconnect or crash (drop handles, abort tasks).
