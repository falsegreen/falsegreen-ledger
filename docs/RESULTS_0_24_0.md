# FalseGreen 0.24.0 campaign — falsegreen-ledger regression gate

**Job `ade30518-e71d-436f-b2dc-f143a71b2888` — ACCEPTED, attempt 1** (one spend, Model W).
Signed report: <https://falsegreen.com/report/ade30518-e71d-436f-b2dc-f143a71b2888>
(PDF `/pdf`, signed JSON `/json`; both committed under `reports/`).
Keyless verification with the 0.24.0-tree shim: **VERIFIED** — schema v17, digest
`5c20215914459edec25307bb1a4e982e1791561ccb626ff2ef97988f938e6567`, key
`fg-report-ab72bef1-…-20261103-1`.

Run on the **0.24.0 shipping job image** (024rc1, includes the Kani
capability-refusal fix and the `falsegreen-testgen` engine cutover).

## What this gate proves

The 0.24 proof stack keeps this crate exactly as verified as the 0.23 twin
(`f0821601`, accepted, 615.9 s wall / 1,282 s CPU):

| dimension | 0.23 (`f0821601`) | 0.24 (`ade30518`) |
|---|---|---|
| verdict | accepted (attempt 1) | accepted (attempt 1) |
| wall / machine | 615.9 s / 1,282 s | 612.5 s / 1,250 s |
| criteria | 4/4 | 4/4 |
| invariant rungs | kani PROVEN + creusot PROVEN (both with `encoding_sha256`) | kani PROVEN + creusot PROVEN (both with `encoding_sha256`) |
| security classes | 4 pass + licence NOT ASSESSED | 4 pass + licence NOT ASSESSED |
| benchmark `append_throughput` | 91,607 ev/s median, inside band | 129,357 ev/s median, inside band |
| scan scope / rows | `crate:falsegreen-ledger` — 1 proven, 1 tested, 9 not-eligible | identical |

The stable-twin row reads `stable twin ran clean; kani could not run (harness
budget 60 s)` where 0.23 said `bolero ran clean; …` — the **engine cutover is a
clean swap, not a regression**: same row shape, same outcome, now running
`falsegreen_testgen` with zero bolero vendored in the image.

## Honest measurement notes

- **Benchmark delta (+41 % median) is NOT a product or code win.** The bench
  harness and the crate code are unchanged; the 0.23 and 0.24 runs share the
  crate `9f65b001719c` tree. A single Job's measured throughput on a shared
  box is recorded evidence, not an improvement claim — the frozen bar is what
  gates, and both runs sit inside it.
- **The tree hash differs** (`4ef424e76888` vs `9f65b001719c`) because the
  published selection now carries the previously saved signed reports in
  `reports/` — content-addressed scope, expected.
- **The scan stage dominates** (524.3 s of 612.5 s wall; 1,015 s of 1,250 s
  CPU), essentially unchanged from 0.23 (526.3 s). The 0.24 run-internal build
  cache shows no measurable saving on this one-attempt crate — its value is
  per-repair-attempt, which this accepted attempt-1 Job never exercised.
