//! CI host support crate — per-run wall-clock timing ledger + self-hosted runner-pool facts.
//!
//! Axis-A consolidation (operator "go uniform", 2026-06-13): the affected-set *selection* mirror
//! was pure shadow — retired. What remains is host support that is NOT selection: the run timing
//! ledger (`receipt`, consumed by `ci_timings_collector`) and the self-hosted runner-pool
//! capacity facts (`runner_pool`, projected from `std.compute_fabric` `RunnerPoolFacts`).

pub mod receipt;
pub mod runner_pool;
