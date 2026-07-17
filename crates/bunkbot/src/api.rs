use axum::{extract::State, routing::get, Router};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config;
use crate::engine::BunkBotEngine;

#[derive(Clone)]
pub struct ApiState {
    pub engine: Arc<RwLock<Option<Arc<BunkBotEngine>>>>,
    pub config_dir: String,
}

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/config", get(get_config).post(post_config))
        .with_state(state)
}

async fn get_config(State(state): State<ApiState>) -> Result<String, axum::http::StatusCode> {
    let path = format!("{}/botbot.yml", state.config_dir);
    tokio::fs::read_to_string(&path).await.map_err(|e| {
        tracing::error!("failed to read config file {}: {}", path, e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn post_config(State(state): State<ApiState>, payload: String) -> axum::http::StatusCode {
    // Validate YAML
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

    // Now reload all files in the directory to reconstruct the full bot list
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

    #[tokio::test]
    async fn test_post_config_invalid_yaml_returns_bad_request() {
        let state = setup_test_state().await;
        let app = router(state.clone());

        let invalid_yaml = "reply-bots: [\n";

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/config")
                    .header("content-type", "application/yaml")
                    .body(Body::from(invalid_yaml))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
