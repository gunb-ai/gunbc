//! Native-driver cost accounting must refuse, not clamp, and must stop the driver.
//!
//! Review 63891: `saturating_sub` printed remainder_nanos 0 on OverAttributed (a fabricated
//! number). The non-Reconciled arms printed a verdict string and exited 0 (an inert law).
//! `[native-prepare]` duplicated `[native-prepare-split]`.
//!
//! The driver source is `v1.compiler.emit_rust` `emit_source_root_eval_driver_main_rs`.
//! The three-verdict law is `std.compiler_entry` `native_driver_cost_account` (also enrolled
//! as dag witnesses). This file executes both: planted over-attribution on the fold, and
//! the emitted main's refusal/exit shape.

use v1_compiler::std_compiler_entry::{
    native_driver_cost_account, native_driver_cost_remainder_tolerance_nanos,
    native_driver_cost_row_standing, native_driver_exclusive_rows, NativeDriverChildStanding,
    NativeDriverCostAccounting, NativeDriverCostRowStanding, NativeDriverExclusiveRowKey,
};
use v1_compiler::std_measure::nanosecond;
use v1_compiler::v1_compiler_emit_rust::emit_source_root_eval_driver_main_rs;

fn driver_main() -> String {
    emit_source_root_eval_driver_main_rs(
        "v1_compiled".to_string(),
        "v2.compiler.compile".to_string(),
    )
    .content
    .clone()
}

#[test]
fn planted_over_attribution_is_over_attributed_not_clamped() {
    let account = native_driver_cost_account(
        nanosecond(100),
        native_driver_exclusive_rows(|k| match k {
            NativeDriverExclusiveRowKey::ExclusiveLoad => nanosecond(40),
            NativeDriverExclusiveRowKey::ExclusiveUniverseDerivation => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveContext => nanosecond(40),
            NativeDriverExclusiveRowKey::ExclusivePrepare => nanosecond(40),
            NativeDriverExclusiveRowKey::ExclusiveEval => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveReceiptAdmission => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveRowSerialization => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveModuleRelease => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveRelayEmit => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveDemandScheduling => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveDriverCollection => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveOccurrenceCensus => nanosecond(0),
        }),
        native_driver_cost_remainder_tolerance_nanos(),
    );
    match account.as_ref() {
        NativeDriverCostAccounting::NativeDriverCostOverAttributed {
            sum_exclusive,
            parent_span,
        } => {
            assert_eq!(sum_exclusive, &nanosecond(120));
            assert_eq!(parent_span, &nanosecond(100));
        }
        other => panic!("planted exclusive > parent must be OverAttributed, got {other:?}"),
    }
}

#[test]
fn reconciled_parent_passes() {
    let account = native_driver_cost_account(
        nanosecond(3439809790856),
        native_driver_exclusive_rows(|k| match k {
            NativeDriverExclusiveRowKey::ExclusiveLoad => nanosecond(432921485),
            NativeDriverExclusiveRowKey::ExclusiveUniverseDerivation => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveContext => nanosecond(3380085706753),
            NativeDriverExclusiveRowKey::ExclusivePrepare => nanosecond(59277814291),
            NativeDriverExclusiveRowKey::ExclusiveEval => nanosecond(3531894),
            NativeDriverExclusiveRowKey::ExclusiveReceiptAdmission => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveRowSerialization => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveModuleRelease => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveRelayEmit => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveDemandScheduling => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveDriverCollection => nanosecond(0),
            NativeDriverExclusiveRowKey::ExclusiveOccurrenceCensus => nanosecond(0),
        }),
        native_driver_cost_remainder_tolerance_nanos(),
    );
    assert!(
        matches!(
            account.as_ref(),
            NativeDriverCostAccounting::NativeDriverCostReconciled { .. }
        ),
        "control: a parent covering exclusive rows within tolerance must reconcile, got {account:?}"
    );
}

