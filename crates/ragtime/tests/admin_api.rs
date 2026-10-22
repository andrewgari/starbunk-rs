//! Contract tests for the ragtime admin API (TDD PR 1).
//!
//! Drive `ragtime::api::router` in-process with `tower::ServiceExt::oneshot`
//! against an in-memory `QuestionRepository`, so no Postgres is needed and they
//! run in CI. They FAIL against the PR 1 stub (empty router → 404 everywhere)
//! and must pass unchanged once PR 2 implements the routes.
//!
//! Wire contract matches `starbunk-ui/src/lib/ragtime-api.ts`.

use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use chrono::Utc;
use http_body_util::BodyExt;
use ragtime::api::{router, ApiState, ADMIN_TOKEN_ENV};
use ragtime::question::{NewQuestion, Question, QuestionId, QuestionUpdate};
use ragtime::repository::QuestionRepository;
use serde_json::{json, Value};
use tower::ServiceExt;

const TOKEN: &str = "testtoken";
const BASE: &str = "/api/ragtime/questions";

// ---------- in-memory repository ----------

#[derive(Default)]
struct MemRepo {
    rows: Mutex<Vec<Question>>,
    next: Mutex<i64>,
    fail: bool,
}

impl MemRepo {
    fn failing() -> Self {
        Self {
            fail: true,
            ..Default::default()
        }
    }
    fn check(&self) -> anyhow::Result<()> {
        if self.fail {
            anyhow::bail!("db down")
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl QuestionRepository for MemRepo {
    async fn list(&self) -> anyhow::Result<Vec<Question>> {
        self.check()?;
        let mut v = self.rows.lock().unwrap().clone();
        v.sort_by_key(|q| q.id);
        Ok(v)
    }
    async fn get(&self, id: QuestionId) -> anyhow::Result<Option<Question>> {
        self.check()?;
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|q| q.id == id)
            .cloned())
    }
    async fn create(&self, new: NewQuestion) -> anyhow::Result<Question> {
        self.check()?;
        let mut n = self.next.lock().unwrap();
        *n += 1;
        let now = Utc::now();
        let q = Question {
            id: *n,
            question: new.question,
            display_answer: new.display_answer,
            acceptable_answers: new.acceptable_answers,
            created_at: now,
            updated_at: now,
        };
        self.rows.lock().unwrap().push(q.clone());
        Ok(q)
    }
    async fn update(&self, id: QuestionId, c: QuestionUpdate) -> anyhow::Result<Option<Question>> {
        self.check()?;
        let mut rows = self.rows.lock().unwrap();
        let Some(q) = rows.iter_mut().find(|q| q.id == id) else {
            return Ok(None);
        };
        if let Some(v) = c.question {
            q.question = v
        }
        if let Some(v) = c.display_answer {
            q.display_answer = v
        }
        if let Some(v) = c.acceptable_answers {
            q.acceptable_answers = v
        }
        q.updated_at = Utc::now();
        Ok(Some(q.clone()))
    }
    async fn delete(&self, id: QuestionId) -> anyhow::Result<bool> {
        self.check()?;
        let mut rows = self.rows.lock().unwrap();
        let before = rows.len();
        rows.retain(|q| q.id != id);
        Ok(rows.len() != before)
    }
    async fn random(&self, n: usize) -> anyhow::Result<Vec<Question>> {
        self.check()?;
        Ok(self.rows.lock().unwrap().iter().take(n).cloned().collect())
    }
}

// ---------- helpers ----------

fn state(repo: Arc<MemRepo>) -> ApiState {
    ApiState {
        repo,
        admin_token: Some(TOKEN.into()),
    }
}

fn new_q(n: usize) -> NewQuestion {
    NewQuestion {
        question: format!("Question {n}?"),
        display_answer: format!("Answer {n}"),
        acceptable_answers: vec![format!("answer {n}")],
    }
}

async fn seeded(count: usize) -> (Arc<MemRepo>, Vec<Question>) {
    let repo = Arc::new(MemRepo::default());
    let mut out = vec![];
    for n in 0..count {
        out.push(repo.create(new_q(n)).await.unwrap());
    }
    (repo, out)
}

async fn send(
    st: ApiState,
    method: Method,
    uri: &str,
    auth: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Option<Value>, String) {
    let mut b = Request::builder().method(method).uri(uri);
    if let Some(a) = auth {
        b = b.header(header::AUTHORIZATION, a);
    }
    let req = match body {
        Some(v) => b
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => b.body(Body::empty()).unwrap(),
    };
    let res = router(st).oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&bytes).to_string();
    (status, serde_json::from_slice(&bytes).ok(), text)
}

