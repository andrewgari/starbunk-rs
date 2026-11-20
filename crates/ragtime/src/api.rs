//! Token-protected admin HTTP API for the trivia question bank.
//!
//! Mirrors bunkbot's `BUNKBOT_API_URL` / `BUNKBOT_ADMIN_TOKEN` pattern: the UI
//! (`starbunk-ui/src/lib/ragtime-api.ts`) calls `RAGTIME_API_URL` (default
//! `http://localhost:9085`) with an `Authorization: Bearer` header carrying the admin token.
//!
//! # Contract (enforced by `tests/admin_api.rs`)
//!
//! Every route requires `Authorization: Bearer <token>` matching `RAGTIME_ADMIN_TOKEN`; a missing/wrong
//! header — or a server with no token configured — returns **401** (fail closed).
//!
//! | Method | Path | Success | Errors |
//! |---|---|---|---|
//! | `GET` | `/api/ragtime/questions` | 200, JSON array ordered by `id` | 401 |
//! | `GET` | `/api/ragtime/questions/{id}` | 200, question | 400 bad id, 401, 404 |
//! | `POST` | `/api/ragtime/questions` | 201, created question | 400/422 invalid body, 401 |
//! | `PUT` | `/api/ragtime/questions/{id}` | 200, updated question (full replace; body `id` ignored) | 400/422, 401, 404 |
//! | `DELETE` | `/api/ragtime/questions/{id}` | 204, empty body | 400 bad id, 401, 404 |
//!
//! Wire shape: `{ id, question, display_answer, acceptable_answers, created_at, updated_at }`
//! with RFC 3339 timestamps. Input: `{ question, display_answer, acceptable_answers }`;
//! `question` / `display_answer` are trimmed and must be non-empty, `acceptable_answers`
//! entries are trimmed and blanks dropped. Error bodies are JSON `{ "error": "..." }`.
//! Repository failures return 500.

// Handlers short-circuit with a ready-made error `Response`; boxing buys nothing here.
#![allow(clippy::result_large_err)]

use std::sync::Arc;

use axum::extract::{rejection::JsonRejection, Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::question::{NewQuestion, Question, QuestionUpdate};
use crate::repository::QuestionRepository;

pub const ADMIN_TOKEN_ENV: &str = "RAGTIME_ADMIN_TOKEN";

#[derive(Clone)]
pub struct ApiState {
    pub repo: Arc<dyn QuestionRepository>,
    pub admin_token: Option<String>,
}

pub fn admin_token_from_env() -> Option<String> {
    std::env::var(ADMIN_TOKEN_ENV)
        .ok()
        .filter(|t| !t.trim().is_empty())
}

#[derive(Serialize)]
struct Wire {
    id: i64,
    question: String,
    display_answer: String,
    acceptable_answers: Vec<String>,
    created_at: String,
    updated_at: String,
}
impl From<Question> for Wire {
    fn from(q: Question) -> Self {
        Wire {
            id: q.id,
            question: q.question,
            display_answer: q.display_answer,
            acceptable_answers: q.acceptable_answers,
            created_at: q.created_at.to_rfc3339(),
            updated_at: q.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Deserialize)]
struct Input {
    question: String,
    display_answer: String,
    acceptable_answers: Vec<String>,
}

fn err(s: StatusCode, m: &str) -> Response {
    (s, Json(serde_json::json!({"error": m}))).into_response()
}
fn ise(_e: anyhow::Error) -> Response {
    err(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
}

fn authed(st: &ApiState, h: &HeaderMap) -> bool {
    let Some(t) = st.admin_token.as_deref().filter(|t| !t.is_empty()) else {
        return false;
    };
    h.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()) == Some(&format!("Bearer {t}"))
}

fn validate(b: Result<Json<Input>, JsonRejection>) -> Result<NewQuestion, Response> {
    let Json(i) = b.map_err(|e| err(StatusCode::BAD_REQUEST, &e.body_text()))?;
    let q = i.question.trim().to_string();
    let a = i.display_answer.trim().to_string();
    if q.is_empty() || a.is_empty() {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "question and display_answer required",
        ));
    }
    Ok(NewQuestion {
        question: q,
        display_answer: a,
        acceptable_answers: i
            .acceptable_answers
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    })
}

fn pid(p: Result<Path<i64>, axum::extract::rejection::PathRejection>) -> Result<i64, Response> {
    p.map(|Path(i)| i)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "bad id"))
}

async fn list(State(st): State<ApiState>, h: HeaderMap) -> Response {
    if !authed(&st, &h) {
        return err(StatusCode::UNAUTHORIZED, "unauthorized");
    }
    match st.repo.list().await {
        Ok(v) => Json(v.into_iter().map(Wire::from).collect::<Vec<_>>()).into_response(),
        Err(e) => ise(e),
    }
}
async fn create(
    State(st): State<ApiState>,
    h: HeaderMap,
    b: Result<Json<Input>, JsonRejection>,
) -> Response {
    if !authed(&st, &h) {
        return err(StatusCode::UNAUTHORIZED, "unauthorized");
    }
    let n = match validate(b) {
        Ok(n) => n,
        Err(r) => return r,
    };
    match st.repo.create(n).await {
        Ok(q) => (StatusCode::CREATED, Json(Wire::from(q))).into_response(),
        Err(e) => ise(e),
    }
}
async fn get_one(
    State(st): State<ApiState>,
    h: HeaderMap,
    p: Result<Path<i64>, axum::extract::rejection::PathRejection>,
) -> Response {
    if !authed(&st, &h) {
        return err(StatusCode::UNAUTHORIZED, "unauthorized");
    }
    let id = match pid(p) {
        Ok(i) => i,
        Err(r) => return r,
    };
    match st.repo.get(id).await {
        Ok(Some(q)) => Json(Wire::from(q)).into_response(),
        Ok(None) => err(StatusCode::NOT_FOUND, "not found"),
        Err(e) => ise(e),
    }
}
async fn put_one(
    State(st): State<ApiState>,
    h: HeaderMap,
    p: Result<Path<i64>, axum::extract::rejection::PathRejection>,
    b: Result<Json<Input>, JsonRejection>,
) -> Response {
    if !authed(&st, &h) {
        return err(StatusCode::UNAUTHORIZED, "unauthorized");
    }
    let id = match pid(p) {
        Ok(i) => i,
        Err(r) => return r,
    };
    let n = match validate(b) {
        Ok(n) => n,
        Err(r) => return r,
    };
    let u = QuestionUpdate {
        question: Some(n.question),
        display_answer: Some(n.display_answer),
        acceptable_answers: Some(n.acceptable_answers),
    };
    match st.repo.update(id, u).await {
        Ok(Some(q)) => Json(Wire::from(q)).into_response(),
        Ok(None) => err(StatusCode::NOT_FOUND, "not found"),
        Err(e) => ise(e),
    }
}
async fn del_one(
    State(st): State<ApiState>,
    h: HeaderMap,
    p: Result<Path<i64>, axum::extract::rejection::PathRejection>,
) -> Response {
    if !authed(&st, &h) {
        return err(StatusCode::UNAUTHORIZED, "unauthorized");
    }
    let id = match pid(p) {
        Ok(i) => i,
        Err(r) => return r,
    };
    match st.repo.delete(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => err(StatusCode::NOT_FOUND, "not found"),
        Err(e) => ise(e),
    }
}

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/api/ragtime/questions", get(list).post(create))
        .route(
            "/api/ragtime/questions/:id",
            get(get_one).put(put_one).delete(del_one),
        )
        .with_state(state)
}
