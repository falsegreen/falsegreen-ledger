//! The wire floor of the web layer: health, the seeded verified board,
//! and an append through the form path that lands on the chain (with the
//! core's validator refusing junk BEFORE the sink).

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // oneshot

use falsegreen_ledger_web::{router, App};

fn app(dir: &tempfile::TempDir) -> Arc<App> {
    Arc::new(App::new(dir.path().to_str().expect("utf-8 root")))
}

async fn get_json(app: &Arc<App>, uri: &str) -> serde_json::Value {
    let res = router(app.clone())
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn health_is_ok() {
    let dir = tempfile::TempDir::new().unwrap();
    let app = app(&dir);
    let res = router(app.clone())
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn the_seeded_board_is_verified_and_served() {
    let dir = tempfile::TempDir::new().unwrap();
    let app = app(&dir);
    let state = get_json(&app, "/api/state").await;
    assert_eq!(
        state["verified"],
        serde_json::json!(true),
        "the seed must chain"
    );
    assert_eq!(state["total_records"], serde_json::json!(3));
    let res = router(app.clone())
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("CHAIN VERIFIED"));
    assert!(html.contains("Audit trail"));
}

#[tokio::test]
async fn a_form_append_lands_on_the_chain_and_junk_is_refused() {
    let dir = tempfile::TempDir::new().unwrap();
    let app = app(&dir);

    // A good form: 303 redirect, and the record count grows.
    let body = "actor_kind=user&actor_id=bevan&action_kind=tool_call&action_name=edit&target=src%2Fmain.rs&outcome=allowed";
    let res = router(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::SEE_OTHER);
    let state = get_json(&app, "/api/state").await;
    assert_eq!(state["total_records"], serde_json::json!(4));
    assert_eq!(state["verified"], serde_json::json!(true));

    // A junk form (empty action name): refused by the core validator,
    // the chain does NOT grow.
    let junk = "actor_kind=user&actor_id=bevan&action_kind=tool_call&action_name=&target=x&outcome=allowed";
    let res = router(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(junk))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let state = get_json(&app, "/api/state").await;
    assert_eq!(
        state["total_records"],
        serde_json::json!(4),
        "the sink was never reached"
    );
}
