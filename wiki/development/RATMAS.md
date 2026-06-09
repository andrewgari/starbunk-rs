Ratmas Bot — Implementation Plan 🐀🎁
A Discord bot that automates a "Secret Santa with rat memes" annual event for a personal server.

Overview
Bot name: Ratmas Bot (or ratmas-bot)
Language/Runtime: Node.js (TypeScript) or Go — agent's choice, but prefer whatever has the best Discord library support for slash commands and scheduled tasks
Discord Library: discord.js (Node) or discordgo (Go)
Persistence: SQLite or PostgreSQL — needs to store participant info, wishlists, assignments, and anonymous message routing across restarts
Deployment: Self-hosted (single-server personal bot)

Configuration
All dates and channel targets should be configurable via a config file (e.g., config.json or environment variables), not hardcoded. Admins should be able to adjust the schedule without touching code.
RATMAS_ANNOUNCE_CHANNEL_ID   — where signup announcement goes
RATMAS_CATEGORY_ID           — Discord category to create the ratmas channel under
RATMAS_ADMIN_ROLE_ID         — role that can run admin commands
RATMAS_YEAR                  — current ratmas year (used for channel naming)

Phase dates (ISO 8601):
  PHASE_SIGNUP_OPEN           — when signup announcement fires
  PHASE_SIGNUP_CLOSE          — when signups lock and assignments happen
  PHASE_LISTS_DUE             — final call reminder date
  PHASE_OPENING_POLL          — when opening day poll fires

Phases
Phase 0 — Channel Creation
Trigger: Scheduled date (early October or November)

Create a new text channel: 🐀-ratmas-{YEAR} (or similar) under the configured category
Pin a welcome message in the new channel with rat meme(s) and a brief explanation of the event
No interaction required from participants yet


Phase 1 — Signup
Trigger: Scheduled date (configurable)

Post an announcement in RATMAS_ANNOUNCE_CHANNEL inviting server members to join Ratmas
Message should include:

A rat meme or themed embed
Instructions to react with 🐀 (or similar emoji) to sign up
Clear cutoff date


On reaction add: assign the user the Ratmas {YEAR} role (create it if it doesn't exist)
On reaction remove: remove the role (opt-out before cutoff)
Role serves as the authoritative participant list going into Phase 2


Phase 2 — Wishlist Collection
Trigger: Scheduled date (after signup closes / or same as signup open — configurable)

Post announcement in 🐀-ratmas-{YEAR}:

Signups are locked
Instruct participants to submit their wishlist info using bot commands
Include another rat meme because of course



User Commands (slash commands):
CommandDescription/ratmas list set <url>Set or update your wishlist URL/ratmas list viewView your own stored wishlist info/ratmas list updatedShow when your wishlist was last updated/ratmas profile set-addressOpen a DM flow to privately submit shipping address (stored encrypted or separately, never shown in-channel)/ratmas profile viewView your own profile/info as stored/ratmas profile clearClear your stored data
All profile/wishlist data is stored per-user per-year. The bot should track wishlist_url, wishlist_last_updated (auto-stamped on set), and any other user-provided notes.

Phase 3 — Assignment
Trigger: Scheduled date (signup cutoff)

Post final call announcement in 🐀-ratmas-{YEAR}: lists are locking, last chance to update
After the deadline passes, the bot:

Collects all users with the Ratmas {YEAR} role
Runs a Secret Santa assignment shuffle (no self-assignment; attempt to avoid repeat pairs from prior years if history is available)
Sends each participant a DM with:

Who they are gifting to (display name + wishlist link if set)
A rat meme
Instructions for anonymous messaging


Logs assignments internally (never exposed publicly)



Edge cases to handle:

Odd number of participants: allow one three-way chain or abort and notify admin
User has DMs disabled: flag to admin, do not silently fail


Phase 4 — Anonymous Messaging
Trigger: Active from assignment through opening day
Participants can message their recipient (or receive messages from their santa) anonymously via the bot.
Commands:
CommandDescription/ratmas message send <text>Send an anonymous message to your assigned recipient/ratmas message reply <text>Reply to the last anonymous message you received
Behavior:

All messages are relayed via bot DMs
No identifying info is included in relayed messages
Messages are prefixed with something like: 🐀 *Your Secret Rat says:*
Bot logs the sender internally (for admin abuse review), but does not expose it to recipients
Only valid assignment pairs can message each other (santa → recipient and recipient → santa for replies)


Phase 5 — Opening Day Poll
Trigger: Scheduled date (configurable)

Post a poll in 🐀-ratmas-{YEAR} to decide the gift-opening day
Poll should propose 3–5 candidate dates (configurable, or generated based on a target window)
Use Discord's native poll feature if available; otherwise emoji-reaction poll

Unanimous consensus enforcement:

The bot monitors the poll
When one option has been selected by every participant (every user with the Ratmas {YEAR} role), it declares that date the winner and posts a confirmation
If the poll closes without unanimous agreement, the bot re-posts the poll with only the top 2 options (runoff), with a stern rat meme
If runoff also fails unanimity, fall back to majority vote and notify participants — admin can override


Admin Commands
All gated behind RATMAS_ADMIN_ROLE_ID.
CommandDescription/ratmas admin statusShow current phase, participant count, assignment status/ratmas admin participantsList current participants (role holders)/ratmas admin advance-phaseManually trigger the next phase (override schedule)/ratmas admin re-dm <user>Resend assignment DM to a specific user/ratmas admin list-allShow all submitted wishlists (admin only)/ratmas admin resetNuclear option — wipe current year's data (confirmation required)

Data Model
participants
  user_id         TEXT
  year            INT
  wishlist_url    TEXT
  wishlist_updated TIMESTAMP
  assigned_to     TEXT (user_id, nullable until Phase 3)

messages
  id              INT
  year            INT
  from_user_id    TEXT
  to_user_id      TEXT
  content         TEXT
  sent_at         TIMESTAMP

phases
  year            INT
  current_phase   INT
  phase_dates     JSON (keyed by phase number)

Rat Meme Strategy

Maintain a /assets/rats/ directory of rat memes/images
Each phase announcement should pull a random rat image from this directory for the embed
Keep it cursed. That's the point.


Out of Scope (for now)

Multi-server support (this is a personal server bot)
Web dashboard
Gift tracking / confirmation of receipt
Budget enforcement


Suggested Project Structure
ratmas-bot/
├── cmd/
│   └── ratmas/
│       └── main.go (or index.ts)
├── internal/
│   ├── phases/       — phase scheduling and trigger logic
│   ├── commands/     — slash command handlers
│   ├── db/           — data access layer
│   ├── assignment/   — shuffle algorithm
│   └── messaging/    — anonymous relay logic
├── assets/
│   └── rats/         — rat meme images
├── config.json
└── README.md