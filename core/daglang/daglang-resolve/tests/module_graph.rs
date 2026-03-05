// Test infrastructure: filesystem access for test fixtures
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use daglang_resolve::{ModuleGraph, ResolveError};
use daglang_syntax::diagnostic::DiagnosticKind;
use std::collections::BTreeMap;
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

#[test]
fn discovers_all_real_dsl_modules() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
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
fn real_corpus_module_order_is_stable_across_discovery_runs() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
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
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
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
fn real_corpus_dependency_counts_match_expected_snapshot() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let graph = ModuleGraph::discover(&[dsl_root]).expect("expected real dsl graph to parse");

    let actual: BTreeMap<String, usize> = graph
        .modules
        .iter()
        .map(|module| (module.module_path.as_dotted(), module.dependencies.len()))
        .collect();
    let expected: BTreeMap<String, usize> = BTreeMap::from([
        ("cloud.aws.credential".into(), 3),
        ("cloud.azure.credential".into(), 3),
        ("cloud.gcp.credential".into(), 6),
        ("config.arch_rules".into(), 1),
        ("config.build_commands".into(), 0),
        ("config.build_policy".into(), 0),
        ("config.build_targets".into(), 1),
        ("config.build_workflows".into(), 0),
        ("config.ci".into(), 2),
        ("config.clippy_disallowed".into(), 1),
        ("config.clippy_policy".into(), 3),
        ("config.codegen_paths".into(), 0),
        ("config.gitignore".into(), 0),
        ("config.gitignore_categories".into(), 0),
        ("config.resources".into(), 0),
        ("config.test_policy".into(), 1),
        ("config.tool_registry".into(), 0),
        ("config.workflow_catalog".into(), 0),
        ("config.workflow_commands".into(), 0),
        ("config.workspace".into(), 0),
        ("examples.abstract_services".into(), 1),
        ("examples.deployment".into(), 9),
        ("examples.integration_tests".into(), 2),
        ("examples.rich_types".into(), 2),
        ("extdeps.api.gcp_ops".into(), 1),
        ("extdeps.api.github_ops".into(), 1),
        ("extdeps.cargo".into(), 1),
        ("extdeps.clippy".into(), 0),
        ("extdeps.cloud.aws.core".into(), 1),
        ("extdeps.cloud.aws.iam".into(), 2),
        ("extdeps.cloud.aws.lambda".into(), 2),
        ("extdeps.cloud.aws.s3".into(), 3),
        ("extdeps.cloud.aws.secrets_manager".into(), 2),
        ("extdeps.cloud.aws.sqs".into(), 2),
        ("extdeps.cloud.azure.blob_storage".into(), 2),
        ("extdeps.cloud.azure.container_apps".into(), 2),
        ("extdeps.cloud.azure.core".into(), 2),
        ("extdeps.cloud.azure.identity".into(), 2),
        ("extdeps.cloud.azure.key_vault".into(), 2),
        ("extdeps.cloud.azure.service_bus".into(), 2),
        ("extdeps.cloud.core".into(), 0),
        ("extdeps.cloud.gcp.cloud_run".into(), 2),
        ("extdeps.cloud.gcp.core".into(), 3),
        ("extdeps.cloud.gcp.iam".into(), 3),
        ("extdeps.cloud.gcp.pubsub".into(), 3),
        ("extdeps.cloud.gcp.secret_manager".into(), 3),
        ("extdeps.cloud.gcp.storage".into(), 2),
        ("extdeps.cloud.gcp.sts".into(), 4),
        ("extdeps.coordination.core".into(), 1),
        ("extdeps.coordination.gcs".into(), 3),
        ("extdeps.coordination.postgres".into(), 2),
        ("extdeps.coordination.sqlite".into(), 1),
        ("extdeps.devenv.devcontainers".into(), 0),
        ("extdeps.git".into(), 2),
        ("extdeps.github.core".into(), 1),
        ("extdeps.github.gists".into(), 4),
        ("extdeps.github.issues".into(), 4),
        ("extdeps.github.pull_requests".into(), 4),
        ("extdeps.github_actions".into(), 0),
        ("extdeps.gitignore".into(), 0),
        ("extdeps.gitlab_ci".into(), 0),
        ("extdeps.gunbc".into(), 0),
        ("extdeps.llm.anthropic".into(), 3),
        ("extdeps.llm.core".into(), 0),
        ("extdeps.llm.openai".into(), 3),
        ("extdeps.llm.pricing".into(), 1),
        ("extdeps.make".into(), 0),
        ("extdeps.secrets.core".into(), 2),
        ("extdeps.secrets.env_file".into(), 1),
        ("extdeps.secrets.gcp_secret_manager".into(), 3),
        ("extdeps.secrets.github_secrets".into(), 1),
        ("extdeps.secrets.vault".into(), 1),
        ("extdeps.shell".into(), 1),
        ("extdeps.tools.gh_cli".into(), 1),
        ("extdeps.tools.package_managers".into(), 1),
        ("extdeps.tools.rust_toolchain".into(), 2),
        ("extdeps.yaml".into(), 2),
        ("funcs.approval_gate".into(), 5),
        ("funcs.retry_budget".into(), 2),
        ("funcs.review_pipeline".into(), 4),
        ("funcs.sdlc_dispatch_runtime".into(), 0),
        ("funcs.sdlc_stages".into(), 10),
        ("funcs.sdlc_validation_runtime".into(), 0),
        ("funcs.sdlc_worker".into(), 7),
        ("funcs.test_control_flow".into(), 1),
        ("infra.aws.config".into(), 1),
        ("infra.aws.resources".into(), 4),
        ("infra.aws.services".into(), 0),
        ("infra.azure.config".into(), 1),
        ("infra.azure.resources".into(), 4),
        ("infra.azure.services".into(), 0),
        ("infra.core".into(), 1),
        ("infra.gcp.config".into(), 1),
        ("infra.gcp.resources".into(), 4),
        ("infra.gcp.services".into(), 0),
        ("infra.sdlc.deploy".into(), 2),
        ("infra.spec".into(), 1),
        ("interfaces.agent_provider".into(), 4),
        ("interfaces.artifact_store".into(), 4),
        ("interfaces.claim_store".into(), 4),
        ("interfaces.credential_provider".into(), 4),
        ("interfaces.issue_provider".into(), 4),
        ("interfaces.outcome_ledger".into(), 4),
        ("interfaces.signal_store".into(), 4),
        ("pipelines.ci".into(), 9),
        ("pipelines.cloud_e2e".into(), 9),
        ("pipelines.reconciler".into(), 6),
        ("pipelines.scale_test".into(), 6),
        ("pipelines.sdlc".into(), 13),
        ("pipelines.sdlc_ci".into(), 10),
        ("profiles.cloud_run".into(), 8),
        ("profiles.gist".into(), 3),
        ("profiles.local".into(), 8),
        ("profiles.sdlc".into(), 7),
        ("profiles.unit_test".into(), 7),
        ("services.review.dimension".into(), 1),
        ("services.sdlc.providers.codex_agent_provider".into(), 2),
        ("services.sdlc.providers.file_claim_store".into(), 3),
        ("services.sdlc.providers.file_outcome_ledger".into(), 3),
        ("services.sdlc.providers.file_signal_store".into(), 2),
        ("services.sdlc.providers.gcp_credential_provider".into(), 9),
        ("services.sdlc.providers.gcs_artifact_store".into(), 3),
        ("services.sdlc.providers.gcs_claim_store".into(), 3),
        ("services.sdlc.providers.gcs_outcome_ledger".into(), 3),
        ("services.sdlc.providers.github_issue_provider".into(), 3),
        ("services.sdlc.providers.health_check".into(), 1),
        ("services.sdlc.providers.inline_artifact_store".into(), 2),
        ("services.sdlc.providers.llm_agent_provider".into(), 7),
        (
            "services.sdlc.providers.local_credential_provider".into(),
            4,
        ),
        ("services.sdlc.providers.pubsub_signal_store".into(), 3),
        ("services.sdlc.providers.rolling_deploy".into(), 2),
        ("services.sdlc.providers.structured_logging".into(), 1),
        ("services.sdlc.providers.stub_credential_provider".into(), 2),
        ("services.sdlc.providers.stub_providers".into(), 7),
        ("shared.codegen".into(), 0),
        ("shared.compilation".into(), 0),
        ("shared.dag_util".into(), 1),
        ("shared.gist_modes".into(), 5),
        ("std.access".into(), 0),
        ("std.behavioral".into(), 0),
        ("std.box_draw".into(), 3),
        ("std.capability".into(), 0),
        ("std.coordination".into(), 0),
        ("std.errors".into(), 0),
        ("std.fermi".into(), 1),
        ("std.fidelity".into(), 2),
        ("std.filesystem".into(), 1),
        ("std.languages".into(), 1),
        ("std.lint".into(), 0),
        ("std.markdown".into(), 0),
        ("std.markdown_render".into(), 1),
        ("std.patterns".into(), 7),
        ("std.provider_config".into(), 1),
        ("std.rate_limit".into(), 1),
        ("std.render".into(), 4),
        ("std.resources".into(), 1),
        ("std.state_machines".into(), 2),
        ("std.symbols".into(), 0),
        ("std.types".into(), 0),
        ("std.unicode".into(), 1),
        ("std.virtual_io".into(), 1),
        ("std.width".into(), 2),
        ("tests.cas_stress_test".into(), 2),
        ("tools.bootstrap".into(), 3),
        ("tools.build".into(), 3),
        ("tools.cigen".into(), 4),
        ("tools.clippy".into(), 5),
        ("tools.codegen".into(), 2),
        ("tools.deps".into(), 4),
        ("tools.deps_config".into(), 1),
        ("tools.design".into(), 2),
        ("tools.docgen".into(), 4),
        ("tools.gist".into(), 7),
        ("tools.infra".into(), 0),
        ("tools.justgen".into(), 3),
        ("tools.makegen".into(), 3),
        ("tools.pragma".into(), 2),
        ("tools.review".into(), 0),
        ("tools.testgen".into(), 2),
        ("tools.workflow".into(), 1),
        ("workflows.bootstrap".into(), 2),
        ("workflows.build_all".into(), 3),
        ("workflows.ci".into(), 0),
        ("workflows.deps".into(), 2),
        ("workflows.gist".into(), 2),
        ("workflows.makegen".into(), 5),
        ("workflows.pragma".into(), 2),
        ("workflows.sdlc".into(), 5),
        ("workflows.test_all".into(), 0),
        ("workflows.webhook".into(), 4),
    ]);

    assert_eq!(actual, expected);
}

#[test]
fn real_corpus_acyclic_dependencies_precede_dependents() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
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
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
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
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
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
fn unresolved_imports_are_tolerated_for_phase_zero_discovery() {
    let root = unique_temp_dir("unresolved");
    write_file(
        &root.join("a/main.dag"),
        "module a.main\nimport missing.dep\nfn run() -> Unit {}",
    );

    let graph = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect("expected graph discovery success");
    assert_eq!(graph.modules.len(), 1);
    assert_eq!(graph.modules[0].module_path.as_dotted(), "a.main");
    assert!(
        graph.modules[0].dependencies.is_empty(),
        "unresolved imports should be ignored in phase-0 graph construction"
    );

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
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
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
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
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
