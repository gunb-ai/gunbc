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
    compile_to_resolved(Rc::new(sources.into()))
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
        diagnostics: Rc::new(im::vector![]),
        source_indices: cached.source_indices,
        complexity: empty_complexity_report(),
        ownership: Rc::new(im::vector![]),
        newline_indices: Rc::new(newline_indices.iter().cloned().collect()),
    })
}

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
            modules: Rc::new(modules.iter().cloned().collect()),
            item_registry: graph.item_registry.clone(),
            diagnostics: graph.diagnostics.clone(),
            emit_graph_info: graph.emit_graph_info.clone(),
        }),
        source_indices: cached.source_indices,
    }
}

fn remap_binding_intern_name_mismatch(cached: CachedResolvedGraph) -> CachedResolvedGraph {
    let graph = cached.graph;
    let modules = Rc::new(
        graph
            .modules
            .iter()
            .map(|m| {
                let mut typed = (**m).clone();
                let mut type_env = (*typed.type_env).clone();
                let mut table = (*type_env.intern_table).clone();
                let victim = type_env.bindings.iter().find_map(|(id, binding)| {
                    if binding.name.is_empty() {
                        None
                    } else {
                        Some((*id, binding.name.clone()))
                    }
                });
                if let Some((id, expected_name)) = victim {
                    let wrong_name = if expected_name == "Box" {
                        "Int".to_string()
                    } else {
                        "Box".to_string()
                    };
                    let mut strings = (*table.strings).clone();
                    if (id as usize) < strings.len() {
                        strings[id as usize] = wrong_name;
                    }
                    table.strings = Rc::new(strings);
                }
                type_env.intern_table = Rc::new(table);
                typed.type_env = Rc::new(type_env);
                Rc::new(typed)
            })
            .collect::<Vec<_>>(),
    );
    CachedResolvedGraph {
        graph: Rc::new(ResolvedGraph {
            modules: Rc::new(modules.iter().cloned().collect()),
            item_registry: graph.item_registry.clone(),
            diagnostics: graph.diagnostics.clone(),
            emit_graph_info: graph.emit_graph_info.clone(),
        }),
        source_indices: cached.source_indices,
    }
}

fn assert_born_mark_guard_rejects(cached: CachedResolvedGraph, case: &str) {
    let guard_err = validate_fixture_intern_table_for_test(&cached)
        .expect_err(&format!("{case} must fail born-mark guard"));
    assert!(
        guard_err.contains("intern-table born-mark mismatch"),
        "{case}: expected loud born-mark diagnostic, got: {guard_err}"
    );
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
    assert_born_mark_guard_rejects(poisoned, "empty intern_table");
}

#[test]
fn ir_fixture_wrong_but_present_intern_table_born_marks_fail_loader_guard() {
    let direct = resolved_from_source();
    let graph = direct.graph.clone().expect("typed graph");
    let payload =
        serialize_fixture_payload_for_test(graph.as_ref(), direct.source_indices.as_ref())
            .expect("serialize fixture");
    let cached = deserialize_fixture_payload_for_test(&payload).expect("deserialize fixture");
    let poisoned = remap_binding_intern_name_mismatch(cached);
    assert_born_mark_guard_rejects(
        poisoned,
        "wrong-but-present intern_table string at binding id",
    );
}
