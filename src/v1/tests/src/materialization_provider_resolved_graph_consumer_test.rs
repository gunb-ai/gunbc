use v1_compiler::cli_run::{
    make_eval_context, resolve_entry_graph, run_claim, serve_resolved_graph_v1_disk_probe_for_test,
    ClaimOutcome, ResolvedGraphProviderOutcome, OUTPUT_COMPILE_CLEAN_DIAGNOSTIC_UNION,
};
use v1_compiler::resolved_graph_cache::{closure_content_digest, transform_content_digest};
use v1_compiler::v1_compiler_compile::SourceFile;
use v1_compiler::v1_interpreter::ExecutionMode;
use v1_compiler::v1_rt;

use std::rc::Rc;

const WITNESS_ENTRY: &str = "dag/test/claim/materialization_provider_witness_test.dag";

fn fixture_sources() -> Vec<Rc<SourceFile>> {
    vec![Rc::new(SourceFile {
        path: "module_a.dag".to_string(),
        content: "module test.a\nfn answer() -> Int { 42 }\n".to_string(),
    })]
}

fn witness_roots() -> Vec<String> {
    let ws = crate::helpers::workspace_root();
    vec![
        ws.join("dag").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ]
}

fn witness_ctx() -> v1_compiler::v1_interpreter::InterpContext {
    let roots = witness_roots();
    let entry = crate::helpers::workspace_root()
        .join(WITNESS_ENTRY)
        .to_string_lossy()
        .into_owned();
    let (graph, si) = resolve_entry_graph(&roots, &entry).expect("resolve witness entry");
    make_eval_context(&graph, si, ExecutionMode::Hermetic)
}

#[test]
fn v1_disk_probe_refuses_incomplete_naming_compile_clean_diagnostic_union() {
    let sources = fixture_sources();
    let closure_digest = closure_content_digest(&sources);
    let compiler_digest = transform_content_digest();
    let content_digest = v1_rt::atom_identity_hash("fixture-payload".to_string());

    let outcome = serve_resolved_graph_v1_disk_probe_for_test(
        &closure_digest,
        &compiler_digest,
        &content_digest,
        128,
    )
    .expect("provider serve");

    match outcome {
        ResolvedGraphProviderOutcome::RefusedIncomplete { missing } => {
            assert_eq!(missing.len(), 1);
            assert_eq!(missing[0], OUTPUT_COMPILE_CLEAN_DIAGNOSTIC_UNION);
        }
        other => panic!("expected ProviderRefusedIncomplete, got {other:?}"),
    }
}

#[test]
fn witness_request_perturbation_flips_provider_outcome_arm() {
    let ctx = witness_ctx();
    assert_eq!(
        run_claim(
            &ctx,
            "incomplete_v1_shape_artifact_refuses_naming_the_diagnostic_union"
        ),
        ClaimOutcome::Pass,
        "v1-shape artifact must refuse incomplete naming the diagnostic union"
    );
    assert_eq!(
        run_claim(
            &ctx,
            "red_declared_input_miss_stale_artifact_refuses_as_wrong_artifact"
        ),
        ClaimOutcome::Pass,
        "perturbed declared input must flip to wrong-artifact refusal, not incomplete"
    );
}