fn bearer() -> Option<&'static str> {
    Some("Bearer testtoken")
}

fn assert_question_shape(v: &Value) {
    for k in [
        "id",
        "question",
        "display_answer",
        "acceptable_answers",
        "created_at",
        "updated_at",
    ] {
        assert!(v.get(k).is_some(), "missing field `{k}` in {v}");
    }
    assert!(v["id"].is_i64());
    assert!(v["acceptable_answers"].is_array());
    chrono::DateTime::parse_from_rfc3339(v["created_at"].as_str().unwrap())
        .expect("created_at must be RFC 3339");
}

// ---------- auth ----------

#[tokio::test]
async fn every_route_rejects_missing_token() {
    let (repo, qs) = seeded(1).await;
    let one = format!("{BASE}/{}", qs[0].id);
    let body = json!({"question":"q","display_answer":"a","acceptable_answers":["a"]});
    let cases = [
        (Method::GET, BASE.to_string(), None),
        (Method::POST, BASE.to_string(), Some(body.clone())),
        (Method::GET, one.clone(), None),
        (Method::PUT, one.clone(), Some(body)),
        (Method::DELETE, one, None),
    ];
    for (m, uri, b) in cases {
        let (s, _, _) = send(state(repo.clone()), m.clone(), &uri, None, b).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED, "{m} {uri}");
    }
    assert_eq!(
        repo.list().await.unwrap().len(),
        1,
        "unauthorized requests must not mutate"
    );
}

#[tokio::test]
async fn wrong_or_malformed_token_is_rejected() {
    let (repo, _) = seeded(1).await;
    for auth in [
        "Bearer nope",
        "testtoken",
        "Basic testtoken",
        "Bearer testtoken2",
        "bearer",
    ] {
        let (s, _, _) = send(state(repo.clone()), Method::GET, BASE, Some(auth), None).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED, "auth header {auth:?}");
    }
}

#[tokio::test]
async fn server_without_token_fails_closed() {
    let (repo, _) = seeded(1).await;
    for tok in [None, Some(String::new())] {
        let st = ApiState {
            repo: repo.clone(),
            admin_token: tok.clone(),
        };
        let (s, _, _) = send(st, Method::GET, BASE, Some("Bearer "), None).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED, "admin_token={tok:?}");
    }
}

#[tokio::test]
async fn valid_token_is_accepted() {
    let (repo, _) = seeded(0).await;
    let (s, _, _) = send(state(repo), Method::GET, BASE, bearer(), None).await;
    assert_eq!(s, StatusCode::OK);
}

#[test]
fn admin_token_from_env_reads_ragtime_admin_token() {
    assert_eq!(ADMIN_TOKEN_ENV, "RAGTIME_ADMIN_TOKEN");
    std::env::set_var(ADMIN_TOKEN_ENV, "s3cret");
    assert_eq!(
        ragtime::api::admin_token_from_env().as_deref(),
        Some("s3cret")
    );
    std::env::set_var(ADMIN_TOKEN_ENV, "   ");
    assert_eq!(
        ragtime::api::admin_token_from_env(),
        None,
        "blank token must be treated as unset"
    );
    std::env::remove_var(ADMIN_TOKEN_ENV);
    assert_eq!(ragtime::api::admin_token_from_env(), None);
}

// ---------- list / get ----------

#[tokio::test]
async fn list_returns_array_ordered_by_id() {
    let (repo, qs) = seeded(3).await;
    let (s, v, _) = send(state(repo), Method::GET, BASE, bearer(), None).await;
    assert_eq!(s, StatusCode::OK);
    let arr = v.unwrap();
    let arr = arr.as_array().expect("list must be a bare JSON array");
    let ids: Vec<i64> = arr.iter().map(|q| q["id"].as_i64().unwrap()).collect();
    assert_eq!(ids, qs.iter().map(|q| q.id).collect::<Vec<_>>());
    arr.iter().for_each(assert_question_shape);
}

