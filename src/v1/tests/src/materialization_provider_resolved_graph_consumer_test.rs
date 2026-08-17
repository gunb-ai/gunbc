//! Provider consumer matrix for the resolved-graph disk seam (Dispatch G).
//!
//! Checkable receipt: `dag/test/retirement/materialization_provider_resolved_graph_consumer_retained.dag`.

use im::Vector;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use v1_compiler::cli_run::{
    load_sources_for_entry, materialization_provider_ctx_build_count_for_test,
    reset_materialization_provider_ctx_for_test, resolve_closure_request_key_from_digests,
    resolve_entry_graph, resolved_graph_parts_semantic_digest,
    serve_resolved_graph_stored_disk_probe_for_test, ResolvedGraphProviderOutcome,
};
use v1_compiler::resolved_graph_cache::{
    closure_content_digest, encode_resolved_graph_parts, transform_content_digest,
    FaithfulResolvedGraphProbeParts,
};
use v1_compiler::v1_std_core::ErrorNode;

fn empty_compile_clean_diags() -> Vector<Rc<ErrorNode>> {
    Vector::new()
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = crate::helpers::workspace_root()
        .join("target")
        .join(format!(
            "gunbc-mp-consumer-{label}-{}-{}",
            std::process::id(),
            nanos
        ));
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn write_fixture(dir: &std::path::Path) -> (Vec<String>, String) {
    let common = "module test.common\n\
        type Box { v: Int }\n\
        fn boxed(n: Int) -> Box { Box { v: n } }\n\
        fn unbox(b: Box) -> Int { b.v }\n";
    let shared1 = "module test.shared1\nfn val() -> Int { 10 }\n";
    // No import statements: this branch's parser refuses them outright, so a
    // cross-module reference is written container.member and THAT reference is
    // the dependency edge.
    let entry_a = "module test.a\n\
        fn witness_a_true() -> Bool { (test.common.unbox(test.common.boxed(test.shared1.val())) + 0) == 10 }\n";
    for (name, src) in [
        ("common.dag", common),
        ("shared1.dag", shared1),
        ("entry_a.dag", entry_a),
    ] {
        fs::write(dir.join(name), src).unwrap_or_else(|e| panic!("write {name}: {e}"));
    }
    let roots = vec![dir.to_string_lossy().into_owned()];
    let a = dir.join("entry_a.dag").to_string_lossy().into_owned();
    (roots, a)
}

fn fixture_parts_and_keys(
    roots: &[String],
    entry: &str,
) -> (
    FaithfulResolvedGraphProbeParts,
    String,
    String,
    String,
    String,
) {
    let (graph, si) = resolve_entry_graph(roots, entry).expect("resolve fixture");
    let sources = load_sources_for_entry(roots, entry).expect("sources");
    let closure_digest = closure_content_digest(&sources);
    let compiler_digest = transform_content_digest();
    let encoded = encode_resolved_graph_parts(&graph, si.as_ref(), &empty_compile_clean_diags())
        .expect("encode parts");
    let stored_request_key =
        resolve_closure_request_key_from_digests(&closure_digest, &compiler_digest)
            .expect("request key");
    let stored_semantic_digest = resolved_graph_parts_semantic_digest(
        &encoded.graph_digest,
        encoded.graph_bytes.len() as u64,
        &encoded.indices_digest,
        encoded.indices_bytes.len() as u64,
        &encoded.union_digest,
        encoded.union_bytes.len() as u64,
    )
    .expect("semantic digest");
    let parts = FaithfulResolvedGraphProbeParts {
        graph_digest: encoded.graph_digest,
        graph_bytes: encoded.graph_bytes.len() as u64,
        indices_digest: encoded.indices_digest,
        indices_bytes: encoded.indices_bytes.len() as u64,
        union_digest: encoded.union_digest,
        union_bytes: encoded.union_bytes.len() as u64,
    };
    (
        parts,
        closure_digest,
        compiler_digest,
        stored_request_key,
        stored_semantic_digest,
    )
}

#[test]
fn stored_disk_probe_hit_builds_provider_ctx_once() {
    reset_materialization_provider_ctx_for_test();
    let dir = temp_dir("hit");
    let (roots, entry) = write_fixture(&dir);
    let (parts, closure, compiler, request_key, semantic) = fixture_parts_and_keys(&roots, &entry);
    let builds_before = materialization_provider_ctx_build_count_for_test();
    assert!(
        builds_before >= 1,
        "fixture key derivation must boot provider ctx once before serve"
    );
    let outcome = serve_resolved_graph_stored_disk_probe_for_test(
        &closure,
        &compiler,
        &request_key,
        &semantic,
        &parts,
    )
    .expect("valid v3 probe must hit");
    assert_eq!(outcome, ResolvedGraphProviderOutcome::Hit);
    assert_eq!(
        materialization_provider_ctx_build_count_for_test(),
        builds_before,
        "first hit must reuse provider ctx built during fixture setup"
    );
    let outcome2 = serve_resolved_graph_stored_disk_probe_for_test(
        &closure,
        &compiler,
        &request_key,
        &semantic,
        &parts,
    )
    .expect("second hit must reuse ctx");
    assert_eq!(outcome2, ResolvedGraphProviderOutcome::Hit);
    assert_eq!(
        materialization_provider_ctx_build_count_for_test(),
        builds_before,
        "second hit must reuse provider ctx"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn stored_disk_probe_wrong_request_key_refuses() {
    reset_materialization_provider_ctx_for_test();
    let dir = temp_dir("wrong-key");
    let (roots, entry) = write_fixture(&dir);
    let (parts, closure, compiler, _request_key, semantic) = fixture_parts_and_keys(&roots, &entry);
    let outcome = serve_resolved_graph_stored_disk_probe_for_test(
        &closure,
        &compiler,
        "ffffffffffffffff",
        &semantic,
        &parts,
    )
    .expect("wrong stored request key must refuse");
    assert_eq!(outcome, ResolvedGraphProviderOutcome::RefusedWrongArtifact);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn stored_disk_probe_wrong_semantic_digest_refuses() {
    reset_materialization_provider_ctx_for_test();
    let dir = temp_dir("wrong-semantic");
    let (roots, entry) = write_fixture(&dir);
    let (parts, closure, compiler, request_key, _semantic) = fixture_parts_and_keys(&roots, &entry);
    let outcome = serve_resolved_graph_stored_disk_probe_for_test(
        &closure,
        &compiler,
        &request_key,
        "ffffffffffffffff",
        &parts,
    )
    .expect("wrong stored semantic digest must refuse via provider_serve");
    assert_eq!(
        outcome,
        ResolvedGraphProviderOutcome::RefusedWrongContent,
        "modeled provider_serve must own semantic digest comparison"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn incomplete_provider_outcome_maps_to_typed_refusal_message() {
    let msg = v1_compiler::cli_run::provider_integrity_refusal_message_for_test(
        ResolvedGraphProviderOutcome::RefusedIncomplete {
            missing: vec!["compile_clean_diagnostic_union".to_string()],
        },
    )
    .expect("incomplete must refuse");
    assert!(
        msg.contains("incomplete") && msg.contains("compile_clean_diagnostic_union"),
        "typed incomplete refusal: {msg}"
    );
}

#[test]
fn synthetic_wrong_semantic_parts_refuse_via_provider_serve() {
    reset_materialization_provider_ctx_for_test();
    let builds_before = materialization_provider_ctx_build_count_for_test();
    let parts = FaithfulResolvedGraphProbeParts {
        graph_digest: "0000000000000000".to_string(),
        graph_bytes: 1,
        indices_digest: "0000000000000000".to_string(),
        indices_bytes: 1,
        union_digest: "0000000000000000".to_string(),
        union_bytes: 1,
    };
    let request_key =
        resolve_closure_request_key_from_digests("0000000000000000", "0000000000000000")
            .expect("request key");
    let outcome = serve_resolved_graph_stored_disk_probe_for_test(
        "0000000000000000",
        "0000000000000000",
        &request_key,
        "0000000000000000",
        &parts,
    )
    .expect("wrong stored semantic digest must refuse via provider_serve");
    assert_eq!(
        outcome,
        ResolvedGraphProviderOutcome::RefusedWrongContent,
        "stored header semantic digest must disagree with derived artifact content"
    );
    assert_eq!(
        materialization_provider_ctx_build_count_for_test(),
        builds_before + 1,
        "stored disk probe must build provider ctx before provider_serve refusal"
    );
}
