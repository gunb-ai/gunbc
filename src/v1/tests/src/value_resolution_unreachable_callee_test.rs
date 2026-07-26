//! Type-ref mask arm: pool-present return type without import must refuse via mask.

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};
use std::rc::Rc;
use v1_compiler::v1_compiler_compile::compile_to_resolved;
use v1_compiler::v1_compiler_infer_env::{
    type_ref_containment_bindable, type_ref_import_chain_reachable, type_ref_mask_allows_binding,
};
use v1_compiler::v1_rt;
use v1_compiler::v1_std_core::{diagnostic_to_message, is_error_diagnostic, CompilerDiagnostic};

const FIXTURE_DIR: &str = "dag/test/fixtures/value_resolution_unreachable_callee";
const UNIMPORTED_RETURN_TYPE: &str = "ContentHash";

fn fixture_roots() -> [std::path::PathBuf; 2] {
    let ws = workspace_root();
    [ws.join("dag"), ws.join(FIXTURE_DIR)]
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(workspace_root().join(FIXTURE_DIR).join(name))
        .unwrap_or_else(|e| panic!("read fixture {name}: {e}"))
}

fn compile_probe_resolved(
    entry_name: &str,
) -> Rc<v1_compiler::v1_compiler_compile::ResolvedPipelineResult> {
    v1_rt::type_ref_pool_coincidence_mask_set_enabled(true);
    let entry_path = format!("{FIXTURE_DIR}/{entry_name}");
    let source = read_fixture(entry_name);
    let sources =
        resolve_imports_transitively_with_source_roots(&entry_path, &source, &fixture_roots());
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    v1_rt::type_ref_pool_coincidence_mask_set_enabled(false);
    resolved
}

#[test]
fn probe_unreachable_callee_mask_blocks_pool_coincidence_type_binding() {
    let resolved = compile_probe_resolved("probe_value_arm.dag");
    let graph = resolved
        .graph
        .as_ref()
        .expect("resolved graph for value-arm probe fixture");
    let probe = graph
        .modules
        .iter()
        .find(|m| m.type_env.module_path.contains("probe_value_arm"))
        .expect("probe module in graph");
    let import_chain =
        type_ref_import_chain_reachable(probe.type_env.clone(), UNIMPORTED_RETURN_TYPE.to_string());
    let containment =
        type_ref_containment_bindable(probe.type_env.clone(), UNIMPORTED_RETURN_TYPE.to_string());
    v1_rt::type_ref_pool_coincidence_mask_set_enabled(true);
    let mask =
        type_ref_mask_allows_binding(probe.type_env.clone(), UNIMPORTED_RETURN_TYPE.to_string());
    v1_rt::type_ref_pool_coincidence_mask_set_enabled(false);
    assert!(
        !import_chain,
        "return type ContentHash must not be import-chain reachable without explicit import"
    );
    assert!(
        !mask,
        "pool coincidence must refuse ContentHash binding (containment={containment})"
    );
}

#[test]
fn probe_unreachable_return_type_refuses_without_import() {
    let resolved = compile_probe_resolved("probe_value_arm.dag");
    let graph = resolved
        .graph
        .as_ref()
        .expect("resolved graph for value-arm probe");
    let hard: Vec<String> = graph
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect();
    assert!(
        hard.iter()
            .any(|m| m.contains("unresolved type") && m.contains(UNIMPORTED_RETURN_TYPE)),
        "pool-present return type must refuse as UnresolvedType; got {hard:?}"
    );
    assert!(
        !hard.iter().any(|m| m.contains("Product(<anon>)")),
        "must not fabricate anonymous product"
    );
    assert!(
        graph.diagnostics.iter().any(|d| {
            matches!(
                &*d.diagnostic,
                CompilerDiagnostic::UnresolvedType { name, .. } if name == UNIMPORTED_RETURN_TYPE
            )
        }),
        "expected located UnresolvedType for {UNIMPORTED_RETURN_TYPE}"
    );
}
