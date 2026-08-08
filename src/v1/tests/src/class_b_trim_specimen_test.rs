//! Class B live specimen: `trim` binds only by pool-membership coincidence.
//!
//! Execution receipts for #6985 — trim is absent from the substrate free-call
//! builtin registry (`compile-clean-forcecheck.md` §6); free-call `trim(s)` resolves
//! only when `std.algebra` is already in the compilation pool (scalar template
//! `trim` on the FreeMonoid carrier). Direct imports list only `std.types`; the
//! consumer never lists `std.algebra`. Method `trim` on a `FreeMonoid<String>`
//! receiver refuses — trim is on the scalar template, not the collection monoid.

use v1_compiler::cli_run::compile_declared_import_closure_only_with_pool;
use v1_compiler::v1_std_core::{module_imports, CompilerDiagnostic};

use crate::helpers::workspace_root;

const SPECIMEN_ENTRY: &str = "fixtures/class_b_trim/specimen.dag";
const MONOID_ENTRY: &str = "fixtures/class_b_trim/monoid_specimen.dag";
const CONSUMER: &str = "test.claim.class_b_trim_specimen";

/// Declared-import footprint pool: `std.types` plus transitive `std.algebra` (via
/// types.dag's import line) and the fixture entry tree — no unrelated ambient modules.
fn trim_declared_import_pool_roots() -> Vec<String> {
    vec!["dag/std".to_string(), "fixtures/class_b_trim".to_string()]
}

fn trim_pool_with_perturbation() -> Vec<String> {
    let mut roots = trim_declared_import_pool_roots();
    roots.insert(0, "fixtures/class_b_trim/perturbation_overlay".to_string());
    roots
}

fn hard_diagnostic_count(
    compiled: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult,
) -> usize {
    compiled
        .diagnostics
        .iter()
        .filter(|d| v1_compiler::cli_run::compile_clean_diagnostic_is_hard(d))
        .count()
}

fn trim_not_found_diagnostic_count(
    compiled: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult,
) -> usize {
    compiled
        .diagnostics
        .iter()
        .filter(|d| {
            matches!(
                d.diagnostic.as_ref(),
                CompilerDiagnostic::InternalError { message, .. }
                    if message.contains("function 'trim' not found in scope")
            )
        })
        .count()
}

fn algebra_module_in_graph(
    compiled: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult,
) -> bool {
    compiled.graph.as_ref().is_some_and(|g| {
        g.modules
            .iter()
            .any(|m| m.type_env.module_path == "std.algebra")
    })
}

fn consumer_lists_algebra_directly(
    compiled: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult,
) -> bool {
    let graph = compiled.graph.as_ref().expect("graph");
    let tm = graph
        .modules
        .iter()
        .find(|m| m.type_env.module_path == CONSUMER)
        .expect("consumer module");
    module_imports(tm.module.clone())
        .iter()
        .any(|imp| imp.name == "std.algebra")
}

#[test]
fn trim_free_call_fails_in_narrow_pool_without_algebra_coincidence() {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let narrow = vec![
        "fixtures/class_b_trim/narrow_pool".to_string(),
        "fixtures/class_b_trim".to_string(),
    ];
    let compiled = compile_declared_import_closure_only_with_pool(&narrow, SPECIMEN_ENTRY, None)
        .expect("compile");
    assert!(
        trim_not_found_diagnostic_count(compiled.as_ref()) > 0
            || hard_diagnostic_count(compiled.as_ref()) > 0,
        "narrow pool (types without algebra authority) must not silently bind trim: {:?}",
        compiled
            .diagnostics
            .iter()
            .map(|d| format!("{:?}", d.diagnostic))
            .collect::<Vec<_>>()
    );
}

#[test]
fn trim_free_call_compiles_when_algebra_pool_coincidence_present() {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let pool = trim_declared_import_pool_roots();
    let compiled = compile_declared_import_closure_only_with_pool(&pool, SPECIMEN_ENTRY, None)
        .expect("compile");
    assert!(compiled.graph.is_some(), "graph required");
    assert_eq!(hard_diagnostic_count(compiled.as_ref()), 0);
    assert_eq!(trim_not_found_diagnostic_count(compiled.as_ref()), 0);
    assert!(
        algebra_module_in_graph(compiled.as_ref()),
        "std.algebra must be in the compiled pool for trim to bind"
    );
}

#[test]
fn trim_binds_by_pool_membership_not_direct_import() {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let pool = trim_declared_import_pool_roots();
    let compiled = compile_declared_import_closure_only_with_pool(&pool, SPECIMEN_ENTRY, None)
        .expect("compile");
    assert!(
        algebra_module_in_graph(compiled.as_ref()),
        "std.algebra must be present in pool"
    );
    assert!(
        !consumer_lists_algebra_directly(compiled.as_ref()),
        "trim specimen must not directly import std.algebra — binding is coincidence, not listed import"
    );
}

#[test]
fn trim_ambient_perturbation_preserves_compile() {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let baseline = compile_declared_import_closure_only_with_pool(
        &trim_declared_import_pool_roots(),
        SPECIMEN_ENTRY,
        None,
    )
    .expect("baseline compile");
    let perturbed = compile_declared_import_closure_only_with_pool(
        &trim_pool_with_perturbation(),
        SPECIMEN_ENTRY,
        None,
    )
    .expect("perturbed compile");
    assert_eq!(hard_diagnostic_count(baseline.as_ref()), 0);
    assert_eq!(hard_diagnostic_count(perturbed.as_ref()), 0);
    assert_eq!(trim_not_found_diagnostic_count(baseline.as_ref()), 0);
    assert_eq!(trim_not_found_diagnostic_count(perturbed.as_ref()), 0);
}

#[test]
fn trim_method_form_fails_on_freemonoid_receiver() {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let pool = trim_declared_import_pool_roots();
    let compiled =
        compile_declared_import_closure_only_with_pool(&pool, MONOID_ENTRY, None).expect("compile");
    assert!(
        trim_not_found_diagnostic_count(compiled.as_ref()) > 0,
        "trim as free-call on FreeMonoid<String> receiver must refuse: {:?}",
        compiled
            .diagnostics
            .iter()
            .map(|d| format!("{:?}", d.diagnostic))
            .collect::<Vec<_>>()
    );
}