#[test]
fn emitted_driver_refuses_over_attribution_instead_of_clamping_and_fails_the_process() {
    let main_rs = driver_main();
    assert!(
        !main_rs.contains("saturating_sub(exclusive_sum)") && !main_rs.contains("checked_sub("),
        "remainder is the fold residual — not a second subtraction that can clamp or fork OverAttributed; emitted:\n{main_rs}"
    );
    assert!(
        main_rs.contains("NativeDriverCostReconciled { residual, .. }")
            || main_rs.contains("NativeDriverCostReconciled { residual, ..}"),
        "remainder_nanos must come from native_driver_cost_account residual; emitted:\n{main_rs}"
    );
    assert!(
        main_rs.contains("std::process::exit(1)")
            && main_rs.contains("NativeDriverCostOverAttributed")
            && main_rs.contains("NativeDriverCostRemainderExceedsTolerance")
            && main_rs.contains("REFUSED: native driver cost OverAttributed")
            && main_rs.contains("REFUSED: native driver cost RemainderExceedsTolerance"),
        "non-Reconciled arms must refuse and exit 1 so the law is not inert; emitted:\n{main_rs}"
    );
    assert!(
        !main_rs.contains("[native-prepare] "),
        "[native-prepare] is a subset of [native-prepare-split]; one representation"
    );
    assert!(
        main_rs.contains("[native-prepare-split]") && main_rs.contains("decls={}"),
        "the split line must carry decls=; emitted:\n{main_rs}"
    );
}

#[test]
fn a_completed_capture_with_no_cost_rows_is_unobserved() {
    let standing = native_driver_cost_row_standing(std::rc::Rc::new(
        NativeDriverChildStanding::NativeDriverChildExited {
            stderr: String::new(),
        },
    ));
    assert!(
        matches!(
            standing,
            NativeDriverCostRowStanding::NativeDriverCostRowsUnobserved
        ),
        "empty stderr after a completed spawn is unobserved, not a green partition"
    );
}

#[test]
fn a_planted_partition_line_is_observed() {
    let stderr = "[native-cost-partition] {\"verdict\":\"NativeDriverCostReconciled\"}\n";
    let standing = native_driver_cost_row_standing(std::rc::Rc::new(
        NativeDriverChildStanding::NativeDriverChildExited {
            stderr: stderr.to_string(),
        },
    ));
    assert!(
        matches!(
            standing,
            NativeDriverCostRowStanding::NativeDriverCostRowsObserved
        ),
        "the partition tag after exit is Observed"
    );
}

#[test]
fn a_child_that_has_not_exited_is_pending_not_unobserved() {
    let standing = native_driver_cost_row_standing(std::rc::Rc::new(
        NativeDriverChildStanding::NativeDriverChildStillRunning,
    ));
    assert!(
        matches!(
            standing,
            NativeDriverCostRowStanding::NativeDriverCostRowsPending
        ),
        "Command::output never returns until exit; empty mid-run is pending, not unobserved"
    );
}

#[test]
fn the_preparation_span_closes_before_any_declaration_is_evaluated() {
    // prepare and eval are exclusive rows of ONE partition, so a preparation interval that
    // contained its module's evaluation intervals would count that work twice and inflate the
    // exclusive sum against the parent. The 6 assertions above read markers and exit shapes and
    // cannot see the endpoint move: the emitted text is identical either way except for WHERE the
    // close sits, which is exactly what this asserts. Moving the close back across the evaluation
    // loop -- the regression this file is being extended for -- reds here and nowhere else.
    let main_rs = driver_main();
    let close = main_rs
        .match_indices("let this_prepare = span_nanos(prepare_started);")
        .map(|(i, _)| i)
        .collect::<Vec<usize>>();
    assert_eq!(
        close.len(),
        3,
        "each preparation arm closes its own span and yields it as the arm's VALUE -- an outer \
         binding assigned from every arm leaves its initialiser dead, which is an error in the \
         emitted crate under -D warnings. Expected one close per arm (context, resolve, infer/accepted); \
         emitted:\n{main_rs}"
    );
    let eval_start = main_rs
        .find("let evaluate_started = Instant::now();")
        .unwrap_or_else(|| {
            panic!("the emitted driver must open an evaluation span; emitted:\n{main_rs}")
        });
    let last_close = close.iter().copied().max().unwrap();
    assert!(
        last_close < eval_start,
        "every preparation close must precede the first declaration evaluation, or prepare contains \
         eval and the two exclusive rows double-count: last close at {last_close}, evaluation opens \
         at {eval_start}; emitted:\n{main_rs}"
    );
    // The accepted arm is the only one that evaluates, so its close is the one that can drift:
    // assert it sits between inference returning and the evaluation loop rather than after it.
    let infer_close = main_rs
        .find("module_infer_nanos = span_nanos(infer_started);")
        .unwrap_or_else(|| {
            panic!("the emitted driver must close an inference span; emitted:\n{main_rs}")
        });
    let accepted_close = close
        .iter()
        .copied()
        .find(|i| *i > infer_close)
        .unwrap_or_else(|| {
            panic!("the accepted arm must close preparation after inference; emitted:\n{main_rs}")
        });
    assert!(
        accepted_close < eval_start,
        "the accepted arm closes preparation at {accepted_close}, after evaluation opens at \
         {eval_start}: the prepared module's evaluation is inside its preparation span"
    );
}

