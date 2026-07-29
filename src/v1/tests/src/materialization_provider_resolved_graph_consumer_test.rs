use v1_compiler::cli_run::{
    build_multi_entry_index, make_eval_context, materialization_provider_consumer,
    resolve_entry_graph, resolve_entry_with_index, run_claim,
    serve_resolved_graph_v1_disk_probe_for_test, ClaimOutcome, ResolvedGraphProviderOutcome,
    OUTPUT_COMPILE_CLEAN_DIAGNOSTIC_UNION,
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
fn provider_ctx_reused_across_disk_probes() {
    materialization_provider_consumer::reset_materialization_provider_ctx_for_test();
    let sources = fixture_sources();
    let closure_digest = closure_content_digest(&sources);
    let compiler_digest = transform_content_digest();
    let content_digest = v1_rt::atom_identity_hash("fixture-payload-reuse".to_string());

    let _ = serve_resolved_graph_v1_disk_probe_for_test(
        &closure_digest,
        &compiler_digest,
        &content_digest,
        128,
    )
    .expect("first probe");
    let builds_after_first =
        materialization_provider_consumer::materialization_provider_ctx_build_count_for_test();
    assert_eq!(
        builds_after_first, 1,
        "first probe must cold-build the provider interpreter context once"
    );

    let _ = serve_resolved_graph_v1_disk_probe_for_test(
        &closure_digest,
        &compiler_digest,
        &content_digest,
        256,
    )
    .expect("second probe");
    assert_eq!(
        materialization_provider_consumer::materialization_provider_ctx_build_count_for_test(),
        1,
        "repeat probes must reuse the process-local provider context"
    );
}

#[test]
fn v1_disk_hit_provider_refusal_stops_resolve_line() {
    let dir = crate::helpers::workspace_root()
        .join("target")
        .join(format!("gunbc-provider-refuse-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let entry_src = "module test.provider_refuse\nfn witness() -> Bool { true }\n";
    let entry = dir.join("entry.dag");
    std::fs::write(&entry, entry_src).expect("write entry");
    let roots = vec![dir.to_string_lossy().into_owned()];
    let entry_path = entry.to_string_lossy().into_owned();
    let cache_dir = dir.join("cache");
    std::fs::create_dir_all(&cache_dir).expect("cache dir");

    let prev = std::env::var_os("GUNBC_RESOLVED_GRAPH_CACHE_DIR");
    std::env::set_var(
        "GUNBC_RESOLVED_GRAPH_CACHE_DIR",
        cache_dir.to_string_lossy().as_ref(),
    );
    let index = build_multi_entry_index(&roots);
    let _ = resolve_entry_with_index(&index, &entry_path).expect("cold resolve warms cache");
    let index2 = build_multi_entry_index(&roots);
    let err = resolve_entry_with_index(&index2, &entry_path)
        .expect_err("disk hit must not widen to cold resolve");
    if let Some(v) = prev {
        std::env::set_var("GUNBC_RESOLVED_GRAPH_CACHE_DIR", v);
    } else {
        std::env::remove_var("GUNBC_RESOLVED_GRAPH_CACHE_DIR");
    }
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        err.contains("provider refused incomplete disk hit"),
        "typed provider refusal must stop the line: {err}"
    );
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
