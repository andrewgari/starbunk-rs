use prometheus::{histogram_opts, opts, Histogram, IntCounter, IntCounterVec, IntGauge, Registry};
use std::sync::Arc;

/// Prometheus metrics for BunkBot.
///
/// Each field corresponds to one Prometheus metric family exposed on `GET /metrics`.
/// Construct once with `BunkBotMetrics::new()` and distribute via `Arc`.
#[derive(Debug)]
pub struct BunkBotMetrics {
    registry: Registry,
    pub messages_received: IntCounter,
    pub bot_triggers: IntCounterVec,
    pub active_bots: IntGauge,
    pub response_latency: Histogram,
    pub errors: IntCounterVec,
}

impl BunkBotMetrics {
    /// Create a new `BunkBotMetrics` with a fresh, isolated `Registry`.
    ///
    /// All five metric families are registered against the private registry so
    /// that this crate's metrics never pollute the default global registry.
    pub fn new() -> Arc<Self> {
        let registry = Registry::new();

        let messages_received = IntCounter::with_opts(opts!(
            "bunkbot_messages_received_total",
            "Total number of Discord messages received by BunkBot"
        ))
        .expect("bunkbot_messages_received_total counter is valid");

        let bot_triggers = IntCounterVec::new(
            opts!(
                "bunkbot_bot_triggers_total",
                "Total number of times each reply bot has triggered"
            ),
            &["bot"],
        )
        .expect("bunkbot_bot_triggers_total counter_vec is valid");

        let active_bots = IntGauge::with_opts(opts!(
            "bunkbot_active_bots",
            "Current number of active (enabled) reply bots"
        ))
        .expect("bunkbot_active_bots gauge is valid");

        let latency_buckets = vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5];
        let response_latency = Histogram::with_opts(histogram_opts!(
            "bunkbot_response_latency_seconds",
            "Latency for bot response dispatch in seconds",
            latency_buckets
        ))
        .expect("bunkbot_response_latency_seconds histogram is valid");

        let errors = IntCounterVec::new(
            opts!(
                "bunkbot_errors_total",
                "Total number of errors encountered by BunkBot, labelled by kind"
            ),
            &["kind"],
        )
        .expect("bunkbot_errors_total counter_vec is valid");

        registry
            .register(Box::new(messages_received.clone()))
            .expect("messages_received registers without conflict");
        registry
            .register(Box::new(bot_triggers.clone()))
            .expect("bot_triggers registers without conflict");
        registry
            .register(Box::new(active_bots.clone()))
            .expect("active_bots registers without conflict");
        registry
            .register(Box::new(response_latency.clone()))
            .expect("response_latency registers without conflict");
        registry
            .register(Box::new(errors.clone()))
            .expect("errors registers without conflict");

        Arc::new(Self {
            registry,
            messages_received,
            bot_triggers,
            active_bots,
            response_latency,
            errors,
        })
    }

    /// Return a reference to the private Prometheus registry.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Encoder;

    #[test]
    fn metrics_new_succeeds() {
        let _ = BunkBotMetrics::new();
    }

    #[test]
    fn messages_received_increments() {
        let m = BunkBotMetrics::new();
        m.messages_received.inc();
        assert_eq!(m.messages_received.get(), 1);
    }

    #[test]
    fn bot_triggers_increments_per_bot() {
        let m = BunkBotMetrics::new();
        m.bot_triggers.with_label_values(&["bluebot"]).inc();
        assert_eq!(m.bot_triggers.with_label_values(&["bluebot"]).get(), 1);
        assert_eq!(m.bot_triggers.with_label_values(&["covabot"]).get(), 0);
    }

    #[test]
    fn active_bots_gauge_set() {
        let m = BunkBotMetrics::new();
        m.active_bots.set(5);
        assert_eq!(m.active_bots.get(), 5);
    }

    #[test]
    fn response_latency_records_observation() {
        let m = BunkBotMetrics::new();
        m.response_latency.observe(0.1);
        // get_sample_count returns the number of observations
        assert_eq!(m.response_latency.get_sample_count(), 1);
    }

    #[test]
    fn errors_increments_per_kind() {
        let m = BunkBotMetrics::new();
        m.errors.with_label_values(&["send"]).inc();
        assert_eq!(m.errors.with_label_values(&["send"]).get(), 1);
        assert_eq!(m.errors.with_label_values(&["db"]).get(), 0);
    }

    #[test]
    fn registry_gather_returns_families() {
        let m = BunkBotMetrics::new();
        // Seed label-set vecs so they appear in gather() output.
        m.bot_triggers.with_label_values(&["test"]).inc();
        m.errors.with_label_values(&["test"]).inc();
        let families = m.registry().gather();
        assert!(
            families.len() >= 5,
            "expected at least 5 metric families, got {}",
            families.len()
        );
    }

    #[test]
    fn text_encoder_output_contains_metric_names() {
        let m = BunkBotMetrics::new();
        // Observe at least one sample so latency histogram and label-set vecs appear in output.
        m.response_latency.observe(0.01);
        m.messages_received.inc();
        m.bot_triggers.with_label_values(&["test"]).inc();
        m.errors.with_label_values(&["test"]).inc();

        let encoder = prometheus::TextEncoder::new();
        let families = m.registry().gather();
        let mut buf = Vec::new();
        encoder
            .encode(&families, &mut buf)
            .expect("encode succeeds");
        let text = String::from_utf8(buf).expect("valid utf8");

        for name in &[
            "bunkbot_messages_received_total",
            "bunkbot_bot_triggers_total",
            "bunkbot_active_bots",
            "bunkbot_response_latency_seconds",
            "bunkbot_errors_total",
        ] {
            assert!(
                text.contains(name),
                "encoded metrics text must contain '{name}'"
            );
        }
    }
}
