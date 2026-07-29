use v1_compiler::cli_run::{
    serve_resolved_graph_v1_disk_probe_for_test, ResolvedGraphProviderOutcome,
    OUTPUT_COMPILE_CLEAN_DIAGNOSTIC_UNION,
};
use v1_compiler::resolved_graph_cache::{closure_content_digest, transform_content_digest};
use v1_compiler::v1_compiler_compile::SourceFile;
use v1_compiler::v1_rt;

use std::rc::Rc;

fn fixture_sources() -> Vec<Rc<SourceFile>> {
    vec![Rc::new(SourceFile {
        path: "module_a.dag".to_string(),
        content: "module test.a\nfn answer() -> Int { 42 }\n".to_string(),
    })]
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
fn perturbed_compiler_identity_flips_provider_outcome_from_incomplete_to_wrong_artifact() {
    let sources = fixture_sources();
    let closure_digest = closure_content_digest(&sources);
    let compiler_digest = transform_content_digest();
    let content_digest = v1_rt::atom_identity_hash("fixture-payload".to_string());

    let matched = serve_resolved_graph_v1_disk_probe_for_test(
        &closure_digest,
        &compiler_digest,
        &content_digest,
        128,
    )
    .expect("matched provider serve");
    assert!(
        matches!(
            matched,
            ResolvedGraphProviderOutcome::RefusedIncomplete { .. }
        ),
        "expected incomplete refusal for matched request, got {matched:?}"
    );

    let perturbed_compiler = v1_rt::atom_identity_hash("different-compiler-identity".to_string());
    let flipped = serve_resolved_graph_v1_disk_probe_for_test(
        &closure_digest,
        &perturbed_compiler,
        &content_digest,
        128,
    )
    .expect("perturbed provider serve");
    assert_eq!(
        flipped,
        ResolvedGraphProviderOutcome::RefusedWrongArtifact,
        "perturbed declared input must refuse as wrong artifact, not incomplete"
    );
}
