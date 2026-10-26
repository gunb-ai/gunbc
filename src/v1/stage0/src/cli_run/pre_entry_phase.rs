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

/// One phase's receipt row. Every row is INCLUSIVE and printed as such: `elapsed` is the whole
/// span (or, for a producer-`record`ed row, the whole time it handed in); nothing is netted.
/// `parent`/`depth` carry the place in the containment tree (nearest enclosing `timed` span at
/// first entry), so a reader computes exclusive time as `parent.ms - Σ children.ms` from figures
/// that are each individually true — see `three_level_nesting_prints_inclusive_rows_with_parent_links`
/// for why a netted figure printed here was wrong (§4 P0.1: say which a row is, never pretend the
/// other). `raw` marks a producer-`record`ed row: its span boundaries are not visible here, so it
/// is labelled `recorded` and never counted as cover.
struct PhaseRow {
    name: &'static str,
    scale: PhaseScale,
    elapsed: Duration,
    /// Nearest enclosing `timed` span at FIRST entry; `None` at top level.
    parent: Option<&'static str>,
    depth: usize,
    /// True once ANY accumulation of this name happened inside a `timed` span: a name seen both
    /// at top level and nested cannot count as a disjoint root in the transaction cover.
    nested_seen: bool,
    raw: bool,
    /// When this row FIRST got time, so the transaction receipt excludes rows a predecessor
    /// recorded before it started — counting one against a total it never contributed to would
    /// fabricate a floor below zero.
    first_recorded_at: std::time::Instant,
}

