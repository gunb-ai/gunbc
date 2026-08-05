//! Class B closure control for `rust_test_fixtures` (#7811 follow-on).
//!
//! Proves `src/v2/extdeps/languages/rust_test_fixtures.dag` binds
//! `v2.extdeps.languages.rust` through its **declared import-edge closure** —
//! not pool membership coincidence. Both arms use a narrow import closure (not
//! whole-tree compile). The positive arm checks the four symbols that redded in
//! the Class B incident; full-file typecheck is intentionally out of scope
//! because the entry uses many symbols outside its single `import` block.

use std::rc::Rc;

use v1_compiler::cli_run::{
    compile_entry_on_declared_import_closure_only,
    compile_stripped_entry_declared_import_closure_only,
};
use v1_compiler::v1_compiler_compile::ResolvedPipelineResult;

use crate::helpers::{read_v2_file, workspace_root};

const ENTRY_REL: &str = "src/v2/extdeps/languages/rust_test_fixtures.dag";
const RUST_DAG_SUFFIX: &str = "src/v2/extdeps/languages/rust.dag";

/// The four cross-module rust symbols that failed under pool-coincidence drift.
const CLASS_B_RUST_BINDING_SYMBOLS: &[&str] = &[
    "rust_selection_policy_node",
    "rust_operator_realizations_catalog_node",
    "rust_grammar_terminal",
    "rust_inhabitant_atom",
];

fn witness_layer_roots() -> Vec<String> {
    let ws = workspace_root();
    vec![
        ws.join("dag").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ]
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
fn positive_declared_import_closure_binds_rust_symbols() {
    let entry = workspace_root().join(ENTRY_REL);
    let resolved = compile_entry_on_declared_import_closure_only(
        &witness_layer_roots(),
        entry.to_str().unwrap(),
    )
    .expect("compile on declared import closure");
    let rust_errors = class_b_rust_binding_errors(&hard_messages(&resolved));
    assert!(
        rust_errors.is_empty(),
        "positive control: Class B rust symbols must bind through the declared import \
         closure (not pool coincidence); rust-binding diags: {rust_errors:?}"
    );
}

#[test]
fn negative_stripped_declared_import_closure_excludes_rust_dag() {
    let entry = workspace_root().join(ENTRY_REL);
    let stripped = strip_import_block(&read_v2_file(ENTRY_REL));
    assert!(
        !stripped.contains("import v2.extdeps.languages.rust"),
        "negative fixture must not retain the rust import block"
    );
    // Zero import lines ⇒ declared import closure is the entry module alone.
    let closure_paths = vec![ENTRY_REL.to_string()];
    assert!(
        !closure_has_rust_dag(&closure_paths),
        "negative control: stripped entry's declared import closure must not include rust.dag"
    );
    let _ = (entry, stripped);
}

#[test]
fn negative_stripped_declared_import_closure_refuses_rust_bindings() {
    let entry = workspace_root().join(ENTRY_REL);
    let stripped = strip_import_block(&read_v2_file(ENTRY_REL));
    let resolved =
        compile_stripped_entry_declared_import_closure_only(entry.to_str().unwrap(), &stripped);
    let rust_errors = class_b_rust_binding_errors(&hard_messages(&resolved));
    assert!(
        !rust_errors.is_empty(),
        "negative control: stripped entry must refuse Class B rust symbol bindings on \
         its declared import closure only; got no rust-binding diags"
    );
}
