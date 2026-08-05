//! Class B closure control for `rust_test_fixtures` (#7811 follow-on).
//!
//! Proves `src/v2/extdeps/languages/rust_test_fixtures.dag` binds
//! `v2.extdeps.languages.rust` through its **declared import-edge closure** —
//! not pool membership coincidence. Each arm compiles on a fresh isolated
//! index (no process resolve-store warm hit). Full-file typecheck on import-only
//! closure is out of scope; the witness targets the Class B binding axis only.

use std::collections::BTreeMap;
use std::fs;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use v1_compiler::cli_run::{
    compile_declared_import_closure_only_with_pool, compile_entry_on_declared_import_closure_only,
    cross_module_binding_receipts_for_symbols, CrossModuleBindingReceipt,
    UnlistedImportBindingSource,
};
use v1_compiler::v1_compiler_compile::ResolvedPipelineResult;
use v1_compiler::v1_compiler_infer_items::ResolvedGraph;

use crate::helpers::{read_v2_file, workspace_root};

const ENTRY_REL: &str = "src/v2/extdeps/languages/rust_test_fixtures.dag";
const RUST_DAG_SUFFIX: &str = "src/v2/extdeps/languages/rust.dag";
const RUST_MODULE: &str = "v2.extdeps.languages.rust";
const CONSUMER_MODULE: &str = "v2.extdeps.languages.rust_test";

/// The four cross-module rust symbols that failed under pool-coincidence drift.
const CLASS_B_RUST_BINDING_SYMBOLS: &[&str] = &[
    "rust_selection_policy_node",
    "rust_operator_realizations_catalog_node",
    "rust_grammar_terminal",
    "rust_inhabitant_atom",
];

/// Unrelated pool member that imports rust.dag for ambient coincidence pressure.
const AMBIENT_RUST_LOADER: &str = "module v2.extdeps.languages.rust_pool_perturb_ambient\n\
import v2.extdeps.languages.rust { rust_selection_policy_node }\n\
data ambient_anchor: Int = 1\n";

/// Homonym decoy: same bare names, wrong defining module — must not win binding.
const HOMONYM_DECOY_MODULE: &str = "module v2.extdeps.languages.rust_test_decoy\n\
fn rust_selection_policy_node() -> Int { 0 }\n\
fn rust_operator_realizations_catalog_node() -> Int { 0 }\n\
fn rust_grammar_terminal() -> Int { 0 }\n\
fn rust_inhabitant_atom() -> Int { 0 }\n";

fn witness_layer_roots() -> Vec<String> {
    let ws = workspace_root();
    vec![
        ws.join("dag").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ]
}

fn perturbation_pool_roots(tmp: &std::path::Path) -> Vec<String> {
    let ws = workspace_root();
    vec![
        tmp.to_string_lossy().into_owned(),
        ws.join("dag").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ]
}

fn write_pool_perturbation_fixtures(tmp: &std::path::Path) {
    let ambient = tmp.join("src/v2/extdeps/languages/rust_pool_perturb_ambient.dag");
    let decoy = tmp.join("src/v2/extdeps/languages/rust_test_decoy.dag");
    fs::create_dir_all(ambient.parent().unwrap()).expect("tmpdir parents");
    fs::write(&ambient, AMBIENT_RUST_LOADER).expect("write ambient loader");
    fs::write(&decoy, HOMONYM_DECOY_MODULE).expect("write homonym decoy");
}

fn hard_messages(resolved: &Rc<ResolvedPipelineResult>) -> Vec<String> {
    resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| {
            !m.starts_with("complexity: ")
                && !m.starts_with("unlisted import use ")
                && !m.starts_with("where-refinement unenforced:")
        })
        .collect()
}

