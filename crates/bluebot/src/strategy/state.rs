use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared state for BlueBot strategies to track reply windows and cooldowns.
#[derive(Debug, Clone)]
pub struct SharedState {
    // Add state fields here (e.g. last_blue_response, last_murder_response)
}

impl SharedState {
    pub fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {}))
    }
}
