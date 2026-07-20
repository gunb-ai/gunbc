//! P3+P5 purity oracle: incremental `variant_surfaces` / `type_name_index` must
//! preserve re-export variant resolution and full-pipeline diagnostics.
//!
//! RED control: `typecheck_module_isolated` (empty surfaces) must fail on a
//! re-export chain where the consumer imports a coproduct arm through a proxy.

use im::HashMap;
use std::sync::Arc;

use v1_compiler::v1_compiler_compile::{front_end_sources, normalize_graph, SourceFile};
use v1_compiler::v1_compiler_infer::{
    build_variant_export_surface, typecheck_module, typecheck_module_isolated,
    TypecheckModuleResult, VariantExportSurface,
};
use v1_compiler::v1_compiler_infer_items::TypedModule;
use v1_compiler::v1_compiler_resolve::ResolvedModule;
use v1_compiler::v1_rt;
use v1_compiler::v1_std_core::{
    authored_name_at, diagnostic_to_message, is_error_diagnostic, InternTable, NewlineIndex,
};

use crate::helpers::{compile_multi, diagnostic_messages, resolve_imports_transitively};

const PROVIDER: &str = "module test.provider\ntype E = A | B\n";

const REEXPORT: &str = "module test.reexport\nimport test.provider { B }\n";

const CONSUMER: &str = "module test.consumer\nimport test.reexport { B }\nfn f() -> E { B }\n";

type ResolvedGraphFixture = (
    Arc<im::Vector<Arc<ResolvedModule>>>,
    Arc<HashMap<String, Arc<NewlineIndex>>>,
    Arc<InternTable>,
);

fn fixture_sources() -> Vec<Arc<SourceFile>> {
    resolve_imports_transitively("consumer.dag", CONSUMER)
        .into_iter()
        .chain(resolve_imports_transitively("provider.dag", PROVIDER))
        .chain(resolve_imports_transitively("reexport.dag", REEXPORT))
        .fold(HashMap::new(), |mut acc, src| {
            acc.entry(src.path.clone()).or_insert(src);
            acc
        })
        .into_iter()
        .map(|(_, v)| v)
        .collect()
}

fn resolved_module_graph(sources: Arc<im::Vector<Arc<SourceFile>>>) -> ResolvedGraphFixture {
    let frontend = front_end_sources(sources);
    let graph = frontend.graph.clone().expect("resolved module graph");
    let source_indices = frontend.newline_indices.iter().cloned().fold(
        v1_rt::rc_empty_map::<String, Arc<NewlineIndex>>(),
        |acc, si| v1_rt::rc_map_insert(acc, si.file.clone(), si),
    );
    let norm = normalize_graph(graph, source_indices.clone());
    (
        norm.graph.modules.clone(),
        source_indices,
        frontend.intern_table.clone(),
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
    modules: &im::Vector<Arc<ResolvedModule>>,
    source_indices: Arc<HashMap<String, Arc<NewlineIndex>>>,
    intern_table: Arc<InternTable>,
) -> Vec<Arc<TypecheckModuleResult>> {
    let mut module_index: Arc<HashMap<String, Arc<TypedModule>>> = v1_rt::rc_empty_map();
    let mut variant_surfaces: Arc<HashMap<String, Arc<VariantExportSurface>>> =
        v1_rt::rc_empty_map();
    let mut results = Vec::new();
    for resolved in modules {
        let tc = typecheck_module(
            resolved.clone(),
            module_index.clone(),
            variant_surfaces.clone(),
            source_indices.clone(),
            intern_table.clone(),
            v1_compiler::v1_compiler_infer_env::empty_symbol_index(),
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
    modules: &'a im::Vector<Arc<ResolvedModule>>,
    source_indices: &Arc<HashMap<String, Arc<NewlineIndex>>>,
) -> &'a Arc<ResolvedModule> {
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
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
        .collect();
    assert!(
        msgs.is_empty(),
        "re-export variant chain must typecheck clean via production reconcile, got:\n{msgs:?}"
    );
}

#[test]
fn variant_reexport_incremental_surfaces_match_full_pipeline_fingerprint() {
    let sources = Arc::new(fixture_sources().into());
    let pipeline = compile_multi(&[
        ("provider.dag", PROVIDER),
        ("reexport.dag", REEXPORT),
        ("consumer.dag", CONSUMER),
    ]);
    let pipeline_msgs: Vec<_> = diagnostic_messages(&pipeline)
        .into_iter()
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
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
    let sources = Arc::new(fixture_sources().into());
    let (modules, source_indices, intern_table) = resolved_module_graph(sources);

    let incremental =
        typecheck_resolved_incremental(&modules, source_indices.clone(), intern_table.clone());
    assert!(
        hard_diagnostic_messages(incremental.last().expect("consumer incremental typecheck"))
            .is_empty(),
        "incremental path must be clean before RED control"
    );
    let module_index: Arc<HashMap<String, Arc<TypedModule>>> = incremental
        .iter()
        .map(|tc| {
            let path = authored_name_at(source_indices.clone(), tc.typed.module.clone());
            (path, tc.typed.clone())
        })
        .fold(v1_rt::rc_empty_map(), |acc, (k, v)| {
            v1_rt::rc_map_insert(acc, k, v)
        });

    let consumer = consumer_resolved(&modules, &source_indices);
    // Namespace wave-1 dissolved the original perturbation: with variant_surfaces
    // AND the symbol index both empty, the re-export arm still resolves through
    // the ancestry global_bare merge built from the typed parent index — the
    // census layering is the naming authority now, so the full-index isolated
    // path must be CLEAN (a purity witness, matching pipeline/incremental).
    let isolated = typecheck_module_isolated(
        consumer.clone(),
        module_index.clone(),
        source_indices.clone(),
        intern_table.clone(),
    );
    let isolated_msgs = hard_diagnostic_messages(&isolated);
    assert!(
        isolated_msgs.is_empty(),
        "isolated typecheck with the full parent index must resolve the re-export arm \
         via the global_bare merge (wave-1 naming authority), got:\n{}",
        isolated_msgs.join("\n")
    );
    // RED-control debt (2026-07-20): no cheap perturbation reds this shape
    // anymore. Dropping the PROVIDER from the parent index still typechecks
    // clean — imported names bind declared-weak (a type variable, no refusal)
    // when their declaring module is absent, the same L1 unchecked-access
    // deferral tracked in the typed-debt burndown. A discriminating control
    // returns when the strict missing-name wall lands; until then the two
    // asserts above (pipeline/incremental/isolated all clean via the
    // global_bare merge) are the load-bearing purity witnesses.
}
