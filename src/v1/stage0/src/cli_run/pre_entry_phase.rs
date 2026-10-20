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

/// One phase's receipt row.
///
/// `nested` is the time this row's span CONTAINED from other phase rows, measured mechanically by
/// `timed` rather than declared by the author. `raw` marks time a producer measured and `record`ed
/// itself (e.g. `pool_acquire`'s tokenize rows): its span boundaries are not visible here, so it
/// cannot be netted and is reported as `attribution=recorded` rather than guessed at. §4 P0.1
/// requires the receipt to say inclusive or exclusive before a reader sums rows; a row claiming an
/// exclusivity it cannot show makes the sum a fiction, which is why only `timed` spans get an
/// `excl_ms` at all.
struct PhaseRow {
    name: &'static str,
    scale: PhaseScale,
    elapsed: Duration,
    nested: u128,
    raw: bool,
    /// When this row FIRST got time, so the transaction receipt can tell rows that began inside
    /// the compile transaction from rows a predecessor recorded before it started (only the
    /// former may be netted against the transaction total — netting a predecessor's row against
    /// a total it never contributed to would fabricate a floor below zero).
    first_recorded_at: std::time::Instant,
}

thread_local! {
    static PHASES: RefCell<Vec<PhaseRow>> = const { RefCell::new(Vec::new()) };
    /// Nanos recorded by any row so far on this thread; `timed` takes the delta across `f` to get
    /// this span's contained time.
    static RECORDED_NANOS: std::cell::Cell<u128> = const { std::cell::Cell::new(0) };
}

/// Add `elapsed` to the named phase (first record fixes its position in the receipt).
/// Scale may climb `tree` → `closure` on a later add: `pool_source_tokenize` of the
/// demand-acquired set must not stay labelled `tree` because the first lex happened
/// before `mark_demand_acquired_path`.
pub fn record(name: &'static str, scale: PhaseScale, elapsed: Duration) {
    PHASES.with(|p| {
        let mut p = p.borrow_mut();
        if let Some(row) = p.iter_mut().find(|r| r.name == name) {
            row.elapsed += elapsed;
            row.raw = true;
            if scale == PhaseScale::Closure {
                row.scale = PhaseScale::Closure;
            }
        } else {
            p.push(PhaseRow {
                name,
                scale,
                elapsed,
                nested: 0,
                raw: true,
                first_recorded_at: std::time::Instant::now(),
            });
        }
    });
    RECORDED_NANOS.with(|t| t.set(t.get() + elapsed.as_nanos()));
}

