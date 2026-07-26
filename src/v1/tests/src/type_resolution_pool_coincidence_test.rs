//! Discriminating witness for type-resolution fail-closed (b):
//! pool-present-but-not-import-reachable type refs refuse as UnresolvedType,
//! never fabricate Product(<anon>).

use crate::helpers::{compile_dag_named_with_source_roots, diagnostic_messages, workspace_root};
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_rt;
use v1_compiler::v1_std_core::{is_error_diagnostic, CompilerDiagnostic};

const FIXTURE_DIR: &str = "dag/test/fixtures/type_resolution_pool_coincidence";

fn fixture_roots() -> [std::path::PathBuf; 2] {
    let ws = workspace_root();
    [ws.join("dag"), ws.join(FIXTURE_DIR)]
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(workspace_root().join(FIXTURE_DIR).join(name))
        .unwrap_or_else(|e| panic!("read fixture {name}: {e}"))
}

fn compile_probe(
    entry_name: &str,
) -> std::rc::Rc<v1_compiler::v1_compiler_compile::PipelineResult> {
    v1_rt::type_ref_pool_coincidence_mask_set_enabled(true);
    let entry_path = format!("{FIXTURE_DIR}/{entry_name}");
    let source = read_fixture(entry_name);
    let result = compile_dag_named_with_source_roots(
        &entry_path,
        &source,
        RenderTarget::Dag,
        &fixture_roots(),
    );
    v1_rt::type_ref_pool_coincidence_mask_set_enabled(false);
    result
}

fn hard_diagnostic_messages(
    result: &std::rc::Rc<v1_compiler::v1_compiler_compile::PipelineResult>,
) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}
fn unresolved_type_messages(
    result: &std::rc::Rc<v1_compiler::v1_compiler_compile::PipelineResult>,
) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::UnresolvedType { .. }))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

fn has_unresolved_type_for(
    result: &std::rc::Rc<v1_compiler::v1_compiler_compile::PipelineResult>,
    name: &str,
) -> bool {
    result.diagnostics.iter().any(|d| {
        matches!(
            &*d.diagnostic,
            CompilerDiagnostic::UnresolvedType { name: n, .. } if n == name
        )
    })
}

#[test]
fn probe_green_stays_clean_with_own_import() {
    let result = compile_probe("probe_green.dag");
    let hard = hard_diagnostic_messages(&result);
    assert!(
        hard.is_empty(),
        "control (import-reachable) must compile clean; got {hard:?}"
    );
}

#[test]
fn probe_silent_refuses_absent_from_closure_pool() {
    let result = compile_probe("probe_silent.dag");
    let unresolved = unresolved_type_messages(&result);
    assert!(
        unresolved.iter().any(|m| m.contains("ContentHash")),
        "name absent from closure pool must refuse as unresolved type; got {:?}",
        diagnostic_messages(&result)
    );
    assert!(
        !diagnostic_messages(&result)
            .iter()
            .any(|m| m.contains("Product(<anon>)")),
        "must not fabricate anonymous product"
    );
}

#[test]
fn probe_one_import_refuses_pool_coincidence_at_signature() {
    let result = compile_probe("probe_one_import.dag");
    assert!(
        has_unresolved_type_for(&result, "ContentHash"),
        "pool coincidence must refuse at the type reference, not if-branch mismatch; got {:?}",
        hard_diagnostic_messages(&result)
    );
    assert!(
        !hard_diagnostic_messages(&result)
            .iter()
            .any(|m| m.contains("incompatible types")),
        "symptom if-branch mismatch must not mask the binding refusal; got {:?}",
        hard_diagnostic_messages(&result)
    );
}

#[test]
fn probe_crossboundary_refuses_silent_fabrication_into_real_consumer() {
    let result = compile_probe("probe_crossboundary.dag");
    assert!(
        has_unresolved_type_for(&result, "ContentHash"),
        "fabricated type fed to real consumer must refuse; got {:?}",
        hard_diagnostic_messages(&result)
    );
    let hard = hard_diagnostic_messages(&result);
    assert!(
        !hard.is_empty(),
        "must not green silently with only advisory diagnostics"
    );
}
