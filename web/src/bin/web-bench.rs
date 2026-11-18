//! The web layer's own harness, frozen as a contract `[[benchmark]]`:
//! the SSR render path (VirtualDom build + render over a populated
//! board) and the board snapshot path (verify the whole chain + read the
//! shards + assemble the props). Both run on the shipped code; the JSON
//! on stdout is what the contract's `json` measures read.

use std::sync::Arc;
use std::time::Instant;

use falsegreen_ledger_web::{render_board, App};

fn main() {
    let dir = std::env::temp_dir().join(format!("ledger-web-bench-{}", std::process::id()));
    let app = Arc::new(App::new(dir.to_str().expect("utf-8 root")));

    // A populated board: the seed plus 200 appended records (the shape a
    // working audit trail has, not an empty page).
    let mut events = Vec::new();
    for i in 0..200 {
        events.push(falsegreen_ledger::Event {
            seq: 0,
            ts_ms: 1_763_000_000_000 + i,
            actor: falsegreen_ledger::Actor {
                kind: "agent".into(),
                id: format!("bench-{i}"),
                version: None,
                on_behalf_of: None,
            },
            action: falsegreen_ledger::Action {
                kind: "tool_call".into(),
                name: "shell.exec".into(),
                target: Some(format!("cargo test -p crate-{i}")),
            },
            correlation: falsegreen_ledger::Correlation {
                session: "bench".into(),
                request: None,
            },
            decision: falsegreen_ledger::Decision {
                outcome: if i % 7 == 0 {
                    "denied".into()
                } else {
                    "allowed".into()
                },
                policy_version: Some("v1".into()),
                reason: None,
            },
            context: serde_json::json!({}),
        });
    }
    app.append_many(&events).expect("bench append");

    // 1) the render path: props -> VirtualDom -> SSR string.
    let board = app.board(None, None);
    let iterations = 300;
    let t0 = Instant::now();
    let mut total_bytes = 0usize;
    for _ in 0..iterations {
        total_bytes += render_board(board.clone()).len();
    }
    let render_s = t0.elapsed().as_secs_f64();
    let renders_per_sec = iterations as f64 / render_s;

    // 2) the snapshot path: verify_ledger + shard reads + props assembly.
    let snaps = 50;
    let t1 = Instant::now();
    for _ in 0..snaps {
        let b = app.board(None, None);
        assert!(b.verified, "the bench chain must verify");
    }
    let snap_s = t1.elapsed().as_secs_f64();
    let snapshots_per_sec = snaps as f64 / snap_s;

    println!(
        "{}",
        serde_json::json!({
            "renders_per_sec": renders_per_sec,
            "board_snapshots_per_sec": snapshots_per_sec,
            "records": 203,
            "html_bytes_per_render": total_bytes / iterations,
        })
    );
    println!(
        "renders_per_sec {} board_snapshots_per_sec {}",
        renders_per_sec as i64, snapshots_per_sec as i64
    );
    let _ = std::fs::remove_dir_all(&dir);
}
