# StarBunk Bot Management UI

> **Location:** `starbunk-ui/` (Next.js app, worktree `feat-bot-management-frontend`)
> **Purpose:** Operational control plane — at-a-glance status, lifecycle management, and settings editing for all bots.

---

## Tech Stack

| Concern | Choice |
|---|---|
| Framework | Next.js 16 (App Router, React Server Components) |
| Styling | Tailwind CSS v4 + `globals.css` (glassmorphism design tokens) |
| Mutations | Next.js Server Actions (`"use server"`) — no separate API routes |
| Bot status | `fetch()` against each bot's existing HTTP API (no orchestrator client) |
| DB (audit log) | `pg` — lazy-loaded, only when `DATABASE_URL` is set |

---

## Running Locally

```bash
cd starbunk-ui
npm install
npm run dev -- --port 3002   # or any free port
```

Open `http://localhost:3002`. No running bots are required — every action degrades gracefully (status shows `Unknown`).

**Environment variables (all optional):**

| Variable | Default | Effect |
|---|---|---|
| `DATABASE_URL` | *(unset)* | Postgres URL for audit history — omit to skip DB |
| `BUNKBOT_API_URL` | `http://localhost:9082` | BunkBot internal HTTP API |
| `COVABOT_API_URL` | `http://localhost:9083` | CovaBot internal HTTP API |
| `DJCOVA_API_URL` | `http://localhost:9084` | DJCova internal HTTP API |

---

## Data Sources

The UI talks only to the bots' own HTTP APIs, the local filesystem, and
(optionally) Postgres. It has no orchestrator client and no Docker socket —
lifecycle (start/stop/restart) belongs to the Compose stack on Tower
(Portainer), not the UI.

**Bot status** (`src/app/actions.ts` → `getBotStatuses()`): probes each bot's
existing HTTP endpoint with a 2s timeout:

| Bot | Probe | Result |
|---|---|---|
| `bunkbot` | `${BUNKBOT_API_URL}/health` | 200 → `Running`, 503 → `Starting` |
| `covabot` | `${COVABOT_API_URL}/api/personality` | 2xx → `Running` |
| `djcova` | `${DJCOVA_API_URL}/state` | 2xx → `Running` |
| `bluebot`, `ratbot` | none | always `Unknown` |

Any other HTTP status is `Down`; a network error or timeout is `Unknown`.
bluebot and ratbot only run the shared health server, which binds
`127.0.0.1` inside their container and is not reachable from the UI.

**Config files** (`getBotConfigs` / `updateBotConfig`): read and write YAML
under `../config/<bot>/` relative to the UI's working directory.

**Bot HTTP APIs** (`djcova/actions.ts`, `bunkbot/actions.ts`, `covabot/personalities/actions.ts`): each wraps `fetch()` in a try/catch and returns `null` / empty on network error. Pages render gracefully when bots are not running.

**Audit DB** (`history/actions.ts`): pool is lazy-initialized only when `DATABASE_URL` is set. The history page shows a "set DATABASE_URL" hint when the env var is absent.

---

## Route Map

| Route | Component | Data source |
|---|---|---|
| `/` | Dashboard | `getBotStatuses()` — HTTP probe per bot (all 5 bots) |
| `/api/sse` | Real-time SSE Stream | EventSource streaming live telemetry & trigger audits |
| `/history` | Audit Log | Postgres `bot_audit_history` table or local mock |
| `/covabot` | CovaBot landing | Model Tier Routing Matrix, Personality Studio & config files |
| `/covabot/personalities` | Persona editor | CovaBot HTTP API (`/config/profiles/…`) |
| `/bunkbot` | BunkBot Magnum Opus | Global & per-bot limiters, "+ Add Bot" (JSON/TOML/YAML loader), SubBot cards |
| `/bunkbot/strategies` | Strategy editor | BunkBot HTTP API (`/config`) |
| `/djcova` | DJCova landing | Streaming Music HUD, voice channel stats & live queue |
| `/djcova/controls` | Queue / skip / kick | DJCova HTTP API, polled every 10s |
| `/bluebot` | BlueBot landing | Pattern inspector, regex matcher coverage & SSE response audit stream |
| `/ratbot` | RatBot landing | Automated (Cron) vs Manual/Ad-hoc mode toggle & Secret Santa control dashboard |

---

## Server Actions

### `src/app/actions.ts` — Infrastructure layer

| Export | What it does |
|---|---|
| `getBotStatuses()` | Returns `Running` / `Starting` / `Down` / `Unknown` for all 5 bots via HTTP probes |
| `getBotConfigs(botName)` | Reads YAML configs from the local config directory |
| `updateBotConfig(botName, file, content)` | Writes/deletes a config file; `content=null` deletes |

### `src/app/djcova/actions.ts`

`getDjcovaState()` · `skipTrack(guildId)` · `kickBot(guildId)`

### `src/app/covabot/personalities/actions.ts`

`listCovaBotProfiles()` · `getCovaBotProfile(id)` · `saveCovaBotProfile(id, yaml)`

### `src/app/bunkbot/actions.ts`

`getBunkBotConfig()` · `saveBunkBotConfig(yaml)`

### `src/app/history/actions.ts`

`getHistory(botName?, limit?)` — queries `bot_audit_history` via `pg`. No-ops silently (returns `[]`) when `DATABASE_URL` is unset.

---

## Key Components

### `src/components/ConfigManager.tsx`

Reusable YAML file manager used on `/covabot` and `/bunkbot`. Props: `configs: Record<string, string>` (filename → content) and `botName`. Supports selecting, editing, creating, and deleting YAML files.

### `src/app/page.tsx` (Dashboard)

Client component. Polls `getBotStatuses()` every 5 seconds. Read-only: there are no start/stop/restart buttons.

---

## Known Gaps / Next Steps

- `/bluebot` — static stub; no dynamic config surface
- `/history` — requires a Postgres connection; no local mock data
- DJCova landing page (`/djcova`) — currently a static server-rendered snapshot; refresh strategy TBD (see `TODO(human)` in `djcova/page.tsx`)
