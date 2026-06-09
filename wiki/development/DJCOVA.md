# DJCova
DJCova is a Discord music bot written in Rust.

## Tech Stack
- **Discord:** `serenity` for bot framework, `songbird` for voice
- **Audio source:** `yt-dlp` subprocess for YouTube (piped audio stream) — more reliable than native Rust YouTube libs
- **File uploads:** Accept Discord file attachments as audio input directly
- **Gif source:** Tenor API (search term: "dancing") — requires `TENOR_API_KEY` env var
- **Future sources:** Abstract audio source behind a trait so Spotify, SoundCloud, etc. can be added later

---

# Architecture

## Playback Flow
1. User runs `/play <song_name_or_url>`
2. Bot resolves the input (YouTube search, direct URL, or file attachment)
3. Bot joins the voice channel the command user is currently in
4. If something is already playing, the song is added to the queue and a confirmation is sent
5. If nothing is playing, playback starts immediately
6. Bot sends a "Now Playing" embed with interactive buttons (Stop / Skip / Restart / Re-queue)

## Dancing Gif Feature
- While music is actively playing, the bot periodically posts a dancing gif in the text channel associated with the voice channel
- Interval is randomized between **45 and 90 seconds**
- Gifs are fetched from the Tenor API using a randomized search (terms like "dancing", "dance party", "grooving") to keep variety
- **Message behavior:** If the bot's gif message is the most recent message in the channel, edit it in place instead of posting a new one
- If it is not the most recent message, post a new message and track the new message ID
- The gif cycle starts when playback begins and stops when the bot disconnects or playback stops
- Gif posts are fire-and-forget — a failure to fetch or post should log a warning but never affect playback

## Auto-disconnect Rules
- If the queue empties and no new songs are queued within **2 minutes**, the bot disconnects
- If the command user leaves the voice channel, a **1-minute countdown** begins; if they don't return, the bot disconnects
- Both timers are independent — whichever fires first wins
- Active playback is not interrupted by the command user leaving; the timer runs in the background

---

# Commands

## User Commands
- `/play <song_name_or_url>` — Play a song, or add to queue if something is already playing. Accepts YouTube URL, search query, or file attachment.
- `/skip` — Skip the current song. Any user can skip.
- `/stop` — Stop playback and disconnect the bot from the voice channel.
- `/queue` — Show the current queue as an embed.
- `/nowplaying` — Show the current song with playback progress.
- `/history` — Show songs played this session (not persisted across restarts).
- `/shuffle` — Shuffle the current queue.
- `/help` — Show available commands.

## Admin Commands (Manage Guild permission required)
- `/volume <0-100>` — Set playback volume. Default is 50. Per-guild setting, not persisted across restarts.
- `/clear` — Clear the queue (does not stop current song).
- `/repeat <off|song|queue>` — Set repeat mode.

---

# Error Handling

The bot must respond with a clear user-facing message for all failure cases. Silent failures are not acceptable.

| Scenario | Response |
|---|---|
| User not in a voice channel | "You need to be in a voice channel to use this command." |
| Bot can't join the voice channel (permissions) | "I don't have permission to join that voice channel." |
| YouTube URL is invalid or dead | "Couldn't load that URL. It may be private, deleted, or unavailable." |
| Age-restricted or region-blocked video | "That video isn't accessible (age-restricted or region-blocked)." |
| yt-dlp search returns no results | "No results found for that search." |
| yt-dlp subprocess fails/crashes | "Something went wrong fetching that audio. Try again." |
| File attachment is unsupported format | "Unsupported file type. Please upload an MP3, FLAC, OGG, or WAV file." |
| /skip or /stop with nothing playing | "Nothing is currently playing." |
| /queue or /history is empty | "The queue is empty." / "No history yet this session." |
| Tenor API unreachable or returns no results | Log warning, skip gif cycle tick silently |

---

# Embeds

## Now Playing
- Thumbnail (YouTube thumbnail or generic music icon for files)
- Song title and source (YouTube / File Upload)
- Requested by (username)
- Duration and live progress indicator if feasible
- Buttons: **Stop** | **Skip** | **Restart** | **Re-queue**

## Queue
- Numbered list of upcoming songs
- Each entry shows title, duration, and who queued it
- No interactive buttons

## History
- Numbered list of songs played this session
- Each entry shows title and who requested it
- No interactive buttons

## Dancing Gif
- Just the gif — no embed wrapper needed, plain message is fine
- Bot tracks the message ID so it can edit in place on the next tick

---

# Environment Variables
- `DISCORD_TOKEN` — Bot token
- `TENOR_API_KEY` — Tenor API key for gif fetching