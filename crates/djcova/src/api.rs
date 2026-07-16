use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use serenity::all::GuildId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::manager::{GuildAudioManager, QueueItem, RepeatMode};

#[derive(Clone)]
pub struct AppState {
    pub managers: Arc<Mutex<HashMap<GuildId, Arc<Mutex<GuildAudioManager>>>>>,
}

#[derive(Serialize)]
pub struct DjcovaStateResponse {
    pub guilds: Vec<GuildStateResponse>,
}

#[derive(Serialize)]
pub struct GuildStateResponse {
    pub guild_id: u64,
    pub volume: u8,
    pub is_paused: bool,
    pub repeat_mode: String,
    pub current_track: Option<TrackResponse>,
    pub queue_length: usize,
    pub history_length: usize,
}

#[derive(Serialize)]
pub struct TrackResponse {
    pub title: String,
    pub url: String,
    pub requester: String,
    pub duration_secs: Option<u64>,
}

impl From<&QueueItem> for TrackResponse {
    fn from(item: &QueueItem) -> Self {
        Self {
            title: item.title.clone(),
            url: item.url.clone(),
            requester: item.requester.clone(),
            duration_secs: item.duration.map(|d| d.as_secs()),
        }
    }
}

pub fn bot_router(state: AppState) -> Router {
    Router::new()
        .route("/state", get(get_state))
        .route("/kick/:guild_id", post(kick_bot))
        .route("/skip/:guild_id", post(skip_bot))
        .with_state(state)
}

async fn get_state(State(state): State<AppState>) -> impl IntoResponse {
    let mut response = DjcovaStateResponse { guilds: Vec::new() };

    let managers = state.managers.lock().await;
    for (guild_id, mgr_mtx) in managers.iter() {
        let mgr = mgr_mtx.lock().await;

        let repeat_mode = match mgr.get_repeat_mode() {
            RepeatMode::Off => "Off",
            RepeatMode::Song => "Song",
            RepeatMode::Queue => "Queue",
        }
        .to_string();

        response.guilds.push(GuildStateResponse {
            guild_id: guild_id.get(),
            volume: mgr.get_volume(),
            is_paused: mgr.is_paused(),
            repeat_mode,
            current_track: mgr.get_current_track().as_ref().map(TrackResponse::from),
            queue_length: mgr.get_queue().len(),
            history_length: mgr.get_history().len(),
        });
    }

    (StatusCode::OK, Json(response))
}

async fn kick_bot(Path(guild_id): Path<u64>, State(state): State<AppState>) -> impl IntoResponse {
    let managers = state.managers.lock().await;
    let guild_id = GuildId::new(guild_id);
    if let Some(mgr_mtx) = managers.get(&guild_id) {
        let mut mgr = mgr_mtx.lock().await;
        let _ = mgr.stop().await;
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn skip_bot(Path(guild_id): Path<u64>, State(state): State<AppState>) -> impl IntoResponse {
    let managers = state.managers.lock().await;
    let guild_id = GuildId::new(guild_id);
    if let Some(mgr_mtx) = managers.get(&guild_id) {
        let mut mgr = mgr_mtx.lock().await;
        let _ = mgr.skip(None).await;
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn state_endpoint_returns_state() {
        let state = AppState {
            managers: Arc::new(Mutex::new(HashMap::new())),
        };
        let app = bot_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/state")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
