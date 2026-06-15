# Observability

starbunk-rs uses the [OpenTelemetry](https://opentelemetry.io/) standard for all
telemetry. Every bot ships logs, traces, and metrics through a single OTLP gRPC
pipeline to a bundled observability stack (Loki / Tempo / Prometheus / Grafana).

---

## Architecture

```
Bot process
  └─ tracing crate (spans + log events)
       ├─ tracing_subscriber::fmt  → stdout (human-readable console)
       ├─ tracing_opentelemetry    → OTLP gRPC → otel-collector:4317
       │                                  ├─ logs   → Loki:3100
       │                                  ├─ traces → Tempo:4317
       │                                  └─ metrics→ Prometheus:9090
       └─ opentelemetry_appender_tracing  (bridges log events to OTEL logs)
```

All telemetry is initialised by a single call in each bot's `main.rs`:

```rust
let _guard = starbunk::telemetry::init("bluebot");
```

The `_guard` must be held for the full lifetime of the process — dropping it
triggers a graceful flush of all in-flight telemetry.

**Never** call `tracing_subscriber::fmt::init()` directly. It bypasses the
entire OTEL pipeline.

---

## Docker stack

The production and dev compose files both include the full LGTM stack:

| Container | Image | Port | Purpose |
|---|---|---|---|
| `otel-collector` | `otel/opentelemetry-collector-contrib` | 4317/4318 | Receives OTLP, fans out |
| `loki` | `grafana/loki` | 3100 | Log aggregation |
| `tempo` | `grafana/tempo` | 3200 | Distributed tracing |
| `prometheus` | `prom/prometheus` | 9090 | Metrics storage |
| `grafana` | `grafana/grafana` | 3000 | Dashboards |

Configuration lives in `observability/`:

```
observability/
  otel-collector.yaml          # Collector pipelines
  loki.yaml                    # Loki single-process config
  tempo.yaml                   # Tempo config + metrics generator
  prometheus.yml               # Prometheus scrape config
  grafana/
    provisioning/
      datasources/datasources.yaml   # Auto-wired Loki + Tempo + Prometheus
      dashboards/dashboards.yaml     # Dashboard folder provider
```

Grafana datasources are auto-provisioned on startup with cross-links:
- Logs → trace IDs link to Tempo
- Traces → links back to Loki logs by service name
- Traces → service graph from Prometheus metrics

---

## Environment variables

| Variable | Default | Effect |
|---|---|---|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `http://otel-collector:4317` | OTLP gRPC endpoint |
| `RUST_LOG` | `info` | tracing filter (e.g. `debug,serenity=warn`) |
| `VERBOSE` | `false` | Enable verbose mode — see below |

---

## Verbose mode

Set `VERBOSE=1` (or `VERBOSE=true`) on any bot to activate:

- `DEBUG`-level default log level (overrideable with `RUST_LOG`)
- Thread IDs and names in console output
- Source file and line numbers in console output
- Span `NEW`/`CLOSE` events in console output

Example (local dev):
```sh
VERBOSE=1 RUST_LOG=debug,serenity=info cargo run --bin covabot
```

Example (per-service in compose):
```yaml
environment:
  - VERBOSE=1
  - RUST_LOG=debug,serenity=warn,sqlx=warn
```

---

## Logging standards

Use the `tracing` macros. **Never** use `println!`, `eprintln!`, or the `log` crate.

### Level guide

| Level | When to use |
|---|---|
| `error!` | Unrecoverable or external failure (Discord send, DB, LLM) |
| `warn!` | Degraded behaviour — bot continues but something is wrong |
| `info!` | Lifecycle events: startup, ready, shutdown, successful operations |
| `debug!` | Internal decisions — gated to VERBOSE / debug builds only |
| `trace!` | Per-message, per-loop iteration — extremely chatty |

### Always use structured fields

Fields are indexed in Loki and become filterable labels in Grafana.

```rust
// ✓ Good — fields are searchable
tracing::info!(bot = "bluebot", channel = %msg.channel_id, "message received");
tracing::error!(strategy = strategy.name(), err = %e, "send failed");
tracing::warn!(provider = "anthropic", "LLM tier unavailable, falling back");

// ✗ Bad — unstructured string
tracing::info!("bluebot got a message in channel {}", msg.channel_id);
```

Use `%` for `Display` (user-facing strings), `?` for `Debug` (internal types).

### Standard fields

Always include these fields where applicable:

| Field | Type | Example |
|---|---|---|
| `bot` | `&str` | `bot = "bluebot"` |
| `channel` | `%ChannelId` | `channel = %msg.channel_id` |
| `guild` | `%GuildId` | `guild = %guild_id` |
| `strategy` | `&str` | `strategy = strategy.name()` |
| `provider` | `&str` | `provider = "anthropic"` |
| `err` | `%Error` | `err = %e` |

---

## Span instrumentation

Add `#[tracing::instrument]` to every public async function that does work
worth tracing. This is how Tempo gets data.

```rust
// Discord event handler
#[tracing::instrument(skip(self, ctx, msg), fields(channel = %msg.channel_id))]
pub async fn handle(&self, ctx: &Context, msg: &Message) { ... }

// LLM call
#[tracing::instrument(skip(self, req), fields(provider = "anthropic", model = %req.model))]
pub async fn generate(&self, req: GenerateRequest) -> anyhow::Result<GenerateResponse> { ... }

// DB operation
#[tracing::instrument(skip(self), fields(db.operation = "save_memory"))]
pub async fn save(&self, memory: &Memory) -> anyhow::Result<()> { ... }
```

Rules:
- `skip` large or sensitive fields (full message content, API keys).
- Add `fields(...)` for attributes you want to filter by in Grafana.
- Every public async fn in a hot path should have `#[tracing::instrument]`.

---

## Metrics

Use `opentelemetry::global::meter("botname")` to get a meter from the
globally registered `SdkMeterProvider`.

### Recommended counters per bot

```rust
use opentelemetry::{global, KeyValue};

let meter = global::meter("bluebot");

let messages_received = meter
    .u64_counter("bot.messages.received")
    .with_description("Total Discord messages seen by this bot")
    .build();

// In the message handler:
messages_received.add(1, &[
    KeyValue::new("bot", "bluebot"),
    KeyValue::new("guild", guild_id.to_string()),
]);
```

### Standard metric names

| Metric | Type | Labels |
|---|---|---|
| `bot.messages.received` | Counter | `bot`, `guild` |
| `bot.messages.sent` | Counter | `bot`, `strategy` |
| `bot.llm.requests` | Counter | `bot`, `provider`, `model` |
| `bot.llm.duration_ms` | Histogram | `bot`, `provider`, `model` |
| `bot.errors` | Counter | `bot`, `kind` |

---

## Grafana access

- **Local dev**: http://localhost:3000 (no login required — anonymous admin)
- **Production (Tower)**: port-forward or Tailscale to the Tower host

### Useful queries

**All logs for bluebot (Loki):**
```logql
{service_name="bluebot"}
```

**Errors across all bots:**
```logql
{namespace="starbunk-rs"} |= "error"
```

**Trace search (Tempo):** Filter by `service.name = bluebot`

**Message rate (Prometheus):**
```promql
rate(bot_messages_received_total[5m])
```
> **Note:** The `bot.messages.received`, `bot.messages.sent`, and `bot.errors` metrics are
> defined in AGENTS.md but not yet instrumented in any bot crate. This query will return
> no data until the metrics are implemented.

---

## GKE Cloud Log Ingestion

GKE bot logs flow to **Google Cloud Logging** via the in-cluster otel-collector's
`googlecloud` exporter. A Cloud Logging sink forwards them to a Pub/Sub topic, and
the local otel-collector (Tower) pulls that subscription into the same Loki instance
that receives local bot logs — so everything is queryable in one place.

### Architecture

```
GKE pods (stdout/stderr)
    ↓  (node fluent-bit DaemonSet)
Google Cloud Logging
    ↓  (Log Sink: grafana-loki-sink)
GCP Pub/Sub topic: grafana-logs
    ↓  (pull subscription: grafana-logs-sub)
otel-collector on Tower  (googlecloudpubsub receiver)
    ↓  transform/gcp_service_name: maps gcp.container_name → service.name
Loki:3100
    ↓
Grafana:3030
```

### GCP resources (already provisioned)

| Resource | Name |
|---|---|
| GCP project | `starbunk-bot` |
| GKE cluster | `starbunk-gke-cluster` (us-central1) |
| Log sink | `grafana-loki-sink` → topic `grafana-logs` |
| Pub/Sub topic | `projects/starbunk-bot/topics/grafana-logs` |
| Pub/Sub subscription | `projects/starbunk-bot/subscriptions/grafana-logs-sub` |
| Service account | `grafana-loki-reader@starbunk-bot.iam.gserviceaccount.com` |
| Key file (Tower) | `/mnt/user/vault/secrets/projects/starbunk-bot/gcp-key.json` (mode 644) |

### Re-provisioning from scratch

```bash
# Service account + role
gcloud iam service-accounts create grafana-loki-reader --project=starbunk-bot
gcloud projects add-iam-policy-binding starbunk-bot \
  --member="serviceAccount:grafana-loki-reader@starbunk-bot.iam.gserviceaccount.com" \
  --role="roles/pubsub.subscriber"

# Pub/Sub topic + log sink
gcloud pubsub topics create grafana-logs --project=starbunk-bot
gcloud logging sinks create grafana-loki-sink \
  pubsub.googleapis.com/projects/starbunk-bot/topics/grafana-logs \
  --log-filter='resource.type="k8s_container" AND resource.labels.cluster_name="starbunk-gke-cluster"' \
  --project=starbunk-bot
gcloud pubsub topics add-iam-policy-binding grafana-logs \
  --member="serviceAccount:service-487937378621@gcp-sa-logging.iam.gserviceaccount.com" \
  --role="roles/pubsub.publisher" --project=starbunk-bot
gcloud pubsub subscriptions create grafana-logs-sub \
  --topic=grafana-logs --ack-deadline=60 --project=starbunk-bot

# Key file
gcloud iam service-accounts keys create /mnt/user/vault/secrets/projects/starbunk-bot/gcp-key.json \
  --iam-account=grafana-loki-reader@starbunk-bot.iam.gserviceaccount.com
chmod 644 /mnt/user/vault/secrets/projects/starbunk-bot/gcp-key.json
```

### otel-collector config

The `googlecloudpubsub` receiver and `transform/gcp_service_name` processor are
already in `observability/otel-collector.yaml`. The transform promotes
`gcp.container_name` → `service.name` for logs where `service.name` is absent
(nil), so GKE pods appear under their bot names in Loki (e.g. `bluebot`, `covabot`)
rather than `unknown_service`. Note: the condition is `== nil`, not `== "unknown_service"` —
Loki synthesizes that string from an absent attribute; the OTEL resource itself is nil.

The Tower production otel-collector (`starbunk-exporter`) must have:
- `GOOGLE_APPLICATION_CREDENTIALS=/etc/gcp-key.json`
- Volume mount: `/mnt/user/vault/secrets/projects/starbunk-bot/gcp-key.json:/etc/gcp-key.json:ro`

Both are set in `/mnt/user/appdata/portainer/compose/46/docker-compose.yml`.

### Querying GKE logs in Grafana (Loki)

**All logs for a specific bot:**
```logql
{service_name="covabot"}
```

**Errors across all GKE bots:**
```logql
{service_name=~"bluebot|covabot|bunkbot|djcova|ratbot"} | json | level="ERROR"
```

**Filter by Kubernetes namespace:**
```logql
{service_name="covabot"} | json | gcp_namespace_name="starbunk"
```

**Cross-correlate with bot traces** — GKE logs share the same Loki instance as
local bot logs, so Grafana's Explore split-panel can show GKE errors alongside
Tempo trace spans from the same time window.

---

## See Also

- `crates/starbunk/src/telemetry.rs` — init code and env var reference
- `observability/` — all collector, Loki, Tempo, Prometheus, Grafana config
- [[../infrastructure/Deployment|Deployment]] — Tower deploy notes