// THE CLOCK THE PARTITION IS JUDGED ON, DISCRIMINATED BY EXECUTION. The law claims the rows
// partition the driver's OWN work, so the two controls below vary exactly the two things a wall
// clock confused: work nobody instrumented (must refuse) and time the host took away (must not).
// Both run the real thread-CPU read (`v1_interpreter::thread_cpu_nanos_checked`, the seed's
// realization of std.realization_measurement ObserveThreadCpuAtSubject) into the real fold.

fn thread_cpu() -> u128 {
    v1_compiler::v1_interpreter::thread_cpu_nanos_checked()
        .expect("CLOCK_THREAD_CPUTIME_ID must be readable on a host that judges the partition")
}

/// Spin until this thread has spent `nanos` of its own CPU. Bounded by CPU, not by iterations,
/// so the injected work is the same quantity on a quiet and a contended host.
fn burn_thread_cpu(nanos: u128) {
    let started = thread_cpu();
    let mut x: u64 = 1;
    while thread_cpu() - started < nanos {
        x = std::hint::black_box(x.wrapping_mul(6364136223846793005).wrapping_add(1));
    }
}

fn account_one_row(parent: u128, row: u128) -> NativeDriverCostAccounting {
    let row = row as i64;
    (*native_driver_cost_account(
        nanosecond(parent as i64),
        native_driver_exclusive_rows(move |k| match k {
            NativeDriverExclusiveRowKey::ExclusiveLoad => nanosecond(row),
            _ => nanosecond(0),
        }),
        native_driver_cost_remainder_tolerance_nanos(),
    ))
    .clone()
}

const TOLERANCE_NANOS: u128 = 50_000_000;
const INJECTED_NANOS: u128 = 300_000_000;

#[test]
fn a_host_stall_outside_every_row_reconciles_on_thread_cpu() {
    let parent_started = thread_cpu();
    let wall_started = std::time::Instant::now();
    let row_started = thread_cpu();
    burn_thread_cpu(20_000_000);
    let row = thread_cpu() - row_started;
    // The stall: the thread is off-CPU for longer than the tolerance, as under host contention.
    std::thread::sleep(std::time::Duration::from_nanos(INJECTED_NANOS as u64));
    let parent = thread_cpu() - parent_started;
    let wall = wall_started.elapsed().as_nanos();
    // The stall is real: judged on the wall, this same run would have refused.
    assert!(
        wall.saturating_sub(row) > TOLERANCE_NANOS,
        "control is vacuous unless the stall exceeds the tolerance on the wall: wall={wall} row={row}"
    );
    match account_one_row(parent, row) {
        NativeDriverCostAccounting::NativeDriverCostReconciled { .. } => {}
        other => panic!(
            "a stall is not driver work and must not refuse on thread CPU: parent={parent} row={row} -> {other:?}"
        ),
    }
}

