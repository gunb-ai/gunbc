//! CI host support crate — per-run wall-clock timing ledger + self-hosted runner-pool facts.
//!
//! Axis-A consolidation (operator "go uniform", 2026-06-13): the affected-set *selection* mirror
//! (`CiComponentAffected`, `ci_changed_path_affects_*`, the `detect-ci-affected-components` /
//! `emit-affected-set-ci-receipt` / `emit-ci-wave3-shadow-receipt` bins, the `git_diff_transport`
//! reader and the `wave3_shadow_receipt`) was pure shadow — no ci.yml job gated on its
//! classification (floor jobs cache-gated; lens/corpus/`ci` `always()`), so the kill-criterion
//! receipts measured `saved_minutes = 0`. With CI run uniform, that selection machinery is retired.
//! What remains is host support that is NOT selection: the run timing ledger (`receipt`, consumed
//! by `ci_timings_collector` for the `affected-set-ci-receipt-timed` latency artifact) and the
//! self-hosted runner-pool capacity facts (`runner_pool`).

pub mod receipt;
pub mod runner_pool;
