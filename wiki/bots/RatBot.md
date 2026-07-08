# RatBot

> Rat-themed Secret Santa bot — organises the guild's "Ratmas" gift exchange.

## Goals & Purpose

RatBot manages **Ratmas**, a rat-themed Secret Santa event for the guild.
It handles sign-ups, randomly pairs gifters with recipients, notifies
participants, and keeps the guild informed with themed announcements.
A key feature is the **Anonymous DM Chat**, which allows participants to message their Secret Santa or their Giftee without revealing their identity.

This bot is not a generic trigger/response bot. Its sole purpose is running
the Ratmas Secret Santa exchange with maximum rat energy.

## Commands

Since users interact with Ratmas via emoji reactions and DMs, slash commands are reserved entirely for Admins to manage the event.

| Command | Description | Permission |
|---|---|---|
| `/ratmas init [channel]` | Starts a new Ratmas event. The bot posts a themed signup message in the specified channel (or current channel) and adds a 🐀 reaction. | Admin Role |
| `/ratmas status` | Shows how many users have signed up so far (counts reactions on the active message). | Admin Role |
| `/ratmas assign` | Closes sign-ups, removes the bot's reaction to stop new signups, performs the Secret Santa randomization, and sends a DM to each participant with their recipient. | Admin Role |
| `/ratmas cancel` | Cancels the active event (useful if started by mistake). | Admin Role |

## Anonymous DM Workflow

Participants can chat anonymously with their Secret Santa or their Giftee directly through the bot's DMs.
To solve the ambiguity of whether a user is trying to reply to their Secret Santa or send a message to their assigned Giftee, the bot uses an interactive prompt flow:

1. User **A** sends a plain text DM to the bot: `"Did you want the blue one or the red one?"`
2. The bot replies to **A** with a message containing two buttons:
   - `[ Send to your Giftee ]`
   - `[ Send to your Secret Santa ]`
3. If **A** clicks `[ Send to your Giftee ]`, user **B** receives a DM:
   `"🐀 Message from your Secret Santa: Did you want the blue one or the red one?"`
   *(Plus a small footer explaining they can reply by just messaging the bot).*
4. If user **B** wants to reply to **A**, they send a DM to the bot. The bot prompts **B** with the same two buttons. **B** clicks `[ Send to your Secret Santa ]`, and **A** receives:
   `"🐀 Message from your Giftee: Blue please!"`

This ensures that the "response" flow doesn't accidentally break the typical use case, as the user always explicitly chooses the destination for every message.

## Ratmas Workflow / Assignment Logic

1. **Sign-up Phase:** Admin runs `/ratmas init`. Guild members opt-in by reacting to the initialization message with the 🐀 emoji.
2. **Assignment Phase:** Admin runs `/ratmas assign`. The bot randomly assigns participants. It uses a derangement algorithm (or cycle) to ensure no self-assignments.
3. **Notification Phase:** The bot DMs each user their target.
4. **Active Event Phase:** The Anonymous DM Workflow is active until the event is explicitly closed/re-initialized for the next year.

## Architecture & Database

**Entry point:** `crates/ratbot/src/main.rs` → `crates/ratbot/src/lib.rs::run()`
**Framework:** `starbunk::run_bot` + `src/shared/discord::MessageService`

Since Postgres is standard for the workspace (`sqlx`), state is persisted across restarts using two simple tables:

**`ratmas_events`**
- `id` (UUID, primary key)
- `guild_id` (BIGINT)
- `signup_message_id` (BIGINT) — To listen for reaction events.
- `signup_channel_id` (BIGINT)
- `status` (VARCHAR) — `open`, `closed`, or `cancelled`.
- `created_at` (TIMESTAMPTZ)

**`ratmas_assignments`**
- `event_id` (UUID, foreign key)
- `gifter_user_id` (BIGINT)
- `recipient_user_id` (BIGINT)

## Dependencies & Configuration

- **Gateway intents:** `GUILD_MESSAGES | MESSAGE_CONTENT | GUILD_MESSAGE_REACTIONS | DIRECT_MESSAGES`

**Environment Variables:**
| Env Var | Purpose |
|---|---|
| `DATABASE_URL` | Postgres database connection string (for `sqlx`). |
| `RATMAS_ADMIN_ROLE_ID` | Restricts the `/ratmas` slash commands. |
| `RATMAS_ANNOUNCEMENT_CHANNEL_ID` | *(Optional)* Default channel for the bot to post initialization/announcements. |

## Edge Cases

- **Self-Assignment:** A participant must never be assigned themselves.
- **Odd Participant Counts:** Handled gracefully by creating a single large cycle of all participants, guaranteeing everyone gives one and gets one.
- **Duplicate Sign-ups:** Deduplicated based on `user_id`.
- **Bot Interactions:** The bot ignores other bots attempting to sign up.
- **DM Failures:** If a user has DMs disabled, the bot should log a warning or notify the admin running the assign command.

## Observability & Metrics

RatBot integrates with the OpenTelemetry and tracing pipeline. Logs and metrics are fully structured:

- **Logs**: Delineated by process with `bot = "ratbot"`, `guild = %guild_id`, `user_id = %user_id`. Major lifecycle actions (init, assign), DM routes, and errors are logged at the `INFO` or `ERROR` level.
- **Metrics**: Standard `bot.errors` (counter) and potentially `ratmas.signups.total` or `ratmas.messages.routed`.

## Testing

- Unit tests for the randomization logic / derangement algorithm should live in `crates/ratbot/src/`.
- TDD workflows must ensure the assignment logic and the anonymous DM routing (sender -> receiver mapping) are thoroughly tested before Discord API integration.

## See Also

- [[../infrastructure/Architecture|Architecture]]
- [[BlueBot|BlueBot]] — another bot in the same monorepo
