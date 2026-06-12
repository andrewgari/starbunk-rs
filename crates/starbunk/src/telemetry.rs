//! OpenTelemetry initialisation for starbunk-rs bots.
//!
//! # Usage
//!
//! ```rust,ignore
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let _guard = starbunk_shared::telemetry::init("mybot")?;
//!     mybot::run().await
//! }
//! ```
//!
//! Hold `_guard` for the lifetime of the process. Dropping it flushes all
//! in-flight telemetry and performs a graceful provider shutdown.
//!
//! # Environment variables
//!
//! | Variable                         | Default                         | Effect                                   |
//! |----------------------------------|---------------------------------|------------------------------------------|
//! | `OTEL_EXPORTER_OTLP_ENDPOINT`   | `http://otel-collector:4317`   | gRPC endpoint for the OTEL collector     |
//! | `RUST_LOG`                       | `info` / `debug` in verbose    | tracing filter directives (e.g. `debug,serenity=warn`) |
//! | `VERBOSE`                        | unset / `false`                | Enable verbose mode — see below          |
//!
//! # Verbose mode
//!
//! Set `VERBOSE=1` (or `VERBOSE=true`) to activate:
//! - `DEBUG`-level default log level (overrideable with `RUST_LOG`)
//! - Thread IDs and names in console output
//! - Source file and line numbers in console output
//! - Span `NEW`/`CLOSE` events in console output (span creation and completion timing)
//!
//! Example:
//! ```sh
//! VERBOSE=1 RUST_LOG=debug,serenity=info cargo run --bin bluebot
//! ```

use anyhow::Context as _;
use opentelemetry::{trace::TracerProvider as _, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    logs::LoggerProvider,
    metrics::{PeriodicReader, SdkMeterProvider},
    runtime::Tokio,
    trace::TracerProvider,
    Resource,
};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

/// Held for the lifetime of the process. Dropping it flushes and shuts down
/// every OTEL pipeline so no telemetry is lost on clean exits.
#[derive(Debug)]
pub struct TelemetryGuard {
    tracer_provider: TracerProvider,
    logger_provider: LoggerProvider,
    meter_provider: SdkMeterProvider,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        // Shut down in reverse init order. Errors here are logged to the
        // console (fmt layer) only — the OTEL log/trace layers are already
        // being torn down, so these messages will not reach Loki or Tempo.
        if let Err(e) = self.tracer_provider.shutdown() {
            tracing::error!(err = %e, "trace provider shutdown error");
        }
        if let Err(e) = self.logger_provider.shutdown() {
            tracing::error!(err = %e, "logger provider shutdown error");
        }
        if let Err(e) = self.meter_provider.shutdown() {
            tracing::error!(err = %e, "meter provider shutdown error");
        }
    }
}

/// Initialise the full OTEL + tracing subscriber stack.
///
/// Must be called once in `main`, before the bot starts. The returned
/// [`TelemetryGuard`] must be held for the entire lifetime of the process.
///
/// Returns an error if the OTEL exporters cannot be built (e.g. invalid
/// `OTEL_EXPORTER_OTLP_ENDPOINT`).
///
/// Pipelines (all via OTLP gRPC → otel-collector):
/// - **Traces** → Tempo
/// - **Logs**   → Loki
/// - **Metrics** → Prometheus
pub fn init(service_name: &'static str) -> anyhow::Result<TelemetryGuard> {
    let verbose = is_verbose();
    let endpoint = otel_endpoint();
    let resource = build_resource(service_name);

    let tracer_provider = build_tracer_provider(&endpoint, resource.clone())
        .context("failed to build OTEL tracer provider — check OTEL_EXPORTER_OTLP_ENDPOINT")?;
    let logger_provider = build_logger_provider(&endpoint, resource.clone())
        .context("failed to build OTEL logger provider — check OTEL_EXPORTER_OTLP_ENDPOINT")?;
    let meter_provider = build_meter_provider(&endpoint, resource)
        .context("failed to build OTEL meter provider — check OTEL_EXPORTER_OTLP_ENDPOINT")?;

    opentelemetry::global::set_tracer_provider(tracer_provider.clone());
    opentelemetry::global::set_meter_provider(meter_provider.clone());

    let tracer = tracer_provider.tracer(service_name);
    let filter = build_filter(verbose);

    let span_events = if verbose {
        FmtSpan::NEW | FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    };

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(verbose)
        .with_thread_names(verbose)
        .with_file(verbose)
        .with_line_number(verbose)
        .with_span_events(span_events);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(OpenTelemetryLayer::new(tracer))
        .with(OpenTelemetryTracingBridge::new(&logger_provider))
        .init();

    // NOTE: service.version is the version of starbunk-shared (the crate that
    // owns this code), not the individual bot binary. If per-bot versioning is
    // ever introduced, pass the version through init() instead.
    if verbose {
        tracing::debug!(
            service = service_name,
            endpoint = %endpoint,
            verbose = true,
            version = env!("CARGO_PKG_VERSION"),
            "telemetry initialised (VERBOSE mode)"
        );
    } else {
        tracing::info!(service = service_name, "telemetry initialised");
    }

    Ok(TelemetryGuard {
        tracer_provider,
        logger_provider,
        meter_provider,
    })
}

