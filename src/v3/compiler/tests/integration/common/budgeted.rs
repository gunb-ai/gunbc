//! Per-test wall-clock budgets for integration tests.
//!
//! Principle: a single `#[test]` that spends multiple seconds is usually
//! either hanging or redoing full-pipeline work that should be shared
//! (`std::sync::OnceLock`, module-level cache, or fewer parametrized cases).
//!
//! **Consumers:** same-PR as the helper (E-6-style discipline). Use the
//! `budgeted_test!` macro from any `tests/*.rs` that declares `mod common;`
//! (each integration test binary is its own crate; the macro expands to
//! `$crate::common::budgeted::...`).

use std::time::{Duration, Instant};

/// Default single-test budget in milliseconds (2s).
pub const DEFAULT_BUDGET_MS: u64 = 2000;

/// Run `f` and panic if wall time exceeds `budget_ms`.
pub fn with_budget_ms<F: FnOnce()>(budget_ms: u64, f: F) {
    let budget = Duration::from_millis(budget_ms);
    let start = Instant::now();
    f();
    let elapsed = start.elapsed();
    assert!(
        elapsed <= budget,
        "test exceeded wall-clock budget: {:?} > {:?} — share expensive setup (OnceLock / \
         module cache) or collapse fine-grained tests; full bootstrap + compile per test is not \
         allowed",
        elapsed,
        budget
    );
}