thread_local! {
    static PHASES: RefCell<Vec<PhaseRow>> = const { RefCell::new(Vec::new()) };
    /// Names of the `timed` spans open on this thread, outermost first: a row's parent/depth
    /// come from its length at the row's start, and rows started with the stack EMPTY are
    /// provably disjoint from each other.
    static ACTIVE: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

/// Add `elapsed` to the named phase (first record fixes its position in the receipt).
/// Scale may climb `tree` → `closure` on a later add: `pool_source_tokenize` of the
/// demand-acquired set must not stay labelled `tree` because the first lex happened
/// before `mark_demand_acquired_path`.
pub fn record(name: &'static str, scale: PhaseScale, elapsed: Duration) {
    let (parent, depth, under_span) = ACTIVE.with(|a| {
        let a = a.borrow();
        (a.last().copied(), a.len(), !a.is_empty())
    });
    PHASES.with(|p| {
        let mut p = p.borrow_mut();
        if let Some(row) = p.iter_mut().find(|r| r.name == name) {
            row.elapsed += elapsed;
            row.raw = true;
            row.nested_seen |= under_span;
            if scale == PhaseScale::Closure {
                row.scale = PhaseScale::Closure;
            }
        } else {
            p.push(PhaseRow {
                name,
                scale,
                elapsed,
                parent,
                depth,
                nested_seen: under_span,
                raw: true,
                first_recorded_at: std::time::Instant::now(),
            });
        }
    });
}

/// Time `f` into the named phase. The row's `ms` is INCLUSIVE of anything that ran inside `f`,
/// and is printed as such with its `parent` and `depth`, so a reader can subtract the children
/// themselves (§4 P0.1: "Check whether each row is inclusive or exclusive before summing").
pub fn timed<T>(name: &'static str, scale: PhaseScale, f: impl FnOnce() -> T) -> T {
    // The parent/depth are read BEFORE the push: this row sits where it was entered, and its own
    // name must not become its parent.
    let (parent, depth) = ACTIVE.with(|a| {
        let a = a.borrow();
        (a.last().copied(), a.len())
    });
    ACTIVE.with(|a| a.borrow_mut().push(name));
    let started = std::time::Instant::now();
    let out = f();
    let elapsed = started.elapsed();
    ACTIVE.with(|a| {
        let mut a = a.borrow_mut();
        // `f` may panic through a nested span; pop the name we pushed even if the stack shape was
        // disturbed, so a later row is not orphaned under a span that already closed.
        if a.last() == Some(&name) {
            a.pop();
        }
    });
    PHASES.with(|p| {
        let mut p = p.borrow_mut();
        if let Some(row) = p.iter_mut().find(|r| r.name == name) {
            row.elapsed += elapsed;
            // This occurrence is nested, so the row cannot count as a disjoint root even though
            // its FIRST occurrence was at top level (`parent` keeps that first value).
            row.nested_seen |= depth > 0;
            if scale == PhaseScale::Closure {
                row.scale = PhaseScale::Closure;
            }
        } else {
            p.push(PhaseRow {
                name,
                scale,
                elapsed,
                parent,
                depth,
                nested_seen: depth > 0,
                raw: false,
                first_recorded_at: std::time::Instant::now(),
            });
        }
    });
    out
}

/// Drain the receipt: one line per phase in first-recorded order.
///
/// The wire format of these lines is a contract: `bin/claim_batch.rs` forwards them and
/// `cli_run/native_product_cache.rs` treats the last line NOT starting with `[pre-entry]` as a
/// refusal reason. Any new row anywhere must keep that prefix.
pub fn take_lines() -> Vec<String> {
    drain_rows()
        .into_iter()
        .map(|row| {
            format!(
                "[pre-entry] phase={} ms={} scale={}",
                row.name,
                row.elapsed.as_millis(),
                row.scale.wire()
            )
        })
        .collect()
}

/// Drain the receipt with the attribution §4 P0.1 asks for. Every row is printed INCLUSIVE
/// (`attribution=inclusive`, or `attribution=recorded` when a producer measured it itself) with
/// its `parent=` link and `depth=`, so a reader computes exclusive time as
/// `parent.ms - Σ children.ms` rather than trusting a subtraction made here. `parent=none` marks
/// a top-level row. No row is netted: see `PhaseRow`'s doc for why the netting that used to live
/// here was wrong.
pub fn take_rows_with_attribution() -> Vec<String> {
    drain_rows()
        .into_iter()
        .map(|row| {
            format!(
                "[pre-entry] phase={} ms={} scale={} attribution={} parent={} depth={}",
                row.name,
                row.elapsed.as_millis(),
                row.scale.wire(),
                if row.raw { "recorded" } else { "inclusive" },
                row.parent.unwrap_or("none"),
                row.depth
            )
        })
        .collect()
}

fn drain_rows() -> Vec<PhaseRow> {
    PHASES.with(|p| std::mem::take(&mut *p.borrow_mut()))
}

// ── THE OUTER BOUNDARY (§5a amendment A) ────────────────────────────────────────────────────
//
// `CompileRun.wall_ms` is sampled before the transaction's locals finish dropping, and output
// materialization and diagnostic rendering happen in the caller. So the receipt printed by
// `compile_emission_over` is missing everything outside the transaction, and a reader summing rows
// against `wall_ms` silently absorbs that gap as zero. The CLI's `main` records the entry instant
// here and prints the invocation rows on its return path, BEFORE it exits -- before every exit,
// including the `Verdict::apply` path, which ends the process with `libc::_exit`. Printing at the
// return path rather than from an `atexit` hook is what makes that path safe: an exit-time hook
// would be skipped by `_exit` and the receipt would silently vanish exactly when a compile fails
// to write its tree.
//
// The boundary this timer spans is MAIN ENTRY TO PRE-EXIT SAMPLING (named in the emitted rows as
// `boundary=main_entry_to_pre_exit_sampling`): from `record_invocation_start`, main's first act,
// to the sample taken at the print site on the return path. Within it sit argv decode, the
// transactions, output materialization, diagnostic rendering, and whatever drops run before the
// print. UNOBSERVED -- not in any row, NOT counted in `process_unattributed`: code before `main`
// (the runtime's own startup, which happens before the start instant) and post-sample cleanup
// (which happens after the sample). A reader must not read `process_unattributed` as "everything
// outside the transactions for the whole process", because it is only everything outside the
// transactions BETWEEN the two sampling instants.

/// Process-lifetime receipts staged for the CLI return path. Process-global, not thread-local:
/// the compile transaction and the printing site are the same thread in the CLI, but a host that
/// drives a compile from a worker thread must not lose the row.
static COMPILE_TRANSACTIONS: std::sync::Mutex<Vec<(String, u128)>> =
    std::sync::Mutex::new(Vec::new());
static INVOCATION_START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
/// The invocation rows print once per process, no matter how many exits the CLI offers.
static INVOCATION_PRINTED: std::sync::Once = std::sync::Once::new();

/// Called by the CLI's `main`, as its first act, so the invocation total includes argv decode.
/// First call wins; a second call (a host re-entering `main` in a test) keeps the first instant,
/// which is the only honest start.
pub fn record_invocation_start() {
    let _ = INVOCATION_START.set(std::time::Instant::now());
}

/// The invocation total, or `None` when `record_invocation_start` was never called -- a library
/// consumer that compiles without a CLI `main`. Printing a total then would attribute the whole
/// process to a transaction sequence the caller never started (§4d: absent, never fabricated).
fn invocation_elapsed() -> Option<Duration> {
    INVOCATION_START.get().map(|started| started.elapsed())
}

/// Print the compile transaction's receipt: the drained phase rows with attribution, the
/// transaction total, and the transaction's own unattributed time, and stage the invocation
/// total for the process rows. One call site: `cli_run::compile_emission_over` (the wrapper),
/// which passes the instant the transaction began so rows a PREDECESSOR recorded before it never
/// count toward the root cover. Staging on EVERY disposition is what lets
/// `invocation_receipt_lines` tell "a verb that never compiled" (print nothing) apart from
/// "a compile that failed" (print everything the rows explain).
/// The time the transaction's rows can PROVABLY account for, and the non-nettable raw total.
/// Cover = timed rows entered with an empty `ACTIVE` stack (`parent=none`), never seen nested,
/// first recorded inside the transaction: two such roots cannot contain each other, so summing
/// them cannot double-count. Predecessor rows, nested rows, and raw rows never cover — each
/// exclusion only raises the reported floor, and a floor may over-report but never conceal (§4d).
fn disjoint_root_cover(transaction_started: std::time::Instant) -> (u128, u128) {
    PHASES.with(|p| {
        let mut attributed = 0u128;
        let mut raw = 0u128;
        for r in p.borrow().iter() {
            if r.raw {
                raw += r.elapsed.as_nanos();
            } else if r.parent.is_none()
                && !r.nested_seen
                && r.first_recorded_at >= transaction_started
            {
                attributed += r.elapsed.as_nanos();
            }
        }
        (attributed, raw)
    })
}

pub fn print_compile_transaction_receipt(subject: &str, transaction_started: std::time::Instant) {
    let total = transaction_started.elapsed();
    let (attributed, raw_nanos) = disjoint_root_cover(transaction_started);
    for line in take_rows_with_attribution() {
        eprintln!("{line}");
    }
    for line in compile_transaction_lines(subject, total, attributed, raw_nanos, index_scale()) {
        eprintln!("{line}");
    }
    if let Ok(mut t) = COMPILE_TRANSACTIONS.lock() {
        t.push((subject.to_string(), total.as_millis()));
    }
}

/// The two transaction rows (total + the floor of what NO row explains), as pure data so the
/// honesty properties are testable without spawning a compile.
fn compile_transaction_lines(
    subject: &str,
    total: Duration,
    attributed: u128,
    raw_nanos: u128,
    scale: PhaseScale,
) -> Vec<String> {
    let scale = scale.wire();
    vec![
        format!(
            "[pre-entry] phase=compile_transaction ms={} scale={scale} attribution=inclusive subject={subject}",
            total.as_millis()
        ),
        // The transaction's time that NO row explains (the floor computed by the caller). This
        // is where a hidden phase shows up: it cannot be concealed, because it is computed over
        // the same accumulator the rows above drain.
        format!(
            "[pre-entry] phase=compile_transaction_unattributed ms={} scale={scale} attribution=floor raw_ms={}",
            Duration::from_nanos(
                total
                    .as_nanos()
                    .saturating_sub(attributed)
                    .try_into()
                    .unwrap_or(u64::MAX)
            )
            .as_millis(),
            Duration::from_nanos(u64::try_from(raw_nanos).unwrap_or(u64::MAX)).as_millis()
        ),
    ]
}

/// Print the invocation rows. Called by the CLI on its return path, before every exit it takes
/// (including `Verdict::apply`'s `libc::_exit`), which is why this is not an `atexit` hook.
pub fn print_invocation_receipt() {
    INVOCATION_PRINTED.call_once(|| {
        print_invocation_receipt_inner();
    });
}

fn print_invocation_receipt_inner() {
    let transactions: Vec<(String, u128)> = COMPILE_TRANSACTIONS
        .lock()
        .map(|t| t.clone())
        .unwrap_or_default();
    // §4d: no recorded invocation start means no rows at all, never a zero.
    for line in invocation_receipt_lines(invocation_elapsed(), &transactions, index_scale()) {
        eprintln!("{line}");
    }
}

/// The two process rows (invocation total + the unattributed-outside-the-transactions figure),
/// as pure data. `transactions` are `(subject, total_ms)` pairs in call order. No recorded
/// invocation start yields NO rows: an absent figure is honest, a zero is a lie
/// (§4d: "Check the instrument reads real data before it is called a receipt").
fn invocation_receipt_lines(
    elapsed: Option<Duration>,
    transactions: &[(String, u128)],
    scale: PhaseScale,
) -> Vec<String> {
    // Rows exist only for an invocation that actually ran a compile transaction: the compile
    // route stages one on EVERY disposition (the wrapper), so an empty list here means a verb
    // that never compiled (run, check, ...) -- and those routes keep their existing stderr.
    // No recorded start is the other omission: a library consumer without a CLI `main` must not
    // get a total that attributes the whole process to transactions nobody started (§4d).
    if elapsed.is_none() || transactions.is_empty() {
        return Vec::new();
    }
    let elapsed = elapsed.expect("checked above");
    let scale = scale.wire();
    let transactions_total_ms: u128 = transactions.iter().map(|(_, ms)| ms).sum();
    vec![
        // `boundary=` names exactly what the timer spans (side-chat guard on the first wording,
        // which implied whole-process coverage): main entry to the sample taken here. Pre-main
        // startup and exit-time cleanup are OUTSIDE both instants -- unobserved, not counted in
        // process_unattributed (see THE OUTER BOUNDARY above).
        format!(
            "[pre-entry] phase=process_invocation_total ms={} scale={scale} attribution=inclusive transactions={} boundary=main_entry_to_pre_exit_sampling",
            elapsed.as_millis(),
            transactions.len()
        ),
        // Unattributed BETWEEN THE TWO SAMPLING INSTANTS: argv decode, output materialization,
        // diagnostic rendering, drops before the print. Pre-main startup and exit-time cleanup
        // are unobserved, NOT part of this figure (the `boundary=` on the total row says which).
        // This is the §5a-A figure; it is visible precisely so that a hidden cost cannot be read
        // as `wall_ms`'s rounding error.
        format!(
            "[pre-entry] phase=process_unattributed ms={} scale={scale} attribution=floor boundary=main_entry_to_pre_exit_sampling",
            elapsed
                .as_millis()
                .saturating_sub(transactions_total_ms)
                .try_into()
                .unwrap_or(u64::MAX)
        ),
    ]
}

thread_local! {
    static DEMAND_PATH: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static DEMAND_ACQUIRED_MODULES: std::cell::Cell<Option<usize>> =
        const { std::cell::Cell::new(None) };
}

/// Mark that this thread's resolve is the demand-acquired `gunbc run` path, so
/// pool tokenize is `scale=closure`. Does not invent a pool size.
pub(crate) fn mark_demand_acquired_path() {
    DEMAND_PATH.with(|c| c.set(true));
}

pub(crate) fn record_demand_acquired_modules(n: usize) {
    DEMAND_PATH.with(|c| c.set(true));
    DEMAND_ACQUIRED_MODULES.with(|c| c.set(Some(n)));
}

/// How many files the demand path acquired, or `None` when this thread did not.
pub fn demand_acquired_modules() -> Option<usize> {
    DEMAND_ACQUIRED_MODULES.with(|c| c.get())
}

/// `tree` when the indexed pool is every file under `--source-root`; `closure` when
/// `gunbc run` already replaced that pool with the demand-acquired set.
pub fn index_scale() -> PhaseScale {
    if DEMAND_PATH.with(|c| c.get()) {
        PhaseScale::Closure
    } else {
        PhaseScale::Tree
    }
}

/// The indexed pool's module count for `source_roots` (the population the `tree` rows scale
/// with), or `None` when no index exists for them. Reads ONLY the index a resolve already
/// memoized: it never builds one, so a resolve that refused on pool discovery is reported as
/// such rather than re-walked -- or, through the panicking `process_shared_index`, turned from a
/// typed refusal into a panic. It does not substitute the demand-acquired set.
pub fn pool_module_count(source_roots: &[String]) -> Option<usize> {
    super::memoized_process_shared_index(source_roots).map(|index| index.source_files.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_row_scale_climbs_from_tree_to_closure() {
        let _ = take_lines();
        record(
            "pool_source_tokenize",
            PhaseScale::Tree,
            Duration::from_millis(10),
        );
        mark_demand_acquired_path();
        record(
            "pool_source_tokenize",
            index_scale(),
            Duration::from_millis(5),
        );
        let lines = take_lines();
        DEMAND_PATH.with(|c| c.set(false));
        DEMAND_ACQUIRED_MODULES.with(|c| c.set(None));
        assert!(
            lines
                .iter()
                .any(|l| l.contains("phase=pool_source_tokenize") && l.contains("scale=closure")),
            "{lines:?}"
        );
    }

    #[test]
    fn pool_module_count_does_not_publish_the_demand_set_as_the_pool() {
        mark_demand_acquired_path();
        record_demand_acquired_modules(58);
        assert_eq!(demand_acquired_modules(), Some(58));
        assert_eq!(pool_module_count(&["dag".to_string()]), None);
        DEMAND_PATH.with(|c| c.set(false));
        DEMAND_ACQUIRED_MODULES.with(|c| c.set(None));
    }

    /// §4d, as a RED: with no recorded invocation start the process rows are OMITTED. A `0` here
    /// would read as "the CLI spent no time outside the transaction" -- the fabricated-zero class
    /// the plan names. (The `record_invocation_start`-happens test is
    /// `the_invocation_total_advances_with_wall_time`.)
    #[test]
    fn an_invocation_without_a_recorded_start_omits_the_process_rows_instead_of_reporting_zero() {
        let lines =
            invocation_receipt_lines(None, &[("subject".to_string(), 7u128)], PhaseScale::Tree);
        assert!(lines.is_empty(), "{lines:?}");
    }

    /// The §5a-A row: the invocation total MINUS the transaction totals is what no row explains
    /// BETWEEN the two sampling instants. A hidden phase in that window (output materialization,
    /// diagnostic rendering, drops before the print) can only surface through this subtraction,
    /// so the arithmetic is the instrument. Pre-main startup and exit-time cleanup sit outside
    /// both instants and are unobserved — never folded into this figure.
    #[test]
    fn process_rows_subtract_transaction_totals_from_the_invocation() {
        let lines = invocation_receipt_lines(
            Some(Duration::from_millis(100)),
            &[("a".to_string(), 30u128), ("b".to_string(), 40u128)],
            PhaseScale::Tree,
        );
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(
            lines[0].contains("phase=process_invocation_total ms=100")
                && lines[0].contains("transactions=2"),
            "{}",
            lines[0]
        );
        // The emitted label names the timer's ACTUAL span (side-chat guard): a reader cannot
        // mistake these rows for whole-process coverage. Pre-main startup and exit-time cleanup
        // are outside both sampling instants -- unobserved, not counted in the figure below.
        assert!(
            lines[0].contains("boundary=main_entry_to_pre_exit_sampling"),
            "{}",
            lines[0]
        );
        assert!(
            lines[1].contains("boundary=main_entry_to_pre_exit_sampling"),
            "{}",
            lines[1]
        );
        assert!(
            lines[1].contains("phase=process_unattributed ms=30"),
            "{}",
            lines[1]
        );
        // Every row keeps the `[pre-entry]` prefix: native_product_cache treats the last line
        // WITHOUT it as a refusal reason, and claim_batch forwards only these.
        assert!(
            lines.iter().all(|l| l.starts_with("[pre-entry] ")),
            "{lines:?}"
        );
    }

    /// A transaction's unattributed floor may OVER-report (raw rows, predecessor rows) but never
    /// under-report: total minus the netted timed cover.
    #[test]
    fn transaction_rows_price_the_unexplained_time() {
        // `attributed` is the provably-disjoint root cover and `raw_nanos` the non-nettable
        // total, both in nanos (what `print_compile_transaction_receipt` computes).
        let lines = compile_transaction_lines(
            "src/main.dag",
            Duration::from_millis(90),
            40_000_000,
            10_000_000,
            PhaseScale::Tree,
        );
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(
            lines[0].contains("phase=compile_transaction ms=90")
                && lines[0].contains("subject=src/main.dag"),
            "{}",
            lines[0]
        );
        assert!(
            lines[1].contains("phase=compile_transaction_unattributed ms=50")
                && lines[1].contains("raw_ms=10"),
            "{}",
            lines[1]
        );
    }

    /// The counter-example that killed the netting, as a RED: A⊃B⊃C at 100/60/40ms, where B's
    /// own work is 20ms and A's own work is 40ms. The removed implementation subtracted a
    /// CUMULATIVE counter of all inclusive durations, so A paid for C twice (through B and
    /// through itself) and reported exclusive 0 where 40 is correct. No row is netted anymore;
    /// every row prints INCLUSIVE with its parent link, and exclusive time is
    /// `parent.ms - Σ children.ms` for the reader.
    #[test]
    fn three_level_nesting_prints_inclusive_rows_with_parent_links() {
        let _ = take_lines();
        timed("probe_grandparent", PhaseScale::Tree, || {
            record(
                "probe_grandparent_leaf",
                PhaseScale::Tree,
                Duration::from_millis(5),
            );
            timed("probe_parent", PhaseScale::Tree, || {
                timed("probe_child", PhaseScale::Tree, || {
                    std::thread::sleep(Duration::from_millis(40));
                });
                // 20ms of B's own work: B.ms - C.ms = 20 is B's exclusive part.
                std::thread::sleep(Duration::from_millis(20));
            });
            // 40ms of A's own work: A.ms - B.ms = 40 is A's exclusive part — the figure the
            // removed netting reported as 0.
            std::thread::sleep(Duration::from_millis(40));
        });
        let lines = take_rows_with_attribution();
        let row = |name: &str| {
            lines
                .iter()
                .find(|l| l.contains(&format!("phase={name} ")))
                .unwrap_or_else(|| panic!("no row for {name} in {lines:?}"))
                .clone()
        };
        let ms_of = |l: &str| -> u128 {
            l.split("ms=")
                .nth(1)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap()
        };
        let grandparent = row("probe_grandparent");
        let parent = row("probe_parent");
        let child = row("probe_child");
        let leaf = row("probe_grandparent_leaf");
        // The printed parent links, asserted per level: depth 0 has none, each level below
        // names its container and deepens.
        assert!(
            grandparent.contains("parent=none depth=0")
                && grandparent.contains("attribution=inclusive"),
            "{grandparent}"
        );
        assert!(
            parent.contains("parent=probe_grandparent depth=1")
                && parent.contains("attribution=inclusive"),
            "{parent}"
        );
        assert!(
            child.contains("parent=probe_parent depth=2")
                && child.contains("attribution=inclusive"),
            "{child}"
        );
        // A raw row nested inside spans is labelled recorded, and still names its parent.
        assert!(
            leaf.contains("parent=probe_grandparent depth=1")
                && leaf.contains("attribution=recorded"),
            "{leaf}"
        );
        // No row carries a netted figure: nothing to sum wrongly, nothing double-subtracted.
        assert!(
            lines.iter().all(|l| !l.contains("excl_ms=")),
            "a netted figure came back: {lines:?}"
        );
        // And the reader's arithmetic lands where the removed netting was wrong: A's only timed
        // child is B, so A.ms - B.ms is A's exclusive part — ~40ms, and under the removed
        // implementation's netting it printed as 0.
        let a_exclusive = ms_of(&grandparent) - ms_of(&parent);
        let b_exclusive = ms_of(&parent) - ms_of(&child);
        assert!(
            (30..=70).contains(&a_exclusive),
            "reader-computed A-exclusive from {grandparent} | {parent}"
        );
        assert!(
            (10..=40).contains(&b_exclusive),
            "reader-computed B-exclusive from {parent} | {child}"
        );
    }

    /// The transaction floor covers ONLY provably-disjoint roots: a nested row is excluded even
    /// though its inclusive `ms` fits inside its root (summing both would double-count and
    /// UNDERSTATE the hidden time — the concealment direction the floor must never go, §4d), and
    /// a raw record never covers anything.
    #[test]
    fn the_transaction_floor_covers_disjoint_roots_only() {
        let _ = take_lines();
        let started = std::time::Instant::now();
        timed("floor_root_a", PhaseScale::Tree, || {
            std::thread::sleep(Duration::from_millis(60));
        });
        timed("floor_root_b", PhaseScale::Tree, || {
            record("floor_raw", PhaseScale::Tree, Duration::from_millis(4));
            timed("floor_nested", PhaseScale::Tree, || {
                std::thread::sleep(Duration::from_millis(60));
            });
        });
        let wall = started.elapsed().as_nanos();
        let (attributed, raw) = disjoint_root_cover(started);
        assert_eq!(raw, 4_000_000, "raw total is reported, never netted");
        // Disjoint roots are spans inside [started, now], so their sum can never exceed the
        // measured wall — true by construction, for any scheduler. Counting a nested child in
        // addition to its root is what breaks it: B already contains the child, so B+C lands
        // ~50% ABOVE the wall. Both sides measure the same run, so this is noise-proof.
        assert!(
            attributed <= wall,
            "cover {attributed} exceeds measured wall {wall}: a nested row was counted twice \
             (understating hidden time, the one direction §4d forbids)"
        );
    }

    /// "Check the instrument reads real data before it is called a receipt": the invocation
    /// total is a wall-clock `Instant` taken at the CLI's entry, so it must ADVANCE between reads
    /// — a constant, or a total computed from anything other than elapsed real time, does not
    /// grow by the sleep. The absolute value is not asserted: the instant is process-global and
    /// first-wins, so an earlier test in the same binary legitimately fixes the base.
    #[test]
    fn the_invocation_total_advances_with_wall_time() {
        record_invocation_start();
        let first = invocation_elapsed().expect("start was recorded above");
        std::thread::sleep(Duration::from_millis(80));
        let second = invocation_elapsed().expect("start was recorded above");
        assert!(
            second > first && second - first < Duration::from_secs(5),
            "invocation total must advance ~1:1 with wall time: first={first:?} second={second:?}"
        );
    }

    /// A verb that never compiled stages no transaction and therefore prints no invocation rows:
    /// `run`/`check`/... keep their existing stderr byte-for-byte, which is what makes this
    /// receipt safe for the 14k-test suite.
    #[test]
    fn an_invocation_without_a_compile_transaction_prints_nothing() {
        let lines =
            invocation_receipt_lines(Some(Duration::from_millis(500)), &[], PhaseScale::Tree);
        assert!(lines.is_empty(), "{lines:?}");
    }
}
