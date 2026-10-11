//! THE PRE-ENTRY PHASE RECEIPT OF `gunbc run`.
//!
//! `gunbc run --entry X --function F` pays a cost before it evaluates `F`, and the
//! recurring-failure row `gunbc.recurring_failure_mode`
//! `gunbc_run_pays_a_large_fixed_pre_entry_cost_per_invocation` names its next-rung trigger
//! as a decomposition of that cost into named phases, sufficient to show which part a
//! small-closure entry does not demand. This is that decomposition's producer: every
//! `gunbc run` prints one `[pre-entry] phase=<name> ms=<n> scale=<tree|closure>` line per
//! phase on stderr, so the instrument is the ordinary invocation and a reader re-derives the
//! figures by running it rather than reading them here (DESIGN §6).
//!
//! `scale` is a declared property of the PHASE, not a measurement: `tree` names work whose
//! input is the whole indexed pool (every file under every `--source-root`), `closure` names
//! work whose input is the entry's resolved closure. The two are separated because the
//! question the row asks is which part scales with the tree, and a figure without that tag
//! answers only how big a number is.
//!
//! Host transport, like `phase_profile`'s `[phase-profile]` lines.
//!
//! Thread-local, like `ResolveStageNanos`: a `gunbc run` resolves on its main thread, and a
//! floor worker that reaches these timers keeps its own accumulator rather than sharing one.

use std::cell::RefCell;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhaseScale {
    Tree,
    Closure,
}

impl PhaseScale {
    fn wire(self) -> &'static str {
        match self {
            PhaseScale::Tree => "tree",
            PhaseScale::Closure => "closure",
        }
    }
}

thread_local! {
    static PHASES: RefCell<Vec<(&'static str, PhaseScale, Duration)>> =
        const { RefCell::new(Vec::new()) };
}

/// Add `elapsed` to the named phase (first record fixes its position in the receipt).
pub fn record(name: &'static str, scale: PhaseScale, elapsed: Duration) {
    PHASES.with(|p| {
        let mut p = p.borrow_mut();
        if let Some(row) = p.iter_mut().find(|(n, _, _)| *n == name) {
            row.2 += elapsed;
        } else {
            p.push((name, scale, elapsed));
        }
    });
}

/// Time `f` into the named phase.
pub fn timed<T>(name: &'static str, scale: PhaseScale, f: impl FnOnce() -> T) -> T {
    let started = std::time::Instant::now();
    let out = f();
    record(name, scale, started.elapsed());
    out
}

/// Drain the receipt: one line per phase in first-recorded order.
pub fn take_lines() -> Vec<String> {
    PHASES.with(|p| {
        std::mem::take(&mut *p.borrow_mut())
            .into_iter()
            .map(|(name, scale, d)| {
                format!(
                    "[pre-entry] phase={name} ms={} scale={}",
                    d.as_millis(),
                    scale.wire()
                )
            })
            .collect()
    })
}

/// The indexed pool's module count for `source_roots` (the population the `tree` rows scale
/// with), or `None` when no index exists for them. Reads ONLY the index a resolve already
/// memoized: it never builds one, so a resolve that refused on pool discovery is reported as
/// such rather than re-walked -- or, through the panicking `process_shared_index`, turned from a
/// typed refusal into a panic.
pub fn pool_module_count(source_roots: &[String]) -> Option<usize> {
    super::memoized_process_shared_index(source_roots).map(|index| index.source_files.len())
}
