# DJCova

> Voice channel music streaming service.

## Goals & Purpose

DJCova joins Discord voice channels on demand and streams YouTube audio. It
manages a per-guild queue so each server has independent playback state. Ported
from starbunk-js DJCova.

## Major Features

- **Slash Commands:** Integrated modern Discord slash commands for all music actions.
- **Interactive Controls:** Posts an interactive button panel when a song is playing, allowing users to stop, skip, restart, or re-queue the current track easily.
- **Tenor GIF Integration:** Searches and displays random dancing GIFs (via Tenor API) at regular intervals during active playback.
- **Auto-Disconnect Timers:**
  - 2-minute idle timer: Leaves the channel automatically if the queue remains empty.
  - 1-minute empty voice channel timer: Leaves the channel if all non-bot members leave.

## Slash Commands

- `/play [input] [file]` — Play a song via query/URL, or upload an audio file (MP3/FLAC/OGG/WAV).
- `/skip` — Skip the current track.
- `/stop` — Stop playback and disconnect from the voice channel.
- `/queue` — View the current queue.
- `/nowplaying` — View details of the currently playing track along with the control buttons.
- `/history` — View a list of recently played tracks.
- `/volume [level]` — Adjust playback volume (0-100%).
- `/repeat [mode]` — Set repeat mode (`off`, `song`, or `queue`).
- `/shuffle` — Shuffle the queue.
- `/clear` — Clear all tracks from the queue.
- `/help` — Display commands help menu.

## Configuration

Requires the following environment variables:
- `DISCORD_TOKEN` — Discord bot token.
- `TENOR_API_KEY` — Tenor API key for fetching dancing GIFs.
- `DEV_GUILD_ID` (optional) — If set, registers slash commands to a specific guild instantly for faster development feedback.

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
