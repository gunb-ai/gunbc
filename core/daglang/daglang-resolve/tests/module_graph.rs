use daglang_resolve::{ModuleGraph, ResolveError};
use daglang_syntax::diagnostic::DiagnosticKind;
use std::collections::BTreeMap;
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

fn expected_dsl_modules_sorted() -> Vec<&'static str> {
    vec![
        "cloud.aws.credential",
        "cloud.azure.credential",
        "cloud.gcp.credential",
        "examples.abstract_services",
        "examples.deployment",
        "examples.integration_tests",
        "examples.rich_types",
        "infra.aws.config",
        "infra.aws.resources",
        "infra.aws.services",
        "infra.azure.config",
        "infra.azure.resources",
        "infra.azure.services",
        "infra.core",
        "infra.gcp.config",
        "infra.gcp.resources",
        "infra.gcp.services",
        "infra.spec",
        "pipelines.ci",
        "services.cargo",
        "services.gcp.iam",
        "services.gcp.secret_manager",
        "services.gcp.sts",
        "services.git",
        "services.github.gist",
        "services.shell",
        "shared.dag_util",
        "shared.gist_modes",
        "std.patterns",
        "std.resources",
        "std.types",
        "tools.bootstrap",
        "tools.build",
        "tools.clippy",
        "tools.codegen",
        "tools.dag_viz",
        "tools.deps",
        "tools.docgen",
        "tools.gist",
        "tools.makegen",
        "tools.pragma",
        "tools.testgen",
    ]
}

#[test]
fn discovers_all_real_dsl_modules() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let graph = ModuleGraph::discover(&[dsl_root]).expect("expected real dsl graph to parse");

    assert_eq!(graph.modules.len(), 42, "expected 42 discovered modules");
    let mut module_names: Vec<String> = graph
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect();
    module_names.sort();
    let expected: Vec<String> = expected_dsl_modules_sorted()
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(module_names, expected);
}

#[test]
fn discovered_module_paths_match_ast_module_declarations() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let graph = ModuleGraph::discover(&[dsl_root]).expect("expected real dsl graph to parse");

    for module in &graph.modules {
        let ast_module_path = module
            .ast
            .module_path
            .as_ref()
            .map(|module| module.node.segments.clone())
            .expect("real corpus files should contain module declarations");
        assert_eq!(
            module.module_path, ast_module_path,
            "resolved module path should match parsed AST module declaration for {}",
            module.path.display()
        );
    }
}

