# BunkBot Rust Port — Implementation Plan

> YAML-driven reply bot engine. 19 production bots. 7 tickets, each following mandatory TDD
> (Tests-only PR → Implementation PR).

## Reference Material

| Source | Path |
|---|---|
| Production config | `/mnt/nfs/appdata/starbunk-js/config/bunkbot/bots.yml` |
| JS framework | `../starbunk-js/src/bunkbot/` |
| Go reference | `../starbunk-go/cmd/bunkbot/` |
| E2E suites | `crates/e2e/suites/` |

---

## Architecture Decisions

| Decision | Choice | Reason |
|---|---|---|
| Config loading | YAML-driven at runtime | Production configs are git-ignored, deployed separately; matches JS model |
| BlueBot | Stays compiled | Simple single-strategy bot, no need for YAML overhead |
| Config parsing | `serde_yaml` with strongly typed structs | Fails loudly at startup, not silently mid-message (improvement over JS/Zod) |
| State persistence | SQLite via `sqlx` | JS state resets on restart — this is a known deficiency to fix |
| Regex caching | `HashMap<String, Regex>` built at load time | YAML regexes compiled once when config loads, not per message |
| URL stripping | Preprocessing step before all condition eval | Prevents `https://blue.com/...` from triggering on the word "blue" |

---

## YAML Schema (Canonical)

```yaml
reply-bots:
  - name: example-bot

    # Identity: static | mimic | random
    identity:
      type: static
      bot_name: ExampleBot          # required for static
      avatar_url: https://...       # required for static
    # identity:
    #   type: mimic
    #   as_member: "113035990725066752"   # Discord user ID (string)
    # identity:
    #   type: random

    # Optional: bot-level response pool (used when trigger has none)
    responses:
      - "Response one"
      - "Response two"

    # Defaults
    ignore_bots: true       # default: true
    ignore_humans: false    # default: false

    triggers:
      - name: my-trigger    # optional, used in logs

        # --- Leaf conditions ---
        conditions:
          contains_phrase: "banana"
          # contains_word: "banana"             # word-boundary match
          # matches_regex: "\\bshe{2,}sh\\b"
          # from_user: "113035990725066752"
          # with_chance: 15                     # N% chance on any matching message
          # always: true                        # unconditional

          # --- Compound conditions ---
          # all_of:
          #   - contains_phrase: "guy"
          #   - with_chance: 10
          # any_of:
          #   - contains_word: "hello"
          #   - contains_word: "hi"
          # none_of:
          #   - from_user: "123456789"

        # Optional: overrides bot-level response pool for this trigger
        responses: "Always bring a :banana: to a party!"
```

### Template Placeholders (in responses)

| Placeholder | Example | Output |
|---|---|---|
| `{start}` | `"{start}-- sorry, go ahead"` | `***Hello world...***-- sorry, go ahead` |
| `{random:min-max:char}` | `"sh{random:2-20:e}sh"` | `sheeeeeeesh` |
| `{swap_message:w1:w2}` | `"{swap_message:check:czech}"` | Original message with words swapped |

### Backward Compatibility

The `always: true` + `with_chance` pattern from JS is redundant but must parse correctly:
```yaml
# JS idiom (redundant but valid)
conditions:
  all_of:
    - always: true
    - with_chance: 1

# Preferred Rust form (same semantics)
conditions:
  with_chance: 1
```

---

## Production Bots (19)

All must work on day one of T-7.

| Bot | Identity | Primary trigger | Templates |
|---|---|---|---|
| botbot | static | 1% chance, bots only | — |
| guy-bot | mimic (Guy `113035990725066752`) | phrase "guy" OR 10% from Guy | `{random}` |
| clanker-bot | static (HK-47) | phrase "clanker" | — |
| spider-bot | static | regex `spider(?!-).*man` | — |
| nice-bot | static | regex `\b(69\|sixty-?nine)\b` | — |
| banana-bot | static | phrase "banana" | — |
| sheesh-bot | static | regex `\bshe{2,}sh\b` | `{random:2-20:e}` |
| pickle-bot | static (GremlinBot) | phrase "gremlin" | — |
| hold-bot | static | regex `^Hold\.?$` | — |
| attitude-bot | static (Xander Crews) | regex `(you\|I\|they\|we) can'?t` | — |
| baby-bot | static | phrase "baby" | — |
| chaos-bot | static | phrase "chaos" | — |
| gundam-bot | static | regex `\bg(u\|a)ndam\b` | — |
| interrupt-bot | static | 1% any message | `{start}` |
| venn-bot | mimic (Venn `151120340343455744`) | phrase "cringe" | — |
| check-bot | static | regex `\bcheck\b` / `\bczech\b` (2 triggers) | — |
| chad-bot | mimic (Chad `85184539906809856`) | 1% any message | — |
| ezio-bot | static | regex `\b(ezio\|assassin)\b` | — |
| homonym-bot | static (Gerald) | 3 triggers, 15% each | — |

