# FalseGreen Ledger — 0.23.0 full-scan results (2026-11-13)

Job `f0821601-412c-47e9-9048-962882e88994` — **ACCEPTED, attempt 1**. One unit spent at the terminal verdict (Model W).

| Surface | Result |
|---|---|
| Criteria (4) | `build`, `tests`, `clippy` (-D warnings), `formatting` — **all green** |
| Proof rungs (frozen invariants) | **2 PROVEN** — Kani (`encoding_sha256 2eb7adea…`) + Creusot (`9e9afede…`), same claim: *chain linking is exact successor arithmetic and never wraps at the u64 ceiling* |
| Whole-crate scan | scope `crate:falsegreen-ledger`, 11 public functions: **1 Kani PROVEN** (`rules::links`, digest `ec8624ea…` lineage), **1 tested** (`rules::shard_name` — deterministic Bolero inputs, a test not a proof), 9 honestly `not_eligible` (reason named per function) |
| Creusot scan rung | 4 goals closed / 2 open (rules/shard_index, rules/shard_name) / 7 targets unattempted — recorded, never gating |
| Declared-model taint (S4) | **model satisfied** — no declared flow among 2 sources x 2 sinks through 3 validators (2.6 s) |
| Dependency/dead-crate census | parsed clean, zero ghost deps; surface: 0 FFI, 0 dynamic-load, 0 unsafe — inventory-only posture |
| Benchmark `append_throughput` | median **91,607 events/s** (runs 90.2k–113.3k) — inside the frozen band 90k–150k, **not breached** |
| Security classes (5) | SCA pass (0 findings) · SAST pass · secret pass · supply-chain pass · licence `not assessed` (no policy frozen) |
| Machine time | wall 615.9 s · CPU 1,282 s · peak RSS 2.56 GiB (0.22.0 B2 numbers, first customer-visible release) |

## Integrity

- Signed JSON + PDF archived in this repo: `reports/f0821601-412c-47e9-9048-962882e88994.json`, `reports/f0821601-412c-47e9-9048-962882e88994.pdf`
- Local keyless verification: **VERIFIED** (Ed25519, embedded public key, digest `d247f817f0b5…`, report schema 17)
- Tampered copy (verdict flipped): **REFUSED** — `NOT_A_JSON_REPORT` before any trust was placed

## Links

- Human report: <https://falsegreen.com/report/f0821601-412c-47e9-9048-962882e88994>
- JSON: <https://falsegreen.com/report/f0821601-412c-47e9-9048-962882e88994/json> · PDF: <https://falsegreen.com/report/f0821601-412c-47e9-9048-962882e88994/pdf>

## What changed vs the 0.22.0 runs

- Proof rows now carry the discharged proposition verbatim + a real `encoding_sha256` (R-Proof0) — no more "proven" with no subject.
- The Creusot scan rung resolves fully offline (sealed vendor world) — `closed 4` where 0.22.0 reported "no matching package" (rows 5–7).
- The cover states scan completeness honestly (R-Scan2), the verdict carries a WHY block (12b), and dependency findings would name their introduction chain (row 11/12 — zero findings here, so unexercised).
