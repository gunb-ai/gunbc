use v1_compiler::cli_run::{
    build_multi_entry_index, closure_subject_for_entry, make_eval_context,
    resolve_entry_with_index, run_claim, run_claim_measured, ClaimOutcome,
};
use v1_compiler::v1_interpreter::ExecutionMode;

fn outcome_tag(o: &ClaimOutcome) -> &'static str {
    match o {
        ClaimOutcome::Pass => "PASS",
        ClaimOutcome::Fail => "FAIL",
        ClaimOutcome::NotBool { .. } => "NOTBOOL",
        ClaimOutcome::RuntimeError { .. } => "RUNTIMEERR",
    }
}

#[test]
fn eval_measurement_does_not_change_witness_verdict() {
    let ws = crate::helpers::workspace_root();
    let source_roots = vec![ws.join("dag").to_string_lossy().into_owned()];
    let entry = ws
        .join("dag/test/claim/realization_measurement_keystone_test.dag")
        .to_string_lossy()
        .into_owned();
    let function = "realization_measurement_keystone_witnesses";

    let index = build_multi_entry_index(&source_roots);
    let closure_subject = closure_subject_for_entry(&index, &entry).expect("closure subject");
    let (graph, source_indices) = resolve_entry_with_index(&index, &entry).expect("resolve");
    let ctx = make_eval_context(&graph, source_indices, ExecutionMode::Wet);

    let plain = run_claim(&ctx, function);
    let (measured, receipt) = run_claim_measured(&ctx, &closure_subject, function);

    assert_eq!(
        outcome_tag(&plain),
        outcome_tag(&measured),
        "measurement must not change witness verdict"
    );
    assert!(
        receipt.wall_nanos > 0,
        "PerformanceReceipt must record positive wall time"
    );
    assert_eq!(
        receipt.work_shape, function,
        "work_shape must match witness function name"
    );
    assert!(
        !receipt.subject_key.is_empty(),
        "subject_key must be content-hash keyed"
    );
}
