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

async fn get_config(State(_state): State<AppState>) -> String {
    // Stub
    String::new()
}

async fn post_config(State(_state): State<AppState>, _payload: String) -> axum::http::StatusCode {
    // Stub
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

    #[tokio::test]
    async fn test_get_config_returns_yaml() {
        let state = Arc::new(RwLock::new(None));
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
            body_str.contains("bots:"),
            "Expected config to contain 'bots:'"
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
                    .body(Body::from(invalid_yaml))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Failing condition: the stub returns OK, test expects BAD_REQUEST
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
