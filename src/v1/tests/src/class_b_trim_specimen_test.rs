//! Class B trim: explicit `import std.algebra { trim }` binds in narrow pools;
//! bare trim refuses even when std.algebra is already in the compilation pool.
//!
//! SCAFFOLD (DESIGN §7 HAND-RUST GATE — explicit deferral): this module exercises
//! the v1 compiler-test harness because Class B trim discrimination needs
//! primary-precedence pool overlays (`fixtures/class_b_trim/narrow_pool/`) and
//! `compile_declared_import_closure_only_with_pool` receipts that cannot be enrolled
//! as discovered `dag/test/claim/*_test.dag` rows while the Class B gate observes only
//! `item_registry` symbols (`run_class_b_import_closure_gate` / `rust_selection_policy_node`)
//! and `trim` has no such registry row. Lane: import-strip witness-discovery cascade
//! (#6985 Class B pool-membership coincidence; `import-strip-witness-discovery-cascade-diagnosis.md`
//! §9, §12). Sole dissolution: migrate `fixtures/class_b_trim/*.dag` into enrolled
//! `dag/test/claim/class_b_trim_*` witness rows on the floor roster and delete this
//! Rust module when trim is enrolled on the Class B observation surface.

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

fn trim_compile_refused(
    compiled: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult,
) -> bool {
    trim_not_found_diagnostic_count(compiled) > 0 || hard_diagnostic_count(compiled) > 0
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
fn trim_free_call_refuses_in_narrow_pool_without_algebra_in_closure() {
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
        trim_compile_refused(compiled.as_ref()),
        "bare trim must refuse when std.algebra is not in pool: {:?}",
        compiled
            .diagnostics
            .iter()
            .map(|d| format!("{:?}", d.diagnostic))
            .collect::<Vec<_>>()
    );
}

#[test]
fn trim_free_call_refuses_when_algebra_in_pool_without_listed_import() {
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
    assert!(
        !consumer_lists_algebra_directly(compiled.as_ref(), COINCIDENCE_CONSUMER),
        "coincidence specimen must not directly import std.algebra"
    );
    assert!(
        trim_compile_refused(compiled.as_ref()),
        "bare trim must refuse even when std.algebra is in pool without listed import: {:?}",
        compiled
            .diagnostics
            .iter()
            .map(|d| format!("{:?}", d.diagnostic))
            .collect::<Vec<_>>()
    );
    let trim_binding = declared_import_closure_binding_observation_from_resolved(
        compiled.as_ref(),
        COINCIDENCE_CONSUMER,
        "trim",
    );
    match trim_binding {
        DeclaredImportClosureBindingObservation::NotRunnable(_) => {}
        DeclaredImportClosureBindingObservation::Observed(
            DeclaredImportClosureBindingObserved {
                symbol_resolves: false,
                ..
            },
        ) => {}
        other => panic!(
            "trim must not resolve via pool coincidence when std.algebra is in closure without listed import: {other:?}"
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
        trim_compile_refused(compiled.as_ref()),
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
    let trim_binding = declared_import_closure_binding_observation_from_resolved(
        compiled.as_ref(),
        BOUND_CONSUMER,
        "trim",
    );
    match trim_binding {
        DeclaredImportClosureBindingObservation::Observed(
            DeclaredImportClosureBindingObserved {
                symbol_resolves: true,
                binding_source: Some(UnlistedImportBindingSource::ListedImport),
                definer_module: Some(definer),
                ..
            },
        ) => {
            assert_eq!(definer, "std.algebra");
        }
        other => {
            panic!("trim with explicit import must bind via ListedImport in narrow pool: {other:?}")
        }
    }
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
        trim_compile_refused(compiled.as_ref()),
        "method trim on FreeMonoid<String> receiver must refuse: {:?}",
        compiled
            .diagnostics
            .iter()
            .map(|d| format!("{:?}", d.diagnostic))
            .collect::<Vec<_>>()
    );
}

const WIRE_ENTRY: &str = "fixtures/class_b_trim/wire_projection_specimen.dag";

/// Pool for the wire-projection specimen. It needs `dag/extdeps` in addition to the roots the
/// hand-authored specimens use, because its subject is a real service-op output declaration
/// (`Filesystem.Read` `content: String from "content"`) rather than a hand-declared carrier.
fn wire_projection_pool_roots() -> Vec<String> {
    vec![
        "dag/std".to_string(),
        "dag/extdeps".to_string(),
        "fixtures/class_b_trim".to_string(),
    ]
}

/// REGRESSION CONTROL for the service-op wire-projection class.
///
/// THE CLASS AS REGISTERED (`extdeps.shell.exec` `service_op_string_wire_projection_method_fork_note`,
/// `gunbc.plans.model_realization_fork`): a service-op output field declared `T from "wire_key"`
/// infers as a FaithfulFreeMonoid/Coproduct carrier when the closure lacks v1-seed sources, so the
/// METHOD form on it refuses (`method trim not found on Coproduct(FreeMonoid)`) although the extdeps
/// contract names String. Both carriers name `monoid_specimen.dag` +
/// `trim_method_form_fails_on_freemonoid_receiver` as the pinning witness.
///
/// THAT WITNESS CANNOT SEE THE CLASS. Its subject is a HAND-DECLARED `FreeMonoid<String>` parameter;
/// the class's subject is a String field MINTED BY A WIRE PROJECTION. Measured: zero `T from "key"`
/// declarations existed anywhere in this fixture directory, so the class had no discriminating
/// evidence and read identically whether it was live or dead.
///
/// MEASURED HERE, 2026-08-17, on a tree that STILL CONTAINS `v1.compiler.infer` `rust_corpus_repr`
/// (i.e. before the Root B cut that deletes it): the method form on a real wire field COMPILES —
/// `hard_diagnostics = 0`, `trim_not_found = 0`, `refused = false`. So removing `rust_corpus_repr` is
/// NOT the condition under which this class disappears; it does not reproduce with that mechanism
/// present. The class is therefore narrower than "every String method on any service-op result
/// field", or already dissolved and its note stale.
///
/// SCOPE OF THE CLAIM, deliberately narrow: one service op (`Filesystem.Read` `content`), one method
/// (`trim`), one pool. It says nothing about `shell.Exec.Run` `stdout`, whose input is a
/// `sole_constructor`-sealed `TransportScript` and so cannot be minted in a fixture.
///
/// This control locks the compiling behaviour in. If the wire projection later starts minting a
/// distinct carrier again, this goes red.
#[test]
fn wire_projection_method_form_compiles_on_service_op_string_field() {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let pool = wire_projection_pool_roots();
    let compiled = compile_declared_import_closure_only_with_pool(&pool, WIRE_ENTRY, None)
        .expect("wire-projection specimen must load its declared-import closure");

    assert_eq!(
        trim_not_found_diagnostic_count(compiled.as_ref()),
        0,
        "method form on a service-op String wire field must not report trim-not-found: {:?}",
        compiled
            .diagnostics
            .iter()
            .map(|d| format!("{:?}", d.diagnostic))
            .collect::<Vec<_>>()
    );
    assert!(
        !trim_compile_refused(compiled.as_ref()),
        "wire-field method form must compile: {:?}",
        compiled
            .diagnostics
            .iter()
            .map(|d| format!("{:?}", d.diagnostic))
            .collect::<Vec<_>>()
    );
}
