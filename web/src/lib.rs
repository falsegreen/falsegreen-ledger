//! The audit board server: an axum API over the falsegreen-ledger core
//! plus a server-rendered Dioxus front end. One binary, full stack.
//!
//! The transport-free core is what the contract verifies; this crate is
//! thin on purpose: read the chain, render it, append events through the
//! core's own validation. The taint posture the contract freezes treats
//! the web form as the untrusted source (`parse_event_form`), the core's
//! validation as the validator (`Event::validate` via
//! `validate_event_form`), and the chaining functions as the sinks.

pub mod ui;

use std::sync::Arc;

use axum::extract::{Form, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::Router;

use falsegreen_ledger::{
    Action, Actor, Correlation, Decision, Event, Ledger, Record, GENESIS_HASH,
};

pub struct App {
    ledger: Ledger,
}

impl App {
    pub fn new(root: &str) -> Self {
        let ledger = Ledger::init(root).expect("init the ledger");
        let app = Self { ledger };
        app.seed().expect("seed the demo events");
        app
    }

    /// A short demo trail so a fresh board is a board, not an empty page.
    fn seed(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (last, _) = self.ledger.head()?;
        if last > 0 {
            return Ok(());
        }
        let now = now_ms();
        let demo = |kind: &str, name: &str, target: &str, outcome: &str| Event {
            seq: 0,
            ts_ms: now,
            actor: Actor {
                kind: "agent".into(),
                id: "omp-main".into(),
                version: None,
                on_behalf_of: Some("bevan".into()),
            },
            action: Action {
                kind: kind.into(),
                name: name.into(),
                target: Some(target.into()),
            },
            correlation: Correlation {
                session: "demo-1".into(),
                request: None,
            },
            decision: Decision {
                outcome: outcome.into(),
                policy_version: Some("v1".into()),
                reason: None,
            },
            context: serde_json::json!({}),
        };
        self.ledger.append(&[
            demo(
                "tool_call",
                "shell.exec",
                "cargo test -p falsegreen-ledger",
                "allowed",
            ),
            demo(
                "decision",
                "policy.eval",
                "allow-list: cargo test",
                "allowed",
            ),
            demo("tool_call", "shell.exec", "rm -rf /", "denied"),
        ])?;
        Ok(())
    }

    /// Append a batch (the benchmark harness path; the form path appends
    /// one at a time through the same core call).
    pub fn append_many(&self, events: &[falsegreen_ledger::Event]) -> Result<usize, String> {
        self.ledger
            .append(events)
            .map(|r| r.len())
            .map_err(|e| e.to_string())
    }

    /// Read every record from the sealed + active shards (the shard files
    /// are the ledger's public on-disk format; seal lines are skipped).
    fn records(&self) -> Vec<Record> {
        let mut out = Vec::new();
        for index in self.ledger.shard_indices().unwrap_or_default() {
            let Ok(text) = std::fs::read_to_string(self.ledger.shard_path(index)) else {
                continue;
            };
            for line in text.lines() {
                if let Ok(record) = serde_json::from_str::<Record>(line) {
                    out.push(record);
                }
            }
        }
        out
    }

    pub fn board(&self, notice: Option<String>, error: Option<String>) -> ui::Board {
        let report = falsegreen_ledger::verify_ledger(&self.ledger, None).ok();
        let verified = report.as_ref().is_some_and(|r| r.verified);
        let total = report.as_ref().map(|r| r.total_records).unwrap_or(0);
        let head = self
            .ledger
            .head()
            .map(|(_, h)| h)
            .unwrap_or_else(|_| GENESIS_HASH.into());
        let rows = self
            .records()
            .into_iter()
            .map(|r| ui::Row {
                seq: r.event.seq,
                ts_ms: r.event.ts_ms,
                actor: format!("{}:{}", r.event.actor.kind, r.event.actor.id),
                action: format!("{}:{}", r.event.action.kind, r.event.action.name),
                target: r.event.action.target.unwrap_or_default(),
                outcome: r.event.decision.outcome,
                hash: r.hash,
            })
            .collect();
        ui::Board {
            verified,
            total_records: total,
            head_hash: head,
            rows,
            notice,
            error,
        }
    }
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// THE untrusted entry point (the contract's S4 taint source): the raw
/// web form. It only becomes an `Event` after `validate_event_form`
/// (the core's own `Event::validate`) and an append that the core
/// re-validates and chains.
pub fn parse_event_form(form: &Form<EventForm>) -> Event {
    let f = &form.0;
    Event {
        seq: 0, // the core rewrites seq at append time
        ts_ms: now_ms(),
        actor: Actor {
            kind: f.actor_kind.clone(),
            id: f.actor_id.clone(),
            version: None,
            on_behalf_of: None,
        },
        action: Action {
            kind: f.action_kind.clone(),
            name: f.action_name.clone(),
            target: Some(f.target.clone()),
        },
        correlation: Correlation {
            session: "web".into(),
            request: None,
        },
        decision: Decision {
            outcome: f.outcome.clone(),
            policy_version: None,
            reason: None,
        },
        context: serde_json::json!({}),
    }
}

/// The validator the contract names: core validation BEFORE the append.
pub fn validate_event_form(event: &Event) -> Result<(), String> {
    event.validate().map_err(|e| format!("{e:?}"))
}

#[derive(serde::Deserialize, Clone)]
pub struct EventForm {
    actor_kind: String,
    actor_id: String,
    action_kind: String,
    action_name: String,
    target: String,
    outcome: String,
}

/// The sink the contract names: the append into the hash chain.
pub fn append_records(app: &App, event: Event) -> Result<usize, String> {
    app.ledger
        .append(&[event])
        .map(|records| records.len())
        .map_err(|e| format!("{e}"))
}

async fn index(State(app): State<Arc<App>>) -> Response {
    let html = render_board(app.board(None, None));
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    (headers, html).into_response()
}

async fn events(
    State(app): State<Arc<App>>,
    form: Result<Form<EventForm>, axum::extract::rejection::FormRejection>,
) -> Response {
    let (status, notice, error) = match form {
        Ok(form) => {
            let event = parse_event_form(&form);
            match validate_event_form(&event) {
                Ok(()) => match append_records(&app, event) {
                    Ok(n) => (
                        StatusCode::SEE_OTHER,
                        Some(format!("appended {n} record(s)")),
                        None,
                    ),
                    Err(e) => (StatusCode::BAD_REQUEST, None, Some(e)),
                },
                Err(e) => (StatusCode::BAD_REQUEST, None, Some(e)),
            }
        }
        Err(rej) => (
            StatusCode::BAD_REQUEST,
            None,
            Some(format!("bad form: {rej}")),
        ),
    };
    if status == StatusCode::SEE_OTHER {
        return Redirect::to("/").into_response();
    }
    (status, Html(render_board(app.board(notice, error)))).into_response()
}

async fn health() -> &'static str {
    "ok"
}

async fn state(State(app): State<Arc<App>>) -> impl IntoResponse {
    let board = app.board(None, None);
    axum::Json(serde_json::json!({
        "verified": board.verified,
        "total_records": board.total_records,
        "head_hash": board.head_hash,
        "rows": board.rows.iter().map(|r| serde_json::json!({
            "seq": r.seq, "actor": r.actor, "action": r.action,
            "target": r.target, "outcome": r.outcome, "hash": r.hash,
        })).collect::<Vec<_>>(),
    }))
}

/// Render the board through the frozen Dioxus VirtualDom seam (the same
/// observation surface the UI evidence lane uses — headless, no browser).
pub fn render_board(board: ui::Board) -> String {
    // The component tree IS a VirtualDom root element; the SSR renderer
    // builds and renders it headless (the same seam the UI evidence lane
    // observes in 0.28.0).
    dioxus::ssr::render_element(ui::App(ui::AppProps { board }))
}

pub fn router(app: Arc<App>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/events", post(events))
        .route("/health", get(health))
        .route("/api/state", get(state))
        .with_state(app)
}

pub fn run() {
    let root = std::env::var("LEDGER_ROOT").unwrap_or_else(|_| "ledger-data".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8090);
    let app = Arc::new(App::new(&root));
    let rt = tokio::runtime::Runtime::new().expect("tokio");
    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
            .await
            .expect("bind");
        eprintln!("falsegreen-ledger web: http://0.0.0.0:{port} (ledger root: {root})");
        axum::serve(listener, router(app)).await.expect("serve");
    });
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn the_ssr_board_renders_the_badge_and_the_form() {
        let html = render_board(ui::Board {
            verified: true,
            total_records: 1,
            head_hash: "abc".repeat(20),
            rows: vec![ui::Row {
                seq: 1,
                ts_ms: 1_000,
                actor: "agent:x".into(),
                action: "tool_call:y".into(),
                target: "t".into(),
                outcome: "allowed".into(),
                hash: "d".repeat(64),
            }],
            notice: None,
            error: None,
        });
        assert!(html.contains("CHAIN VERIFIED"));
        assert!(html.contains("agent:x"));
        assert!(html.contains("action=\"/events\""));
    }

    #[test]
    fn the_core_validator_blocks_an_empty_action_name_before_the_sink() {
        let form = Form(EventForm {
            actor_kind: "agent".into(),
            actor_id: "x".into(),
            action_kind: "tool_call".into(),
            action_name: String::new(),
            target: "t".into(),
            outcome: "allowed".into(),
        });
        let event = parse_event_form(&form);
        assert!(
            validate_event_form(&event).is_err(),
            "empty action name must not reach the chain"
        );
    }
}
