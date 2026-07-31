use crate::personality::{CovabotPersonalityYml, PersonalityStore};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

pub fn router(store: Arc<PersonalityStore>) -> Router {
    Router::new()
        .route(
            "/config/profiles/:id",
            get(get_profile_yaml).post(update_profile_yaml),
        )
        .route(
            "/api/personality",
            get(get_personality_json).patch(patch_personality_json),
        )
        .with_state(store)
}

async fn get_profile_yaml(
    Path(_id): Path<String>,
    State(store): State<Arc<PersonalityStore>>,
) -> Result<String, axum::http::StatusCode> {
    store
        .sync_to_yaml()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

async fn update_profile_yaml(
    Path(_id): Path<String>,
    State(store): State<Arc<PersonalityStore>>,
    body: String,
) -> Result<(), axum::http::StatusCode> {
    store
        .sync_from_yaml(&body)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)
}

async fn get_personality_json(
    State(store): State<Arc<PersonalityStore>>,
) -> Result<Json<CovabotPersonalityYml>, axum::http::StatusCode> {
    let p = store
        .get_personality()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(p))
}

#[derive(Deserialize)]
pub struct PatchPersonalityDto {
    pub identity: Option<String>,
    pub speech_patterns: Option<Vec<String>>,
    pub affinities: Option<Vec<String>>,
    pub relationships: Option<HashMap<String, String>>,
}

async fn patch_personality_json(
    State(store): State<Arc<PersonalityStore>>,
    Json(update): Json<PatchPersonalityDto>,
) -> Result<(), axum::http::StatusCode> {
    let mut current = store
        .get_personality()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(identity) = update.identity {
        current.identity = identity;
    }
    if let Some(patterns) = update.speech_patterns {
        current.speech_patterns = patterns;
    }
    if let Some(affinities) = update.affinities {
        current.affinities = affinities;
    }
    if let Some(relationships) = update.relationships {
        current.relationships = relationships;
    }

    store
        .update_personality(&current)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;

    // A simple smoke test to ensure the router can be created and bound.
    // Testing the actual handlers with the real Postgres DB in this unit test
    // is possible since setup_store initializes it.

    #[tokio::test]
    async fn router_can_be_created() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://starbunk:starbunk@localhost:5432/starbunk_memory".to_string()
        });
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(120))
            .connect(&db_url)
            .await
            .unwrap();
        let store = Arc::new(PersonalityStore::new(pool));

        let app = router(store);

        let req = Request::builder()
            .uri("/api/personality")
            .body(axum::body::Body::empty())
            .unwrap();

        // We just ensure it runs and we get a response (it might be 500 if DB is missing schema, but router works).
        let _res = app.oneshot(req).await.unwrap();
    }
}