#[test]
fn real_corpus_dependency_counts_match_expected_snapshot() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let graph = ModuleGraph::discover(&[dsl_root]).expect("expected real dsl graph to parse");

    let actual: BTreeMap<String, usize> = graph
        .modules
        .iter()
        .map(|module| (module.module_path.join("."), module.dependencies.len()))
        .collect();
    let expected: BTreeMap<String, usize> = BTreeMap::from([
        ("cloud.aws.credential".into(), 2),
        ("cloud.azure.credential".into(), 2),
        ("cloud.gcp.credential".into(), 5),
        ("examples.abstract_services".into(), 0),
        ("examples.deployment".into(), 9),
        ("examples.integration_tests".into(), 1),
        ("examples.rich_types".into(), 0),
        ("infra.aws.config".into(), 2),
        ("infra.aws.resources".into(), 3),
        ("infra.aws.services".into(), 0),
        ("infra.azure.config".into(), 2),
        ("infra.azure.resources".into(), 3),
        ("infra.azure.services".into(), 0),
        ("infra.core".into(), 0),
        ("infra.gcp.config".into(), 2),
        ("infra.gcp.resources".into(), 3),
        ("infra.gcp.services".into(), 0),
        ("infra.spec".into(), 1),
        ("pipelines.ci".into(), 10),
        ("services.cargo".into(), 0),
        ("services.gcp.iam".into(), 0),
        ("services.gcp.secret_manager".into(), 0),
        ("services.gcp.sts".into(), 0),
        ("services.git".into(), 0),
        ("services.github.gist".into(), 0),
        ("services.shell".into(), 0),
        ("shared.dag_util".into(), 2),
        ("shared.gist_modes".into(), 2),
        ("std.patterns".into(), 4),
        ("std.resources".into(), 0),
        ("std.types".into(), 0),
        ("tools.bootstrap".into(), 4),
        ("tools.build".into(), 3),
        ("tools.clippy".into(), 4),
        ("tools.codegen".into(), 2),
        ("tools.dag_viz".into(), 5),
        ("tools.deps".into(), 4),
        ("tools.docgen".into(), 2),
        ("tools.gist".into(), 6),
        ("tools.makegen".into(), 3),
        ("tools.pragma".into(), 3),
        ("tools.testgen".into(), 2),
    ]);

    assert_eq!(actual, expected);
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
fn parse_errors_are_independent_of_root_argument_order() {
    let root = unique_temp_dir("parse_error_root_order");
    let a = root.join("a");
    let b = root.join("b");
    write_file(&a.join("a_parse.dag"), "module sample.a\nfn");
    write_file(&b.join("b_lex.dag"), "module sample.b\n$\n");

    let first = ModuleGraph::discover(&[a.clone(), b.clone()])
        .expect_err("first discover should produce parse errors");
    let second =
        ModuleGraph::discover(&[b, a]).expect_err("second discover should produce parse errors");

    let (files_first, files_second) = match (first, second) {
        (ResolveError::ParseErrors(files_first), ResolveError::ParseErrors(files_second)) => {
            (files_first, files_second)
        }
        (left, right) => panic!(
            "expected ParseErrors for both runs, got left={left:?}, right={right:?}"
        ),
    };

    assert_eq!(files_first, files_second);
    assert_eq!(files_first.len(), 2, "expected diagnostics from both roots");

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

#[test]
fn discovery_deduplicates_parse_errors_from_duplicate_roots() {
    let root = unique_temp_dir("duplicate_roots_parse_errors");
    write_file(&root.join("broken.dag"), "module sample.broken\nfn");

    let err = ModuleGraph::discover(&[root.clone(), root.clone()])
        .expect_err("expected parse error for duplicate roots");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(
                files.len(),
                1,
                "duplicate roots should not duplicate parse error file entries"
            );
            assert_eq!(
                files[0].0.file_name().and_then(|name| name.to_str()),
                Some("broken.dag")
            );
        }
        other => panic!("expected ParseErrors, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[cfg(unix)]
#[test]
fn discovery_deduplicates_files_from_symlink_and_real_roots() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("symlink_and_real_roots");
    let real = root.join("real");
    let link = root.join("link");
    write_file(
        &real.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    );
    symlink(&real, &link).expect("failed to create root symlink");

    let graph = ModuleGraph::discover(&[real.clone(), link])
        .expect("symlink+real roots should deduplicate discovered files");
    assert_eq!(graph.modules.len(), 1);
    assert_eq!(graph.modules[0].module_path.join("."), "sample.main");

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[cfg(unix)]
#[test]
fn discovery_is_independent_of_symlink_alias_root_order() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("symlink_alias_root_order");
    let real = root.join("real");
    let link = root.join("link");
    write_file(
        &real.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    );
    symlink(&real, &link).expect("failed to create root symlink");

    let first = ModuleGraph::discover(&[real.clone(), link.clone()]).expect("first discover");
    let second = ModuleGraph::discover(&[link, real]).expect("second discover");

    let first_paths: Vec<String> = first
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect();
    let second_paths: Vec<String> = second
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect();
    assert_eq!(first_paths, second_paths);
    assert_eq!(first.display_tree(), second.display_tree());

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[cfg(unix)]
#[test]
fn discovery_parse_errors_are_independent_of_symlink_alias_root_order() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("symlink_alias_root_order_parse_errors");
    let real = root.join("real");
    let link = root.join("link");
    write_file(&real.join("broken.dag"), "module sample.broken\nfn");
    symlink(&real, &link).expect("failed to create root symlink");

    let first = ModuleGraph::discover(&[real.clone(), link.clone()])
        .expect_err("first discover should return parse errors");
    let second = ModuleGraph::discover(&[link, real])
        .expect_err("second discover should return parse errors");

    match (first, second) {
        (ResolveError::ParseErrors(first_files), ResolveError::ParseErrors(second_files)) => {
            assert_eq!(first_files, second_files);
        }
        (left, right) => panic!(
            "expected ParseErrors for both runs, got left={left:?}, right={right:?}"
        ),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[cfg(unix)]
#[test]
fn discovery_fallback_module_path_is_independent_of_symlink_alias_root_order() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("symlink_alias_root_order_no_module");
    let real = root.join("real");
    let link = root.join("link");
    write_file(&real.join("nested/no_module.dag"), "fn ok() -> Unit {}");
    symlink(&real, &link).expect("failed to create root symlink");

    let first = ModuleGraph::discover(&[real.clone(), link.clone()]).expect("first discover");
    let second = ModuleGraph::discover(&[link, real]).expect("second discover");

    let first_paths: Vec<String> = first
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect();
    let second_paths: Vec<String> = second
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect();
    assert_eq!(first_paths, second_paths);
    assert_eq!(first_paths, vec!["nested.no_module".to_string()]);

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn discovery_fallback_module_path_is_independent_of_overlapping_root_order() {
    let root = unique_temp_dir("overlapping_root_order_no_module");
    let nested = root.join("nested");
    write_file(&nested.join("no_module.dag"), "fn ok() -> Unit {}");

    let first = ModuleGraph::discover(&[root.clone(), nested.clone()]).expect("first discover");
    let second = ModuleGraph::discover(&[nested, root.clone()]).expect("second discover");

    let first_paths: Vec<String> = first
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect();
    let second_paths: Vec<String> = second
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect();
    assert_eq!(first_paths, second_paths);
    assert_eq!(first_paths, vec!["no_module".to_string()]);

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[cfg(unix)]
#[test]
fn discovery_handles_directory_symlink_cycle_without_recursing_forever() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("dir_symlink_cycle");
    let nested = root.join("nested");
    write_file(
        &nested.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    );
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let graph = ModuleGraph::discover(&[root.clone()])
        .expect("discover should handle directory cycle symlink");
    assert_eq!(graph.modules.len(), 1);
    assert_eq!(graph.modules[0].module_path.join("."), "sample.main");

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[cfg(unix)]
#[test]
fn discovery_deduplicates_parse_errors_in_directory_symlink_cycle() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("dir_symlink_cycle_parse_errors");
    let nested = root.join("nested");
    write_file(&nested.join("broken.dag"), "module sample.broken\nfn");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let err = ModuleGraph::discover(&[root.clone()])
        .expect_err("discover should return parse errors for malformed source");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(
                files.len(),
                1,
                "directory symlink cycle should not duplicate parse-error files"
            );
            assert_eq!(
                files[0].0.file_name().and_then(|name| name.to_str()),
                Some("broken.dag")
            );
        }
        other => panic!("expected ParseErrors, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[cfg(unix)]
#[test]
fn discovery_parse_errors_are_deterministic_in_directory_symlink_cycle() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("dir_symlink_cycle_deterministic_parse_errors");
    let nested = root.join("nested");
    write_file(&nested.join("broken.dag"), "module sample.broken\nfn");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let first = ModuleGraph::discover(&[root.clone()])
        .expect_err("first discover should return parse errors");
    let second = ModuleGraph::discover(&[root.clone()])
        .expect_err("second discover should return parse errors");

    match (first, second) {
        (ResolveError::ParseErrors(first_files), ResolveError::ParseErrors(second_files)) => {
            assert_eq!(first_files, second_files);
        }
        (left, right) => panic!(
            "expected ParseErrors for both runs, got left={left:?}, right={right:?}"
        ),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[cfg(unix)]
#[test]
fn discovery_deduplicates_parse_errors_from_symlink_and_real_roots() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("symlink_and_real_roots_parse_errors");
    let real = root.join("real");
    let link = root.join("link");
    write_file(&real.join("broken.dag"), "module sample.broken\nfn");
    symlink(&real, &link).expect("failed to create root symlink");

    let err = ModuleGraph::discover(&[real.clone(), link])
        .expect_err("expected parse error for symlink+real roots");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(
                files.len(),
                1,
                "real+symlink roots should not duplicate parse error file entries"
            );
            assert_eq!(
                files[0].0.file_name().and_then(|name| name.to_str()),
                Some("broken.dag")
            );
        }
        other => panic!("expected ParseErrors, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[cfg(unix)]
#[test]
fn discovery_derives_module_path_for_symlink_root_without_module_decl() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("symlink_root_module_path_fallback");
    let real = root.join("real");
    let link = root.join("link");
    write_file(&real.join("nested/no_module.dag"), "fn ok() -> Unit {}");
    symlink(&real, &link).expect("failed to create root symlink");

    let graph = ModuleGraph::discover(&[link]).expect("discover should succeed");
    assert_eq!(graph.modules.len(), 1);
    assert_eq!(
        graph.modules[0].module_path,
        vec!["nested".to_string(), "no_module".to_string()]
    );

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[cfg(unix)]
#[test]
fn discovery_reports_io_error_for_dangling_dag_symlink() {
    use std::io::ErrorKind;
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("dangling_dag_symlink");
    fs::create_dir_all(&root).expect("failed to create temp directory");
    let dangling_target = root.join("missing.dag");
    let dangling_link = root.join("broken.dag");
    symlink(&dangling_target, &dangling_link).expect("failed to create dangling symlink");

    let err =
        ModuleGraph::discover(&[root.clone()]).expect_err("dangling symlink should fail discover");
    match err {
        ResolveError::IoError(path, io_error) => {
            assert_eq!(
                path.file_name().and_then(|name| name.to_str()),
                Some("broken.dag")
            );
            assert_eq!(io_error.kind(), ErrorKind::NotFound);
        }
        other => panic!("expected IoError, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}
