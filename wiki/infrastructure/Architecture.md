# Architecture

## Overview

Starbunk-Rs is a Rust monorepo. Each bot is an independent binary with its own
Docker container and Discord token. The runtime is async/await with the Tokio
executor; Discord is handled by the Serenity framework.

```
starbunk-rs/
  src/
    bin/
      bluebot.rs    # binary entry point
      bunkbot.rs
      covabot.rs
      djcova.rs
      ratbot.rs
    bots/           # per-bot implementation modules
      bluebot/
      bunkbot/
      covabot/
      djcova/
      ratbot/
      mod.rs
    shared/         # shared libraries
      discord/      # Identity, MessageService, WebhookService
      llm/          # LLM abstraction + multi-provider clients
      memory/       # Semantic memory with pgvector
      middleware/   # MessageFilter composable gates
      replybot/     # Reply bot dispatcher (Strategy pattern)
      mod.rs
    lib.rs          # run_bot(), default_intents(), re-exports
    main.rs         # stub — use specific bot binaries
  docker/
    Dockerfile              # single multi-stage build; BOT_NAME arg selects binary
    docker-compose.yml      # local dev — builds from source
  docker-compose.yml        # production — pulls GHCR images
  .github/workflows/
    ci.yml      # PR checks
    main.yml    # build + push images on merge
    deploy.yml  # deploy to Tower server
```

## Shared Libraries (`src/shared/`)

### `src/lib.rs`

- `run_bot(name, intents, handler)` — reads `DISCORD_TOKEN`, creates a Serenity
  client with the provided handler, starts the gateway, blocks until shutdown.
- `default_intents()` — returns `GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT`.

### `src/shared/discord`

Three single-responsibility layers:

- **`Identity` / `IdentityProvider`** — first-class persona concept.
  `Identity { username, nickname, avatar_url }` belongs to any message send.
  `DiscordIdentityProvider` resolves live Discord identities, preferring guild
  member details over global user details.

- **`WebhookService`** — internal implementation detail; callers never use it
  directly. Manages the full lifecycle of per-channel webhooks:
  - Lazily creates a webhook named `"Starbunk Webhook"` on first use (found by
    name — all bots share one slot per channel, within Discord's 15/channel limit).
  - Caches entries in a `HashMap<channel_id, {webhook, last_used}>` registry.
  - Background reaper (every 1 minute) deletes webhooks idle longer than 5 minutes.
  - `close()` stops the reaper and deletes all owned webhooks.

- **`MessageService`** — the only caller-facing send API. Callers say *what* to
  send and *as whom*; the implementation decides how to deliver it.

  ```rust
  #[async_trait]
  pub trait MessageService: Send + Sync {
      // Bot's own identity — admin, errors, ephemeral messages
      async fn send_message(&self, channel_id: ChannelId, content: &str) -> Result<Message>;
      // Caller-provided identity — service decides transport (direct API vs webhook)
      async fn send_message_with_identity(&self, channel_id: ChannelId, content: &str, id: &Identity) -> Result<Message>;
      async fn reply(&self, channel_id: ChannelId, message_id: MessageId, content: &str) -> Result<Message>;
      async fn edit(&self, channel_id: ChannelId, message_id: MessageId, content: &str) -> Result<Message>;
      async fn delete(&self, channel_id: ChannelId, message_id: MessageId) -> Result<()>;
      async fn close(&self) -> Result<()>;
  }
  ```

### `src/shared/llm`

- `LlmService` trait — unified abstraction for bots to interact with Large Language Models.
- `Registry` trait — factory pattern for separating High, Medium, and Low capability tiers via `.env`.
- `TieredRegistry` — concrete implementation using env-configured providers.
- Multi-provider clients: Anthropic, Google, Ollama, OpenAI.

### `src/shared/memory`

- `MemoryService` trait — semantic memory system.
- Asynchronously uses Low-tier LLMs to extract facts from incoming messages.
- Uses `pgvector` inside PostgreSQL to store text alongside embedding vectors.
- `recall()` injects past facts into the bot's system prompt context.

### `src/shared/middleware`

Composable message filter gates. Every bot must supply a `MessageFilter` to
`run_bot`; no message can be processed without passing the filter.

**Trait**

```rust
pub trait MessageFilter: Send + Sync {
    fn check(&self, ctx: &Context, msg: &Message) -> bool;
}
```

**Primitives** (by file)

| File | Filters |
|---|---|
| `author.rs` | `NOT_SELF`, `NOT_BOT`, `IS_BOT`, `author_id(id)`, `author_named(name)`, `author_has_role(role_id)` |
| `content.rs` | `HAS_CONTENT`, `content_contains(substr)`, `content_matches(re)`, `HAS_ATTACHMENT` |
| `context.rs` | `GUILD_ONLY`, `DM_ONLY`, `in_channel(id)`, `on_weekdays(days...)` |
| `random.rs`  | `chance(p)` — passes with probability p |

**Combinators**

```rust
pub fn all_of(filters: Vec<Arc<dyn MessageFilter>>) -> Arc<dyn MessageFilter>
pub fn any_of(filters: Vec<Arc<dyn MessageFilter>>) -> Arc<dyn MessageFilter>
pub fn not(f: Arc<dyn MessageFilter>) -> Arc<dyn MessageFilter>
```

**Example composition**

```rust
// Bots always fail. Non-bots pass freely, except a specific user
// who only triggers when the message contains "bingo".
let filter = all_of(vec![
    NOT_BOT.clone(),
    any_of(vec![
        not(author_id("111111")),
        content_contains("bingo"),
    ]),
]);
```

See [[../development/MessageFiltering|Message Filtering]] for the full design.

## Bot Pattern

```rust
use starbunk::shared::middleware;

struct MyHandler {
    filter: Arc<dyn MessageFilter>,
    sender: Arc<dyn MessageService>,
}

#[async_trait]
impl EventHandler for MyHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !self.filter.check(&ctx, &msg) {
            return;
        }
        // Audit has already passed. Business logic here.
        self.sender.send_message(msg.channel_id, "response").await.ok();
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = middleware::all_of(vec![
        middleware::NOT_SELF.clone(),
        middleware::NOT_BOT.clone(),
        middleware::GUILD_ONLY.clone(),
        middleware::HAS_CONTENT.clone(),
    ]);
    let handler = MyHandler { filter, sender: ... };
    starbunk::run_bot("BotName", starbunk::default_intents(), handler).await
}
```

## Discord Intents

Default: `GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT`.
DJCova additionally needs `GatewayIntents::GUILD_VOICE_STATES`.

## See Also

- [[Deployment|Deployment]]
- [[Configuration|Configuration]]
