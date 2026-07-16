use crate::helpers::workspace_root;
use v1_compiler::cli_run::{make_eval_context, resolve_entry_graph, run_claim, ClaimOutcome};
use v1_compiler::v1_interpreter::ExecutionMode;

#[test]
fn c_compilation_unit_witnesses_green_by_execution() {
    let roots = vec![
        workspace_root()
            .join("src/v2")
            .to_string_lossy()
            .into_owned(),
        workspace_root().join("dag").to_string_lossy().into_owned(),
    ];
    let entry = workspace_root()
        .join("src/v2/test/claim/c_compilation_unit_witness_test.dag")
        .to_string_lossy()
        .into_owned();
    let (graph, si) = resolve_entry_graph(&roots, &entry).expect("resolve c witness entry");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Hermetic);
    let outcome = run_claim(&ctx, "c_compilation_unit_witnesses");
    assert!(
        matches!(outcome, ClaimOutcome::Pass),
        "c_compilation_unit_witnesses must pass by execution, got {:?}",
        outcome
    );
}
