//! Probe: can we discriminate declared-import closure from bare-reference / pool coincidence
//! for `rust_test_fixtures` with src/v2-only pool (no dag/ witness roots)?

use std::fs;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use v1_compiler::cli_run::{
    import_closure_live_paths_with_facts, load_sources_for_entry_with_pool_index,
    resolve_entry_graph,
};
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};

use crate::helpers::{read_v2_file, workspace_root};

const ENTRY_REL: &str = "src/v2/extdeps/languages/rust_test_fixtures.dag";
const RUST_DAG_SUFFIX: &str = "src/v2/extdeps/languages/rust.dag";

fn v2_roots() -> Vec<String> {
    vec![workspace_root().join("src/v2").to_string_lossy().into_owned()]
}

fn hard_messages(resolved: &ResolvedPipelineResult) -> Vec<String> {
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
    paths.iter().any(|p| p.replace('\\', "/").ends_with(RUST_DAG_SUFFIX))
}

#[test]
fn probe_positive_import_closure_includes_rust_dag_src_v2_only() {
    let facts = v1_compiler::cli_run::build_module_graph_facts_live(&v2_roots());
    let import_closure =
        import_closure_live_paths_with_facts(ENTRY_REL, &facts);
    assert!(
        closure_has_rust_dag(&import_closure),
        "declared import closure must include rust.dag; got {} paths",
        import_closure.len()
    );
}

#[test]
fn probe_positive_resolves_on_src_v2_only_pool() {
    let entry = workspace_root().join(ENTRY_REL);
    let (_graph, _) = resolve_entry_graph(&v2_roots(), entry.to_str().unwrap())
        .expect("positive resolve_entry_graph");
}

#[test]
fn probe_negative_stripped_import_closure_excludes_rust_dag() {
    let content = read_v2_file(ENTRY_REL);
    let stripped = strip_import_block(&content);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = workspace_root()
        .join("target")
        .join(format!("rtf-stripped-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&dir).expect("tmpdir");
    let rel = "extdeps/languages/rust_test_fixtures.dag";
    let entry_path = dir.join(rel);
    fs::create_dir_all(entry_path.parent().unwrap()).expect("parent");
    fs::write(&entry_path, &stripped).expect("write stripped");

    let facts = v1_compiler::cli_run::build_module_graph_facts_live(&v2_roots());
    let import_closure =
        import_closure_live_paths_with_facts(entry_path.to_str().unwrap(), &facts);
    assert!(
        !closure_has_rust_dag(&import_closure),
        "stripped file import-closure must NOT include rust.dag; got {:?}",
        import_closure
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn probe_negative_stripped_both_closure_resolve_on_src_v2_only() {
    let content = read_v2_file(ENTRY_REL);
    let stripped = strip_import_block(&content);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = workspace_root()
        .join("target")
        .join(format!("rtf-stripped-resolve-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&dir).expect("tmpdir");
    let rel = "extdeps/languages/rust_test_fixtures.dag";
    let entry_path = dir.join(rel);
    fs::create_dir_all(entry_path.parent().unwrap()).expect("parent");
    fs::write(&entry_path, &stripped).expect("write stripped");

    let sources = load_sources_for_entry_with_pool_index(
        &v2_roots(),
        entry_path.to_str().unwrap(),
        true,
    )
    .expect("load both-closure sources");
    let has_rust = sources.iter().any(|s| {
        s.path.replace('\\', "/").ends_with(RUST_DAG_SUFFIX)
    });
    eprintln!(
        "stripped both-closure: {} modules, rust.dag present = {has_rust}",
        sources.len()
    );

    let resolved = compile_to_resolved(Rc::new(sources.into()));
    let hard = hard_messages(&resolved);
    eprintln!("stripped resolve hard diags: {hard:?}");
    let _ = fs::remove_dir_all(&dir);
}