// ──────────────────────────── internal helpers ───────────────────────────────

fn is_verbose() -> bool {
    std::env::var("VERBOSE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn otel_endpoint() -> String {
    std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://otel-collector:4317".to_string())
}

fn build_resource(service_name: &'static str) -> Resource {
    Resource::new([
        KeyValue::new("service.name", service_name),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        KeyValue::new("service.namespace", "starbunk-rs"),
    ])
}

fn build_tracer_provider(endpoint: &str, resource: Resource) -> anyhow::Result<TracerProvider> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;
    Ok(TracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter, Tokio)
        .build())
}

fn build_logger_provider(endpoint: &str, resource: Resource) -> anyhow::Result<LoggerProvider> {
    let exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;
    Ok(LoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter, Tokio)
        .build())
}

fn build_meter_provider(endpoint: &str, resource: Resource) -> anyhow::Result<SdkMeterProvider> {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;
    let reader = PeriodicReader::builder(exporter, Tokio).build();
    Ok(SdkMeterProvider::builder()
        .with_resource(resource)
        .with_reader(reader)
        .build())
}

fn build_filter(verbose: bool) -> EnvFilter {
    let default_level = if verbose { "debug" } else { "info" };
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn is_verbose_false_by_default() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::remove_var("VERBOSE");
        assert!(!is_verbose());
    }

    #[test]
    fn is_verbose_true_for_one() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("VERBOSE", "1");
        assert!(is_verbose());
        std::env::remove_var("VERBOSE");
    }

    #[test]
    fn is_verbose_true_for_true_string() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("VERBOSE", "true");
        assert!(is_verbose());
        std::env::remove_var("VERBOSE");
    }

    #[test]
    fn is_verbose_true_case_insensitive() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("VERBOSE", "TRUE");
        assert!(is_verbose());
        std::env::remove_var("VERBOSE");
    }

    #[test]
    fn is_verbose_false_for_other_values() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("VERBOSE", "false");
        assert!(!is_verbose());
        std::env::remove_var("VERBOSE");
    }

    #[test]
    fn otel_endpoint_default() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        assert_eq!(otel_endpoint(), "http://otel-collector:4317");
    }

    #[test]
    fn otel_endpoint_custom() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317");
        assert_eq!(otel_endpoint(), "http://localhost:4317");
        std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    }

    #[test]
    fn build_filter_uses_info_by_default() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::remove_var("RUST_LOG");
        let filter = build_filter(false);
        // EnvFilter's Display shows the directives; "info" should be present.
        assert!(filter.to_string().contains("info"));
    }

    #[test]
    fn build_filter_uses_debug_in_verbose_mode() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::remove_var("RUST_LOG");
        let filter = build_filter(true);
        assert!(filter.to_string().contains("debug"));
    }

    #[test]
    fn build_filter_respects_rust_log_override() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("RUST_LOG", "warn");
        let filter = build_filter(false);
        assert!(filter.to_string().contains("warn"));
        std::env::remove_var("RUST_LOG");
    }
}