---

## Ticket Breakdown

### T-1 — Config Schema & YAML Loader

**Goal:** Rust types for the full YAML config + a directory loader that fails loudly on bad input.

**New files:**
- `crates/bunkbot/src/config.rs` — all serde types
- `crates/bunkbot/src/loader.rs` — directory scanner + parser

**Structs/enums to define:**

```rust
// crates/bunkbot/src/config.rs

pub struct BotConfig {
    pub name: String,
    pub identity: IdentityConfig,
    pub responses: Vec<String>,        // bot-level pool
    pub ignore_bots: bool,             // default: true
    pub ignore_humans: bool,           // default: false
    pub triggers: Vec<TriggerConfig>,
}

pub enum IdentityConfig {
    Static { bot_name: String, avatar_url: String },
    Mimic { as_member: String },       // Discord user ID
    Random,
}

pub struct TriggerConfig {
    pub name: Option<String>,
    pub conditions: ConditionNode,
    pub responses: Vec<String>,        // trigger-level pool (overrides bot-level)
}

pub enum ConditionNode {
    // Leaf
    ContainsPhrase(String),
    ContainsWord(String),
    MatchesRegex(String),
    FromUser(String),
    WithChance(u8),                    // 0-100
    Always,
    // Compound
    AllOf(Vec<ConditionNode>),
    AnyOf(Vec<ConditionNode>),
    NoneOf(Vec<ConditionNode>),
}
```

