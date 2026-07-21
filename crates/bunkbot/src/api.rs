use axum::{extract::State, routing::get, Router};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::engine::BunkBotEngine;

pub type AppState = Arc<RwLock<Option<BunkBotEngine>>>;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/config", get(get_config).post(post_config))
        .with_state(state)
}

async fn get_config(State(state): State<AppState>) -> String {
    let engine_guard = state.read().await;
    if let Some(engine) = engine_guard.as_ref() {
        let configs = engine.bot_configs().to_vec();
        let file = crate::config::ReplyBotsFile {
            reply_bots: configs,
        };
        serde_yaml::to_string(&file).unwrap_or_default()
    } else {
        String::new()
    }
}

async fn post_config(State(state): State<AppState>, payload: String) -> axum::http::StatusCode {
    match crate::config::parse_bots(&payload) {
        Ok(bots) => {
            let mut engine_guard = state.write().await;
            if let Some(engine) = engine_guard.as_mut() {
                engine.reload_bots(bots);
                axum::http::StatusCode::OK
            } else {
                axum::http::StatusCode::SERVICE_UNAVAILABLE
            }
        }
        Err(e) => {
            tracing::warn!("Failed to parse uploaded config: {}", e);
            axum::http::StatusCode::BAD_REQUEST
        }
    }
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

    #[tokio::test]
    async fn test_get_config_returns_yaml() {
        let engine = BunkBotEngine::new(
            vec![],
            Arc::new(crate::engine::tests::DummySender),
            Arc::new(crate::engine::tests::DummyIdentity),
            Arc::new(crate::state::InMemoryBotStateManager::new()),
            Arc::new(starbunk::audit::AuditStore::dummy()),
        );
        let state = Arc::new(RwLock::new(Some(engine)));
        let app = router(state);

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

        // Failing condition: the stub returns empty string, test expects YAML content
        assert!(
            body_str.contains("reply-bots:"),
            "Expected config to contain 'reply-bots:'"
        );
    }

    #[tokio::test]
    async fn test_post_config_invalid_yaml_returns_bad_request() {
        let state = Arc::new(RwLock::new(None));
        let app = router(state);

        let invalid_yaml = "bots: [\n";

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

        // Failing condition: the stub returns OK, test expects BAD_REQUEST
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
