use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::{self, BotConfig};
use crate::engine::BunkBotEngine;

#[derive(Clone)]
pub struct ApiState {
    pub engine: Arc<RwLock<Option<Arc<BunkBotEngine>>>>,
    pub config_dir: String,
}

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/config", get(get_config).post(post_config))
        .route("/api/bots", get(get_bots).put(put_bots))
        .route("/api/bots/status", get(get_bots_status))
        .route("/api/bots/:name/enable", post(enable_bot))
        .route("/api/bots/:name/disable", post(disable_bot))
        .route("/api/bots/:name/frequency", post(set_bot_frequency))
        .with_state(state)
}

async fn get_config(State(state): State<ApiState>) -> Result<String, axum::http::StatusCode> {
    let path = format!("{}/botbot.yml", state.config_dir);
    tokio::fs::read_to_string(&path).await.map_err(|e| {
        tracing::error!("failed to read config file {}: {}", path, e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn post_config(
    headers: axum::http::HeaderMap,
    State(state): State<ApiState>,
    payload: String,
) -> axum::http::StatusCode {
    if !is_authorized(&headers) {
        return axum::http::StatusCode::UNAUTHORIZED;
    }
    let _parsed_bots = match config::parse_bots(&payload) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("invalid yaml submitted: {}", e);
            return axum::http::StatusCode::BAD_REQUEST;
        }
    };

    let path = format!("{}/botbot.yml", state.config_dir);
    if let Err(e) = tokio::fs::write(&path, &payload).await {
        tracing::error!("failed to write config file {}: {}", path, e);
        return axum::http::StatusCode::INTERNAL_SERVER_ERROR;
    }

    reload_all_bots(&state).await
}

async fn reload_all_bots(state: &ApiState) -> axum::http::StatusCode {
    let mut all_bots = Vec::new();
    let mut read_dir = match tokio::fs::read_dir(&state.config_dir).await {
        Ok(dir) => dir,
        Err(e) => {
            tracing::error!(err = %e, "failed to read config directory");
            return axum::http::StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let p = entry.path();
        if p.is_file()
            && (p.extension().unwrap_or_default() == "yml"
                || p.extension().unwrap_or_default() == "yaml")
        {
            let yaml = match tokio::fs::read_to_string(&p).await {
                Ok(content) => content,
                Err(e) => {
                    tracing::error!(err = %e, file = %p.display(), "failed to read config file");
                    return axum::http::StatusCode::INTERNAL_SERVER_ERROR;
                }
            };
            match config::parse_bots(&yaml) {
                Ok(mut parsed) => all_bots.append(&mut parsed),
                Err(e) => {
                    tracing::error!(err = %e, file = %p.display(), "failed to parse config file");
                    return axum::http::StatusCode::INTERNAL_SERVER_ERROR;
                }
            }
        }
    }

    let mut engine_lock = state.engine.write().await;
    if let Some(engine_arc) = engine_lock.as_mut() {
        let new_engine = engine_arc.reload_bots_as_new(all_bots);
        *engine_arc = Arc::new(new_engine);
    }

    axum::http::StatusCode::OK
}

async fn get_bots(
    State(state): State<ApiState>,
) -> Result<Json<Vec<BotConfig>>, axum::http::StatusCode> {
    let mut all_bots = Vec::new();
    let mut read_dir = match tokio::fs::read_dir(&state.config_dir).await {
        Ok(dir) => dir,
        Err(e) => {
            tracing::error!(err = %e, "failed to read config directory");
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let p = entry.path();
        if p.is_file()
            && (p.extension().unwrap_or_default() == "yml"
                || p.extension().unwrap_or_default() == "yaml")
        {
            let yaml = match tokio::fs::read_to_string(&p).await {
                Ok(content) => content,
                Err(_) => continue,
            };
            if let Ok(mut parsed) = config::parse_bots(&yaml) {
                all_bots.append(&mut parsed);
            }
        }
    }
    Ok(Json(all_bots))
}

#[derive(Serialize)]
pub struct BotStatus {
    pub name: String,
    pub enabled: bool,
    pub current_frequency: u8,
    pub original_frequency: u8,
    pub triggers_today: u64,
}

async fn get_bots_status(
    State(state): State<ApiState>,
) -> Result<Json<Vec<BotStatus>>, axum::http::StatusCode> {
    let engine_lock = state.engine.read().await;
    let engine = match &*engine_lock {
        Some(e) => e,
        None => return Err(axum::http::StatusCode::SERVICE_UNAVAILABLE),
    };

    let state_service = engine.state_service();
    let states = state_service.get_all_states();
    let frequencies = state_service.get_all_frequencies();
    let triggers = state_service.get_all_triggers_today();

    let configs = engine.bot_configs();
    let mut result = Vec::new();

    for (name, orig_freq) in configs {
        let enabled = states.get(&name).copied().unwrap_or(true);
        let current_frequency = frequencies
            .get(&name)
            .map(|f| f.current_frequency)
            .unwrap_or(orig_freq);
        let triggers_today = triggers.get(&name).copied().unwrap_or(0);

        result.push(BotStatus {
            name,
            enabled,
            current_frequency,
            original_frequency: orig_freq,
            triggers_today,
        });
    }

    Ok(Json(result))
}

async fn enable_bot(
    headers: axum::http::HeaderMap,
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> axum::http::StatusCode {
    if !is_authorized(&headers) {
        return axum::http::StatusCode::UNAUTHORIZED;
    }
    let engine_lock = state.engine.read().await;
    if let Some(engine) = &*engine_lock {
        engine.state_service().enable_bot(&name);
    }
    axum::http::StatusCode::OK
}

async fn disable_bot(
    headers: axum::http::HeaderMap,
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> axum::http::StatusCode {
    if !is_authorized(&headers) {
        return axum::http::StatusCode::UNAUTHORIZED;
    }
    let engine_lock = state.engine.read().await;
    if let Some(engine) = &*engine_lock {
        engine.state_service().disable_bot(&name);
    }
    axum::http::StatusCode::OK
}

#[derive(Deserialize)]
pub struct FrequencyPayload {
    frequency: u8,
}

async fn set_bot_frequency(
    headers: axum::http::HeaderMap,
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Json(payload): Json<FrequencyPayload>,
) -> axum::http::StatusCode {
    if !is_authorized(&headers) {
        return axum::http::StatusCode::UNAUTHORIZED;
    }
    let engine_lock = state.engine.read().await;
    if let Some(engine) = &*engine_lock {
        let orig = engine
            .bot_configs()
            .into_iter()
            .find(|(n, _)| n == &name)
            .map(|(_, f)| f)
            .unwrap_or(100);
        engine
            .state_service()
            .set_frequency(&name, payload.frequency, "admin_ui", orig);
    }
    axum::http::StatusCode::OK
}

async fn put_bots(
    headers: axum::http::HeaderMap,
    State(state): State<ApiState>,
    Json(bots): Json<Vec<BotConfig>>,
) -> axum::http::StatusCode {
    if !is_authorized(&headers) {
        return axum::http::StatusCode::UNAUTHORIZED;
    }
    let file = config::ReplyBotsFile { reply_bots: bots };
    let yaml = match serde_yaml::to_string(&file) {
        Ok(y) => y,
        Err(e) => {
            tracing::error!("failed to serialize bots: {}", e);
            return axum::http::StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let path = format!("{}/botbot.yml", state.config_dir);
    if let Err(e) = tokio::fs::write(&path, &yaml).await {
        tracing::error!("failed to write config file {}: {}", path, e);
        return axum::http::StatusCode::INTERNAL_SERVER_ERROR;
    }

    reload_all_bots(&state).await
}

fn is_authorized(headers: &axum::http::HeaderMap) -> bool {
    let token = match std::env::var("BUNKBOT_ADMIN_TOKEN") {
        Ok(t) => t,
        Err(_) => return false,
    };
    if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str == format!("Bearer {}", token) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn setup_test_state() -> ApiState {
        let dir = std::env::temp_dir().join(format!(
            "bunkbot_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        ApiState {
            engine: Arc::new(RwLock::new(None)),
            config_dir: dir.to_string_lossy().to_string(),
        }
    }

    #[tokio::test]
    async fn test_get_config_returns_yaml() {
        let state = setup_test_state().await;

        let path = format!("{}/botbot.yml", state.config_dir);
        let dummy_yaml =
            "reply-bots:\n  - name: test_bot\n    triggers: []\n    identity:\n      type: random";
        tokio::fs::write(&path, dummy_yaml).await.unwrap();

        let app = router(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/config")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(
            body_str.contains("reply-bots:"),
            "Expected config to contain 'reply-bots:'"
        );
    }
}