**Tests (PR 1):**
- Parse all 3 identity types
- Parse all 6 leaf condition types
- Parse all 3 compound condition types, including nested
- Parse bot-level response pool (single string and array)
- Parse trigger-level response pool
- Parse `ignore_bots` / `ignore_humans` defaults
- Loader: reads multiple `.yml` files from a directory
- Loader: returns clear error on invalid YAML (doesn't silently skip)
- Loader: parses actual `bots.yml` from production without error (snapshot test)
- Backward compat: `always: true` + `with_chance` inside `all_of` parses correctly

---

### T-2 — Strategy Builder & URL Preprocessor

**Goal:** Convert `BotConfig` → `Box<dyn Strategy>` using existing middleware. Wire into `lib.rs`.

**New files:**
- `crates/bunkbot/src/strategy_builder.rs`
- `crates/bunkbot/src/preprocessor.rs`

**Key logic:**
- `ConditionNode` → `Arc<dyn MessageFilter>` using existing `starbunk-shared` middleware
- URL stripping: strip `https?://\S+` from message content before any condition evaluation
- `build_strategy(config: &BotConfig, state: Arc<dyn BotStateStore>) -> Box<dyn Strategy>`
- `lib.rs`: load YAML dir on `ready`, pass strategies to `ReplyBot::new()`

**Condition → middleware mapping:**

| ConditionNode | Middleware |
|---|---|
| `ContainsPhrase(s)` | `content_contains(s)` |
| `ContainsWord(s)` | `content_matches(format!("\\b{}\\b", s))` |
| `MatchesRegex(r)` | `content_matches(r)` |
| `FromUser(id)` | `author_id(id)` |
| `WithChance(n)` | `chance(n as f64 / 100.0)` |
| `Always` | no filter (pass-through) |
| `AllOf(nodes)` | `all_of(nodes.map(build_filter))` |
| `AnyOf(nodes)` | `any_of(nodes.map(build_filter))` |
| `NoneOf(nodes)` | `not(any_of(nodes.map(build_filter)))` |

**Tests (PR 1):**
- `contains_phrase` triggers on matching message, not on unrelated
- `matches_regex` triggers correctly
- `from_user` triggers only for correct user ID
- `with_chance: 100` always fires; `with_chance: 0` never fires
- `all_of`: requires all conditions (short-circuits)
- `any_of`: fires on first matching condition
- `none_of`: fires only when none match
- URL in message does not trigger phrase match on URL content
- First matching trigger wins; remaining triggers skipped
- Bot-level response pool used when trigger has no responses
- Trigger-level response pool overrides bot-level
- `ignore_bots: true` (default): bot messages skipped
- `ignore_humans: true` (botbot): human messages skipped
- `ignore_bots: false` + `ignore_humans: true` (botbot): only bot messages pass

---

### T-3 — Identity Resolution

**Goal:** All 3 identity types resolve to a webhook `Identity`, with caching to avoid rate limits.

**New files:**
- `crates/bunkbot/src/identity.rs`

**Trait:**
```rust
#[async_trait]
pub trait IdentityResolver: Send + Sync {
    async fn resolve(&self, ctx: &Context, msg: &Message) -> Option<Identity>;
}
```

**Implementations:**
- `StaticResolver`: returns fixed `Identity { name, avatar_url }` — no network call
- `MimicResolver`: fetches Discord user by ID via `ctx.http`, caches result in `DashMap<UserId, Identity>` (profiles rarely change — cache forever per session)
- `RandomResolver`: fetches guild members via `ctx.http`, caches list with 5-minute TTL, samples one per trigger

**Tests (PR 1):**
- Static resolver returns correct name and avatar_url
- Mimic resolver maps Discord user to Identity (mock `ctx.http`)
- Mimic resolver caches: second call does not re-fetch
- Random resolver samples from guild member list (mock guild)
- Random resolver returns a valid member every call
- Random cache invalidates after TTL

---

### T-4 — Response Templates

**Goal:** Template placeholder resolution in response strings.

**New files:**
- `crates/starbunk-shared/src/template.rs` (shared utility — other bots may use this)

**Function signature:**
```rust
pub fn resolve_template(template: &str, msg: &Message) -> String
```

**Placeholder specs:**
- `{start}`: split message by whitespace, take words until cumulative length > 15 chars, join with space, append `...` if truncated, wrap in `***...***`
- `{random:min-max:char}`: parse min/max as `usize`, repeat `char` N times where N ∈ [min, max], cap at 1000
- `{swap_message:word1:word2}`: case-insensitive replace of word1→word2 and word2→word1 in original message, preserve surrounding case

**Tests (PR 1):**
- `{start}` with message shorter than 15 chars (no truncation, no `...`)
- `{start}` with message longer than 15 chars (truncated with `...`)
- `{random:2-5:e}` output length always in [2, 5]
- `{random:1-1:x}` always returns `"x"`
- `{random:3-3:Mister }` repeats multi-char string exactly 3 times
- `{swap_message:check:czech}` swaps both directions in one message
- `{swap_message}` handles case variants (Check → Czech, CHECK → CZECH)
- Response with no placeholder returns original string unchanged
- Response with multiple placeholders resolves all

---

### T-5 — Persistent State Store

**Goal:** Bot enable/disable, frequency overrides, and comment overrides survive container restarts.

**New files:**
- `crates/bunkbot/src/state.rs`

**Trait:**
```rust
#[async_trait]
pub trait BotStateStore: Send + Sync {
    async fn is_enabled(&self, bot_name: &str) -> bool;
    async fn set_enabled(&self, bot_name: &str, enabled: bool) -> Result<()>;
    async fn get_frequency(&self, bot_name: &str) -> Option<u8>;   // None = use YAML default
    async fn set_frequency(&self, bot_name: &str, pct: u8) -> Result<()>;
    async fn reset_frequency(&self, bot_name: &str) -> Result<()>;
    async fn get_comments(&self, bot_name: &str) -> Vec<String>;
    async fn set_comments(&self, bot_name: &str, comments: Vec<String>) -> Result<()>;
    async fn clear_comments(&self, bot_name: &str) -> Result<()>;
    async fn list_bots(&self) -> Vec<BotState>;
}
```

**Implementation:** SQLite via `sqlx`. DB file at path from env `BUNKBOT_STATE_DB`
(default: `data/bunkbot_state.db`).

**Tests (PR 1):**
- Bot is enabled by default
- Disable persists: recreating store from same DB returns disabled state
- Frequency override applied to `WithChance` condition evaluation
- `reset_frequency` returns `None` (strategy uses YAML default)
- Comment override replaces response pool for that bot
- Clear comments: pool reverts to YAML responses
- Multiple bots maintain independent state
- `list_bots` returns all known bots with current state

---

### T-6 — Slash Commands

**Goal:** Full admin interface matching JS bunkbot, backed by T-5 state store.

**New files:**
- `crates/bunkbot/src/commands.rs` (module root)
- `crates/bunkbot/src/commands/ping.rs`
- `crates/bunkbot/src/commands/bot.rs`
- `crates/bunkbot/src/commands/comments.rs`
- `crates/bunkbot/src/commands/webhooks.rs`

**Commands:**

| Command | Subcommand | Permission | Description |
|---|---|---|---|
| `/ping` | — | none | Responds "Pong." |
| `/bot` | `list` | Admin | All bots, enabled/disabled, freq overrides |
| `/bot` | `enable <name>` | Admin | Enable a disabled bot |
| `/bot` | `disable <name>` | Admin | Disable a bot |
| `/bot` | `override frequency <name> <pct>` | Admin | 0–100% override |
| `/bot` | `reset frequency <name>` | Admin | Back to YAML default |
| `/comments` | `get <name>` | Admin | Current comment pool |
| `/comments` | `set <name> <text>` | Admin | Set pool (`\|` or newline separator) |
| `/comments` | `append <name> <text>` | Admin | Add to pool |
| `/comments` | `clear <name>` | Admin | Clear pool |
| `/comments` | `list` | Admin | All bots + comment counts |
| `/clearwebhooks` | — | Manage Webhooks | Delete all "Starbunk Bot" webhooks |

All `<name>` arguments have autocomplete backed by the loaded bot registry.

**Tests (PR 1):**
- `/ping` responds "Pong."
- `/bot enable` and `/bot disable` round-trip via state store
- `/bot list` reflects current state
- `/bot override frequency` clamps to [0, 100]
- `/comments set` splits on `|` and newline
- `/comments append` adds to existing pool
- `/comments clear` returns to YAML pool
- Admin commands rejected for non-admin users
- Autocomplete returns all registered bot names

---

### T-7 — E2E Test Suite

**Goal:** End-to-end coverage for all 19 production bots.

**New file:** `crates/e2e/suites/bunkbot.json`

**Test cases:**

| Bot | Case | Expect |
|---|---|---|
| botbot | bot sends any message | response |
| botbot | human sends any message | no response |
| guy-bot | human says "hey guy what's up" | response (mimic Guy) |
| guy-bot | unrelated message from Guy, chance forced 100% | response |
| clanker-bot | "you clanker" | response |
| spider-bot | "spiderman is cool" | hyphen correction |
| spider-bot | "Spider-Man is cool" | no response |
| nice-bot | "haha 69" | "Nice." |
| banana-bot | "I like banana" | response |
| sheesh-bot | "sheeeesh" | response matches `sh[e]{2,}sh` |
| pickle-bot | "stop being a gremlin" | response |
| hold-bot | "Hold" | "Hold." |
| hold-bot | "Hold on a second" | no response |
| attitude-bot | "we can't do that" | attitude response |
| baby-bot | "baby steps" | GIF URL response |
| chaos-bot | "pure chaos" | response |
| gundam-bot | "I love gundam" | Gandum response |
| gundam-bot | "I love gandam" | Gandum response |
| interrupt-bot | any message, chance forced 100% | response contains message excerpt |
| venn-bot | "that's so cringe" | response (mimic Venn) |
| check-bot | "check the list" | czech correction |
| check-bot | "czech republic" | check correction |
| chad-bot | any message, chance forced 100% | response (mimic Chad) |
| ezio-bot | "ezio is cool" | assassin quote |
| ezio-bot | "assassin's creed" | assassin quote |
| homonym-bot | "their going" chance forced 100% | there correction |
| homonym-bot | "your welcome" chance forced 100% | you're correction |
| homonym-bot | "I want to go" chance forced 100% | too correction |

---

## Execution Order

```
T-1 (Config Schema)           ← start here
  └── T-2 (Strategy Builder)  ← unblocks everything
        ├── T-3 (Identity)    ← parallel with T-4
        ├── T-4 (Templates)   ← parallel with T-3
        └── T-7 (E2E)         ← final validation gate

T-5 (State Store)             ← parallel with T-2
  └── T-6 (Slash Commands)   ← depends on T-5
```

**Sprint sequence:**
1. T-1 tests → T-1 impl
2. T-2 tests → T-2 impl + T-5 tests (parallel)
3. T-3 tests + T-4 tests + T-5 impl (parallel)
4. T-3 impl + T-4 impl → T-6 tests
5. T-6 impl → T-7

---

## Definition of Done

- [ ] All 19 production bots load and respond correctly
- [ ] `cargo test` passes across workspace
- [ ] `bash scripts/devops-validate.sh` exits cleanly
- [ ] E2E suite (`bunkbot.json`) passes
- [ ] State persists across container restart
- [ ] Slash commands work with autocomplete
- [ ] Wiki updated (`wiki/bots/BunkBot.md`)
- [ ] Changelog entry added
