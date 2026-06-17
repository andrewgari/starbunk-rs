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

/// Start a lightweight, zero-dependency async HTTP health server.
/// Binds to the port specified in the `HEALTH_PORT` environment variable or
/// defaults to a unique port based on the bot name.
pub fn start_health_server(bot_name: &str, health_monitor: Option<Arc<HealthMonitor>>) {
    let bot_name = bot_name.to_string();
    let port = std::env::var("HEALTH_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or_else(|| match bot_name.to_lowercase().as_str() {
            "bluebot" => 8081,
            "bunkbot" => 8082,
            "covabot" => 8083,
            "djcova" => 8084,
            "ratbot" => 8085,
            _ => 8080,
        });

    tokio::spawn(async move {
        let addr = format!("0.0.0.0:{}", port);
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!(
                    bot = bot_name.as_str(),
                    "Failed to bind health server on {}: {}",
                    addr,
                    e
                );
                return;
            }
        };
        tracing::info!(
            bot = bot_name.as_str(),
            "Health server listening on http://{}",
            addr
        );

        loop {
            match listener.accept().await {
                Ok((mut socket, _)) => {
                    let monitor = health_monitor.clone();
                    let bot_name_clone = bot_name.clone();
                    tokio::spawn(async move {
                        use tokio::io::{AsyncReadExt, AsyncWriteExt};
                        let mut buf = [0; 512];
                        let _ = tokio::time::timeout(
                            tokio::time::Duration::from_millis(500),
                            socket.read(&mut buf),
                        )
                        .await;

                        let is_ok = monitor.as_ref().map(|m| m.is_connected()).unwrap_or(true);
                        let (status_code, body) = if is_ok {
                            ("200 OK", "{\"status\":\"ok\"}")
                        } else {
                            ("503 Service Unavailable", "{\"status\":\"connecting\"}")
                        };

                        let response = format!(
                            "HTTP/1.1 {}\r\n\
                             Content-Type: application/json\r\n\
                             Content-Length: {}\r\n\
                             Connection: close\r\n\r\n\
                             {}",
                            status_code,
                            body.len(),
                            body
                        );

                        if let Err(e) = socket.write_all(response.as_bytes()).await {
                            tracing::debug!(
                                bot = bot_name_clone.as_str(),
                                "Failed to write health response: {}",
                                e
                            );
                        }
                        let _ = socket.flush().await;
                    });
                }
                Err(e) => {
                    tracing::error!(bot = bot_name.as_str(), "Health server accept error: {}", e);
                }
            }
        }
    });
}
