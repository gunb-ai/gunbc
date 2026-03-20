// Test infrastructure: filesystem access for test fixtures
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use daglang_resolve::unused_imports::{
    build_module_export_index, find_unused_imports_with_export_index, UnusedImport,
};
use daglang_resolve::{ModuleGraph, ResolveError};
use daglang_syntax::ast::{Expr, Item, Literal, SourceFile};
use daglang_syntax::diagnostic::DiagnosticKind;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use gunbc_test::unique_temp_dir;

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent directories");
    }
    fs::write(path, content).expect("failed to write test file");
}

fn expected_dsl_modules_sorted(dsl_root: &Path) -> Vec<String> {
    let mut modules = Vec::new();
    collect_dsl_modules(dsl_root, dsl_root, &mut modules);
    modules.sort();
    modules
}

fn collect_dsl_modules(dsl_root: &Path, dir: &Path, modules: &mut Vec<String>) {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read dsl directory {}: {e}", dir.display()))
        .map(|entry| {
            entry.unwrap_or_else(|e| panic!("failed to read dsl entry in {}: {e}", dir.display()))
        })
        .collect();
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_dsl_modules(dsl_root, &path, modules);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("dag") {
            continue;
        }
        modules.push(parse_module_declaration(dsl_root, &path));
    }
}

fn parse_module_declaration(dsl_root: &Path, path: &Path) -> String {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read dsl source {}: {e}", path.display()));
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("module ") {
            if let Some(module_id) = rest.split_whitespace().next() {
                return module_id.to_string();
            }
        }
    }
    panic!(
        "missing module declaration in dsl source {} (under {})",
        path.display(),
        dsl_root.display()
    );
}

fn find_data_def<'a>(ast: &'a SourceFile, name: &str) -> &'a daglang_syntax::ast::DataDef {
    ast.items
        .iter()
        .find_map(|item| match &item.node {
            Item::DataDef(data_def) if data_def.name == name => Some(data_def),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing data definition `{name}`"))
}

fn string_list_data(ast: &SourceFile, name: &str) -> Vec<String> {
    let data_def = find_data_def(ast, name);
    let Expr::List(entries) = &data_def.value else {
        panic!("data definition `{name}` must stay a list literal");
    };
    entries
        .iter()
        .map(|entry| match entry {
            Expr::Literal(Literal::String(value)) => value.clone(),
            _ => panic!("data definition `{name}` must contain only string literals"),
        })
        .collect()
}

#[test]
fn discovers_all_real_dsl_modules() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl");
    let graph = ModuleGraph::discover(std::slice::from_ref(&dsl_root))
        .expect("expected real dsl graph to parse");

    let expected = expected_dsl_modules_sorted(&dsl_root);
    assert_eq!(
        graph.modules.len(),
        expected.len(),
        "resolved module count should match dsl/**/*.dag filesystem discovery"
    );
    let mut module_names: Vec<String> = graph
        .modules
        .iter()
        .map(|module| module.module_path.as_dotted())
        .collect();
    module_names.sort();
    assert_eq!(module_names, expected);
}

#[test]
fn real_corpus_includes_contractual_extdeps_language_modules() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl");
    let graph = ModuleGraph::discover(std::slice::from_ref(&dsl_root))
        .expect("expected real dsl graph to parse");
    let discovered_modules: HashSet<String> = graph
        .modules
        .iter()
        .map(|module| module.module_path.as_dotted())
        .collect();
    let std_languages = graph
        .modules
        .iter()
        .find(|module| module.module_path.as_dotted() == "std.languages")
        .unwrap_or_else(|| panic!("missing `std.languages` module in real corpus"));
    let required_modules = string_list_data(
        &std_languages.ast,
        "contractual_extdeps_language_modules",
    );

    let missing: Vec<_> = required_modules
        .iter()
        .filter(|module| !discovered_modules.contains(*module))
        .cloned()
        .collect();
    assert!(
        missing.is_empty(),
        "real dsl corpus must retain contractual extdeps language modules: missing {missing:?}"
    );
}

