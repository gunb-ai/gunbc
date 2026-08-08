//! Class B trim: explicit `import std.algebra { trim }` binds in narrow pools;
//! coincidence binding remains a regression control only.
//!
//! SCAFFOLD (DESIGN §7 HAND-RUST GATE — explicit deferral): this module exercises
//! the v1 compiler-test harness because Class B trim discrimination needs
//! primary-precedence pool overlays (`fixtures/class_b_trim/narrow_pool/`) and
//! `compile_declared_import_closure_only_with_pool` receipts that cannot be enrolled
//! as discovered `dag/test/claim/*_test.dag` rows while the Class B gate observes only
//! `item_registry` symbols (`run_class_b_import_closure_gate` / `rust_selection_policy_node`)
//! and `trim` has no such registry row. Lane: import-strip witness-discovery cascade
//! (#6985 Class B pool-membership coincidence; `import-strip-witness-discovery-cascade-diagnosis.md`
//! §9, §12). Sole dissolution: the change landing closure-independent binding for bare
//! free-call `trim` (dissolve trigger on `trim_free_function_authority_note` in
//! `std.algebra` — bare trim without listed import refuses repo-wide) must migrate
//! `fixtures/class_b_trim/*.dag` into enrolled `dag/test/claim/class_b_trim_*` witness
//! rows on the floor roster and delete this Rust module in the same change.

use v1_compiler::cli_run::{
    compile_declared_import_closure_only_with_pool,
    declared_import_closure_binding_observation_from_resolved,
    DeclaredImportClosureBindingObservation, DeclaredImportClosureBindingObserved,
    UnlistedImportBindingSource,
};
use v1_compiler::v1_std_core::{module_imports, CompilerDiagnostic};

use crate::helpers::workspace_root;

const SPECIMEN_ENTRY: &str = "fixtures/class_b_trim/specimen.dag";
const COINCIDENCE_ENTRY: &str = "fixtures/class_b_trim/coincidence_specimen.dag";
const MONOID_ENTRY: &str = "fixtures/class_b_trim/monoid_specimen.dag";
const BOUND_CONSUMER: &str = "test.claim.class_b_trim_specimen";
const COINCIDENCE_CONSUMER: &str = "test.claim.class_b_trim_coincidence_specimen";

fn trim_narrow_pool_roots() -> Vec<String> {
    vec![
        "fixtures/class_b_trim/narrow_pool".to_string(),
        "fixtures/class_b_trim".to_string(),
        "dag/std".to_string(),
    ]
}

fn trim_narrow_pool_without_algebra_roots() -> Vec<String> {
    vec![
        "fixtures/class_b_trim/narrow_pool".to_string(),
        "fixtures/class_b_trim".to_string(),
    ]
}

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
                        || message.contains("method 'trim' not found")
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
    consumer: &str,
) -> bool {
    let graph = compiled.graph.as_ref().expect("graph");
    let tm = graph
        .modules
        .iter()
        .find(|m| m.type_env.module_path == consumer)
        .expect("consumer module");
    module_imports(tm.module.clone())
        .iter()
        .any(|imp| imp.name == "std.algebra")
}

#[test]
fn trim_free_call_fails_in_narrow_pool_without_algebra_coincidence() {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let compiled = compile_declared_import_closure_only_with_pool(
        &trim_narrow_pool_without_algebra_roots(),
        COINCIDENCE_ENTRY,
        None,
    )
    .expect("compile");
    assert!(
        !algebra_module_in_graph(compiled.as_ref()),
        "std.algebra must be absent from narrow pool without dag/std"
    );
    assert!(
        !consumer_lists_algebra_directly(compiled.as_ref(), COINCIDENCE_CONSUMER),
        "coincidence specimen must not directly import std.algebra"
    );
    let trim_binding = declared_import_closure_binding_observation_from_resolved(
        compiled.as_ref(),
        COINCIDENCE_CONSUMER,
        "trim",
    );
    match trim_binding {
        DeclaredImportClosureBindingObservation::Observed(
            DeclaredImportClosureBindingObserved {
                symbol_resolves: false,
                ..
            },
        ) => {}
        DeclaredImportClosureBindingObservation::Observed(observed) => {
            panic!(
                "trim must not resolve via pool coincidence when std.algebra is absent from narrow pool: {observed:?}"
            );
        }
        DeclaredImportClosureBindingObservation::NotRunnable(reason) => {
            panic!("trim binding observation must be runnable: {reason}");
        }
    }
}