#[tokio::test]
async fn list_empty_is_empty_array() {
    let (repo, _) = seeded(0).await;
    let (s, v, _) = send(state(repo), Method::GET, BASE, bearer(), None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v.unwrap(), json!([]));
}

#[tokio::test]
async fn get_returns_question() {
    let (repo, qs) = seeded(2).await;
    let q = &qs[1];
    let (s, v, _) = send(
        state(repo),
        Method::GET,
        &format!("{BASE}/{}", q.id),
        bearer(),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let v = v.unwrap();
    assert_question_shape(&v);
    assert_eq!(v["id"], q.id);
    assert_eq!(v["question"], q.question);
    assert_eq!(v["display_answer"], q.display_answer);
    assert_eq!(v["acceptable_answers"], json!(q.acceptable_answers));
}

#[tokio::test]
async fn get_missing_is_404_and_bad_id_is_400() {
    let (repo, _) = seeded(1).await;
    let (s, v, _) = send(
        state(repo.clone()),
        Method::GET,
        &format!("{BASE}/999"),
        bearer(),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    assert!(
        v.and_then(|v| v.get("error").cloned()).is_some(),
        "404 body must be {{error}}"
    );
    let (s, _, _) = send(
        state(repo),
        Method::GET,
        &format!("{BASE}/abc"),
        bearer(),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

// ---------- create ----------

#[tokio::test]
async fn create_returns_201_and_persists() {
    let (repo, _) = seeded(0).await;
    let body = json!({"question":"Capital of France?","display_answer":"Paris","acceptable_answers":["paris","パリ"]});
    let (s, v, _) = send(
        state(repo.clone()),
        Method::POST,
        BASE,
        bearer(),
        Some(body),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    let v = v.unwrap();
    assert_question_shape(&v);
    assert_eq!(v["question"], "Capital of France?");
    assert_eq!(v["acceptable_answers"], json!(["paris", "パリ"]));
    let stored = repo
        .get(v["id"].as_i64().unwrap())
        .await
        .unwrap()
        .expect("persisted");
    assert_eq!(stored.display_answer, "Paris");
}

#[tokio::test]
async fn create_trims_and_drops_blank_answers() {
    let (repo, _) = seeded(0).await;
    let body = json!({"question":"  Q?  ","display_answer":" A ","acceptable_answers":[" a ","", "  ","b"]});
    let (s, v, _) = send(state(repo), Method::POST, BASE, bearer(), Some(body)).await;
    assert_eq!(s, StatusCode::CREATED);
    let v = v.unwrap();
    assert_eq!(v["question"], "Q?");
    assert_eq!(v["display_answer"], "A");
    assert_eq!(v["acceptable_answers"], json!(["a", "b"]));
}

#[tokio::test]
async fn create_rejects_invalid_bodies() {
    let (repo, _) = seeded(0).await;
    let bad = [
        json!({"question":"","display_answer":"A","acceptable_answers":[]}),
        json!({"question":"   ","display_answer":"A","acceptable_answers":[]}),
        json!({"question":"Q","display_answer":"","acceptable_answers":[]}),
        json!({"display_answer":"A","acceptable_answers":[]}),
        json!({"question":"Q","display_answer":"A","acceptable_answers":"a"}),
        json!("nope"),
    ];
    for b in bad {
        let (s, _, _) = send(
            state(repo.clone()),
            Method::POST,
            BASE,
            bearer(),
            Some(b.clone()),
        )
        .await;
        assert!(
            s == StatusCode::BAD_REQUEST || s == StatusCode::UNPROCESSABLE_ENTITY,
            "body {b} gave {s}"
        );
    }
    // Malformed JSON text.
    let req = Request::builder()
        .method(Method::POST)
        .uri(BASE)
        .header(header::AUTHORIZATION, "Bearer testtoken")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{not json"))
        .unwrap();
    let s = router(state(repo.clone()))
        .oneshot(req)
        .await
        .unwrap()
        .status();
    assert!(
        s.is_client_error() && s != StatusCode::NOT_FOUND,
        "malformed json gave {s}"
    );
    assert!(
        repo.list().await.unwrap().is_empty(),
        "invalid creates must not persist"
    );
}

// ---------- update ----------

#[tokio::test]
async fn put_replaces_fields_and_bumps_updated_at() {
    let (repo, qs) = seeded(2).await;
    let q = &qs[0];
    // UI sends `{ id, ...input }`; body id is ignored in favour of the path.
    let body =
        json!({"id": 12345, "question":"New?","display_answer":"New","acceptable_answers":["new"]});
    let (s, v, _) = send(
        state(repo.clone()),
        Method::PUT,
        &format!("{BASE}/{}", q.id),
        bearer(),
        Some(body),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let v = v.unwrap();
    assert_question_shape(&v);
    assert_eq!(v["id"], q.id);
    assert_eq!(v["question"], "New?");
    let stored = repo.get(q.id).await.unwrap().unwrap();
    assert_eq!(stored.acceptable_answers, vec!["new".to_string()]);
    assert!(stored.updated_at >= q.updated_at);
    assert!(repo.get(12345).await.unwrap().is_none());
    assert_eq!(
        repo.get(qs[1].id).await.unwrap().unwrap(),
        qs[1],
        "other rows untouched"
    );
}

#[tokio::test]
async fn put_missing_is_404_and_invalid_is_rejected() {
    let (repo, qs) = seeded(1).await;
    let ok = json!({"question":"Q","display_answer":"A","acceptable_answers":[]});
    let (s, _, _) = send(
        state(repo.clone()),
        Method::PUT,
        &format!("{BASE}/999"),
        bearer(),
        Some(ok),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let bad = json!({"question":"","display_answer":"A","acceptable_answers":[]});
    let (s, _, _) = send(
        state(repo.clone()),
        Method::PUT,
        &format!("{BASE}/{}", qs[0].id),
        bearer(),
        Some(bad),
    )
    .await;
    assert!(
        s == StatusCode::BAD_REQUEST || s == StatusCode::UNPROCESSABLE_ENTITY,
        "got {s}"
    );
    assert_eq!(
        repo.get(qs[0].id).await.unwrap().unwrap(),
        qs[0],
        "invalid update must not mutate"
    );
}

// ---------- delete ----------

#[tokio::test]
async fn delete_returns_204_then_404() {
    let (repo, qs) = seeded(2).await;
    let uri = format!("{BASE}/{}", qs[0].id);
    let (s, _, text) = send(state(repo.clone()), Method::DELETE, &uri, bearer(), None).await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    assert!(text.is_empty());
    assert!(repo.get(qs[0].id).await.unwrap().is_none());
    assert!(repo.get(qs[1].id).await.unwrap().is_some());
    let (s, _, _) = send(state(repo), Method::DELETE, &uri, bearer(), None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
}

// ---------- errors ----------

#[tokio::test]
async fn repository_failure_is_500_with_error_body() {
    let repo = Arc::new(MemRepo::failing());
    let (s, v, text) = send(state(repo), Method::GET, BASE, bearer(), None).await;
    assert_eq!(s, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(
        v.and_then(|v| v.get("error").cloned()).is_some(),
        "500 body must be {{error}}"
    );
    assert!(
        !text.contains("db down"),
        "internal error details must not leak: {text}"
    );
}

#[tokio::test]
async fn full_crud_round_trip() {
    let (repo, _) = seeded(0).await;
    let st = || state(repo.clone());
    let (s, v, _) = send(
        st(),
        Method::POST,
        BASE,
        bearer(),
        Some(json!({"question":"Q","display_answer":"A","acceptable_answers":["a"]})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    let id = v.unwrap()["id"].as_i64().unwrap();
    let one = format!("{BASE}/{id}");
    let (s, v, _) = send(
        st(),
        Method::PUT,
        &one,
        bearer(),
        Some(json!({"question":"Q2","display_answer":"A2","acceptable_answers":["a2"]})),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v.unwrap()["question"], "Q2");
    let (_, v, _) = send(st(), Method::GET, BASE, bearer(), None).await;
    assert_eq!(v.unwrap().as_array().unwrap().len(), 1);
    let (s, _, _) = send(st(), Method::DELETE, &one, bearer(), None).await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    let (_, v, _) = send(st(), Method::GET, BASE, bearer(), None).await;
    assert_eq!(v.unwrap(), json!([]));
}
