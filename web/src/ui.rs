//! The Dioxus component tree of the audit board (server-rendered; the
//! same component type the 0.28.0 UI evidence lane's VirtualDom seam
//! observes — native, deterministic, no browser in the render path).

use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Row {
    pub seq: u64,
    pub ts_ms: i64,
    pub actor: String,
    pub action: String,
    pub target: String,
    pub outcome: String,
    pub hash: String,
}

#[derive(Clone, PartialEq)]
pub struct Board {
    pub verified: bool,
    pub total_records: u64,
    pub head_hash: String,
    pub rows: Vec<Row>,
    pub notice: Option<String>,
    pub error: Option<String>,
}

fn clock_label(ts_ms: i64) -> String {
    let secs = ts_ms / 1000;
    let d = secs / 86_400;
    let h = (secs % 86_400) / 3_600;
    let m = (secs % 3_600) / 60;
    let s = secs % 60;
    format!("{d}d {h:02}:{m:02}:{s:02}")
}

#[component]
pub fn App(board: Board) -> Element {
    rsx! {
        head { title { "falsegreen-ledger — audit board" } }
        body {
            style { {CSS} }
            main {
                class: "board",
                header {
                    class: "top",
                    span { class: "brand",
                        "falsegreen" span { class: "dot", "." } "audit-board"
                    }
                    h1 { "Audit trail" }
                    p {
                        class: if board.verified { "badge ok" } else { "badge bad" },
                        if board.verified { "CHAIN VERIFIED" } else { "CHAIN BROKEN" }
                    }
                    p { class: "muted",
                        "records: {board.total_records}  head: {short(&board.head_hash)}"
                    }
                }
                if let Some(notice) = board.notice {
                    p { class: "notice", {notice} }
                }
                if let Some(err) = board.error {
                    p { class: "error", {err} }
                }
                table {
                    thead { tr {
                        th { "seq" } th { "age" } th { "actor" }
                        th { "action" } th { "target" } th { "decision" } th { "hash" }
                    } }
                    tbody {
                        for row in board.rows.iter().rev() {
                            tr {
                                td { {row.seq.to_string()} }
                                td { {clock_label(row.ts_ms)} }
                                td { {row.actor.clone()} }
                                td { {row.action.clone()} }
                                td { class: "mono", {row.target.clone()} }
                                td {
                                    class: if row.outcome == "allowed" { "ok" } else { "warn" },
                                    {row.outcome.clone()}
                                }
                                td { class: "mono", {short(&row.hash)} }
                            }
                        }
                    }
                }
                section {
                    h2 { "Append an event" }
                    form {
                        method: "post",
                        action: "/events",
                        label { "actor kind" input { name: "actor_kind", value: "agent" } }
                        label { "actor id" input { name: "actor_id", value: "omp-main" } }
                        label { "action kind" input { name: "action_kind", value: "tool_call" } }
                        label { "action name" input { name: "action_name", value: "shell.exec" } }
                        label { "target" input { name: "target", value: "cargo test -p falsegreen-ledger" } }
                        label { "outcome" input { name: "outcome", value: "allowed" } }
                        button { r#type: "submit", "append"}
                    }
                }
                footer { p { class: "muted", "tamper-evident hash chain — falsegreen-ledger 0.2.0" } }
            }
        }
    }
}

fn short(hex: &str) -> String {
    hex.chars().take(12).collect()
}

const CSS: &str = r#"
/* falsegreen.com design language (0.2.0): same tokens as the marketing
   site — -apple-system stack, body #222222 on #f8f9fa, primary teal
   #0c8e99, radius 13px, soft navy-tinted shadows. */
:root {
  --fg: #222222; --muted: #6c757d; --bg: #f8f9fa; --surface: #ffffff;
  --border: #dee2e6; --primary: #0c8e99; --primary-dark: #08767f;
  --primary-soft: #e7f4f5; --success: #157347; --success-soft: #e8f5ee;
  --danger: #b02a37; --danger-soft: #fbe9eb; --radius: 13px;
  --shadow: 0px 6px 24px -6px rgba(45, 67, 121, 0.15);
}
* { box-sizing: border-box; }
body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
    "Helvetica Neue", Arial, sans-serif;
  background: var(--bg); color: var(--fg); margin: 0;
  font-size: 16px; line-height: 1.6;
}
main.board { max-width: 1080px; margin: 0 auto; padding: 28px 20px 48px; }
header.top { display: flex; align-items: center; gap: 16px; flex-wrap: wrap;
  padding-bottom: 14px; border-bottom: 1px solid var(--border); }
.brand { font-weight: 700; letter-spacing: -0.02em; color: var(--primary); }
.brand .dot { color: var(--primary-dark); }
h1 { font-size: clamp(20px, 3.2vw, 26px); font-weight: 700; margin: 0;
  letter-spacing: -0.02em; }
h2 { font-size: 17px; font-weight: 700; letter-spacing: -0.01em; }
.badge { padding: 3px 12px; border-radius: 999px; font-size: 12px; font-weight: 600;
  letter-spacing: 0.04em; }
.badge.ok { background: var(--success-soft); color: var(--success); }
.badge.bad { background: var(--danger-soft); color: var(--danger); }
.muted { color: var(--muted); font-size: 14px; margin: 4px 0; }
section.card { background: var(--surface); border: 1px solid var(--border);
  border-radius: var(--radius); box-shadow: var(--shadow); padding: 18px 20px;
  margin: 18px 0; }
table { border-collapse: collapse; width: 100%; font-size: 14px; }
th { color: var(--muted); font-weight: 600; text-align: left;
  border-bottom: 2px solid var(--border); padding: 8px 10px; }
td { border-bottom: 1px solid var(--border); padding: 8px 10px; }
tbody tr:hover { background: var(--primary-soft); }
.mono { font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
  font-size: 13px; color: var(--muted); }
.ok { color: var(--success); font-weight: 600; } .warn { color: var(--danger); font-weight: 600; }
.notice { color: var(--success); } .error { color: var(--danger); }
form { display: grid; grid-template-columns: repeat(3, minmax(180px, 1fr)); gap: 12px; }
label { display: grid; gap: 4px; font-size: 13px; color: var(--muted); }
input { background: var(--surface); color: var(--fg); border: 1px solid var(--border);
  border-radius: var(--radius); padding: 8px 10px; }
input:focus { outline: none; border-color: var(--primary); box-shadow: 0 0 0 3px var(--primary-soft); }
button { background: var(--primary); color: #fff; border: 0;
  border-radius: var(--radius); padding: 10px 18px; font-weight: 600; cursor: pointer; }
button:hover { background: var(--primary-dark); }
footer { margin-top: 32px; }
"#;
