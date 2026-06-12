use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::Instant;

/// Runtime health monitor for a single bot instance.
///
/// Tracks whether the bot has successfully connected to the Discord gateway,
/// and counts user-facing errors (send failures, handler panics, etc.).
///
/// Emits structured log events with consistent `health.*` fields so that
/// `scripts/deployment/health-check.sh` can grep for them in Loki/container logs:
///
/// | Event                    | Log message                  | `health.status`     |
/// |--------------------------|------------------------------|---------------------|
/// | Gateway `ready` fired    | `"health: startup ok"`       | `"connected"`       |
/// | Ready not seen in 30s    | `"health: startup timeout"`  | `"startup_timeout"` |
/// | User-facing send failure | `"health: user-facing error"`| `"error"`           |
#[derive(Debug)]
pub struct HealthMonitor {
    bot: &'static str,
    connected: AtomicBool,
    error_count: AtomicU64,
    started_at: Instant,
}

impl HealthMonitor {
    /// Create a new monitor wrapped in an `Arc` so it can be shared between
    /// the event handler and the startup watchdog task.
    pub fn new(bot: &'static str) -> Arc<Self> {
        Arc::new(Self {
            bot,
            connected: AtomicBool::new(false),
            error_count: AtomicU64::new(0),
            started_at: Instant::now(),
        })
    }

    /// Call from `EventHandler::ready`. Marks the bot as connected and emits
    /// a structured startup-success log entry.
    pub fn on_connected(&self) {
        self.connected.store(true, Ordering::SeqCst);
        tracing::info!(
            bot = self.bot,
            health.status = "connected",
            startup_ms = self.started_at.elapsed().as_millis() as u64,
            "health: startup ok"
        );
    }

    /// Call whenever a user-facing operation fails (e.g. failed Discord send).
    ///
    /// Increments the error counter and emits a structured log entry at ERROR
    /// level so the deployment health check and Loki alerts can detect it.
    pub fn on_error(&self, context: &str, err: &dyn std::fmt::Display) {
        let count = self.error_count.fetch_add(1, Ordering::SeqCst) + 1;
        tracing::error!(
            bot = self.bot,
            health.status = "error",
            health.errors = count,
            health.context = context,
            err = %err,
            "health: user-facing error"
        );
    }

    /// Returns `true` once `on_connected` has been called.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Total user-facing errors recorded since startup.
    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::SeqCst)
    }
}
