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

To pull logs from a GKE cluster into the local Loki instance, use **Grafana Alloy**
with a GCP Pub/Sub receiver. This routes Cloud Logging output into the same Loki
instance that receives bot OTEL logs, so GKE and bot logs are correlated in one place.

### Architecture

```
GKE cluster (Cloud Logging)
    ↓  (Log Sink)
GCP Pub/Sub topic
    ↓  (pull subscription)
Grafana Alloy  (Docker container on Tower)
    ↓  (HTTP push)
Loki:3100
    ↓
Grafana:3000
```

### Step 1 — GCP: Create a Log Sink

In GCP Console → Logging → Log Router → **Create Sink**:

- **Sink destination:** Cloud Pub/Sub topic (create a new topic, e.g. `grafana-logs`)
- **Inclusion filter** — scope to your GKE cluster to avoid pulling everything:

```
resource.type="k8s_container"
resource.labels.cluster_name="YOUR_CLUSTER"
```

Then create a Pub/Sub **subscription** for Alloy to pull from:

```bash
gcloud pubsub subscriptions create grafana-logs-sub \
  --topic=grafana-logs \
  --ack-deadline=60
```

### Step 2 — GCP: Service Account

```bash
gcloud iam service-accounts create grafana-loki-reader \
  --display-name="Grafana Loki Log Reader"

gcloud projects add-iam-policy-binding YOUR_PROJECT \
  --member="serviceAccount:grafana-loki-reader@YOUR_PROJECT.iam.gserviceaccount.com" \
  --role="roles/pubsub.subscriber"

gcloud iam service-accounts keys create ~/gcp-key.json \
  --iam-account=grafana-loki-reader@YOUR_PROJECT.iam.gserviceaccount.com
```

Place `gcp-key.json` on Tower at a path accessible to Docker (e.g. `~/secrets/gcp-key.json`).

### Step 3 — Docker: Add Alloy service

In the production `docker-compose.yml`, add alongside the existing LGTM stack:

```yaml
alloy:
  image: grafana/alloy:latest
  volumes:
    - ./alloy/config.alloy:/etc/alloy/config.alloy
    - ~/secrets/gcp-key.json:/etc/alloy/gcp-key.json:ro
  environment:
    - GOOGLE_APPLICATION_CREDENTIALS=/etc/alloy/gcp-key.json
  command: run /etc/alloy/config.alloy
  depends_on:
    - loki
  restart: unless-stopped
```

### Step 4 — Alloy config

Create `observability/alloy/config.alloy`:

```hcl
loki.source.pubsub "gke_logs" {
  project_id           = "YOUR_GCP_PROJECT"
  subscription         = "projects/YOUR_GCP_PROJECT/subscriptions/grafana-logs-sub"
  use_incoming_timestamp = true

  forward_to = [loki.write.local.receiver]

  labels = {
    source  = "gcp",
    cluster = "YOUR_CLUSTER",
  }
}

loki.write "local" {
  endpoint {
    url = "http://loki:3100/loki/api/v1/push"
  }
}
```

### Querying GKE logs in Grafana (Loki)

GKE log entries arrive with their original Cloud Logging structure. Useful LogQL queries:

**All logs from a specific namespace:**
```logql
{source="gcp", cluster="YOUR_CLUSTER"} | json | kubernetes_namespace_name="your-ns"
```

**Pod logs by name:**
```logql
{source="gcp"} | json | kubernetes_pod_name=~"my-pod-.*"
```

**Errors across the cluster:**
```logql
{source="gcp"} | json | severity="ERROR"
```

**Cross-correlate with bot traces** — GKE logs land in the same Loki instance, so
you can use Grafana's split-panel view to show GKE errors alongside bot trace spans
from Tempo.

---

## See Also

- `crates/starbunk/src/telemetry.rs` — init code and env var reference
- `observability/` — all collector, Loki, Tempo, Prometheus, Grafana config
- [[../infrastructure/Deployment|Deployment]] — Tower deploy notes