#[test]
fn real_corpus_contractual_extdeps_language_modules_resolve_via_public_bindings() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl");
    let dsl_graph = ModuleGraph::discover(std::slice::from_ref(&dsl_root))
        .expect("expected real dsl graph to parse");
    let std_languages = dsl_graph
        .modules
        .iter()
        .find(|module| module.module_path.as_dotted() == "std.languages")
        .unwrap_or_else(|| panic!("missing `std.languages` module in real corpus"));
    let required_modules = string_list_data(
        &std_languages.ast,
        "contractual_extdeps_language_modules",
    );
    let witnesses = [
        ("extdeps.languages.go.runtime", "format_func", "String"),
        ("extdeps.languages.go.types", "visibility_by_case", "Bool"),
        (
            "extdeps.languages.python.types",
            "type_checker_strict",
            "String",
        ),
        ("extdeps.languages.rust.runtime", "string_literal_suffix", "String"),
        ("extdeps.languages.rust.types", "pass_copy_by_value", "Bool"),
    ];

    let witness_modules: HashSet<String> = witnesses
        .iter()
        .map(|(module, _, _)| (*module).to_string())
        .collect();
    let required_module_set: HashSet<String> = required_modules.iter().cloned().collect();
    let missing_witnesses: Vec<_> = required_modules
        .iter()
        .filter(|module| !witness_modules.contains(*module))
        .cloned()
        .collect();
    let unexpected_witnesses: Vec<_> = witness_modules
        .difference(&required_module_set)
        .cloned()
        .collect();
    assert!(
        missing_witnesses.is_empty() && unexpected_witnesses.is_empty(),
        "public-binding witnesses must stay aligned with std.languages contractual module authority: \
         missing witnesses {missing_witnesses:?}, unexpected witnesses {unexpected_witnesses:?}"
    );

    let root = unique_temp_dir("contractual_language_module_surface");
    let importer_path = root.join("sample/language_contracts.dag");
    let import_lines = witnesses
        .iter()
        .map(|(module, binding, _)| format!("import {module} {{ {binding} }}"))
        .collect::<Vec<_>>()
        .join("\n");
    let output_fields = witnesses
        .iter()
        .map(|(_, binding, ty)| format!("{binding}: {ty}"))
        .collect::<Vec<_>>()
        .join(", ");
    let return_fields = witnesses
        .iter()
        .map(|(_, binding, _)| format!("{binding}: {binding}"))
        .collect::<Vec<_>>()
        .join(", ");

    write_file(
        &importer_path,
        &format!(
            "module sample.language_contracts\n\n{import_lines}\n\nfunc proof() -> {{ {output_fields} }} {{\n  return {{ {return_fields} }}\n}}\n"
        ),
    );

    let graph = ModuleGraph::discover(&[root.clone(), dsl_root.clone()]).expect(
        "real contractual extdeps language modules should resolve when imported by a consumer",
    );
    let export_index = build_module_export_index(&graph.modules);

    for (module, binding, _) in witnesses {
        let exports = export_index
            .get(module)
            .unwrap_or_else(|| panic!("missing export index entry for `{module}`"));
        assert!(
            exports.contains(binding),
            "contractual language module `{module}` must export witness binding `{binding}`"
        );
    }

    let importer = graph
        .modules
        .iter()
        .find(|module| module.module_path.as_dotted() == "sample.language_contracts")
        .expect("expected importer module to be present");
    let unused = find_unused_imports_with_export_index(&importer.ast, &export_index);
    assert!(
        unused.is_empty(),
        "contractual language module imports should stay usable through their exported bindings, got: {unused:?}"
    );

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn real_corpus_module_order_is_stable_across_discovery_runs() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl");
    let first =
        ModuleGraph::discover(std::slice::from_ref(&dsl_root)).expect("expected real dsl graph");
    let second = ModuleGraph::discover(&[dsl_root]).expect("expected real dsl graph");
    let first_order: Vec<String> = first
        .modules
        .iter()
        .map(|module| module.module_path.as_dotted())
        .collect();
    let second_order: Vec<String> = second
        .modules
        .iter()
        .map(|module| module.module_path.as_dotted())
        .collect();
    assert_eq!(first_order, second_order);
}

#[test]
fn discovered_module_paths_match_ast_module_declarations() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl");
    let graph = ModuleGraph::discover(&[dsl_root]).expect("expected real dsl graph to parse");

    for module in &graph.modules {
        let ast_module_path = module
            .ast
            .module_path
            .as_ref()
            .map(|module| module.node.clone())
            .expect("real corpus files should contain module declarations");
        assert_eq!(
            module.module_path,
            ast_module_path,
            "resolved module path should match parsed AST module declaration for {}",
            module.path.display()
        );
    }
}