/// Time `f` into the named phase. The row's `ms` is INCLUSIVE of any phase row recorded inside
/// `f`; the contained time is subtracted to print `excl_ms`, so rows can be summed without
/// double-counting (§4 P0.1: "Check whether each row is inclusive or exclusive before summing").
pub fn timed<T>(name: &'static str, scale: PhaseScale, f: impl FnOnce() -> T) -> T {
    let started = std::time::Instant::now();
    let recorded_before = RECORDED_NANOS.with(|t| t.get());
    let out = f();
    let elapsed = started.elapsed();
    let nested = RECORDED_NANOS.with(|t| t.get().saturating_sub(recorded_before));
    PHASES.with(|p| {
        let mut p = p.borrow_mut();
        if let Some(row) = p.iter_mut().find(|r| r.name == name) {
            row.elapsed += elapsed;
            row.nested += nested;
            if scale == PhaseScale::Closure {
                row.scale = PhaseScale::Closure;
            }
        } else {
            p.push(PhaseRow {
                name,
                scale,
                elapsed,
                nested,
                raw: false,
                first_recorded_at: std::time::Instant::now(),
            });
        }
    });
    RECORDED_NANOS.with(|t| t.set(t.get() + elapsed.as_nanos()));
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

/// Drain the receipt with the inclusive/exclusive attribution §4 P0.1 asks for:
/// `attribution=exclusive` (this row's span contained no other row; `ms` may be summed with the
/// other rows' `excl_ms`), `attribution=inclusive` (it contained other rows; their recorded time
/// is subtracted into `excl_ms`), `attribution=recorded` (the producer measured and `record`ed
/// this row itself, so its span boundaries are not visible and it is reported unnetted).
pub fn take_rows_with_attribution() -> Vec<String> {
    drain_rows()
        .into_iter()
        .map(|row| {
            let (attribution, excl) = if row.raw {
                ("recorded", row.elapsed.as_nanos())
            } else if row.nested > 0 {
                (
                    "inclusive",
                    row.elapsed.as_nanos().saturating_sub(row.nested),
                )
            } else {
                ("exclusive", row.elapsed.as_nanos())
            };
            format!(
                "[pre-entry] phase={} ms={} excl_ms={} scale={} attribution={}",
                row.name,
                row.elapsed.as_millis(),
                Duration::from_nanos(u64::try_from(excl).unwrap_or(u64::MAX),).as_millis(),
                row.scale.wire(),
                attribution
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
// Boundary this measures: argv decode (before the transaction), output materialization,
// diagnostic rendering, teardown. It does NOT reach code before `main` (the runtime's own
// startup) -- that residue shows up in `process_unattributed` as time no row explains, which is
// where a hidden phase is supposed to surface.

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
/// which passes the instant the transaction began so rows a PREDECESSOR recorded before it are
/// excluded from the netting. Staging on EVERY disposition is what lets
/// `invocation_receipt_lines` tell "a verb that never compiled" (print nothing) apart from
/// "a compile that failed" (print everything the rows explain).
pub fn print_compile_transaction_receipt(subject: &str, transaction_started: std::time::Instant) {
    let total = transaction_started.elapsed();
    // The transaction's time that NO row explains, as a FLOOR: total minus the disjoint cover of
    // the transaction's OWN timed rows (for a containment tree, a parent's `excl_ms` plus its
    // children's `excl_ms` equals the parent's span, so summing those `excl_ms` is the covered
    // time). Three honesty rules, each pushing the reported hidden time UP rather than down —
    // a floor may over-report, never conceal (§4d):
    // - rows first recorded before `transaction_started` are excluded: they are a predecessor's
    //   cost and were never part of this total (netting them would understate the floor);
    // - rows a producer `record`ed itself are excluded (their span boundaries are invisible
    //   here, so they cannot be shown disjoint) and printed as `raw_ms=` for a reader to judge;
    // - a raw record inside a timed span was already netted out of that span's `excl_ms` by
    //   `timed`, so excluding raw rows loses nothing.
    let (attributed, raw_nanos): (u128, u128) = PHASES.with(|p| {
        let mut attributed = 0u128;
        let mut raw = 0u128;
        for r in p.borrow().iter() {
            if r.raw {
                raw += r.elapsed.as_nanos();
            } else if r.first_recorded_at >= transaction_started {
                attributed += r.elapsed.as_nanos().saturating_sub(r.nested);
            }
        }
        (attributed, raw)
    });
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
            "[pre-entry] phase=compile_transaction_unattributed ms={} scale={scale} attribution=exclusive raw_ms={}",
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
        format!(
            "[pre-entry] phase=process_invocation_total ms={} scale={scale} attribution=inclusive transactions={}",
            elapsed.as_millis(),
            transactions.len()
        ),
        // Unattributed OUTSIDE every transaction: argv decode, output materialization, diagnostic
        // rendering, teardown. This is the §5a-A figure; it is visible precisely so that a hidden
        // cost cannot be read as `wall_ms`'s rounding error.
        format!(
            "[pre-entry] phase=process_unattributed ms={} scale={scale} attribution=exclusive",
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

    /// The §5a-A row: the invocation total MINUS the transaction totals is what no row explains.
    /// A hidden phase outside the transaction (output materialization, diagnostic rendering,
    /// teardown) can only surface through this subtraction, so the arithmetic is the instrument.
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
        // `attributed` is the netted COVER and `raw_nanos` the non-nettable total, both in
        // nanos (what `print_compile_transaction_receipt` computes).
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

    /// `timed` measures the inclusive/exclusive attribution MECHANICALLY (§4 P0.1): a parent's
    /// `ms` is its span, `excl_ms` is that span minus what the nested rows recorded inside it,
    /// and a producer-`record`ed row is reported `recorded` rather than netted.
    #[test]
    fn timed_rows_are_netted_against_nested_rows_by_construction() {
        let _ = take_lines();
        timed("probe_parent", PhaseScale::Tree, || {
            timed("probe_child", PhaseScale::Tree, || {
                std::thread::sleep(Duration::from_millis(30));
            });
            // 20ms of the parent's span belongs to no row: that is its exclusive part.
            std::thread::sleep(Duration::from_millis(20));
        });
        record("probe_raw", PhaseScale::Tree, Duration::from_millis(5));
        let lines = take_rows_with_attribution();
        let parent = lines
            .iter()
            .find(|l| l.contains("phase=probe_parent"))
            .expect("parent row");
        let child = lines
            .iter()
            .find(|l| l.contains("phase=probe_child"))
            .expect("child row");
        let raw = lines
            .iter()
            .find(|l| l.contains("phase=probe_raw"))
            .expect("raw row");
        assert!(
            parent.contains("attribution=inclusive"),
            "parent contains a timed child: {parent}"
        );
        let excl: u128 = parent
            .split("excl_ms=")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse().ok())
            .expect("excl_ms on parent row");
        // The parent spanned ~50ms; its exclusive part must exclude the child's ~30ms but the
        // floor is generous to scheduler noise: netting must be visible, not borderline.
        assert!(exl_usable(excl), "parent excl_ms={excl}: {parent}");
        assert!(
            child.contains("attribution=exclusive"),
            "child timed nothing: {child}"
        );
        assert!(
            raw.contains("attribution=recorded"),
            "producer-recorded rows are not netted: {raw}"
        );
    }

    fn exl_usable(excl: u128) -> bool {
        (15..=45).contains(&excl)
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