#[test]
fn trim_coincidence_free_call_binds_via_pool_when_algebra_in_closure() {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let compiled = compile_declared_import_closure_only_with_pool(
        &trim_declared_import_pool_roots(),
        COINCIDENCE_ENTRY,
        None,
    )
    .expect("compile");
    assert!(
        algebra_module_in_graph(compiled.as_ref()),
        "std.algebra must be in declared-import closure via std.types transitive import"
    );
    let trim_binding = declared_import_closure_binding_observation_from_resolved(
        compiled.as_ref(),
        COINCIDENCE_CONSUMER,
        "trim",
    );
    match trim_binding {
        DeclaredImportClosureBindingObservation::Observed(
            DeclaredImportClosureBindingObserved {
                symbol_resolves: true,
                binding_source: Some(UnlistedImportBindingSource::PoolCoincidence),
                definer_module: Some(definer),
                ..
            },
        ) => {
            assert_eq!(definer, "std.algebra");
        }
        other => panic!(
            "trim must resolve via pool coincidence when std.algebra is in closure without direct import: {other:?}"
        ),
    }
}

#[test]
fn trim_free_call_refuses_in_narrow_pool_without_algebra_authority() {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let compiled = compile_declared_import_closure_only_with_pool(
        &trim_narrow_pool_without_algebra_roots(),
        SPECIMEN_ENTRY,
        None,
    )
    .expect("compile");
    assert!(
        !algebra_module_in_graph(compiled.as_ref()),
        "std.algebra must be absent from narrow pool without dag/std"
    );
    assert!(
        trim_not_found_diagnostic_count(compiled.as_ref()) > 0
            || hard_diagnostic_count(compiled.as_ref()) > 0,
        "explicit import std.algebra {{ trim }} must refuse when std.algebra is not in pool: {:?}",
        compiled
            .diagnostics
            .iter()
            .map(|d| format!("{:?}", d.diagnostic))
            .collect::<Vec<_>>()
    );
}

#[test]
fn trim_free_call_compiles_via_explicit_import_in_narrow_pool() {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let compiled = compile_declared_import_closure_only_with_pool(
        &trim_narrow_pool_roots(),
        SPECIMEN_ENTRY,
        None,
    )
    .expect("compile");
    assert!(compiled.graph.is_some(), "graph required");
    assert_eq!(hard_diagnostic_count(compiled.as_ref()), 0);
    assert_eq!(trim_not_found_diagnostic_count(compiled.as_ref()), 0);
    assert!(
        consumer_lists_algebra_directly(compiled.as_ref(), BOUND_CONSUMER),
        "bound specimen must list std.algebra via explicit trim import"
    );
}

#[test]
fn trim_coincidence_binding_still_works_without_explicit_import() {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let pool = trim_declared_import_pool_roots();
    let compiled = compile_declared_import_closure_only_with_pool(&pool, COINCIDENCE_ENTRY, None)
        .expect("compile");
    assert!(compiled.graph.is_some(), "graph required");
    assert_eq!(hard_diagnostic_count(compiled.as_ref()), 0);
    assert_eq!(trim_not_found_diagnostic_count(compiled.as_ref()), 0);
    assert!(
        algebra_module_in_graph(compiled.as_ref()),
        "std.algebra must be in pool for coincidence binding"
    );
    assert!(
        !consumer_lists_algebra_directly(compiled.as_ref(), COINCIDENCE_CONSUMER),
        "coincidence specimen must not directly import std.algebra"
    );
}

#[test]
fn trim_explicit_import_not_required_when_pool_coincidence_present() {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let compiled = compile_declared_import_closure_only_with_pool(
        &trim_declared_import_pool_roots(),
        SPECIMEN_ENTRY,
        None,
    )
    .expect("compile");
    assert_eq!(hard_diagnostic_count(compiled.as_ref()), 0);
    assert_eq!(trim_not_found_diagnostic_count(compiled.as_ref()), 0);
    assert!(
        consumer_lists_algebra_directly(compiled.as_ref(), BOUND_CONSUMER),
        "bound specimen lists trim import even when coincidence pool is present"
    );
}

#[test]
fn trim_ambient_perturbation_preserves_explicit_import_compile() {
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
        trim_not_found_diagnostic_count(compiled.as_ref()) > 0
            || hard_diagnostic_count(compiled.as_ref()) > 0,
        "method trim on FreeMonoid<String> receiver must refuse: {:?}",
        compiled
            .diagnostics
            .iter()
            .map(|d| format!("{:?}", d.diagnostic))
            .collect::<Vec<_>>()
    );
}