#[test]
fn real_corpus_acyclic_dependencies_precede_dependents() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl");
    let graph = ModuleGraph::discover(&[dsl_root]).expect("expected real dsl graph to parse");
    for (module_idx, module) in graph.modules.iter().enumerate() {
        let module_name = module.module_path.as_dotted();
        for dep_idx in &module.dependencies {
            let dep_name = graph.modules[*dep_idx].module_path.as_dotted();

            assert!(
                *dep_idx < module_idx,
                "dependency should precede dependent for acyclic modules: {dep_name} (index {dep_idx}) should be before {module_name} (index {module_idx})"
            );
        }
    }
}

#[test]
fn real_corpus_dependency_indices_are_within_bounds() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl");
    let graph = ModuleGraph::discover(&[dsl_root]).expect("expected real dsl graph to parse");

    for module in &graph.modules {
        for dep_idx in &module.dependencies {
            assert!(
                *dep_idx < graph.modules.len(),
                "dependency index {} is out of bounds for module {}",
                dep_idx,
                module.module_path.as_dotted()
            );
        }
    }
}

#[test]
fn real_corpus_dependency_indices_match_declared_imports() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl");
    let graph = ModuleGraph::discover(&[dsl_root]).expect("expected real dsl graph to parse");
    use daglang_syntax::ast::ModulePath;
    let module_index: HashMap<ModulePath, usize> = graph
        .modules
        .iter()
        .enumerate()
        .map(|(idx, module)| (module.module_path.clone(), idx))
        .collect();

    for module in &graph.modules {
        let declared_imports: HashSet<ModulePath> = module
            .ast
            .imports
            .iter()
            .map(|import| import.node.path.clone())
            .collect();

        let resolved_dependencies: HashSet<ModulePath> = module
            .dependencies
            .iter()
            .map(|dep_idx| graph.modules[*dep_idx].module_path.clone())
            .collect();

        for dep_path in &resolved_dependencies {
            assert!(
                declared_imports.contains(dep_path),
                "resolved dependency {} was not declared by module {}",
                dep_path.as_dotted(),
                module.module_path.as_dotted()
            );
        }

        for import in &declared_imports {
            if module_index.contains_key(import) {
                assert!(
                    resolved_dependencies.contains(import),
                    "declared import {} should resolve as dependency for module {}",
                    import.as_dotted(),
                    module.module_path.as_dotted()
                );
            }
        }
    }
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

    let err = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect_err("expected duplicate module error");
    match err {
        ResolveError::DuplicateModule(path) => {
            assert_eq!(path.as_dotted(), "dup.mod");
        }
        other => panic!("expected DuplicateModule, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn missing_discovery_root_is_rejected() {
    let root = unique_temp_dir("missing_root");
    let err = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect_err("expected invalid root error");
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

    let err = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect_err("expected invalid root error");
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
    let missing_err = ModuleGraph::discover(std::slice::from_ref(&missing_root))
        .expect_err("expected invalid root error");
    let missing_rendered = missing_err.to_string();
    assert!(missing_rendered.contains("invalid discovery root"));
    assert!(missing_rendered.contains(&missing_root.display().to_string()));
    assert!(missing_rendered.contains("does not exist"));

    let non_dir_root = unique_temp_dir("display_non_dir_root");
    write_file(&non_dir_root, "not a directory");
    let non_dir_err = ModuleGraph::discover(std::slice::from_ref(&non_dir_root))
        .expect_err("expected invalid root error");
    let non_dir_rendered = non_dir_err.to_string();
    assert!(non_dir_rendered.contains("invalid discovery root"));
    assert!(non_dir_rendered.contains(&non_dir_root.display().to_string()));
    assert!(non_dir_rendered.contains("is not a directory"));

    fs::remove_file(non_dir_root).expect("failed to clean temp file");
}

#[test]
fn unresolved_imports_fail_closed_during_discovery() {
    let root = unique_temp_dir("unresolved");
    write_file(
        &root.join("a/main.dag"),
        "module a.main\nimport missing.dep\nfn run() -> Unit {}",
    );

    let err = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect_err("expected unresolved import discovery failure");
    assert!(
        matches!(err, ResolveError::UnresolvedImport { .. }),
        "expected unresolved import error, got {err:?}"
    );
    let rendered = err.to_string();
    assert!(rendered.contains("missing.dep"));
    assert!(rendered.contains("a.main"));

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn cyclic_dependencies_are_reported_as_resolve_errors() {
    let root = unique_temp_dir("cycle");
    write_file(
        &root.join("a/a.dag"),
        "module cycle.a\nimport cycle.b\nfn a() -> Unit {}",
    );
    write_file(
        &root.join("b/b.dag"),
        "module cycle.b\nimport cycle.a\nfn b() -> Unit {}",
    );

    let err = ModuleGraph::discover_strict(std::slice::from_ref(&root))
        .expect_err("expected cycle error");
    match err {
        ResolveError::CyclicDependency(cycle) => {
            let module_names = cycle
                .iter()
                .map(|module| module.as_dotted())
                .collect::<Vec<_>>();
            assert!(module_names.iter().any(|name| name == "cycle.a"));
            assert!(module_names.iter().any(|name| name == "cycle.b"));
        }
        other => panic!("expected CyclicDependency, got {other:?}"),
    }

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn parse_error_contains_file_line_col_rendering() {
    let root = unique_temp_dir("parse_error_location");
    write_file(&root.join("broken.dag"), "module sample.broken\nfn");

    let err = ModuleGraph::discover(std::slice::from_ref(&root)).expect_err("expected parse error");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(files.len(), 1);
            assert_eq!(
                files[0].0.file_name().and_then(|n| n.to_str()),
                Some("broken.dag")
            );
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

    let err = ModuleGraph::discover(std::slice::from_ref(&root)).expect_err("expected lex error");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(files.len(), 1);
            assert_eq!(
                files[0].0.file_name().and_then(|n| n.to_str()),
                Some("broken.dag")
            );
            let diag = files[0].1.first().expect("expected a lexical diagnostic");
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

    let err = ModuleGraph::discover(std::slice::from_ref(&root)).expect_err("expected lex errors");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(
                files.len(),
                2,
                "expected diagnostics for both lex-broken files"
            );
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
                    diagnostics
                        .iter()
                        .all(|diag| diag.kind == DiagnosticKind::Lex),
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

    let err = ModuleGraph::discover(std::slice::from_ref(&root)).expect_err("expected lex errors");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(files.len(), 1);
            let diagnostics = &files[0].1;
            assert_eq!(
                diagnostics.len(),
                2,
                "expected both lexical diagnostics for one file"
            );
            assert!(diagnostics
                .iter()
                .all(|diag| diag.kind == DiagnosticKind::Lex));
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

    let err =
        ModuleGraph::discover(std::slice::from_ref(&root)).expect_err("expected parse errors");
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

    let err =
        ModuleGraph::discover(std::slice::from_ref(&root)).expect_err("expected parse errors");
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

    let err =
        ModuleGraph::discover(std::slice::from_ref(&root)).expect_err("expected mixed errors");
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
        (left, right) => {
            panic!("expected ParseErrors for both runs, got left={left:?}, right={right:?}")
        }
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

    let err =
        ModuleGraph::discover(std::slice::from_ref(&root)).expect_err("expected parse errors");
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
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl");
    let graph_a = ModuleGraph::discover(std::slice::from_ref(&dsl_root))
        .expect("first discover should succeed");
    let graph_b = ModuleGraph::discover(&[dsl_root]).expect("second discover should succeed");

    let order_a: Vec<String> = graph_a
        .modules
        .iter()
        .map(|module| module.module_path.as_dotted())
        .collect();
    let order_b: Vec<String> = graph_b
        .modules
        .iter()
        .map(|module| module.module_path.as_dotted())
        .collect();

    assert_eq!(order_a, order_b, "module discovery order should be stable");
}

#[test]
fn display_tree_output_is_deterministic_across_runs() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl");
    let graph_a = ModuleGraph::discover(std::slice::from_ref(&dsl_root))
        .expect("first discover should succeed");
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
        .map(|module| module.module_path.as_dotted())
        .collect();
    let order_ba: Vec<String> = graph_ba
        .modules
        .iter()
        .map(|module| module.module_path.as_dotted())
        .collect();
    assert_eq!(order_ab, order_ba);
    assert_eq!(graph_ab.display_tree(), graph_ba.display_tree());

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn display_tree_contains_expected_summary_fields() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl");
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

    let graph =
        ModuleGraph::discover(std::slice::from_ref(&root)).expect("discover should ignore non-dag");
    assert_eq!(graph.modules.len(), 1);
    assert_eq!(graph.modules[0].module_path.as_dotted(), "sample.main");

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
    assert_eq!(graph.modules[0].module_path.as_dotted(), "sample.main");

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
    assert_eq!(graph.modules[0].module_path.as_dotted(), "sample.main");

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn discovery_deduplicates_files_from_equivalent_curdir_suffix_roots() {
    let root = unique_temp_dir("equivalent_curdir_suffix_roots");
    write_file(
        &root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    );

    let graph = ModuleGraph::discover(&[root.clone(), root.join(".")])
        .expect("discover should dedupe equivalent root entries");
    assert_eq!(graph.modules.len(), 1);
    assert_eq!(graph.modules[0].module_path.as_dotted(), "sample.main");

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

#[test]
fn discovery_deduplicates_parse_errors_from_equivalent_curdir_suffix_roots() {
    let root = unique_temp_dir("equivalent_curdir_suffix_roots_parse_errors");
    write_file(&root.join("broken.dag"), "module sample.broken\nfn");

    let err = ModuleGraph::discover(&[root.clone(), root.join(".")])
        .expect_err("expected parse error for equivalent root entries");
    match err {
        ResolveError::ParseErrors(files) => {
            assert_eq!(
                files.len(),
                1,
                "equivalent roots should not duplicate parse error file entries"
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

#[test]
fn discovery_is_independent_of_equivalent_curdir_suffix_root_order() {
    let root = unique_temp_dir("equivalent_curdir_suffix_root_order");
    write_file(
        &root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    );

    let first = ModuleGraph::discover(&[root.clone(), root.join(".")]).expect("first discover");
    let second = ModuleGraph::discover(&[root.join("."), root.clone()]).expect("second discover");

    let first_paths: Vec<String> = first
        .modules
        .iter()
        .map(|module| module.module_path.as_dotted())
        .collect();
    let second_paths: Vec<String> = second
        .modules
        .iter()
        .map(|module| module.module_path.as_dotted())
        .collect();
    assert_eq!(first_paths, second_paths);
    assert_eq!(first.display_tree(), second.display_tree());

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn discovery_parse_errors_are_independent_of_equivalent_curdir_suffix_root_order() {
    let root = unique_temp_dir("equivalent_curdir_suffix_root_order_parse_errors");
    write_file(&root.join("broken.dag"), "module sample.broken\nfn");

    let first = ModuleGraph::discover(&[root.clone(), root.join(".")])
        .expect_err("first discover should return parse errors");
    let second = ModuleGraph::discover(&[root.join("."), root.clone()])
        .expect_err("second discover should return parse errors");

    match (first, second) {
        (ResolveError::ParseErrors(first_files), ResolveError::ParseErrors(second_files)) => {
            assert_eq!(first_files, second_files);
        }
        (left, right) => panic!("expected parse errors for both runs, got {left:?} and {right:?}"),
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
    assert_eq!(graph.modules[0].module_path.as_dotted(), "sample.main");

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
        .map(|module| module.module_path.as_dotted())
        .collect();
    let second_paths: Vec<String> = second
        .modules
        .iter()
        .map(|module| module.module_path.as_dotted())
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
        (left, right) => {
            panic!("expected ParseErrors for both runs, got left={left:?}, right={right:?}")
        }
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
        .map(|module| module.module_path.as_dotted())
        .collect();
    let second_paths: Vec<String> = second
        .modules
        .iter()
        .map(|module| module.module_path.as_dotted())
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
        .map(|module| module.module_path.as_dotted())
        .collect();
    let second_paths: Vec<String> = second
        .modules
        .iter()
        .map(|module| module.module_path.as_dotted())
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

    let graph = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect("discover should handle directory cycle symlink");
    assert_eq!(graph.modules.len(), 1);
    assert_eq!(graph.modules[0].module_path.as_dotted(), "sample.main");

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

    let err = ModuleGraph::discover(std::slice::from_ref(&root))
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

    let first = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect_err("first discover should return parse errors");
    let second = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect_err("second discover should return parse errors");

    match (first, second) {
        (ResolveError::ParseErrors(first_files), ResolveError::ParseErrors(second_files)) => {
            assert_eq!(first_files, second_files);
        }
        (left, right) => {
            panic!("expected ParseErrors for both runs, got left={left:?}, right={right:?}")
        }
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
        daglang_syntax::ast::ModulePath::new(vec!["nested".to_string(), "no_module".to_string()])
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

    let err = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect_err("dangling symlink should fail discover");
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

#[test]
fn dependency_counts_match_import_structure() {
    let root = unique_temp_dir("dep_counts");

    // base: no imports → 0 deps
    write_file(
        &root.join("base/base.dag"),
        "module dep.base\nfn base_fn() -> Unit {}",
    );

    // mid: imports base → 1 dep
    write_file(
        &root.join("mid/mid.dag"),
        "module dep.mid\nimport dep.base\nfn mid_fn() -> Unit {}",
    );

    // top: imports base + mid → 2 deps
    write_file(
        &root.join("top/top.dag"),
        "module dep.top\nimport dep.base\nimport dep.mid\nfn top_fn() -> Unit {}",
    );

    // leaf: imports only mid → 1 dep
    write_file(
        &root.join("leaf/leaf.dag"),
        "module dep.leaf\nimport dep.mid\nfn leaf_fn() -> Unit {}",
    );

    let graph = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect("expected synthetic graph to parse");

    assert_eq!(graph.modules.len(), 4);

    let counts: HashMap<String, usize> = graph
        .modules
        .iter()
        .map(|m| (m.module_path.as_dotted(), m.dependencies.len()))
        .collect();

    assert_eq!(counts["dep.base"], 0, "base has no imports");
    assert_eq!(counts["dep.mid"], 1, "mid imports base");
    assert_eq!(counts["dep.top"], 2, "top imports base + mid");
    assert_eq!(counts["dep.leaf"], 1, "leaf imports mid");

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn removed_top_level_data_export_disappears_from_module_export_index() {
    let root = unique_temp_dir("removed_top_level_data_export");
    let module_path = root.join("sample/contracts.dag");

    write_file(
        &module_path,
        r#"
        module sample.contracts

        data stable_surface: String = "stable"
        data legacy_surface: String = "legacy"
        "#,
    );

    let before_graph = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect("expected synthetic graph with both data exports to parse");
    let before_export_index = build_module_export_index(&before_graph.modules);
    let before_exports = before_export_index
        .get("sample.contracts")
        .expect("provider module should exist in export index before removal");
    assert!(before_exports.contains("stable_surface"));
    assert!(before_exports.contains("legacy_surface"));

    write_file(
        &module_path,
        r#"
        module sample.contracts

        data stable_surface: String = "stable"
        "#,
    );

    let after_graph = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect("expected synthetic graph after removing data export to parse");
    let after_export_index = build_module_export_index(&after_graph.modules);
    let after_exports = after_export_index
        .get("sample.contracts")
        .expect("provider module should still exist in export index after removal");
    assert!(after_exports.contains("stable_surface"));
    assert!(
        !after_exports.contains("legacy_surface"),
        "removed top-level data exports must disappear from build_module_export_index"
    );

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn module_import_used_via_exported_service_namespaces_is_not_reported() {
    let root = unique_temp_dir("unused_import_exported_service_namespaces");

    write_file(
        &root.join("extdeps/cloud/gcp/sts.dag"),
        r#"
        module extdeps.cloud.gcp.sts

        service gcp.STS {
            operation Exchange {
                input {}
                output {}
                transport shell { argv: ["echo", "sts"] }
            }
        }

        service github.OIDC {
            operation GetToken {
                input {}
                output {}
                transport shell { argv: ["echo", "oidc"] }
            }
        }
        "#,
    );

    write_file(
        &root.join("gunbc/auth/patterns.dag"),
        r#"
        module gunbc.auth.patterns

        import extdeps.cloud.gcp.sts

        func credential_chain() -> { ok: Bool } {
            sts = gcp.STS.Exchange()
            oidc = github.OIDC.GetToken()
            return { ok: true }
        }
        "#,
    );

    let graph = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect("expected synthetic graph to parse");
    let export_index = build_module_export_index(&graph.modules);
    let module = graph
        .modules
        .iter()
        .find(|module| module.module_path.as_dotted() == "gunbc.auth.patterns")
        .expect("expected importer module to be present");

    let unused = find_unused_imports_with_export_index(&module.ast, &export_index);
    assert!(
        unused.is_empty(),
        "expected exported service namespaces to mark import as used, got: {unused:?}"
    );

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn module_import_used_via_exported_github_service_namespace_is_not_reported() {
    let root = unique_temp_dir("unused_import_exported_github_namespace");

    write_file(
        &root.join("extdeps/github/gists.dag"),
        r#"
        module extdeps.github.gists

        service github.Gist {
            operation Create {
                input {}
                output {}
                transport rest { method: POST, path: "/gists" }
            }
        }
        "#,
    );

    write_file(
        &root.join("gunbc/tools/gist.dag"),
        r#"
        module gunbc.tools.gist

        import extdeps.github.gists

        func gist() -> { ok: Bool } {
            created = github.Gist.Create()
            return { ok: true }
        }
        "#,
    );

    let graph = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect("expected synthetic graph to parse");
    let export_index = build_module_export_index(&graph.modules);
    let module = graph
        .modules
        .iter()
        .find(|module| module.module_path.as_dotted() == "gunbc.tools.gist")
        .expect("expected importer module to be present");

    let unused = find_unused_imports_with_export_index(&module.ast, &export_index);
    assert!(
        unused.is_empty(),
        "expected exported github service namespace to mark import as used, got: {unused:?}"
    );

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn aliased_module_import_used_via_original_exported_namespace_is_reported_unused() {
    let root = unique_temp_dir("unused_import_aliased_exported_namespace");

    write_file(
        &root.join("extdeps/cloud/gcp/sts.dag"),
        r#"
        module extdeps.cloud.gcp.sts

        service gcp.STS {
            operation Exchange {
                input {}
                output {}
                transport shell { argv: ["echo", "sts"] }
            }
        }
        "#,
    );

    write_file(
        &root.join("gunbc/auth/patterns.dag"),
        r#"
        module gunbc.auth.patterns

        import extdeps.cloud.gcp.sts as provider

        func credential_chain() -> { ok: Bool } {
            sts = gcp.STS.Exchange()
            return { ok: true }
        }
        "#,
    );

    let graph = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect("expected synthetic graph to parse");
    let export_index = build_module_export_index(&graph.modules);
    let module = graph
        .modules
        .iter()
        .find(|module| module.module_path.as_dotted() == "gunbc.auth.patterns")
        .expect("expected importer module to be present");

    let unused = find_unused_imports_with_export_index(&module.ast, &export_index);
    assert_eq!(
        unused,
        vec![UnusedImport {
            module_path: "extdeps.cloud.gcp.sts".into(),
            binding: None,
        }]
    );

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn imported_bindings_do_not_make_downstream_module_imports_used() {
    let root = unique_temp_dir("unused_import_non_reexported_binding");

    write_file(
        &root.join("dep/base.dag"),
        r#"
        module dep.base

        type Summary {
            value: String
        }
        "#,
    );

    write_file(
        &root.join("dep/mid.dag"),
        r#"
        module dep.mid

        import dep.base { Summary }

        fn mid_fn() -> Unit {}
        "#,
    );

    write_file(
        &root.join("dep/top.dag"),
        r#"
        module dep.top

        import dep.mid

        fn top(summary: Summary) -> Unit {}
        "#,
    );

    let graph = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect("expected synthetic graph to parse");
    let export_index = build_module_export_index(&graph.modules);
    let module = graph
        .modules
        .iter()
        .find(|module| module.module_path.as_dotted() == "dep.top")
        .expect("expected importer module to be present");

    let unused = find_unused_imports_with_export_index(&module.ast, &export_index);
    assert_eq!(
        unused,
        vec![UnusedImport {
            module_path: "dep.mid".into(),
            binding: None,
        }]
    );

    fs::remove_dir_all(root).expect("failed to clean temp directory");
}

#[test]
fn real_corpus_has_no_unused_imports() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl");
    let graph = ModuleGraph::discover(std::slice::from_ref(&dsl_root))
        .expect("expected real dsl graph to parse");
    let export_index = build_module_export_index(&graph.modules);

    let mut violations = Vec::new();
    for module in &graph.modules {
        let module_path = module.module_path.as_dotted();
        let unused = find_unused_imports_with_export_index(&module.ast, &export_index);
        for u in unused {
            let binding_desc = match &u.binding {
                Some(name) => format!("binding `{name}` from `{}`", u.module_path),
                None => format!("module `{}`", u.module_path),
            };
            violations.push(format!("{module_path}: unused import {binding_desc}"));
        }
    }
    assert!(
        violations.is_empty(),
        "found unused imports in dsl corpus:\n  {}",
        violations.join("\n  ")
    );
}
