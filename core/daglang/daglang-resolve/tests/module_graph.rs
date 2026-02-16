use daglang_resolve::{ModuleGraph, ResolveError};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("daglang_resolve_{name}_{}_{}", std::process::id(), nanos))
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent directories");
    }
    fs::write(path, content).expect("failed to write test file");
}

#[test]
fn discovers_all_real_dsl_modules() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let graph = ModuleGraph::discover(&[dsl_root]).expect("expected real dsl graph to parse");

    assert_eq!(graph.modules.len(), 42, "expected 42 discovered modules");
    let module_names: Vec<String> = graph
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect();
    assert!(module_names.iter().any(|m| m == "tools.makegen"));
    assert!(module_names.iter().any(|m| m == "std.types"));
    assert!(module_names.iter().any(|m| m == "infra.core"));
    assert!(module_names.iter().any(|m| m == "pipelines.ci"));
}

#[test]
fn duplicate_module_paths_are_rejected() {
    let root = unique_temp_dir("duplicate");
    write_file(
        &root.join("a/one.dag"),
        "module dup.mod\nfn one() -> Unit {}",
    );
    write_file(
        &root.join("b/two.dag"),
        "module dup.mod\nfn two() -> Unit {}",
    );

    let err = ModuleGraph::discover(&[root.clone()]).expect_err("expected duplicate module error");
    match err {
        ResolveError::DuplicateModule(path) => {
            assert_eq!(path.join("."), "dup.mod");
        }
        other => panic!("expected DuplicateModule, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn unresolved_imports_are_tolerated_for_phase_zero_discovery() {
    let root = unique_temp_dir("unresolved");
    write_file(
        &root.join("a/main.dag"),
        "module a.main\nimport missing.dep\nfn run() -> Unit {}",
    );

    let graph = ModuleGraph::discover(&[root.clone()]).expect("expected graph discovery success");
    assert_eq!(graph.modules.len(), 1);
    assert_eq!(graph.modules[0].module_path.join("."), "a.main");
    assert!(
        graph.modules[0].dependencies.is_empty(),
        "unresolved imports should be ignored in phase-0 graph construction"
    );

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn cyclic_dependencies_are_tolerated_for_phase_zero_discovery() {
    let root = unique_temp_dir("cycle");
    write_file(
        &root.join("a/a.dag"),
        "module cycle.a\nimport cycle.b\nfn a() -> Unit {}",
    );
    write_file(
        &root.join("b/b.dag"),
        "module cycle.b\nimport cycle.a\nfn b() -> Unit {}",
    );

    let graph = ModuleGraph::discover(&[root.clone()]).expect("expected cycle-tolerant discover");
    let module_names: Vec<String> = graph
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect();
    assert!(module_names.iter().any(|name| name == "cycle.a"));
    assert!(module_names.iter().any(|name| name == "cycle.b"));

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}
