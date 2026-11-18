//! The binary: the audit board server (axum + Dioxus SSR over the
//! falsegreen-ledger core). All logic lives in the lib so the tests and
//! the benchmark drive the same router the process serves.

fn main() {
    falsegreen_ledger_web::run();
}
