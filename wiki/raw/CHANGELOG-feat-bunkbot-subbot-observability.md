## [Unreleased] — feat/bunkbot-subbot-observability

### Added
- Wire `bunkbot_bot_triggers_total` Prometheus counter into engine dispatch — each subbot trigger now increments the metric with a `bot` label.
- Wire `bunkbot_response_latency_seconds` histogram into engine dispatch — records time from send call to completion per subbot trigger.
- Wire `bunkbot_errors_total` counter with `kind=send` label on dispatch failures.
- Set `bunkbot_active_bots` gauge on engine initialization with count of loaded bots.
- Add `#[tracing::instrument]` on `dispatch_bot` with `bot` field — produces Tempo spans per subbot evaluation.
- Add `tracing::debug!` log event on successful trigger with bot, channel, and latency_ms fields for Loki queries.
- Enable Prometheus scrape of BunkBot `/metrics` endpoint (port 9082) in `observability/prometheus.yml`.
- Add Grafana dashboard JSON (`observability/grafana/dashboards/bunkbot-subbots.json`) with:
  - Trigger rate per subbot (time series)
  - Pie chart of total triggers by subbot
  - Messages received stat
  - Active bots gauge
  - Errors by kind
  - Response latency percentiles (p50/p95/p99)
  - Trigger heatmap (hourly buckets)
  - Loki log table of subbot trigger events
  - Top 10 subbots bar chart (24h)
  - Message rate vs trigger rate overlay
- Mount `observability/grafana/dashboards/` into Grafana container in docker-compose.

### Changed
- `BunkBotEngine::new` and `new_with_comment_config` now accept an optional `metrics: Option<Arc<BunkBotMetrics>>` parameter.
