# Tier-3 mirror perf fixtures (C1)

Phase 1 benchmarks in `../tier3_mirror_perf.rs` currently use **inline**
deterministic inputs (Peano depth 32, fixed `DescentEvidence` variants, a small
linear idempotent workflow). Shared corpus files for stricter like-for-like
Phase 1 vs Phase 2 comparison land when the Evaluator-backed bench harness
(`tier3_eval_perf.rs`, worker brief deliverable 2) is authored.

See [`docs/briefs/r3-pb-tier3-perf-budget-worker.md`](../../../../../docs/briefs/r3-pb-tier3-perf-budget-worker.md).
