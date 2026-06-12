# DJCova

> Voice channel music streaming service. Streams YouTube audio per guild with a
> persistent queue, interactive button controls, and Tenor GIF reactions.

## Goals & Purpose

DJCova joins Discord voice channels on demand and streams YouTube audio via
`yt-dlp`. Each guild has an independent `GuildAudioManager` with its own queue,
history, volume, and repeat state. Ported from starbunk-js DJCova.

## Commands

| Command | Description | Permission |
|---|---|---|
| `/play <url\|query> [file]` | Join voice and play/queue a YouTube URL, search query, or uploaded audio file (MP3/FLAC/OGG/WAV) | Everyone |
| `/skip` | Skip the current track (own tracks only; admins skip any) | Everyone |
| `/skipnext <user>` | Remove the next queued track by a specific user | Everyone |
| `/skiplast <user>` | Remove the last queued track by a specific user | Everyone |
| `/stop` | Stop playback and disconnect from voice | Everyone |
| `/pause` | Toggle pause/resume on the current track | `MANAGE_MESSAGES` |
| `/queue` | Display the current guild queue | Everyone |
| `/nowplaying` | Show current track with interactive buttons | Everyone |
| `/history` | List recently played tracks | Everyone |
| `/shuffle` | Shuffle the remaining queue | Everyone |
| `/repeat [off\|song\|queue]` | Set repeat mode | Everyone |
| `/volume <1–100>` | Set playback volume | Everyone |
| `/clear` | Clear the current queue | Everyone |
| `/help` | Show the DJCova help menu | Everyone |

## Interactive Button Controls

`/nowplaying` and `/play` responses include a button row:

| Button | Action |
|---|---|
| Stop | Stop playback and disconnect |
| Skip | Skip to the next queued track |
| Restart | Restart the current track from the beginning |
| Re-queue | Add the current track back to the end of the queue |

## Auto-Disconnect Behaviour

- **Empty queue idle** — bot leaves voice after 2 minutes with no activity.
- **Empty channel** — when all non-bot members leave the voice channel, a 60-second leave
  timer starts. If a user returns the timer is cancelled.

## Architecture

```
lib.rs          — Handler (EventHandler): ready, interaction_create, voice_state_update
main.rs         — entry point: telemetry init, calls run()
manager.rs      — GuildAudioManager: queue, history, volume, repeat, idle/leave timers
voice.rs        — VoiceService trait + DiscordVoiceService (songbird)
gif_client.rs   — GifService trait + TenorGifClient (Tenor API)
commands/       — slash command handlers (play, skip, skipnext, skiplast, pause, stop, …)
commands/buttons.rs — interactive button component handler
```

Key types:

```rust
pub struct QueueItem {
    pub title: String,
    pub url: String,
    pub requester: String,
    pub duration: Option<Duration>,
    pub thumbnail_url: Option<String>,
}

pub enum RepeatMode { Off, Song, Queue }

pub struct GuildAudioManager {
    queue: VecDeque<QueueItem>,
    history: Vec<QueueItem>,
    current_track: Option<QueueItem>,
    volume: u8,          // 1–100, default 50
    repeat_mode: RepeatMode,
    is_paused: bool,
    idle_timer_active: bool,
    leave_timer_active: bool,
    // injected deps
    voice: Arc<dyn VoiceService>,
    gif: Arc<dyn GifService>,
}
```

## Dependencies

- **`songbird`** — Discord voice via Serenity (`register_songbird()` required on client build)
- **`yt-dlp`** — external binary; must be on PATH in the container
- **Tenor API** — `TENOR_API_KEY` env var required for GIF reactions
- **Gateway intents** — `GUILD_MESSAGES | MESSAGE_CONTENT | GUILD_VOICE_STATES | GUILDS`

## Configuration

| Env Var | Default | Purpose |
|---|---|---|
| `DISCORD_TOKEN` | — | Required |
| `TENOR_API_KEY` | — | Tenor GIF reactions (gracefully disabled if absent) |
| `DEV_GUILD_ID` | — | Register commands as guild commands (instant); omit for global |

## Ownership & Permission Model

- `/skip` — non-admins may only skip their own tracks. Users with `MANAGE_MESSAGES` may skip any track.
- `/pause` — restricted to `MANAGE_MESSAGES`.
- `/skipnext` and `/skiplast` — scan the queue by requester name; anyone may remove their own tracks.

## Edge Cases

- Concurrent `/play` requests during active playback are enqueued, not dropped.
- `yt-dlp` process is cleaned up on stop/disconnect.
- Songbird player handles are dropped on `stop()` to prevent resource leaks.
- Empty-channel leave timer is cancelled if a user returns within 60 seconds.
- Queue races: `GuildAudioManager` is wrapped in `Arc<Mutex<_>>` per guild; all mutations are serialised.
- Geo-restricted or unavailable YouTube videos return an error embed.

## Testing

Unit tests for `GuildAudioManager` command logic live in `crates/djcova/src/`. Failing tests for `skipnext`, `skiplast`, `pause`, and `resume` have been added as the TDD tests-first PR; implementation follows in the next PR.

E2E health tests live in `crates/e2e/suites/health_djcova.json` and cover: startup channel access (ping), bot self-filter, help command, and unknown input handling.

E2E voice tests are **not** supported by the webhook-based E2E runner — voice channel joining requires a different test harness.

## See Also

- [[../infrastructure/Architecture|Architecture]] — voice intents and per-guild isolation
- [[../development/Testing|Testing]] — unit test patterns for `GuildAudioManager`
- [[../development/Observability|Observability]] — structured log fields
