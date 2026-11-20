# ragtime

Discord trivia contest bot. Full docs: `wiki/bots/Ragtime.md`.

## Playing a round

| Step | How |
|---|---|
| Open a lobby | @mention the bot in a channel. You become the host. A second @mention while a round is running gets "already running". |
| Join | React with 🙋 on the lobby message. The host joins the same way (not automatic). Joining closes when the round starts. |
| Start | Host runs `/ragtime start`. Up to 20 random questions are loaded from the question bank (all of them if fewer). If nobody joined, the round is closed silently. |
| Answer | Click **Answer** under the question and type into the modal. The acknowledgement is private and never says whether you were right. Non-players get "not in this round"; a second answer is refused. |
| Reveal | When every player has answered, or the timer runs out, the bot posts the answer and mentions who got it right, then the next question. |
| Tally | After the last question the bot posts the ranked leaderboard (1 point per correct answer, ties share a rank). |
| Cancel | Host runs `/ragtime cancel` at any time. |

Non-host `/ragtime start` / `cancel` get an ephemeral "Only the host can do that."

## Timeouts

| Timer | Value | Effect |
|---|---|---|
| Question | 2 min | The question closes and is revealed |
| Lobby | 10 min after the mention | An unstarted lobby is closed with a short notice |
| Abandon | 6 min with no answers from anyone during a round | The round ends with a short notice; the timer resets on every answer |

Constants live in `src/round.rs` (`QUESTION_TIMEOUT`, `LOBBY_TIMEOUT`, `HOST_ABANDON_TIMEOUT`, `QUESTIONS_PER_ROUND`).

## Grading

Grading currently uses an interim case- and whitespace-insensitive exact match against the display answer and the acceptable answers (`src/bot/grading.rs`). The fuzzy `grade()` from card t_a1639804 replaces it once that lands.

## Running

Needs `DISCORD_TOKEN` and `DATABASE_URL`. Migrations run at startup. Required gateway intents: `GUILDS`, `GUILD_MESSAGES`, `MESSAGE_CONTENT`, `GUILD_MESSAGE_REACTIONS`. The bot registers the global `/ragtime` command on `ready`.

```bash
cargo test -p ragtime                     # unit + wiring tests (no Discord, no DB)
DATABASE_URL=postgres://… cargo test -p ragtime -- --ignored   # repository contract tests
```
