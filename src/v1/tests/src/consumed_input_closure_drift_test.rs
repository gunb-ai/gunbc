// Deliverable 1 of the intent-linearity ImportGraph representation: the
// consumed-input-closure DRIFT WALL.
//
// `dsl/tools/rust_stage0_gates.dag` declares `declared_consumed_input_closures`
// -- a `ConsumedInputClosure { unit, consumed_dag_paths: List<String> }` whose
// path list is HAND-TYPED (rust_stage0_gates.dag:22 `slice1_status` admits the
// fail-open: "declaration drift silently re-opens the .dag->rust fail-open").
//
// That declared list is pure parallel representation of a value the compiler
// already derives: the transitive import-graph closure of the unit's entry
// .dag. This oracle is the intent-linearity wall for that representation --
// redundancy = (declared description) - (minimal/derived description), and the
// wall is `declared == derived`. Any drift (a dropped path, or a real .dag read
// added without updating the declaration) goes RED, closing the fail-open.

use std::collections::BTreeSet;
use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};
use v1_compiler::v1_std_core::diagnostic_to_message;

use crate::helpers::{
    resolve_imports_transitively_with_source_roots, v2_layer_roots, workspace_root,
};

const GATES_ENTRY: &str = "dsl/tools/rust_stage0_gates.dag";
const DECLARED_DATA: &str = "declared_consumed_input_closures";

struct DeclaredClosure {
    unit: String,
    declared_paths: Vec<String>,
}

fn blocking_diagnostics(resolved: &ResolvedPipelineResult) -> Vec<String> {
    resolved
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect()
}

