//! P5-SPIKE: serialized ResolvedGraph fixture seam soundness (§5 fail-closed).
//!
//! (b) Round-trip: compile -> cache payload serde -> load -> emit == direct emit.
//! (c) Born-mark trap: fixture with stripped intern_table fails the loader guard loudly.

use std::rc::Rc;

use v1_compiler::resolved_graph_cache::{
    deserialize_fixture_payload_for_test, serialize_fixture_payload_for_test,
    validate_fixture_intern_table_for_test, CachedResolvedGraph,
};
use v1_compiler::v1_compiler_compile::{
    compile_to_resolved, emit_resolved_for_target, empty_complexity_report, ResolvedPipelineResult,
};
use v1_compiler::v1_compiler_infer_items::ResolvedGraph;
use v1_compiler::v1_compiler_languages::RenderTarget;
use v1_compiler::v1_std_core::{empty_intern_table, is_error_diagnostic, NewlineIndex};

use crate::helpers::{compile_dag, resolve_imports_transitively};

const FIXTURE_SOURCE: &str = r#"module fixture_seam
type Box { v: Int }
fn wrap(n: Int) -> Box { Box { v: n } }
fn unbox(b: Box) -> Int { b.v }
"#;

fn resolved_from_source() -> Rc<ResolvedPipelineResult> {
    let sources = resolve_imports_transitively("fixture_seam.dag", FIXTURE_SOURCE);
    compile_to_resolved(Rc::new(sources))
}

fn emit_rust(
    resolved: Rc<ResolvedPipelineResult>,
) -> Rc<v1_compiler::v1_compiler_compile::PipelineResult> {
    emit_resolved_for_target(resolved, RenderTarget::Rust)
}

fn emit_files_fingerprint(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> String {
    let mut pairs: Vec<(String, String)> = result
        .files
        .iter()
        .map(|f| (f.path.clone(), f.content.clone()))
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    pairs
        .into_iter()
        .map(|(p, c)| format!("{p}\n{c}"))
        .collect::<Vec<_>>()
        .join("\n---\n")
}

fn resolved_pipeline_from_cached(cached: CachedResolvedGraph) -> Rc<ResolvedPipelineResult> {
    let newline_indices = Rc::new(
        cached
            .source_indices
            .values()
            .cloned()
            .collect::<Vec<Rc<NewlineIndex>>>(),
    );
    Rc::new(ResolvedPipelineResult {
        graph: Some(cached.graph),
        diagnostics: Rc::new(vec![]),
        source_indices: cached.source_indices,
        complexity: empty_complexity_report(),
        ownership: Rc::new(vec![]),
        newline_indices,
    })
}

/// Simulate naive load: keep binding ids but drop the embedded intern_table (born-mark trap).
fn strip_intern_table_from_fixture(cached: CachedResolvedGraph) -> CachedResolvedGraph {
    let graph = cached.graph;
    let modules = Rc::new(
        graph
            .modules
            .iter()
            .map(|m| {
                let mut typed = (**m).clone();
                let mut type_env = (*typed.type_env).clone();
                type_env.intern_table = empty_intern_table();
                typed.type_env = Rc::new(type_env);
                Rc::new(typed)
            })
            .collect::<Vec<_>>(),
    );
    CachedResolvedGraph {
        graph: Rc::new(ResolvedGraph {
            modules,
            item_registry: graph.item_registry.clone(),
            diagnostics: graph.diagnostics.clone(),
            emit_graph_info: graph.emit_graph_info.clone(),
        }),
        source_indices: cached.source_indices,
    }
}

#[test]
fn ir_fixture_round_trip_emit_is_bit_identical_to_direct() {
    let direct = resolved_from_source();
    let graph = direct.graph.clone().expect("typed graph");
    assert!(
        direct
            .diagnostics
            .iter()
            .all(|d| !is_error_diagnostic(d.diagnostic.clone())),
        "direct resolve must not error: {:?}",
        direct.diagnostics
    );

    let payload =
        serialize_fixture_payload_for_test(graph.as_ref(), direct.source_indices.as_ref())
            .expect("serialize fixture");
    let cached = deserialize_fixture_payload_for_test(&payload).expect("deserialize fixture");
    validate_fixture_intern_table_for_test(&cached).expect("valid fixture intern table");

    let direct_fp = emit_files_fingerprint(&emit_rust(direct.clone()));
    let fixture_fp = emit_files_fingerprint(&emit_rust(resolved_pipeline_from_cached(cached)));
    assert_eq!(
        direct_fp, fixture_fp,
        "fixture-loaded emit must be bit-identical to direct path"
    );

    // Cross-check against helpers::compile_dag (full source->pipeline entry).
    let via_helpers = compile_dag(FIXTURE_SOURCE);
    assert_eq!(
        emit_files_fingerprint(&via_helpers),
        direct_fp,
        "compile_dag oracle must match compile_to_resolved emit"
    );
}

#[test]
fn ir_fixture_wrong_intern_table_born_marks_fail_loader_guard() {
    let direct = resolved_from_source();
    let graph = direct.graph.clone().expect("typed graph");
    let payload =
        serialize_fixture_payload_for_test(graph.as_ref(), direct.source_indices.as_ref())
            .expect("serialize fixture");
    let cached = deserialize_fixture_payload_for_test(&payload).expect("deserialize fixture");
    let poisoned = strip_intern_table_from_fixture(cached);

    let guard_err = validate_fixture_intern_table_for_test(&poisoned)
        .expect_err("stripped intern_table must fail born-mark guard");
    assert!(
        guard_err.contains("intern-table born-mark mismatch"),
        "expected loud born-mark diagnostic, got: {guard_err}"
    );
}