#[test]
fn uninstrumented_driver_work_refuses_on_thread_cpu() {
    let parent_started = thread_cpu();
    let row_started = thread_cpu();
    burn_thread_cpu(20_000_000);
    let row = thread_cpu() - row_started;
    // Real work outside every row: the residual the tolerance exists to catch.
    burn_thread_cpu(INJECTED_NANOS);
    let parent = thread_cpu() - parent_started;
    match account_one_row(parent, row) {
        NativeDriverCostAccounting::NativeDriverCostRemainderExceedsTolerance {
            residual, ..
        } => {
            assert!(nanosecond_count_of(&residual) >= INJECTED_NANOS);
        }
        other => {
            panic!("uninstrumented CPU work must refuse: parent={parent} row={row} -> {other:?}")
        }
    }
}

fn nanosecond_count_of(n: &v1_compiler::std_measure::Nanosecond) -> u128 {
    v1_compiler::std_measure::nanosecond_count(n.clone()) as u128
}

/// The emitted driver judges on that clock: the parent and every row are CPU spans, and the one
/// wall read is printed as an observation and never reaches the fold.
#[test]
fn the_emitted_driver_judges_the_partition_on_thread_cpu_and_only_observes_the_wall() {
    let main_rs = driver_main();
    let start = main_rs
        .find("fn run_adjudication(")
        .expect("adjudication fn");
    let body = &main_rs[start..];
    let body = &body[..body.find("\n}\n").expect("adjudication fn end")];
    assert!(body.contains("let driver_start = cpu_mark();"));
    assert!(body.contains("let parent_span_nanos = cpu_span_nanos(driver_start);"));
    assert!(body.contains("nanosecond(native_cost_i64(parent_span_nanos))"));
    assert_eq!(
        body.matches("Instant::now()").count(),
        1,
        "exactly one wall read in adjudication -- the observation around the run"
    );
    assert!(
        !body.contains("parent_wall_nanos)),"),
        "the wall must not enter the fold"
    );
    assert!(body.contains("\"basis\": \"native_driver_thread_cpu\""));
    assert!(main_rs.contains("observed_thread_cpu_nanos(String::new())"));
}

/// The scheduling row is the engine CALL's measured span minus the demand time the engine
/// attributed, and the driver's collection after the call is its own row. The bookkeeping-only
/// realization -- summing the engine's internal spans -- is what left the work between them timed by
/// no row, so its spelling is refused here; and an over-attribution inside the call refuses rather
/// than clamping scheduling to zero.
#[test]
fn scheduling_is_the_engine_call_span_and_collection_is_its_own_row() {
    let main_rs = driver_main();
    let call = main_rs
        .find("let schedule_started = cpu_mark();")
        .expect("the engine call is spanned");
    let run = main_rs
        .find("let run = native_demand_schedule_universe(")
        .expect("engine call");
    let close = main_rs
        .find("let schedule_span_nanos = cpu_span_nanos(schedule_started);")
        .expect("engine call span closes");
    assert!(call < run && run < close, "the span must bracket the call");
    assert!(
        main_rs.contains("let demand_scheduling_nanos = schedule_span_nanos - attributed_in_call;")
    );
    // Every engine-attributed demand row leaves the call span, the occurrence census included:
    // leaving one out would count its windows in scheduling AND in its own row.
    assert!(main_rs.contains(
        "let attributed_in_call = engine_prepare_nanos + engine_eval_nanos + occurrence_census_nanos;"
    ));
    assert!(
        !main_rs.contains("let demand_scheduling_nanos = native_demand_scheduling_nanos("),
        "the bookkeeping-only sum must not be the scheduling row"
    );
    assert!(main_rs.contains("REFUSED: native driver cost OverAttributed inside the engine call"));
    let collect = main_rs
        .find("let collection_started = cpu_mark();")
        .expect("collection span");
    let push = main_rs
        .find("population.push(row.clone());")
        .expect("population collection");
    let collect_close = main_rs
        .find("let driver_collection_nanos = cpu_span_nanos(collection_started);")
        .expect("collection span closes");
    assert!(close < collect && collect < push && push < collect_close);
    assert!(main_rs.contains(
        "NativeDriverExclusiveRowKey::ExclusiveDriverCollection => nanosecond(native_cost_i64(driver_collection_nanos))"
    ));
}