fn strip_import_block(content: &str) -> String {
    let mut out = String::new();
    let mut skipping = false;
    for line in content.lines() {
        if line.starts_with("import v2.extdeps.languages.rust") {
            skipping = true;
            continue;
        }
        if skipping {
            if line.trim() == "}" {
                skipping = false;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn closure_has_rust_dag(paths: &[String]) -> bool {
    paths
        .iter()
        .any(|p| p.replace('\\', "/").ends_with(RUST_DAG_SUFFIX))
}

fn class_b_rust_binding_errors(messages: &[String]) -> Vec<String> {
    messages
        .iter()
        .filter(|m| {
            CLASS_B_RUST_BINDING_SYMBOLS
                .iter()
                .any(|sym| m.contains(sym))
        })
        .cloned()
        .collect()
}

fn resolved_graph_or_panic(resolved: &Rc<ResolvedPipelineResult>) -> &ResolvedGraph {
    resolved
        .graph
        .as_ref()
        .expect("Class B witness requires a resolved graph (not parse-only refusal)")
        .as_ref()
}

fn class_b_binding_receipts(
    resolved: &Rc<ResolvedPipelineResult>,
) -> BTreeMap<String, CrossModuleBindingReceipt> {
    cross_module_binding_receipts_for_symbols(
        resolved_graph_or_panic(resolved),
        CONSUMER_MODULE,
        CLASS_B_RUST_BINDING_SYMBOLS,
    )
}

#[derive(Debug, PartialEq, Eq)]
struct ClassBBindingProfile {
    receipts: BTreeMap<String, CrossModuleBindingReceipt>,
    rust_errors: Vec<String>,
}

fn class_b_binding_profile(resolved: &Rc<ResolvedPipelineResult>) -> ClassBBindingProfile {
    ClassBBindingProfile {
        receipts: class_b_binding_receipts(resolved),
        rust_errors: class_b_rust_binding_errors(&hard_messages(resolved)),
    }
}

fn assert_exact_rust_declaration_identities(
    receipts: &BTreeMap<String, CrossModuleBindingReceipt>,
) {
    for sym in CLASS_B_RUST_BINDING_SYMBOLS {
        let receipt = receipts
            .get(*sym)
            .unwrap_or_else(|| panic!("missing binding receipt for Class B symbol {sym}"));
        assert_eq!(
            receipt.definer_module.as_deref(),
            Some(RUST_MODULE),
            "symbol {sym} must bind to the rust.dag authority, not a homonym"
        );
        assert_eq!(
            receipt.binding_source,
            Some(UnlistedImportBindingSource::ListedImport),
            "symbol {sym} must bind through the declared import edge, not pool coincidence"
        );
    }
}

#[test]
fn positive_declared_import_closure_includes_rust_dag() {
    let entry = workspace_root().join(ENTRY_REL);
    let paths = v1_compiler::cli_run::declared_import_closure_live_paths(
        &witness_layer_roots(),
        entry.to_str().unwrap(),
    )
    .expect("declared import closure paths");
    assert!(
        closure_has_rust_dag(&paths),
        "positive control: declared import closure must include rust.dag; got {} paths",
        paths.len()
    );
}

#[test]
fn positive_declared_import_closure_binds_exact_rust_declaration_identities() {
    let entry = workspace_root().join(ENTRY_REL);
    let resolved = compile_entry_on_declared_import_closure_only(
        &witness_layer_roots(),
        entry.to_str().unwrap(),
    )
    .expect("compile on declared import closure");
    let profile = class_b_binding_profile(&resolved);
    assert!(
        profile.rust_errors.is_empty(),
        "positive control: Class B rust symbols must bind without diagnostics; got {:#?}",
        profile.rust_errors
    );
    assert_exact_rust_declaration_identities(&profile.receipts);
}

#[test]
fn pool_perturbation_preserves_class_b_binding_identities() {
    let entry = workspace_root().join(ENTRY_REL);
    let entry_str = entry.to_str().unwrap();

    let baseline = class_b_binding_profile(
        &compile_entry_on_declared_import_closure_only(&witness_layer_roots(), entry_str)
            .expect("baseline compile"),
    );

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = workspace_root().join("target").join(format!(
        "rtf-pool-perturb-{}-{}",
        std::process::id(),
        nanos
    ));
    write_pool_perturbation_fixtures(&dir);
    let perturbed = class_b_binding_profile(
        &compile_entry_on_declared_import_closure_only(&perturbation_pool_roots(&dir), entry_str)
            .expect("perturbed compile"),
    );
    let _ = fs::remove_dir_all(&dir);

    assert_eq!(
        baseline.receipts, perturbed.receipts,
        "pool perturbation must not change Class B declaration identities"
    );
    assert_eq!(
        baseline.rust_errors, perturbed.rust_errors,
        "pool perturbation must not change Class B refusal population"
    );
    assert_exact_rust_declaration_identities(&perturbed.receipts);
}

#[test]
fn negative_stripped_declared_import_closure_refuses_rust_bindings() {
    let entry = workspace_root().join(ENTRY_REL);
    let stripped = strip_import_block(&read_v2_file(ENTRY_REL));
    assert!(
        !stripped.contains("import v2.extdeps.languages.rust"),
        "negative fixture must not retain the rust import block"
    );
    let resolved = compile_declared_import_closure_only_with_pool(
        &witness_layer_roots(),
        entry.to_str().unwrap(),
        Some(&stripped),
    )
    .expect("compile stripped entry on entry-only closure");
    let rust_errors = class_b_rust_binding_errors(&hard_messages(&resolved));
    assert!(
        !rust_errors.is_empty(),
        "negative control: stripped entry must refuse Class B rust symbol bindings"
    );
}

#[test]
fn negative_stripped_refuses_even_when_ambient_pool_imports_rust() {
    let entry = workspace_root().join(ENTRY_REL);
    let stripped = strip_import_block(&read_v2_file(ENTRY_REL));
    assert!(
        !stripped.contains("import v2.extdeps.languages.rust"),
        "negative fixture must not retain the rust import block"
    );

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = workspace_root().join("target").join(format!(
        "rtf-ambient-negative-{}-{}",
        std::process::id(),
        nanos
    ));
    write_pool_perturbation_fixtures(&dir);

    let resolved = compile_declared_import_closure_only_with_pool(
        &perturbation_pool_roots(&dir),
        entry.to_str().unwrap(),
        Some(&stripped),
    )
    .expect("compile stripped entry with ambient rust loader in pool");
    let _ = fs::remove_dir_all(&dir);

    let rust_errors = class_b_rust_binding_errors(&hard_messages(&resolved));
    assert!(
        !rust_errors.is_empty(),
        "negative control: stripped entry must still refuse Class B rust bindings even when \
         an unrelated ambient module imports rust.dag into the pool"
    );
}
