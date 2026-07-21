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
| K8s client | `@kubernetes/client-node` v1.4 (object-param API) |
| DB (audit log) | `pg` — lazy-loaded, only when `DATABASE_URL` is set |

---

## Running Locally

```bash
cd starbunk-ui
npm install
npm run dev -- --port 3002   # or any free port
```

Open `http://localhost:3002`. No Kubernetes or running bots are required — every action has a local fallback.

**Environment variables (all optional):**

| Variable | Default | Effect |
|---|---|---|
| `K8S_NAMESPACE` | `starbunk` | K8s namespace to read/patch |
| `DATABASE_URL` | *(unset)* | Postgres URL for audit history — omit to skip DB |
| `BUNKBOT_API_URL` | `http://localhost:9082` | BunkBot internal HTTP API |
| `COVABOT_API_URL` | `http://localhost:9083` | CovaBot internal HTTP API |
| `DJCOVA_API_URL` | `http://localhost:9084` | DJCova internal HTTP API |

---

## Dual-Environment Architecture

Every server action has two code paths selected at module init time:

```
K8s available?  ──yes──▶  Kubernetes API  (read/patch Deployments & ConfigMaps)
                ──no───▶  Local fallback  (mock state / local filesystem)
```

**K8s client** (`src/app/actions.ts`): initialised once at module load by calling `kc.loadFromDefault()`. If that throws (no kubeconfig), `k8sAppsApi` and `k8sCoreApi` are `null` and every action falls back to local mode.

**Bot HTTP APIs** (`djcova/actions.ts`, `bunkbot/actions.ts`, `covabot/personalities/actions.ts`): each wraps `fetch()` in a try/catch and returns `null` / empty on network error. Pages render gracefully when bots are not running.

**Audit DB** (`history/actions.ts`): pool is lazy-initialized only when `DATABASE_URL` is set. The history page shows a "set DATABASE_URL" hint when the env var is absent.

---

## Route Map

| Route | Component | Data source |
|---|---|---|
| `/` | Dashboard | `getBotDeployments()` — K8s or mock (all 5 bots) |
| `/api/sse` | Real-time SSE Stream | EventSource streaming live telemetry & trigger audits |
| `/history` | Audit Log | Postgres `bot_audit_history` table or local mock |
| `/covabot` | CovaBot landing | Model Tier Routing Matrix, Personality Studio & ConfigMap |
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
| `getBotDeployments()` | Returns deployment status for all 4 bots |
| `setBotState(name, action)` | Starts, stops, or rolling-restarts a bot via K8s patch |
| `getBotConfigs(botName)` | Reads YAML configs from K8s ConfigMap or local FS |
| `updateBotConfig(botName, file, content)` | Writes/deletes a config entry; `content=null` deletes |

**K8s API note (v1.4):** All calls use the object-param form: `api.readNamespacedDeployment({ name, namespace })`. The response is the resource directly — no `.body` wrapper. The `restart` action patches `kubectl.kubernetes.io/restartedAt` and falls back from `replace` to `add` if the annotation path doesn't exist yet.

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

Client component. Polls `getBotDeployments()` every 5 seconds. Uses `useTransition` so lifecycle button clicks are non-blocking.

---

## Known Gaps / Next Steps

- `/bluebot` — static stub; no dynamic config surface
- `/history` — requires a Postgres connection; no local mock data
- DJCova landing page (`/djcova`) — currently a static server-rendered snapshot; refresh strategy TBD (see `TODO(human)` in `djcova/page.tsx`)