fn v2_source_root_strings() -> Vec<String> {
    v2_layer_roots()
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

/// Read `declared_consumed_input_closures` straight out of the .dag (the
/// declared side of the parallel representation).
fn read_declared_closures() -> Vec<DeclaredClosure> {
    let roots = v2_source_root_strings();
    let entry = workspace_root().join(GATES_ENTRY);
    let entry = entry.to_string_lossy().to_string();
    let sources = cli_run::load_sources_for_entry(&roots, &entry)
        .unwrap_or_else(|e| panic!("failed to load {GATES_ENTRY}: {e}"));
    let resolved = compile_to_resolved(Rc::new(sources));
    let msgs = blocking_diagnostics(&resolved);
    assert!(msgs.is_empty(), "{GATES_ENTRY} should resolve: {msgs:?}");
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    let value = v1_interpreter::eval_data_item_value(&ctx, DECLARED_DATA)
        .unwrap_or_else(|e| panic!("eval {DECLARED_DATA}: {e}"))
        .unwrap_or_else(|| panic!("{DECLARED_DATA} is not a declared data item"));
    let unit_sym = ctx.sym("unit");
    let paths_sym = ctx.sym("consumed_dag_paths");
    let items = match value {
        Value::List(xs) => xs,
        other => panic!("expected List for {DECLARED_DATA}, got {other:?}"),
    };
    items
        .iter()
        .map(|item| {
            let Value::Record { fields, .. } = item else {
                panic!("expected ConsumedInputClosure record, got {item:?}");
            };
            let unit = match v1_compiler::v1_interpreter::fields_get(fields, unit_sym) {
                Some(Value::Str(s)) => s.clone(),
                other => panic!("expected Str `unit`, got {other:?}"),
            };
            let declared_paths = match v1_compiler::v1_interpreter::fields_get(fields, paths_sym) {
                Some(Value::List(xs)) => xs
                    .iter()
                    .map(|v| match v {
                        Value::Str(s) => s.clone(),
                        other => panic!("expected Str path, got {other:?}"),
                    })
                    .collect(),
                other => panic!("expected List `consumed_dag_paths`, got {other:?}"),
            };
            DeclaredClosure {
                unit,
                declared_paths,
            }
        })
        .collect()
}

/// Convention: the unit's entry .dag is the declared path whose file stem equals
/// the unit name. (`coproduct_reflection_conformance_test` <->
/// `.../coproduct_reflection_conformance_test.dag`.) Fail-closed: a unit with no
/// matching entry is a declaration error, not a silent skip.
fn entry_path_for(closure: &DeclaredClosure) -> String {
    closure
        .declared_paths
        .iter()
        .find(|p| {
            std::path::Path::new(p)
                .file_stem()
                .map(|s| s.to_string_lossy() == closure.unit)
                .unwrap_or(false)
        })
        .unwrap_or_else(|| {
            panic!(
                "no declared path matches unit `{}` (entry must be `<unit>.dag`): {:?}",
                closure.unit, closure.declared_paths
            )
        })
        .clone()
}

/// The minimal/derived side: the transitive import-graph closure the compiler
/// already walks, as the same workspace-relative .dag path set.
///
/// Derived over `v2_layer_roots()` = [src/v2, dsl] -- deliberately the SAME root
/// set the unit is actually compiled under (the coproduct conformance test's
/// `cert_sources` resolves via `v2_layer_roots()` too), so derived and declared
/// describe the same module universe. NOTE (the Axis-2 dependency cool-cat-421
/// flagged): the *choice* of roots is itself the LayerDAG parallel-representation
/// -- ImportGraph's derived side is parameterized by where modules are declared,
/// so Axis-1 sits on top of Axis-2, not beside it. The module index is keyed by
/// module name (last-scan-wins); a name collision across roots would silently
/// shift the derived set, which is exactly the LayerDAG roster Axis-2 will
/// consolidate. Until then this root set is the single authority for both sides.
fn derive_closure_paths(entry: &str) -> BTreeSet<String> {
    let content = std::fs::read_to_string(workspace_root().join(entry))
        .unwrap_or_else(|e| panic!("read entry {entry}: {e}"));
    resolve_imports_transitively_with_source_roots(entry, &content, &v2_layer_roots())
        .iter()
        .map(|s| s.path.clone())
        .collect()
}

fn declared_set(closure: &DeclaredClosure) -> BTreeSet<String> {
    closure.declared_paths.iter().cloned().collect()
}

/// THE WALL (run over the live corpus): every declared consumed-input closure
/// must equal its import-graph-derived closure. Drift in either direction --
/// over-declaration (redundant, the §2 angle) or under-declaration (a real .dag
/// read not declared, the §5 fail-open) -- fails this test.
#[test]
fn consumed_input_closure_declared_equals_import_graph_derived() {
    let closures = read_declared_closures();
    assert!(
        !closures.is_empty(),
        "expected at least one declared ConsumedInputClosure (vacuous wall)"
    );
    for closure in &closures {
        let entry = entry_path_for(closure);
        let derived = derive_closure_paths(&entry);
        let declared = declared_set(closure);
        let missing: Vec<&String> = derived.difference(&declared).collect();
        let extra: Vec<&String> = declared.difference(&derived).collect();
        assert!(
            missing.is_empty() && extra.is_empty(),
            "consumed-input-closure DRIFT for unit `{}`:\n  under-declared (real reads not declared, FAIL-OPEN): {:?}\n  over-declared (redundant): {:?}",
            closure.unit,
            missing,
            extra
        );
    }
}

/// DISCRIMINATING INPUT (the §5 prove-by-execution control): the wall is not
/// vacuously green. Dropping any declared path makes declared != derived, and
/// adding a bogus path does too. If these did not go red, the wall above would
/// be meaningless.
#[test]
fn drift_oracle_goes_red_on_perturbed_declaration() {
    let closures = read_declared_closures();
    assert!(!closures.is_empty(), "need a closure to perturb");
    let closure = &closures[0];
    let entry = entry_path_for(closure);
    let derived = derive_closure_paths(&entry);
    let declared = declared_set(closure);
    assert_eq!(declared, derived, "baseline must match before perturbation");

    // Drop a path -> drift (the under-declared fail-open direction).
    let dropped: BTreeSet<String> = declared.iter().filter(|p| **p != entry).cloned().collect();
    assert_ne!(
        dropped, derived,
        "dropping a declared path must break declared == derived"
    );

    // Add a bogus path -> drift (the over-declared redundancy direction).
    let mut added = declared.clone();
    added.insert("src/v2/std/__bogus_never_imported__.dag".to_string());
    assert_ne!(
        added, derived,
        "adding a bogus declared path must break declared == derived"
    );
}
