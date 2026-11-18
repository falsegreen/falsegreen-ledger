//! Throughput bench for the ledger's append path: canonicalize → SHA-256 →
//! link check, one record at a time over the SHIPPED code in `src/chain.rs`
//! (the same functions the chain and the verifier call).
//!
//! The frozen `[[benchmark]]` section in `falsegreen.toml` runs this binary
//! and parses the JSON document line; the second line is the human-readable
//! form for the regex measure. It never opines about speed — the frozen bar
//! in the contract decides (a demonstrated breach fails the Job; without a
//! bar the numbers are recorded evidence only).
//!
//! The PROOF build (`cargo creusot`, `--cfg creusot`) compiles the whole
//! crate including this bin, but the crate root excludes the storage/CLI
//! modules under that cfg — so the bench compiles to a no-op main there.
//! The bench only ever runs on the stable toolchain.

#[cfg(creusot)]
fn main() {
    // The proof build compiles this bin too, and the creusot driver refuses
    // any crate in the build that depends on creusot-std without loading the
    // prelude — same requirement as `src/main.rs`. The bench itself stays a
    // no-op under this cfg.
    use creusot_std::prelude::*;
}

#[cfg(not(creusot))]
use std::time::Instant;

#[cfg(not(creusot))]
use falsegreen_ledger::{
    chain::GENESIS_HASH,
    event::{Action, Actor, Correlation, Decision, Event},
    Record,
};

/// The frozen workload size (part of what the measured number means).
#[cfg(not(creusot))]
const RECORDS: u64 = 200_000;

#[cfg(not(creusot))]
fn event(seq: u64) -> Event {
    Event {
        seq,
        ts_ms: 1_700_000_000_000 + seq as i64,
        actor: Actor {
            kind: "agent".into(),
            id: "bench".into(),
            version: None,
            on_behalf_of: Some("user-1".into()),
        },
        action: Action {
            kind: "tool_call".into(),
            name: "ledger.append".into(),
            target: Some("shard-0001".into()),
        },
        correlation: Correlation {
            session: "bench".into(),
            request: None,
        },
        decision: Decision {
            outcome: "allowed".into(),
            policy_version: Some("v1".into()),
            reason: None,
        },
        context: serde_json::Map::new().into(),
    }
}

#[cfg(not(creusot))]
fn main() {
    let mut prev_hash = GENESIS_HASH.to_string();
    let mut last: Option<Record> = None;
    let start = Instant::now();
    for seq in 1..=RECORDS {
        let rec = Record::new(event(seq), &prev_hash);
        assert!(rec.check_hash());
        if let Some(p) = &last {
            assert!(rec.check_link(p));
        }
        prev_hash = rec.hash.clone();
        last = Some(rec);
    }
    let secs = start.elapsed().as_secs_f64();
    let per_sec = RECORDS as f64 / secs;
    // JSON document line first (the `json` measure reads it), then the
    // human line (the `regex` measure reads it).
    println!("{{\"records\": {RECORDS}, \"events_per_sec\": {per_sec}}}");
    println!("append_events_per_sec {per_sec:.0}");
    let _ = last;
}

// e2e repair attempt 1 (the driver keeps trying; the code stays broken)

// e2e repair attempt 2 (the driver keeps trying; the code stays broken)

// e2e repair attempt 3 (the driver keeps trying; the code stays broken)

// e2e repair attempt 4 (the driver keeps trying; the code stays broken)

// e2e repair attempt 5 (the driver keeps trying; the code stays broken)

// e2e repair attempt 1 (the driver keeps trying; the code stays broken)

// e2e repair attempt 2 (the driver keeps trying; the code stays broken)

// e2e repair attempt 3 (the driver keeps trying; the code stays broken)

// e2e repair attempt 4 (the driver keeps trying; the code stays broken)

// e2e repair attempt 5 (the driver keeps trying; the code stays broken)

// e2e repair attempt 1 (the driver keeps trying; the code stays broken)

// e2e repair attempt 2 (the driver keeps trying; the code stays broken)

// e2e repair attempt 3 (the driver keeps trying; the code stays broken)

// e2e repair attempt 4 (the driver keeps trying; the code stays broken)

// e2e repair attempt 1 (the driver keeps trying; the code stays broken)

// e2e repair attempt 2 (the driver keeps trying; the code stays broken)

// e2e repair attempt 3 (the driver keeps trying; the code stays broken)
