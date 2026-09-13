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
    native_driver_cost_row_standing, NativeDriverChildStanding, NativeDriverCostAccounting,
    NativeDriverCostRowStanding, NativeDriverExclusiveRows,
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
        std::rc::Rc::new(NativeDriverExclusiveRows {
            load: nanosecond(40),
            context: nanosecond(40),
            prepare: nanosecond(40),
            eval: nanosecond(0),
            universe_derivation: nanosecond(0),
            receipt_admission: nanosecond(0),
            row_serialization: nanosecond(0),
            module_release: nanosecond(0),
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
        std::rc::Rc::new(NativeDriverExclusiveRows {
            load: nanosecond(432921485),
            context: nanosecond(3380085706753),
            prepare: nanosecond(59277814291),
            eval: nanosecond(3531894),
            universe_derivation: nanosecond(0),
            receipt_admission: nanosecond(0),
            row_serialization: nanosecond(0),
            module_release: nanosecond(0),
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
