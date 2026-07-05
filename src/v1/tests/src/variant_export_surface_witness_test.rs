//! P3+P5 purity oracle: incremental `variant_surfaces` / `type_name_index` must
//! preserve re-export variant resolution and full-pipeline diagnostics.
//!
//! RED control: `typecheck_module_isolated` (empty surfaces) must fail on a
//! re-export chain where the consumer imports a coproduct arm through a proxy.

use std::collections::HashMap;
use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{front_end_sources, normalize_graph, SourceFile};
use v1_compiler::v1_compiler_infer::{
    build_variant_export_surface, typecheck_module, typecheck_module_isolated,
    TypecheckModuleResult, VariantExportSurface,
};
use v1_compiler::v1_compiler_infer_items::TypedModule;
use v1_compiler::v1_compiler_resolve::ResolvedModule;
use v1_compiler::v1_std_core::{
    authored_name_at, diagnostic_to_message, empty_intern_table, is_error_diagnostic,
    CompilerDiagnostic, InternTable, NewlineIndex,
};
use v1_compiler::v1_rt;

use crate::helpers::{compile_multi, diagnostic_messages, resolve_imports_transitively};

const PROVIDER: &str = "module test.provider\ntype Shape = Circle { r: Int } | Square { s: Int }\n";

const REEXPORT: &str = "module test.reexport\n\
import test.provider { Circle }\n";

const CONSUMER: &str = "module test.consumer\n\
import test.reexport { Circle }\n\
fn mk() -> Circle { Circle { r: 1 } }\n";

fn fixture_sources() -> Vec<Rc<SourceFile>> {
    resolve_imports_transitively("consumer.dag", CONSUMER)
        .into_iter()
        .chain(resolve_imports_transitively("provider.dag", PROVIDER))
        .chain(resolve_imports_transitively("reexport.dag", REEXPORT))
        .fold(HashMap::new(), |mut acc, src| {
            acc.entry(src.path.clone()).or_insert(src);
            acc
        })
        .into_values()
        .collect()
}

fn resolved_module_graph(
    sources: Rc<Vec<Rc<SourceFile>>>,
) -> (
    Rc<Vec<Rc<ResolvedModule>>>,
    Rc<HashMap<String, Rc<NewlineIndex>>>,
    Rc<InternTable>,
) {
    let frontend = front_end_sources(sources);
    let graph = frontend.graph.expect("resolved module graph");
    let source_indices = frontend
        .newline_indices
        .iter()
        .cloned()
        .fold(
            v1_rt::rc_empty_map::<String, Rc<NewlineIndex>>(),
            |acc, si| v1_rt::rc_map_insert(acc, si.file.clone(), si),
        );
    let norm = normalize_graph(graph, source_indices.clone());
    (
        norm.graph.modules.clone(),
        source_indices,
        frontend.intern_table,
    )
}

fn hard_diagnostic_messages(result: &TypecheckModuleResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

fn typecheck_resolved_incremental(
    modules: &[Rc<ResolvedModule>],
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    intern_table: Rc<InternTable>,
) -> Vec<Rc<TypecheckModuleResult>> {
    let mut module_index: Rc<HashMap<String, Rc<TypedModule>>> = v1_rt::rc_empty_map();
    let mut variant_surfaces: Rc<HashMap<String, Rc<VariantExportSurface>>> =
        v1_rt::rc_empty_map();
    let mut results = Vec::new();
    for resolved in modules {
        let tc = typecheck_module(
            resolved.clone(),
            module_index.clone(),
            variant_surfaces.clone(),
            source_indices.clone(),
            intern_table.clone(),
        );
        let typed = tc.typed.clone();
        let path = authored_name_at(source_indices.clone(), typed.module.clone());
        variant_surfaces = v1_rt::rc_map_insert(
            variant_surfaces.clone(),
            path.clone(),
            build_variant_export_surface(
                typed.clone(),
                variant_surfaces.clone(),
                source_indices.clone(),
            ),
        );
        module_index = v1_rt::rc_map_insert(module_index, path, typed);
        results.push(tc);
    }
    results
}

fn consumer_resolved<'a>(
    modules: &'a [Rc<ResolvedModule>],
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> &'a Rc<ResolvedModule> {
    modules
        .iter()
        .find(|m| authored_name_at(source_indices.clone(), m.module.clone()) == "test.consumer")
        .expect("consumer module in resolved graph")
}

#[test]
fn variant_reexport_chain_full_pipeline_is_clean() {
    let result = compile_multi(&[
        ("provider.dag", PROVIDER),
        ("reexport.dag", REEXPORT),
        ("consumer.dag", CONSUMER),
    ]);
    let msgs: Vec<_> = diagnostic_messages(&result)
        .into_iter()
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty(),
        "re-export variant chain must typecheck clean via production reconcile, got:\n{msgs:?}"
    );
}

#[test]
fn variant_reexport_incremental_surfaces_match_full_pipeline_fingerprint() {
    let sources = Rc::new(fixture_sources());
    let pipeline = compile_multi(&[
        ("provider.dag", PROVIDER),
        ("reexport.dag", REEXPORT),
        ("consumer.dag", CONSUMER),
    ]);
    let pipeline_msgs: Vec<_> = diagnostic_messages(&pipeline)
        .into_iter()
        .filter(|m| !m.starts_with("complexity: "))
        .collect();

    let (modules, source_indices, intern_table) = resolved_module_graph(sources);
    let incremental = typecheck_resolved_incremental(&modules, source_indices, intern_table);
    let consumer_tc = incremental.last().expect("consumer typecheck result");
    let incremental_msgs = hard_diagnostic_messages(consumer_tc);

    assert!(
        pipeline_msgs.is_empty() && incremental_msgs.is_empty(),
        "incremental-surface path must match full pipeline (both clean), pipeline={pipeline_msgs:?} incremental={incremental_msgs:?}"
    );
}

#[test]
fn variant_reexport_empty_surfaces_red_control() {
    let sources = Rc::new(fixture_sources());
    let (modules, source_indices, intern_table) = resolved_module_graph(sources);

    let incremental =
        typecheck_resolved_incremental(&modules, source_indices.clone(), intern_table.clone());
    let module_index: Rc<HashMap<String, Rc<TypedModule>>> = incremental
        .iter()
        .map(|tc| {
            let path = authored_name_at(source_indices.clone(), tc.typed.module.clone());
            (path, tc.typed.clone())
        })
        .fold(v1_rt::rc_empty_map(), |acc, (k, v)| v1_rt::rc_map_insert(acc, k, v));

    let consumer = consumer_resolved(&modules, &source_indices);
    let isolated = typecheck_module_isolated(
        consumer.clone(),
        module_index,
        source_indices.clone(),
        intern_table,
    );
    let has_unresolved = isolated.diagnostics.iter().any(|d| {
        matches!(
            &*d.diagnostic,
            CompilerDiagnostic::UnresolvedType { name, .. } if name == "Circle"
        )
    });
    assert!(
        has_unresolved,
        "isolated typecheck (empty variant_surfaces) must RED on re-export arm import, got:\n{}",
        hard_diagnostic_messages(&isolated).join("\n")
    );
}
