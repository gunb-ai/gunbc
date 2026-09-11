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
    native_driver_cost_rows_observed, NativeDriverCostAccounting, NativeDriverExclusiveRows,
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
    assert!(
        !native_driver_cost_rows_observed(String::new()),
        "empty stderr after a completed spawn is unobserved, not a green partition"
    );
}

#[test]
fn a_planted_shared_and_partition_line_is_observed() {
    let stderr =
        "[native-cost-shared] {\"producer\":\"std.compiler_entry.SourceRootEvalDriver\"}\n\
         [native-cost-partition] {\"verdict\":\"NativeDriverCostReconciled\"}\n";
    assert!(
        native_driver_cost_rows_observed(stderr.to_string()),
        "a planted completed capture must be observed"
    );
}
