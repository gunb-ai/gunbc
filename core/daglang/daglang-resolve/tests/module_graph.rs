use daglang_resolve::{ModuleGraph, ResolveError};
use daglang_syntax::diagnostic::DiagnosticKind;
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
fn discovery_with_empty_roots_returns_empty_graph() {
    let graph = ModuleGraph::discover(&[]).expect("empty roots should be valid");
    assert!(graph.modules.is_empty(), "expected no discovered modules");
    assert!(
        graph.display_tree().is_empty(),
        "empty graph display should be empty"
    );
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
fn missing_discovery_root_is_rejected() {
    let root = unique_temp_dir("missing_root");
    let err = ModuleGraph::discover(&[root.clone()]).expect_err("expected invalid root error");
    match err {
        ResolveError::InvalidRootPath { path, reason } => {
            assert_eq!(path, root);
            assert!(reason.contains("does not exist"));
        }
        other => panic!("expected InvalidRootPath, got {other:?}"),
    }
}

#[test]
fn discovery_fails_when_any_root_is_missing() {
    let valid_root = unique_temp_dir("mixed_valid_missing_valid");
    write_file(
        &valid_root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    );
    let missing_root = unique_temp_dir("mixed_valid_missing_missing");

    let err = ModuleGraph::discover(&[valid_root.clone(), missing_root.clone()])
        .expect_err("expected invalid root error");
    match err {
        ResolveError::InvalidRootPath { path, reason } => {
            assert_eq!(path, missing_root);
            assert!(reason.contains("does not exist"));
        }
        other => panic!("expected InvalidRootPath, got {other:?}"),
    }

    fs::remove_dir_all(valid_root).expect("failed to clean temp directory");
}

#[test]
fn non_directory_discovery_root_is_rejected() {
    let root = unique_temp_dir("non_directory_root");
    write_file(&root, "not a directory");

    let err = ModuleGraph::discover(&[root.clone()]).expect_err("expected invalid root error");
    match err {
        ResolveError::InvalidRootPath { path, reason } => {
            assert_eq!(path, root);
            assert!(reason.contains("is not a directory"));
        }
        other => panic!("expected InvalidRootPath, got {other:?}"),
    }

    fs::remove_file(root).expect("failed to clean temp file");
}

#[test]
fn discovery_fails_when_any_root_is_non_directory() {
    let valid_root = unique_temp_dir("mixed_valid_nondir_valid");
    write_file(
        &valid_root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    );
    let non_directory_root = unique_temp_dir("mixed_valid_nondir_file");
    write_file(&non_directory_root, "not a directory");

    let err = ModuleGraph::discover(&[valid_root.clone(), non_directory_root.clone()])
        .expect_err("expected invalid root error");
    match err {
        ResolveError::InvalidRootPath { path, reason } => {
            assert_eq!(path, non_directory_root);
            assert!(reason.contains("is not a directory"));
        }
        other => panic!("expected InvalidRootPath, got {other:?}"),
    }

    fs::remove_dir_all(valid_root).expect("failed to clean temp directory");
    fs::remove_file(non_directory_root).expect("failed to clean temp file");
}

#[test]
fn invalid_root_path_error_display_includes_path_and_reason() {
    let missing_root = unique_temp_dir("display_missing_root");
    let missing_err =
        ModuleGraph::discover(&[missing_root.clone()]).expect_err("expected invalid root error");
    let missing_rendered = missing_err.to_string();
    assert!(missing_rendered.contains("invalid discovery root"));
    assert!(missing_rendered.contains(&missing_root.display().to_string()));
    assert!(missing_rendered.contains("does not exist"));

    let non_dir_root = unique_temp_dir("display_non_dir_root");
    write_file(&non_dir_root, "not a directory");
    let non_dir_err =
        ModuleGraph::discover(&[non_dir_root.clone()]).expect_err("expected invalid root error");
    let non_dir_rendered = non_dir_err.to_string();
    assert!(non_dir_rendered.contains("invalid discovery root"));
    assert!(non_dir_rendered.contains(&non_dir_root.display().to_string()));
    assert!(non_dir_rendered.contains("is not a directory"));

    fs::remove_file(non_dir_root).expect("failed to clean temp file");
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

#[test]
fn parse_error_contains_file_line_col_rendering() {
    let root = unique_temp_dir("parse_error_location");
    write_file(&root.join("broken.dag"), "module sample.broken\nfn");

    let err = ModuleGraph::discover(&[root.clone()]).expect_err("expected parse error");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(files.len(), 1);
            assert_eq!(files[0].0.file_name().and_then(|n| n.to_str()), Some("broken.dag"));
            assert!(
                files[0].1.iter().any(|diag| diag.render().contains(":2:")),
                "expected diagnostic to include line/column: {:?}",
                files[0].1
            );
            assert!(
                files[0]
                    .1
                    .iter()
                    .all(|diag| diag.kind == DiagnosticKind::Parse),
                "expected parser diagnostics for malformed syntax"
            );
        }
        other => panic!("expected ParseErrors, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn lex_error_is_reported_with_lex_diagnostic_kind() {
    let root = unique_temp_dir("lex_error_location");
    write_file(&root.join("broken.dag"), "module sample.broken\n$\n");

    let err = ModuleGraph::discover(&[root.clone()]).expect_err("expected lex error");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(files.len(), 1);
            assert_eq!(files[0].0.file_name().and_then(|n| n.to_str()), Some("broken.dag"));
            let diag = files[0]
                .1
                .first()
                .expect("expected a lexical diagnostic");
            assert_eq!(diag.kind, DiagnosticKind::Lex);
            assert!(diag.render().contains(":2:1:"));
        }
        other => panic!("expected ParseErrors, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn lex_error_file_order_is_deterministic() {
    let root = unique_temp_dir("lex_error_order");
    write_file(&root.join("z_lex.dag"), "module sample.z\n$\n");
    write_file(&root.join("a_lex.dag"), "module sample.a\n$\n");

    let err = ModuleGraph::discover(&[root.clone()]).expect_err("expected lex errors");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(files.len(), 2, "expected diagnostics for both lex-broken files");
            let names: Vec<String> = files
                .iter()
                .map(|(path, _)| path.file_name().unwrap().to_string_lossy().to_string())
                .collect();
            assert_eq!(
                names,
                vec!["a_lex.dag".to_string(), "z_lex.dag".to_string()],
                "lex error file ordering should be deterministic and path-sorted"
            );
            for (_path, diagnostics) in &files {
                assert!(
                    diagnostics.iter().all(|diag| diag.kind == DiagnosticKind::Lex),
                    "expected lexical diagnostic kinds for lex-broken files"
                );
            }
        }
        other => panic!("expected ParseErrors, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn lex_errors_are_aggregated_within_single_file() {
    let root = unique_temp_dir("lex_error_multi");
    write_file(&root.join("broken.dag"), "module sample.broken\n$\n&\n");

    let err = ModuleGraph::discover(&[root.clone()]).expect_err("expected lex errors");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(files.len(), 1);
            let diagnostics = &files[0].1;
            assert_eq!(
                diagnostics.len(),
                2,
                "expected both lexical diagnostics for one file"
            );
            assert!(diagnostics.iter().all(|diag| diag.kind == DiagnosticKind::Lex));
            assert_eq!(diagnostics[0].line, Some(2));
            assert_eq!(diagnostics[0].column, Some(1));
            assert_eq!(diagnostics[1].line, Some(3));
            assert_eq!(diagnostics[1].column, Some(1));
        }
        other => panic!("expected ParseErrors, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn parse_errors_are_aggregated_across_multiple_files() {
    let root = unique_temp_dir("parse_error_many");
    write_file(&root.join("broken_a.dag"), "module sample.a\nfn");
    write_file(&root.join("broken_b.dag"), "module sample.b\nimport");

    let err = ModuleGraph::discover(&[root.clone()]).expect_err("expected parse errors");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(files.len(), 2, "expected diagnostics for both broken files");
            let names: Vec<String> = files
                .into_iter()
                .map(|(path, _)| path.file_name().unwrap().to_string_lossy().to_string())
                .collect();
            assert!(names.iter().any(|name| name == "broken_a.dag"));
            assert!(names.iter().any(|name| name == "broken_b.dag"));
        }
        other => panic!("expected ParseErrors, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn parse_error_file_order_is_deterministic() {
    let root = unique_temp_dir("parse_error_order");
    write_file(&root.join("z_broken.dag"), "module sample.z\nfn");
    write_file(&root.join("a_broken.dag"), "module sample.a\nimport");

    let err = ModuleGraph::discover(&[root.clone()]).expect_err("expected parse errors");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(files.len(), 2, "expected diagnostics for both broken files");
            let names: Vec<String> = files
                .iter()
                .map(|(path, _)| path.file_name().unwrap().to_string_lossy().to_string())
                .collect();
            assert_eq!(
                names,
                vec!["a_broken.dag".to_string(), "z_broken.dag".to_string()],
                "parse error file ordering should be deterministic and path-sorted"
            );
        }
        other => panic!("expected ParseErrors, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn mixed_lex_and_parse_error_file_order_is_path_sorted() {
    let root = unique_temp_dir("mixed_error_order");
    write_file(&root.join("a_parse.dag"), "module sample.a\nfn");
    write_file(&root.join("z_lex.dag"), "module sample.z\n$\n");

    let err = ModuleGraph::discover(&[root.clone()]).expect_err("expected mixed errors");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(files.len(), 2, "expected diagnostics for both broken files");
            let names: Vec<String> = files
                .iter()
                .map(|(path, _)| path.file_name().unwrap().to_string_lossy().to_string())
                .collect();
            assert_eq!(
                names,
                vec!["a_parse.dag".to_string(), "z_lex.dag".to_string()],
                "mixed error file ordering should remain path-sorted"
            );
            assert!(
                files[0]
                    .1
                    .iter()
                    .all(|diag| diag.kind == DiagnosticKind::Parse),
                "first file should preserve parse diagnostic kind"
            );
            assert!(
                files[1]
                    .1
                    .iter()
                    .all(|diag| diag.kind == DiagnosticKind::Lex),
                "second file should preserve lex diagnostic kind"
            );
        }
        other => panic!("expected ParseErrors, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn parse_error_display_includes_all_files_and_locations() {
    let root = unique_temp_dir("parse_error_display");
    write_file(&root.join("broken_a.dag"), "module sample.a\nfn");
    write_file(&root.join("broken_b.dag"), "module sample.b\nimport");

    let err = ModuleGraph::discover(&[root.clone()]).expect_err("expected parse errors");
    let rendered = err.to_string();
    assert!(
        rendered.contains("broken_a.dag"),
        "display should include first broken file path"
    );
    assert!(
        rendered.contains("broken_b.dag"),
        "display should include second broken file path"
    );
    assert!(
        rendered.contains(":2:"),
        "display should include line/column data"
    );

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn discovery_order_is_deterministic_across_runs() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let graph_a = ModuleGraph::discover(&[dsl_root.clone()]).expect("first discover should succeed");
    let graph_b = ModuleGraph::discover(&[dsl_root]).expect("second discover should succeed");

    let order_a: Vec<String> = graph_a
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect();
    let order_b: Vec<String> = graph_b
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect();

    assert_eq!(order_a, order_b, "module discovery order should be stable");
}

#[test]
fn display_tree_output_is_deterministic_across_runs() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let graph_a = ModuleGraph::discover(&[dsl_root.clone()]).expect("first discover should succeed");
    let graph_b = ModuleGraph::discover(&[dsl_root]).expect("second discover should succeed");

    assert_eq!(
        graph_a.display_tree(),
        graph_b.display_tree(),
        "display tree output should be deterministic"
    );
}

#[test]
fn discovery_is_independent_of_root_argument_order() {
    let root = unique_temp_dir("root_order");
    let a = root.join("a");
    let b = root.join("b");
    write_file(&a.join("a.dag"), "module sample.a\nfn ok() -> Unit {}");
    write_file(&b.join("b.dag"), "module sample.b\nfn ok() -> Unit {}");

    let graph_ab = ModuleGraph::discover(&[a.clone(), b.clone()]).expect("first discover");
    let graph_ba = ModuleGraph::discover(&[b, a]).expect("second discover");

    let order_ab: Vec<String> = graph_ab
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect();
    let order_ba: Vec<String> = graph_ba
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect();
    assert_eq!(order_ab, order_ba);
    assert_eq!(graph_ab.display_tree(), graph_ba.display_tree());

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn display_tree_contains_expected_summary_fields() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let graph = ModuleGraph::discover(&[dsl_root]).expect("discover should succeed");
    let tree = graph.display_tree();

    assert!(
        tree.contains("std.types"),
        "display tree should include module path names"
    );
    assert!(
        tree.contains("items"),
        "display tree should include item counts"
    );
    assert!(
        tree.contains("deps"),
        "display tree should include dependency counts"
    );
}

#[test]
fn discovery_ignores_non_dag_files() {
    let root = unique_temp_dir("ignore_non_dag");
    write_file(
        &root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    );
    write_file(&root.join("notes.txt"), "module not.real\n$");

    let graph = ModuleGraph::discover(&[root.clone()]).expect("discover should ignore non-dag");
    assert_eq!(graph.modules.len(), 1);
    assert_eq!(graph.modules[0].module_path.join("."), "sample.main");

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn discovery_deduplicates_files_from_overlapping_roots() {
    let root = unique_temp_dir("overlapping_roots");
    let nested = root.join("nested");
    write_file(
        &nested.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    );

    let graph =
        ModuleGraph::discover(&[root.clone(), nested]).expect("discover should dedupe file paths");
    assert_eq!(graph.modules.len(), 1);
    assert_eq!(graph.modules[0].module_path.join("."), "sample.main");

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn discovery_deduplicates_files_from_duplicate_roots() {
    let root = unique_temp_dir("duplicate_roots");
    write_file(
        &root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    );

    let graph = ModuleGraph::discover(&[root.clone(), root.clone()])
        .expect("discover should dedupe duplicate root entries");
    assert_eq!(graph.modules.len(), 1);
    assert_eq!(graph.modules[0].module_path.join("."), "sample.main");

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}
