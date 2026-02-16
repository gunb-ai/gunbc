use daglang_resolve::ModuleGraph;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn daglang_bin() -> &'static str {
    env!("CARGO_BIN_EXE_daglang")
}

fn unique_temp_file(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("daglang_cli_{name}_{}_{}.dag", std::process::id(), nanos))
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("daglang_cli_{name}_{}_{}", std::process::id(), nanos))
}

fn assert_no_compile_stage_banners(stderr: &str) {
    assert!(
        !stderr.contains("typecheck errors"),
        "unexpected typecheck-stage banner in non-compile command path: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "unexpected lower-stage banner in non-compile command path: {stderr}"
    );
}

fn unique_name(name: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    format!("daglang_cli_{name}_{}_{}", std::process::id(), nanos)
}

fn expected_check_success_stdout(parsed_files: usize) -> String {
    format!("OK: parsed {parsed_files} file(s)\n")
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

fn expected_real_corpus_module_order() -> Vec<&'static str> {
    vec![
        "std.types",
        "std.resources",
        "services.shell",
        "tools.codegen",
        "services.github.gist",
        "services.git",
        "services.gcp.sts",
        "services.gcp.secret_manager",
        "services.gcp.iam",
        "std.patterns",
        "tools.testgen",
        "tools.docgen",
        "shared.dag_util",
        "tools.pragma",
        "tools.makegen",
        "tools.deps",
        "tools.bootstrap",
        "services.cargo",
        "tools.clippy",
        "tools.build",
        "pipelines.ci",
        "infra.gcp.services",
        "infra.core",
        "infra.spec",
        "infra.gcp.resources",
        "infra.gcp.config",
        "infra.azure.services",
        "infra.azure.resources",
        "infra.azure.config",
        "infra.aws.services",
        "infra.aws.resources",
        "infra.aws.config",
        "examples.rich_types",
        "examples.integration_tests",
        "examples.abstract_services",
        "cloud.gcp.credential",
        "cloud.azure.credential",
        "cloud.aws.credential",
        "examples.deployment",
        "shared.gist_modes",
        "tools.dag_viz",
        "tools.gist",
    ]
}

fn reported_modules_in_order(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter_map(|line| line.split_once("  (").map(|(module, _)| module.trim().to_string()))
        .collect()
}

fn reported_modules_sorted(stdout: &str) -> Vec<String> {
    let mut modules = reported_modules_in_order(stdout);
    modules.sort();
    modules
}

fn reported_module_summary(stdout: &str) -> BTreeMap<String, (usize, usize)> {
    stdout
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter_map(|line| line.split_once("  ("))
        .filter_map(|(module, rest)| {
            let counts = rest.split(")  [").next()?;
            let (items_part, deps_part) = counts.split_once(", ")?;
            let item_count = items_part.trim_end_matches(" items").parse::<usize>().ok()?;
            let dep_count = deps_part.trim_end_matches(" deps").parse::<usize>().ok()?;
            Some((module.trim().to_string(), (item_count, dep_count)))
        })
        .collect()
}

fn expected_viz_self_mermaid() -> &'static str {
    concat!(
        "flowchart TB\n",
        "subgraph daglang_compiler_pipeline[\"daglang-compiler-pipeline\"]\n",
        "    daglang_compiler_pipeline_discover_files[discover_files]\n",
        "    daglang_compiler_pipeline_parse_all[parse_all]\n",
        "    daglang_compiler_pipeline_build_module_graph[build_module_graph]\n",
        "    daglang_compiler_pipeline_report_modules[report_modules]\n",
        "    daglang_compiler_pipeline_discover_files -->|files:files| daglang_compiler_pipeline_parse_all\n",
        "    daglang_compiler_pipeline_discover_files -->|diagnostics:diagnostics| daglang_compiler_pipeline_parse_all\n",
        "    daglang_compiler_pipeline_parse_all -->|parsed_modules:parsed_modules| daglang_compiler_pipeline_build_module_graph\n",
        "    daglang_compiler_pipeline_parse_all -->|diagnostics:diagnostics| daglang_compiler_pipeline_build_module_graph\n",
        "    daglang_compiler_pipeline_build_module_graph -->|module_graph:module_graph| daglang_compiler_pipeline_report_modules\n",
        "    daglang_compiler_pipeline_build_module_graph -->|diagnostics:diagnostics| daglang_compiler_pipeline_report_modules\n",
        "end\n\n",
    )
}

fn resolve_discovered_module_order() -> Vec<String> {
    ModuleGraph::discover(&[workspace_root().join("dsl")])
        .expect("resolve discovery should succeed for real corpus")
        .modules
        .into_iter()
        .map(|module| module.module_path.join("."))
        .collect()
}

fn resolve_discovered_module_summary() -> BTreeMap<String, (usize, usize)> {
    ModuleGraph::discover(&[workspace_root().join("dsl")])
        .expect("resolve discovery should succeed for real corpus")
        .modules
        .into_iter()
        .map(|module| {
            (
                module.module_path.join("."),
                (module.ast.items.len(), module.dependencies.len()),
            )
        })
        .collect()
}

fn reported_diagnostics_in_order(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .skip_while(|line| *line != "Diagnostics:")
        .skip(1)
        .take_while(|line| !line.is_empty())
        .filter_map(|line| line.strip_prefix("  ").map(String::from))
        .collect()
}

fn expected_real_corpus_modules_diagnostics() -> Vec<String> {
    let unresolved_file = workspace_root().join("dsl/examples/integration_tests.dag");
    let unresolved_file = unresolved_file
        .canonicalize()
        .unwrap_or(unresolved_file);
    vec![
        "cyclic dependencies detected among modules: examples.deployment, shared.gist_modes, tools.dag_viz, tools.gist".to_string(),
        format!(
            "{}: unresolved import: examples.integration_tests -> infra.gcp",
            unresolved_file.display()
        ),
    ]
}

#[test]
fn check_command_parses_full_dsl_corpus() {
    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check");

    assert!(
        output.status.success(),
        "check command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OK: parsed 42 file(s)"),
        "unexpected check output: {stdout}"
    );
    assert!(
        output.stderr.is_empty(),
        "check over golden corpus should not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !stdout.contains("Diagnostics:"),
        "parse-only check over golden corpus should not emit diagnostics: {stdout}"
    );
}

#[test]
fn check_command_real_corpus_stdout_matches_expected_snapshot() {
    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check");

    assert!(
        output.status.success(),
        "check command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, expected_check_success_stdout(42));
    assert!(
        output.stderr.is_empty(),
        "check over golden corpus should not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_command_default_root_matches_explicit_dsl_output() {
    let explicit = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run explicit-root daglang check");
    assert!(
        explicit.status.success(),
        "explicit-root check should succeed: {}",
        String::from_utf8_lossy(&explicit.stderr)
    );

    let default = Command::new(daglang_bin())
        .arg("check")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run default-root daglang check");
    assert!(
        default.status.success(),
        "default-root check should succeed: {}",
        String::from_utf8_lossy(&default.stderr)
    );

    assert_eq!(
        default.stdout, explicit.stdout,
        "default check output should match explicit 'check dsl' output"
    );
    assert_eq!(
        default.stderr, explicit.stderr,
        "default check stderr should match explicit-root stderr"
    );
}

#[test]
fn check_command_relative_and_absolute_root_are_equivalent() {
    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run relative-root daglang check");
    assert!(
        relative.status.success(),
        "relative-root check should succeed: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    let absolute_root = workspace_root().join("dsl");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run absolute-root daglang check");
    assert!(
        absolute.status.success(),
        "absolute-root check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute root check outputs should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute root check stderr should match"
    );
}

#[test]
fn check_command_absolute_mixed_segment_root_matches_canonical_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_mixed = workspace_root().join(".").join("dsl");
    let absolute_canonical = workspace_root().join("dsl");

    let mixed = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_mixed)
        .current_dir(&cwd)
        .output()
        .expect("failed to run mixed-segment absolute-root daglang check");
    assert!(
        mixed.status.success(),
        "mixed-segment absolute-root check should succeed: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_canonical)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute-root daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute-root check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute-root check stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute-root check stderr should match"
    );
}

#[test]
fn check_command_absolute_parent_segment_root_matches_canonical_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_parent_segment = workspace_root().join("dsl/std/..");
    let absolute_canonical = workspace_root().join("dsl");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment)
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment absolute-root daglang check");
    assert!(
        parent_segment.status.success(),
        "parent-segment absolute-root check should succeed: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_canonical)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute-root daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute-root check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute-root check stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute-root check stderr should match"
    );
}

#[test]
fn check_command_absolute_double_separator_root_matches_canonical_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_double_separator = PathBuf::from(format!("{}//dsl", workspace_root().display()));
    let absolute_canonical = workspace_root().join("dsl");

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator absolute-root daglang check");
    assert!(
        double_separator.status.success(),
        "double-separator absolute-root check should succeed: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_canonical)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute-root daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute-root check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute-root check stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute-root check stderr should match"
    );
}

#[test]
fn check_command_absolute_trailing_slash_root_matches_canonical_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_trailing_slash = PathBuf::from(format!("{}/", workspace_root().join("dsl").display()));
    let absolute_canonical = workspace_root().join("dsl");

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_trailing_slash)
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash absolute-root daglang check");
    assert!(
        trailing_slash.status.success(),
        "trailing-slash absolute-root check should succeed: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_canonical)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute-root daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute-root check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute-root check stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute-root check stderr should match"
    );
}

#[test]
fn check_command_absolute_double_separator_trailing_slash_root_matches_canonical_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_double_separator_trailing =
        PathBuf::from(format!("{}//dsl/", workspace_root().display()));
    let absolute_canonical = workspace_root().join("dsl");

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator_trailing)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator-trailing absolute-root daglang check");
    assert!(
        double_separator_trailing.status.success(),
        "double-separator-trailing absolute-root check should succeed: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_canonical)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute-root daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute-root check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator-trailing and canonical absolute-root check stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator-trailing and canonical absolute-root check stderr should match"
    );
}

#[test]
fn check_command_absolute_parent_segment_missing_root_matches_canonical_output() {
    let cwd = unique_temp_dir("check_absolute_parent_segment_missing_root");
    let anchor = cwd.join("anchor");
    std::fs::create_dir_all(&anchor).expect("failed to create anchor directory");
    let missing_root = cwd.join("missing_root");
    let absolute_parent_segment = cwd.join("anchor/../missing_root");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment)
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment absolute missing-root daglang check");
    assert!(
        !parent_segment.status.success(),
        "parent-segment absolute missing-root check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute missing-root daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-root check should fail"
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute missing-root check stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_absolute_double_separator_missing_root_matches_canonical_output() {
    let cwd = unique_temp_dir("check_absolute_double_separator_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");
    let absolute_double_separator = PathBuf::from(format!("{}//missing_root", cwd.display()));

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator absolute missing-root daglang check");
    assert!(
        !double_separator.status.success(),
        "double-separator absolute missing-root check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute missing-root daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-root check should fail"
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute missing-root check stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_absolute_trailing_slash_missing_root_matches_canonical_output() {
    let cwd = unique_temp_dir("check_absolute_trailing_slash_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");
    let absolute_trailing_slash = PathBuf::from(format!("{}/", missing_root.display()));

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_trailing_slash)
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash absolute missing-root daglang check");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash absolute missing-root check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute missing-root daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-root check should fail"
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute missing-root check stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_absolute_double_separator_trailing_slash_missing_root_matches_canonical_output() {
    let cwd = unique_temp_dir("check_absolute_double_separator_trailing_slash_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");
    let absolute_double_separator_trailing =
        PathBuf::from(format!("{}//missing_root/", cwd.display()));

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator_trailing)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator-trailing absolute missing-root daglang check");
    assert!(
        !double_separator_trailing.status.success(),
        "double-separator-trailing absolute missing-root check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute missing-root daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-root check should fail"
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator-trailing and canonical absolute missing-root check stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator-trailing and canonical absolute missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator_trailing.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_absolute_parent_segment_non_directory_root_matches_canonical_output() {
    let cwd = unique_temp_dir("check_absolute_parent_segment_non_directory_root");
    let anchor = cwd.join("anchor");
    std::fs::create_dir_all(&anchor).expect("failed to create anchor directory");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");
    let absolute_parent_segment = cwd.join("anchor/../input.txt");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment)
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment absolute non-directory-root daglang check");
    assert!(
        !parent_segment.status.success(),
        "parent-segment absolute non-directory-root check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute non-directory-root daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute non-directory-root check should fail"
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute non-directory-root check stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_absolute_trailing_slash_non_directory_root_matches_canonical_output() {
    let cwd = unique_temp_dir("check_absolute_trailing_slash_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");
    let absolute_trailing_slash = PathBuf::from(format!("{}/", root_file.display()));

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_trailing_slash)
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash absolute non-directory-root daglang check");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash absolute non-directory-root check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute non-directory-root daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute non-directory-root check should fail"
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute non-directory-root check stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_absolute_double_separator_trailing_slash_non_directory_root_matches_canonical_output() {
    let cwd = unique_temp_dir("check_absolute_double_separator_trailing_slash_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");
    let absolute_double_separator_trailing =
        PathBuf::from(format!("{}//input.txt/", cwd.display()));

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator_trailing)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator-trailing absolute non-directory-root daglang check");
    assert!(
        !double_separator_trailing.status.success(),
        "double-separator-trailing absolute non-directory-root check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute non-directory-root daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute non-directory-root check should fail"
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator-trailing and canonical absolute non-directory-root check stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator-trailing and canonical absolute non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator_trailing.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_absolute_double_separator_non_directory_root_matches_canonical_output() {
    let cwd = unique_temp_dir("check_absolute_double_separator_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");
    let absolute_double_separator = PathBuf::from(format!("{}//input.txt", cwd.display()));

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator absolute non-directory-root daglang check");
    assert!(
        !double_separator.status.success(),
        "double-separator absolute non-directory-root check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute non-directory-root daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute non-directory-root check should fail"
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute non-directory-root check stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_absolute_mixed_segment_missing_root_matches_canonical_output() {
    let cwd = unique_temp_dir("check_absolute_mixed_segment_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");
    let absolute_mixed = cwd.join(".").join("missing_root");

    let mixed = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_mixed)
        .current_dir(&cwd)
        .output()
        .expect("failed to run mixed-segment absolute missing-root daglang check");
    assert!(
        !mixed.status.success(),
        "mixed-segment absolute missing-root check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute missing-root daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-root check should fail"
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute missing-root check stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&mixed.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_absolute_mixed_segment_non_directory_root_matches_canonical_output() {
    let cwd = unique_temp_dir("check_absolute_mixed_segment_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");
    let absolute_mixed = cwd.join(".").join("input.txt");

    let mixed = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_mixed)
        .current_dir(&cwd)
        .output()
        .expect("failed to run mixed-segment absolute non-directory-root daglang check");
    assert!(
        !mixed.status.success(),
        "mixed-segment absolute non-directory-root check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute non-directory-root daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute non-directory-root check should fail"
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute non-directory-root check stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&mixed.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_parent_segment_root_matches_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_root = workspace_root().join("dsl");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("../dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment-root daglang check");
    assert!(
        parent_segment.status.success(),
        "parent-segment-root check should succeed: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-root daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-root check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute-root check outputs should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute-root check stderr should match"
    );
}

#[test]
fn check_command_parent_double_separator_root_matches_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_root = workspace_root().join("dsl");

    let parent_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("..//dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator-root daglang check");
    assert!(
        parent_double_separator.status.success(),
        "parent-double-separator-root check should succeed: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-root daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-root check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_double_separator.stdout, absolute.stdout,
        "parent-double-separator and absolute-root check outputs should match"
    );
    assert_eq!(
        parent_double_separator.stderr, absolute.stderr,
        "parent-double-separator and absolute-root check stderr should match"
    );
}

#[test]
fn check_command_parent_double_separator_trailing_slash_root_matches_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_root = workspace_root().join("dsl");

    let parent_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg("..//dsl/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator-trailing-root daglang check");
    assert!(
        parent_double_separator_trailing.status.success(),
        "parent-double-separator-trailing-root check should succeed: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-root daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-root check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_double_separator_trailing.stdout, absolute.stdout,
        "parent-double-separator-trailing and absolute-root check outputs should match"
    );
    assert_eq!(
        parent_double_separator_trailing.stderr, absolute.stderr,
        "parent-double-separator-trailing and absolute-root check stderr should match"
    );
}

#[test]
fn check_command_parent_curdir_double_separator_root_matches_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_root = workspace_root().join("dsl");

    let parent_curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".././dsl//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-double-separator-root daglang check");
    assert!(
        parent_curdir_double_separator.status.success(),
        "parent-curdir-double-separator-root check should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-root daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-root check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_curdir_double_separator.stdout, absolute.stdout,
        "parent-curdir-double-separator and absolute-root check outputs should match"
    );
    assert_eq!(
        parent_curdir_double_separator.stderr, absolute.stderr,
        "parent-curdir-double-separator and absolute-root check stderr should match"
    );
}

#[test]
fn check_command_curdir_segment_root_matches_plain_relative_output() {
    let cwd = workspace_root();

    let curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("./dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-segment-root daglang check");
    assert!(
        curdir_segment.status.success(),
        "curdir-segment-root check should succeed: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative-root daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative-root check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        curdir_segment.stdout, plain_relative.stdout,
        "curdir-segment and plain-relative check outputs should match"
    );
    assert_eq!(
        curdir_segment.stderr, plain_relative.stderr,
        "curdir-segment and plain-relative check stderr should match"
    );
}

#[test]
fn check_command_trailing_slash_root_matches_plain_relative_output() {
    let cwd = workspace_root();

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash-root daglang check");
    assert!(
        trailing_slash.status.success(),
        "trailing-slash-root check should succeed: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative-root daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative-root check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        trailing_slash.stdout, plain_relative.stdout,
        "trailing-slash and plain-relative check outputs should match"
    );
    assert_eq!(
        trailing_slash.stderr, plain_relative.stderr,
        "trailing-slash and plain-relative check stderr should match"
    );
}

#[test]
fn check_command_trailing_slash_missing_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("check_trailing_slash_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("missing_root/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash missing-root daglang check");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash missing-root check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative missing-root daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-root check should fail"
    );

    assert_eq!(
        trailing_slash.stdout, plain_relative.stdout,
        "trailing-slash and plain-relative missing-root check stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, plain_relative.stderr,
        "trailing-slash and plain-relative missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_trailing_slash_non_directory_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("check_trailing_slash_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("input.txt/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash non-directory-root daglang check");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash non-directory-root check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative non-directory-root daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative non-directory-root check should fail"
    );

    assert_eq!(
        trailing_slash.stdout, plain_relative.stdout,
        "trailing-slash and plain-relative non-directory-root check stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, plain_relative.stderr,
        "trailing-slash and plain-relative non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_mixed_segment_root_matches_plain_relative_output() {
    let cwd = workspace_root();

    let mixed_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl/./std/..")
        .current_dir(&cwd)
        .output()
        .expect("failed to run mixed-segment-root daglang check");
    assert!(
        mixed_segment.status.success(),
        "mixed-segment-root check should succeed: {}",
        String::from_utf8_lossy(&mixed_segment.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative-root daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative-root check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        mixed_segment.stdout, plain_relative.stdout,
        "mixed-segment and plain-relative check outputs should match"
    );
    assert_eq!(
        mixed_segment.stderr, plain_relative.stderr,
        "mixed-segment and plain-relative check stderr should match"
    );
}

#[test]
fn check_command_double_separator_root_matches_plain_relative_output() {
    let cwd = workspace_root();

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator-root daglang check");
    assert!(
        double_separator.status.success(),
        "double-separator-root check should succeed: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative-root daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative-root check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        double_separator.stdout, plain_relative.stdout,
        "double-separator and plain-relative check stdout should match"
    );
    assert_eq!(
        double_separator.stderr, plain_relative.stderr,
        "double-separator and plain-relative check stderr should match"
    );
}

#[test]
fn check_command_double_separator_missing_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("check_double_separator_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("missing_root//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator missing-root daglang check");
    assert!(
        !double_separator.status.success(),
        "double-separator missing-root check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative missing-root daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-root check should fail"
    );

    assert_eq!(
        double_separator.stdout, plain_relative.stdout,
        "double-separator and plain-relative missing-root check stdout should match"
    );
    assert_eq!(
        double_separator.stderr, plain_relative.stderr,
        "double-separator and plain-relative missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_double_separator_non_directory_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("check_double_separator_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("input.txt//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator non-directory-root daglang check");
    assert!(
        !double_separator.status.success(),
        "double-separator non-directory-root check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative non-directory-root daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative non-directory-root check should fail"
    );

    assert_eq!(
        double_separator.stdout, plain_relative.stdout,
        "double-separator and plain-relative non-directory-root check stdout should match"
    );
    assert_eq!(
        double_separator.stderr, plain_relative.stderr,
        "double-separator and plain-relative non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_curdir_double_separator_root_matches_plain_relative_output() {
    let cwd = workspace_root();

    let curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("./dsl//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-double-separator-root daglang check");
    assert!(
        curdir_double_separator.status.success(),
        "curdir-double-separator-root check should succeed: {}",
        String::from_utf8_lossy(&curdir_double_separator.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative-root daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative-root check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        curdir_double_separator.stdout, plain_relative.stdout,
        "curdir-double-separator and plain-relative check stdout should match"
    );
    assert_eq!(
        curdir_double_separator.stderr, plain_relative.stderr,
        "curdir-double-separator and plain-relative check stderr should match"
    );
}

#[test]
fn check_command_curdir_double_separator_missing_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("check_curdir_double_separator_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");

    let curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("./missing_root//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-double-separator missing-root daglang check");
    assert!(
        !curdir_double_separator.status.success(),
        "curdir-double-separator missing-root check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative missing-root daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-root check should fail"
    );

    assert_eq!(
        curdir_double_separator.stdout, plain_relative.stdout,
        "curdir-double-separator and plain-relative missing-root check stdout should match"
    );
    assert_eq!(
        curdir_double_separator.stderr, plain_relative.stderr,
        "curdir-double-separator and plain-relative missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_double_separator.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_curdir_double_separator_non_directory_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("check_curdir_double_separator_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("./input.txt//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-double-separator non-directory-root daglang check");
    assert!(
        !curdir_double_separator.status.success(),
        "curdir-double-separator non-directory-root check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative non-directory-root daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative non-directory-root check should fail"
    );

    assert_eq!(
        curdir_double_separator.stdout, plain_relative.stdout,
        "curdir-double-separator and plain-relative non-directory-root check stdout should match"
    );
    assert_eq!(
        curdir_double_separator.stderr, plain_relative.stderr,
        "curdir-double-separator and plain-relative non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_curdir_segment_missing_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("check_curdir_segment_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");

    let curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("./missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-segment missing-root daglang check");
    assert!(
        !curdir_segment.status.success(),
        "curdir-segment missing-root check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative missing-root daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-root check should fail"
    );

    assert_eq!(
        curdir_segment.stdout, plain_relative.stdout,
        "curdir-segment and plain-relative missing-root check stdout should match"
    );
    assert_eq!(
        curdir_segment.stderr, plain_relative.stderr,
        "curdir-segment and plain-relative missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_curdir_segment_non_directory_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("check_curdir_segment_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("./input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-segment non-directory-root daglang check");
    assert!(
        !curdir_segment.status.success(),
        "curdir-segment non-directory-root check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative non-directory-root daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative non-directory-root check should fail"
    );

    assert_eq!(
        curdir_segment.stdout, plain_relative.stdout,
        "curdir-segment and plain-relative non-directory-root check stdout should match"
    );
    assert_eq!(
        curdir_segment.stderr, plain_relative.stderr,
        "curdir-segment and plain-relative non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_relative_and_absolute_missing_roots_are_equivalent() {
    let missing_relative = unique_name("check_relative_absolute_missing_root");
    let cwd = workspace_root()
        .canonicalize()
        .unwrap_or_else(|_| workspace_root());

    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_relative)
        .current_dir(&cwd)
        .output()
        .expect("failed to run relative missing-root daglang check");
    assert!(
        !relative.status.success(),
        "relative missing-root check should fail"
    );

    let absolute_root = cwd.join(&missing_relative);
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-root daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-root check should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute missing-root check stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative.stderr).contains(&format!(
            "input root does not exist: {}",
            absolute_root.display()
        )),
        "missing-root diagnostic should contain normalized absolute path: {}",
        String::from_utf8_lossy(&relative.stderr)
    );
}

#[test]
fn check_command_parent_segment_missing_root_is_normalized_and_equivalent() {
    let parent = unique_temp_dir("check_parent_segment_missing_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let absolute_root = parent.join("missing_root");

    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("../missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment relative missing-root daglang check");
    assert!(
        !relative.status.success(),
        "parent-segment relative missing-root check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-root daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-root check should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "parent-segment relative and absolute missing-root check stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "parent-segment relative and absolute missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative.stderr).contains(&format!(
            "input root does not exist: {}",
            absolute_root.display()
        )),
        "missing-root diagnostic should contain normalized parent-segment path: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_double_separator_missing_root_is_normalized_and_equivalent() {
    let parent = unique_temp_dir("check_parent_double_separator_missing_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let absolute_root = parent.join("missing_root");

    let parent_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("..//missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator relative missing-root daglang check");
    assert!(
        !parent_double_separator.status.success(),
        "parent-double-separator relative missing-root check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-root daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-root check should fail"
    );

    assert_eq!(
        parent_double_separator.stdout, absolute.stdout,
        "parent-double-separator and absolute missing-root check stdout should match"
    );
    assert_eq!(
        parent_double_separator.stderr, absolute.stderr,
        "parent-double-separator and absolute missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator.stderr).contains(&format!(
            "input root does not exist: {}",
            absolute_root.display()
        )),
        "missing-root diagnostic should contain normalized parent-double-separator path: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_double_separator_trailing_slash_missing_root_is_normalized_and_equivalent() {
    let parent = unique_temp_dir("check_parent_double_separator_trailing_slash_missing_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let absolute_root = parent.join("missing_root");

    let parent_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg("..//missing_root/")
        .current_dir(&cwd)
        .output()
        .expect(
            "failed to run parent-double-separator-trailing relative missing-root daglang check",
        );
    assert!(
        !parent_double_separator_trailing.status.success(),
        "parent-double-separator-trailing relative missing-root check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-root daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-root check should fail"
    );

    assert_eq!(
        parent_double_separator_trailing.stdout, absolute.stdout,
        "parent-double-separator-trailing and absolute missing-root check stdout should match"
    );
    assert_eq!(
        parent_double_separator_trailing.stderr, absolute.stderr,
        "parent-double-separator-trailing and absolute missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr).contains(&format!(
            "input root does not exist: {}",
            absolute_root.display()
        )),
        "missing-root diagnostic should contain normalized parent-double-separator-trailing path: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_curdir_double_separator_missing_root_is_normalized_and_equivalent() {
    let parent = unique_temp_dir("check_parent_curdir_double_separator_missing_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let absolute_root = parent.join("missing_root");

    let parent_curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".././missing_root//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-double-separator relative missing-root daglang check");
    assert!(
        !parent_curdir_double_separator.status.success(),
        "parent-curdir-double-separator relative missing-root check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-root daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-root check should fail"
    );

    assert_eq!(
        parent_curdir_double_separator.stdout, absolute.stdout,
        "parent-curdir-double-separator and absolute missing-root check stdout should match"
    );
    assert_eq!(
        parent_curdir_double_separator.stderr, absolute.stderr,
        "parent-curdir-double-separator and absolute missing-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr).contains(&format!(
            "input root does not exist: {}",
            absolute_root.display()
        )),
        "missing-root diagnostic should contain normalized parent-curdir-double-separator path: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_segment_non_directory_root_matches_absolute_output() {
    let parent = unique_temp_dir("check_parent_segment_non_directory_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = parent.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("../input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment non-directory-root daglang check");
    assert!(
        !parent_segment.status.success(),
        "parent-segment non-directory-root check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute non-directory-root daglang check");
    assert!(
        !absolute.status.success(),
        "absolute non-directory-root check should fail"
    );

    assert_eq!(
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute non-directory-root check stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized parent-segment path: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_double_separator_non_directory_root_matches_absolute_output() {
    let parent = unique_temp_dir("check_parent_double_separator_non_directory_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = parent.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let parent_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("..//input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator non-directory-root daglang check");
    assert!(
        !parent_double_separator.status.success(),
        "parent-double-separator non-directory-root check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute non-directory-root daglang check");
    assert!(
        !absolute.status.success(),
        "absolute non-directory-root check should fail"
    );

    assert_eq!(
        parent_double_separator.stdout, absolute.stdout,
        "parent-double-separator and absolute non-directory-root check stdout should match"
    );
    assert_eq!(
        parent_double_separator.stderr, absolute.stderr,
        "parent-double-separator and absolute non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized parent-double-separator path: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_double_separator_trailing_slash_non_directory_root_matches_absolute_output() {
    let parent = unique_temp_dir("check_parent_double_separator_trailing_slash_non_directory_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = parent.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let parent_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg("..//input.txt/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator-trailing non-directory-root daglang check");
    assert!(
        !parent_double_separator_trailing.status.success(),
        "parent-double-separator-trailing non-directory-root check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute non-directory-root daglang check");
    assert!(
        !absolute.status.success(),
        "absolute non-directory-root check should fail"
    );

    assert_eq!(
        parent_double_separator_trailing.stdout, absolute.stdout,
        "parent-double-separator-trailing and absolute non-directory-root check stdout should match"
    );
    assert_eq!(
        parent_double_separator_trailing.stderr, absolute.stderr,
        "parent-double-separator-trailing and absolute non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized parent-double-separator-trailing path: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_curdir_double_separator_non_directory_root_matches_absolute_output() {
    let parent = unique_temp_dir("check_parent_curdir_double_separator_non_directory_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = parent.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let parent_curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".././input.txt//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-double-separator non-directory-root daglang check");
    assert!(
        !parent_curdir_double_separator.status.success(),
        "parent-curdir-double-separator non-directory-root check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute non-directory-root daglang check");
    assert!(
        !absolute.status.success(),
        "absolute non-directory-root check should fail"
    );

    assert_eq!(
        parent_curdir_double_separator.stdout, absolute.stdout,
        "parent-curdir-double-separator and absolute non-directory-root check stdout should match"
    );
    assert_eq!(
        parent_curdir_double_separator.stderr, absolute.stderr,
        "parent-curdir-double-separator and absolute non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized parent-curdir-double-separator path: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_relative_and_absolute_non_directory_roots_are_equivalent() {
    let cwd = unique_temp_dir("check_relative_absolute_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp directory");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run relative non-directory-root daglang check");
    assert!(
        !relative.status.success(),
        "relative non-directory-root check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute non-directory-root daglang check");
    assert!(
        !absolute.status.success(),
        "absolute non-directory-root check should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute non-directory-root check stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute non-directory-root check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostic should contain normalized absolute path: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_relative_and_absolute_single_file_targets_are_equivalent() {
    let root = unique_temp_dir("check_relative_absolute_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative-target daglang check");
    assert!(
        relative.status.success(),
        "relative-target check should succeed: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    let absolute_target = root.join("main.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute-target daglang check");
    assert!(
        absolute.status.success(),
        "absolute-target check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&relative.stdout),
        expected_check_success_stdout(1),
        "relative-target check should parse one file"
    );
    assert!(
        relative.stderr.is_empty(),
        "relative-target check should not emit stderr: {}",
        String::from_utf8_lossy(&relative.stderr)
    );
    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute single-file check outputs should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute single-file check stderr should match"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_parent_segment_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("check_parent_segment_single_file");
    let cwd = root.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    std::fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("../main.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment target daglang check");
    assert!(
        parent_segment.status.success(),
        "parent-segment target check should succeed: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    let absolute_target = root.join("main.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-target daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-target check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_segment.stdout),
        expected_check_success_stdout(1),
        "parent-segment single-file check should parse exactly one file"
    );
    assert!(
        parent_segment.stderr.is_empty(),
        "parent-segment single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_parent_double_separator_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("check_parent_double_separator_single_file");
    let cwd = root.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    std::fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let parent_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("..//main.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator target daglang check");
    assert!(
        parent_double_separator.status.success(),
        "parent-double-separator target check should succeed: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    let absolute_target = root.join("main.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-target daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-target check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_double_separator.stdout, absolute.stdout,
        "parent-double-separator and absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_double_separator.stderr, absolute.stderr,
        "parent-double-separator and absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_double_separator.stdout),
        expected_check_success_stdout(1),
        "parent-double-separator single-file check should parse exactly one file"
    );
    assert!(
        parent_double_separator.stderr.is_empty(),
        "parent-double-separator single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_parent_double_separator_trailing_slash_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("check_parent_double_separator_trailing_slash_single_file");
    let cwd = root.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    std::fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let parent_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg("..//main.dag/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator-trailing target daglang check");
    assert!(
        parent_double_separator_trailing.status.success(),
        "parent-double-separator-trailing target check should succeed: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    let absolute_target = root.join("main.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-target daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-target check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_double_separator_trailing.stdout, absolute.stdout,
        "parent-double-separator-trailing and absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_double_separator_trailing.stderr, absolute.stderr,
        "parent-double-separator-trailing and absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_double_separator_trailing.stdout),
        expected_check_success_stdout(1),
        "parent-double-separator-trailing single-file check should parse exactly one file"
    );
    assert!(
        parent_double_separator_trailing.stderr.is_empty(),
        "parent-double-separator-trailing single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_parent_curdir_double_separator_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("check_parent_curdir_double_separator_single_file");
    let cwd = root.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    std::fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let parent_curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".././main.dag//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-double-separator target daglang check");
    assert!(
        parent_curdir_double_separator.status.success(),
        "parent-curdir-double-separator target check should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    let absolute_target = root.join("main.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-target daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-target check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_curdir_double_separator.stdout, absolute.stdout,
        "parent-curdir-double-separator and absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_curdir_double_separator.stderr, absolute.stderr,
        "parent-curdir-double-separator and absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_curdir_double_separator.stdout),
        expected_check_success_stdout(1),
        "parent-curdir-double-separator single-file check should parse exactly one file"
    );
    assert!(
        parent_curdir_double_separator.stderr.is_empty(),
        "parent-curdir-double-separator single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_curdir_segment_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_segment_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("./main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment target daglang check");
    assert!(
        curdir_segment.status.success(),
        "curdir-segment target check should succeed: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        curdir_segment.stdout, plain_relative.stdout,
        "curdir-segment and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        curdir_segment.stderr, plain_relative.stderr,
        "curdir-segment and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_segment.stdout),
        expected_check_success_stdout(1),
        "curdir-segment single-file check should parse exactly one file"
    );
    assert!(
        curdir_segment.stderr.is_empty(),
        "curdir-segment single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_trailing_slash_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_trailing_slash_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash target daglang check");
    assert!(
        trailing_slash.status.success(),
        "trailing-slash target check should succeed: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        trailing_slash.stdout, plain_relative.stdout,
        "trailing-slash and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, plain_relative.stderr,
        "trailing-slash and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&trailing_slash.stdout),
        expected_check_success_stdout(1),
        "trailing-slash single-file check should parse exactly one file"
    );
    assert!(
        trailing_slash.stderr.is_empty(),
        "trailing-slash single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_double_separator_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_double_separator_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator target daglang check");
    assert!(
        double_separator.status.success(),
        "double-separator target check should succeed: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        double_separator.stdout, plain_relative.stdout,
        "double-separator and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        double_separator.stderr, plain_relative.stderr,
        "double-separator and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&double_separator.stdout),
        expected_check_success_stdout(1),
        "double-separator single-file check should parse exactly one file"
    );
    assert!(
        double_separator.stderr.is_empty(),
        "double-separator single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_double_separator_trailing_slash_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_double_separator_trailing_slash_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(".//main.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator-trailing target daglang check");
    assert!(
        double_separator_trailing.status.success(),
        "double-separator-trailing target check should succeed: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        double_separator_trailing.stdout, plain_relative.stdout,
        "double-separator-trailing and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, plain_relative.stderr,
        "double-separator-trailing and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&double_separator_trailing.stdout),
        expected_check_success_stdout(1),
        "double-separator-trailing single-file check should parse exactly one file"
    );
    assert!(
        double_separator_trailing.stderr.is_empty(),
        "double-separator-trailing single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_curdir_double_separator_missing_single_file_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_double_separator_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("./missing.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-double-separator missing-target daglang check");
    assert!(
        !curdir_double_separator.status.success(),
        "curdir-double-separator missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        curdir_double_separator.stdout, plain_relative.stdout,
        "curdir-double-separator and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        curdir_double_separator.stderr, plain_relative.stderr,
        "curdir-double-separator and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_double_separator_trailing_slash_missing_single_file_matches_plain_relative_output() {
    let root = unique_temp_dir("check_double_separator_trailing_slash_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(".//missing.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator-trailing missing-target daglang check");
    assert!(
        !double_separator_trailing.status.success(),
        "double-separator-trailing missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        double_separator_trailing.stdout, plain_relative.stdout,
        "double-separator-trailing and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, plain_relative.stderr,
        "double-separator-trailing and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator_trailing.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_segment_missing_single_file_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_segment_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("./missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment missing-target daglang check");
    assert!(
        !curdir_segment.status.success(),
        "curdir-segment missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        curdir_segment.stdout, plain_relative.stdout,
        "curdir-segment and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        curdir_segment.stderr, plain_relative.stderr,
        "curdir-segment and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_trailing_slash_missing_single_file_matches_plain_relative_output() {
    let root = unique_temp_dir("check_trailing_slash_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash missing-target daglang check");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        trailing_slash.stdout, plain_relative.stdout,
        "trailing-slash and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, plain_relative.stderr,
        "trailing-slash and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_relative_and_absolute_invalid_single_file_targets_are_equivalent() {
    let root = unique_temp_dir("check_relative_absolute_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative-target daglang check");
    assert!(
        !relative.status.success(),
        "relative-target check should fail for malformed source"
    );

    let absolute_target = root.join("broken.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute-target check should fail for malformed source"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute invalid single-file check stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute invalid single-file check stderr should match"
    );
    let canonical_target = root
        .join("broken.dag")
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&relative.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with location for malformed single-file target: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_parent_segment_invalid_single_file_target_matches_absolute_output() {
    let parent = unique_temp_dir("check_parent_segment_invalid_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let broken_file = parent.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("../broken.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment invalid-target daglang check");
    assert!(
        !parent_segment.status.success(),
        "parent-segment invalid-target check should fail for malformed source"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute invalid-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute invalid-target check should fail for malformed source"
    );

    assert_eq!(
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute invalid-target check stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for parent-segment invalid target: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_double_separator_invalid_single_file_target_matches_absolute_output() {
    let parent = unique_temp_dir("check_parent_double_separator_invalid_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let broken_file = parent.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let parent_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("..//broken.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator invalid-target daglang check");
    assert!(
        !parent_double_separator.status.success(),
        "parent-double-separator invalid-target check should fail for malformed source"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute invalid-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute invalid-target check should fail for malformed source"
    );

    assert_eq!(
        parent_double_separator.stdout, absolute.stdout,
        "parent-double-separator and absolute invalid-target check stdout should match"
    );
    assert_eq!(
        parent_double_separator.stderr, absolute.stderr,
        "parent-double-separator and absolute invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for parent-double-separator invalid target: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_double_separator_trailing_slash_invalid_single_file_target_matches_absolute_output()
{
    let parent = unique_temp_dir("check_parent_double_separator_trailing_slash_invalid_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let broken_file = parent.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let parent_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg("..//broken.dag/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator-trailing invalid-target daglang check");
    assert!(
        !parent_double_separator_trailing.status.success(),
        "parent-double-separator-trailing invalid-target check should fail for malformed source"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute invalid-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute invalid-target check should fail for malformed source"
    );

    assert_eq!(
        parent_double_separator_trailing.stdout, absolute.stdout,
        "parent-double-separator-trailing and absolute invalid-target check stdout should match"
    );
    assert_eq!(
        parent_double_separator_trailing.stderr, absolute.stderr,
        "parent-double-separator-trailing and absolute invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for parent-double-separator-trailing invalid target: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_curdir_double_separator_invalid_single_file_target_matches_absolute_output() {
    let parent = unique_temp_dir("check_parent_curdir_double_separator_invalid_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let broken_file = parent.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let parent_curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".././broken.dag//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-double-separator invalid-target daglang check");
    assert!(
        !parent_curdir_double_separator.status.success(),
        "parent-curdir-double-separator invalid-target check should fail for malformed source"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute invalid-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute invalid-target check should fail for malformed source"
    );

    assert_eq!(
        parent_curdir_double_separator.stdout, absolute.stdout,
        "parent-curdir-double-separator and absolute invalid-target check stdout should match"
    );
    assert_eq!(
        parent_curdir_double_separator.stderr, absolute.stderr,
        "parent-curdir-double-separator and absolute invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for parent-curdir-double-separator invalid target: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_curdir_segment_invalid_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_segment_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("./broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment invalid-target daglang check");
    assert!(
        !curdir_segment.status.success(),
        "curdir-segment invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        curdir_segment.stdout, plain_relative.stdout,
        "curdir-segment and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        curdir_segment.stderr, plain_relative.stderr,
        "curdir-segment and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_segment.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for curdir invalid target: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_trailing_slash_invalid_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_trailing_slash_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash invalid-target daglang check");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        trailing_slash.stdout, plain_relative.stdout,
        "trailing-slash and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, plain_relative.stderr,
        "trailing-slash and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for trailing-slash invalid target: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_double_separator_invalid_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_double_separator_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("./broken.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-double-separator invalid-target daglang check");
    assert!(
        !curdir_double_separator.status.success(),
        "curdir-double-separator invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        curdir_double_separator.stdout, plain_relative.stdout,
        "curdir-double-separator and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        curdir_double_separator.stderr, plain_relative.stderr,
        "curdir-double-separator and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for curdir-double-separator invalid target: {}",
        String::from_utf8_lossy(&curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_double_separator_trailing_slash_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_double_separator_trailing_slash_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(".//broken.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator-trailing invalid-target daglang check");
    assert!(
        !double_separator_trailing.status.success(),
        "double-separator-trailing invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        double_separator_trailing.stdout, plain_relative.stdout,
        "double-separator-trailing and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, plain_relative.stderr,
        "double-separator-trailing and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&double_separator_trailing.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for double-separator-trailing invalid target: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_double_separator_invalid_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_double_separator_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator invalid-target daglang check");
    assert!(
        !double_separator.status.success(),
        "double-separator invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        double_separator.stdout, plain_relative.stdout,
        "double-separator and plain-relative invalid-target stdout should match"
    );
    assert_eq!(
        double_separator.stderr, plain_relative.stderr,
        "double-separator and plain-relative invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonical path for double-separator invalid target: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_double_separator_missing_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_double_separator_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator missing-target daglang check");
    assert!(
        !double_separator.status.success(),
        "double-separator missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        double_separator.stdout, plain_relative.stdout,
        "double-separator and plain-relative missing-target stdout should match"
    );
    assert_eq!(
        double_separator.stderr, plain_relative.stderr,
        "double-separator and plain-relative missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_double_separator_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_double_separator_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_double_separator = PathBuf::from(format!("{}//broken.dag", root.display()));

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator absolute invalid-target daglang check");
    assert!(
        !double_separator.status.success(),
        "double-separator absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_double_separator_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_double_separator_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_double_separator = PathBuf::from(format!("{}//missing.dag", root.display()));

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator absolute missing-target daglang check");
    assert!(
        !double_separator.status.success(),
        "double-separator absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_mixed_segment_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_mixed_segment_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_mixed = root.join(".").join("main.dag");

    let mixed = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_mixed)
        .current_dir(&root)
        .output()
        .expect("failed to run mixed-segment absolute single-file daglang check");
    assert!(
        mixed.status.success(),
        "mixed-segment absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&mixed.stdout),
        expected_check_success_stdout(1),
        "mixed-segment absolute single-file check should parse exactly one file"
    );
    assert!(
        mixed.stderr.is_empty(),
        "mixed-segment absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_parent_segment_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_parent_segment_single_file");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_parent_segment = root.join("nested/../main.dag");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment absolute single-file daglang check");
    assert!(
        parent_segment.status.success(),
        "parent-segment absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_segment.stdout),
        expected_check_success_stdout(1),
        "parent-segment absolute single-file check should parse exactly one file"
    );
    assert!(
        parent_segment.stderr.is_empty(),
        "parent-segment absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_double_separator_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_double_separator_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_double_separator = PathBuf::from(format!("{}//main.dag", root.display()));

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator absolute single-file daglang check");
    assert!(
        double_separator.status.success(),
        "double-separator absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&double_separator.stdout),
        expected_check_success_stdout(1),
        "double-separator absolute single-file check should parse exactly one file"
    );
    assert!(
        double_separator.stderr.is_empty(),
        "double-separator absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_trailing_slash_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_trailing_slash_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_trailing_slash = PathBuf::from(format!("{}/", main_file.display()));

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_trailing_slash)
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash absolute single-file daglang check");
    assert!(
        trailing_slash.status.success(),
        "trailing-slash absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&trailing_slash.stdout),
        expected_check_success_stdout(1),
        "trailing-slash absolute single-file check should parse exactly one file"
    );
    assert!(
        trailing_slash.stderr.is_empty(),
        "trailing-slash absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_trailing_slash_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_trailing_slash_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_trailing_slash = PathBuf::from(format!("{}/", broken_file.display()));

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_trailing_slash)
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash absolute invalid-target daglang check");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_trailing_slash_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_trailing_slash_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_trailing_slash = PathBuf::from(format!("{}/", missing_target.display()));

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_trailing_slash)
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash absolute missing-target daglang check");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_double_separator_trailing_slash_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("check_absolute_double_separator_trailing_slash_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_double_separator_trailing =
        PathBuf::from(format!("{}//main.dag/", root.display()));

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator-trailing absolute single-file daglang check");
    assert!(
        double_separator_trailing.status.success(),
        "double-separator-trailing absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator-trailing and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator-trailing and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&double_separator_trailing.stdout),
        expected_check_success_stdout(1),
        "double-separator-trailing absolute single-file check should parse exactly one file"
    );
    assert!(
        double_separator_trailing.stderr.is_empty(),
        "double-separator-trailing absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_double_separator_trailing_slash_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_double_separator_trailing_slash_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_double_separator_trailing =
        PathBuf::from(format!("{}//broken.dag/", root.display()));

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator-trailing absolute invalid-target daglang check");
    assert!(
        !double_separator_trailing.status.success(),
        "double-separator-trailing absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator-trailing and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator-trailing and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&double_separator_trailing.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_double_separator_trailing_slash_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_double_separator_trailing_slash_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_double_separator_trailing =
        PathBuf::from(format!("{}//missing.dag/", root.display()));

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator-trailing absolute missing-target daglang check");
    assert!(
        !double_separator_trailing.status.success(),
        "double-separator-trailing absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator-trailing and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator-trailing and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator_trailing.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_mixed_segment_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_mixed_segment_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_mixed = root.join(".").join("broken.dag");

    let mixed = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_mixed)
        .current_dir(&root)
        .output()
        .expect("failed to run mixed-segment absolute invalid-target daglang check");
    assert!(
        !mixed.status.success(),
        "mixed-segment absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&mixed.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_mixed_segment_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_mixed_segment_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_mixed = root.join(".").join("missing.dag");

    let mixed = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_mixed)
        .current_dir(&root)
        .output()
        .expect("failed to run mixed-segment absolute missing-target daglang check");
    assert!(
        !mixed.status.success(),
        "mixed-segment absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&mixed.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_segment_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_parent_segment_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_parent_segment = root.join("nested/../broken.dag");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment absolute invalid-target daglang check");
    assert!(
        !parent_segment.status.success(),
        "parent-segment absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_segment_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_parent_segment_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_parent_segment = root.join("nested/../missing.dag");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment absolute missing-target daglang check");
    assert!(
        !parent_segment.status.success(),
        "parent-segment absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn modules_command_prints_module_graph_summary() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules");

    assert!(
        output.status.success(),
        "modules command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Discovered modules:"));
    let reported_modules = reported_modules_sorted(&stdout);
    let expected_modules: Vec<String> = expected_dsl_modules_sorted()
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        reported_modules, expected_modules,
        "modules command should report the complete 42-module corpus"
    );
}

#[test]
fn modules_command_reports_expected_real_corpus_diagnostics() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules");

    assert!(
        output.status.success(),
        "modules command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Diagnostics:"),
        "modules output should include diagnostics section for real corpus: {stdout}"
    );
    assert!(
        stdout.contains(
            "cyclic dependencies detected among modules: examples.deployment, shared.gist_modes, tools.dag_viz, tools.gist"
        ),
        "modules output should include expected cycle diagnostic: {stdout}"
    );
    assert!(
        stdout.contains("unresolved import: examples.integration_tests -> infra.gcp"),
        "modules output should include expected unresolved import diagnostic: {stdout}"
    );
}

#[test]
fn modules_command_real_corpus_diagnostics_match_expected_snapshot() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules");

    assert!(
        output.status.success(),
        "modules command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = reported_diagnostics_in_order(&stdout);
    let expected = expected_real_corpus_modules_diagnostics();
    assert_eq!(actual, expected);
}

#[test]
fn modules_command_real_corpus_order_matches_expected_snapshot() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules");

    assert!(
        output.status.success(),
        "modules command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = reported_modules_in_order(&stdout);
    let expected: Vec<String> = expected_real_corpus_module_order()
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn modules_command_real_corpus_order_matches_resolve_discovery() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules");

    assert!(
        output.status.success(),
        "modules command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = reported_modules_in_order(&stdout);
    let expected = resolve_discovered_module_order();
    assert_eq!(actual, expected);
}

#[test]
fn modules_command_real_corpus_summary_matches_resolve_discovery() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules");

    assert!(
        output.status.success(),
        "modules command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = reported_module_summary(&stdout);
    let expected = resolve_discovered_module_summary();
    assert_eq!(actual, expected);
}

#[test]
fn modules_command_default_root_matches_explicit_dsl_output() {
    let explicit = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run explicit-root daglang modules");
    assert!(
        explicit.status.success(),
        "explicit-root modules should succeed: {}",
        String::from_utf8_lossy(&explicit.stderr)
    );

    let default = Command::new(daglang_bin())
        .arg("modules")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run default-root daglang modules");
    assert!(
        default.status.success(),
        "default-root modules should succeed: {}",
        String::from_utf8_lossy(&default.stderr)
    );

    assert_eq!(
        default.stdout, explicit.stdout,
        "default modules output should match explicit 'modules dsl' output"
    );
    assert_eq!(
        default.stderr, explicit.stderr,
        "default modules stderr should match explicit-root stderr"
    );
}

#[test]
fn modules_command_relative_and_absolute_root_are_equivalent() {
    let relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run relative-root daglang modules");
    assert!(
        relative.status.success(),
        "relative-root modules should succeed: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    let absolute_root = workspace_root().join("dsl");
    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run absolute-root daglang modules");
    assert!(
        absolute.status.success(),
        "absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute root modules outputs should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute root modules stderr should match"
    );
}

#[test]
fn modules_command_absolute_mixed_segment_root_matches_canonical_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_mixed = workspace_root().join(".").join("dsl");
    let absolute_canonical = workspace_root().join("dsl");

    let mixed = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_mixed)
        .current_dir(&cwd)
        .output()
        .expect("failed to run mixed-segment absolute-root daglang modules");
    assert!(
        mixed.status.success(),
        "mixed-segment absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_canonical)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute-root daglang modules");
    assert!(
        canonical.status.success(),
        "canonical absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute-root modules stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute-root modules stderr should match"
    );
}

#[test]
fn modules_command_absolute_parent_segment_root_matches_canonical_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_parent_segment = workspace_root().join("dsl/std/..");
    let absolute_canonical = workspace_root().join("dsl");

    let parent_segment = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_parent_segment)
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment absolute-root daglang modules");
    assert!(
        parent_segment.status.success(),
        "parent-segment absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_canonical)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute-root daglang modules");
    assert!(
        canonical.status.success(),
        "canonical absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute-root modules stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute-root modules stderr should match"
    );
}

#[test]
fn modules_command_absolute_double_separator_root_matches_canonical_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_double_separator = PathBuf::from(format!("{}//dsl", workspace_root().display()));
    let absolute_canonical = workspace_root().join("dsl");

    let double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_double_separator)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator absolute-root daglang modules");
    assert!(
        double_separator.status.success(),
        "double-separator absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_canonical)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute-root daglang modules");
    assert!(
        canonical.status.success(),
        "canonical absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute-root modules stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute-root modules stderr should match"
    );
}

#[test]
fn modules_command_absolute_trailing_slash_root_matches_canonical_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_trailing_slash = PathBuf::from(format!("{}/", workspace_root().join("dsl").display()));
    let absolute_canonical = workspace_root().join("dsl");

    let trailing_slash = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_trailing_slash)
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash absolute-root daglang modules");
    assert!(
        trailing_slash.status.success(),
        "trailing-slash absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_canonical)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute-root daglang modules");
    assert!(
        canonical.status.success(),
        "canonical absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute-root modules stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute-root modules stderr should match"
    );
}

#[test]
fn modules_command_absolute_double_separator_trailing_slash_root_matches_canonical_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_double_separator_trailing =
        PathBuf::from(format!("{}//dsl/", workspace_root().display()));
    let absolute_canonical = workspace_root().join("dsl");

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_double_separator_trailing)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator-trailing absolute-root daglang modules");
    assert!(
        double_separator_trailing.status.success(),
        "double-separator-trailing absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_canonical)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute-root daglang modules");
    assert!(
        canonical.status.success(),
        "canonical absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator-trailing and canonical absolute-root modules stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator-trailing and canonical absolute-root modules stderr should match"
    );
}

#[test]
fn modules_command_absolute_parent_segment_missing_root_matches_canonical_output() {
    let cwd = unique_temp_dir("modules_absolute_parent_segment_missing_root");
    let anchor = cwd.join("anchor");
    std::fs::create_dir_all(&anchor).expect("failed to create anchor directory");
    let missing_root = cwd.join("missing_root");
    let absolute_parent_segment = cwd.join("anchor/../missing_root");

    let parent_segment = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_parent_segment)
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment absolute missing-root daglang modules");
    assert!(
        !parent_segment.status.success(),
        "parent-segment absolute missing-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&missing_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute missing-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-root modules should fail"
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute missing-root modules stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_double_separator_missing_root_matches_canonical_output() {
    let cwd = unique_temp_dir("modules_absolute_double_separator_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");
    let absolute_double_separator = PathBuf::from(format!("{}//missing_root", cwd.display()));

    let double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_double_separator)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator absolute missing-root daglang modules");
    assert!(
        !double_separator.status.success(),
        "double-separator absolute missing-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&missing_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute missing-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-root modules should fail"
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute missing-root modules stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_trailing_slash_missing_root_matches_canonical_output() {
    let cwd = unique_temp_dir("modules_absolute_trailing_slash_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");
    let absolute_trailing_slash = PathBuf::from(format!("{}/", missing_root.display()));

    let trailing_slash = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_trailing_slash)
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash absolute missing-root daglang modules");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash absolute missing-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&missing_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute missing-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-root modules should fail"
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute missing-root modules stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_double_separator_trailing_slash_missing_root_matches_canonical_output() {
    let cwd = unique_temp_dir("modules_absolute_double_separator_trailing_slash_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");
    let absolute_double_separator_trailing =
        PathBuf::from(format!("{}//missing_root/", cwd.display()));

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_double_separator_trailing)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator-trailing absolute missing-root daglang modules");
    assert!(
        !double_separator_trailing.status.success(),
        "double-separator-trailing absolute missing-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&missing_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute missing-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-root modules should fail"
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator-trailing and canonical absolute missing-root modules stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator-trailing and canonical absolute missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator_trailing.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_parent_segment_non_directory_root_matches_canonical_output() {
    let cwd = unique_temp_dir("modules_absolute_parent_segment_non_directory_root");
    let anchor = cwd.join("anchor");
    std::fs::create_dir_all(&anchor).expect("failed to create anchor directory");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");
    let absolute_parent_segment = cwd.join("anchor/../input.txt");

    let parent_segment = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_parent_segment)
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment absolute non-directory-root daglang modules");
    assert!(
        !parent_segment.status.success(),
        "parent-segment absolute non-directory-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute non-directory-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute non-directory-root modules should fail"
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute non-directory-root modules stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_double_separator_trailing_slash_non_directory_root_matches_canonical_output() {
    let cwd =
        unique_temp_dir("modules_absolute_double_separator_trailing_slash_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");
    let absolute_double_separator_trailing =
        PathBuf::from(format!("{}//input.txt/", cwd.display()));

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_double_separator_trailing)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator-trailing absolute non-directory-root daglang modules");
    assert!(
        !double_separator_trailing.status.success(),
        "double-separator-trailing absolute non-directory-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute non-directory-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute non-directory-root modules should fail"
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator-trailing and canonical absolute non-directory-root modules stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator-trailing and canonical absolute non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator_trailing.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_trailing_slash_non_directory_root_matches_canonical_output() {
    let cwd = unique_temp_dir("modules_absolute_trailing_slash_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");
    let absolute_trailing_slash = PathBuf::from(format!("{}/", root_file.display()));

    let trailing_slash = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_trailing_slash)
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash absolute non-directory-root daglang modules");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash absolute non-directory-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute non-directory-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute non-directory-root modules should fail"
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute non-directory-root modules stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_double_separator_trailing_slash_single_file_root_matches_canonical_output(
) {
    let cwd = unique_temp_dir("modules_absolute_double_separator_trailing_slash_single_file_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let file_path = cwd.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write .dag file");
    let absolute_double_separator_trailing =
        PathBuf::from(format!("{}//one.dag/", cwd.display()));

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_double_separator_trailing)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator-trailing absolute single-file-root daglang modules");
    assert!(
        !double_separator_trailing.status.success(),
        "double-separator-trailing absolute single-file-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&file_path)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute single-file-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute single-file-root modules should fail"
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator-trailing and canonical absolute single-file-root modules stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator-trailing and canonical absolute single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator_trailing.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_parent_segment_single_file_root_matches_canonical_output() {
    let cwd = unique_temp_dir("modules_absolute_parent_segment_single_file_root");
    let anchor = cwd.join("anchor");
    std::fs::create_dir_all(&anchor).expect("failed to create anchor directory");
    let file_path = cwd.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write .dag file");
    let absolute_parent_segment = cwd.join("anchor/../one.dag");

    let parent_segment = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_parent_segment)
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment absolute single-file-root daglang modules");
    assert!(
        !parent_segment.status.success(),
        "parent-segment absolute single-file-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&file_path)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute single-file-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute single-file-root modules should fail"
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute single-file-root modules stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_trailing_slash_single_file_root_matches_canonical_output() {
    let cwd = unique_temp_dir("modules_absolute_trailing_slash_single_file_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let file_path = cwd.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write .dag file");
    let absolute_trailing_slash = PathBuf::from(format!("{}/", file_path.display()));

    let trailing_slash = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_trailing_slash)
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash absolute single-file-root daglang modules");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash absolute single-file-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&file_path)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute single-file-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute single-file-root modules should fail"
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute single-file-root modules stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_double_separator_non_directory_root_matches_canonical_output() {
    let cwd = unique_temp_dir("modules_absolute_double_separator_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");
    let absolute_double_separator = PathBuf::from(format!("{}//input.txt", cwd.display()));

    let double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_double_separator)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator absolute non-directory-root daglang modules");
    assert!(
        !double_separator.status.success(),
        "double-separator absolute non-directory-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute non-directory-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute non-directory-root modules should fail"
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute non-directory-root modules stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_mixed_segment_missing_root_matches_canonical_output() {
    let cwd = unique_temp_dir("modules_absolute_mixed_segment_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");
    let absolute_mixed = cwd.join(".").join("missing_root");

    let mixed = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_mixed)
        .current_dir(&cwd)
        .output()
        .expect("failed to run mixed-segment absolute missing-root daglang modules");
    assert!(
        !mixed.status.success(),
        "mixed-segment absolute missing-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&missing_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute missing-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-root modules should fail"
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute missing-root modules stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&mixed.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_mixed_segment_non_directory_root_matches_canonical_output() {
    let cwd = unique_temp_dir("modules_absolute_mixed_segment_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");
    let absolute_mixed = cwd.join(".").join("input.txt");

    let mixed = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_mixed)
        .current_dir(&cwd)
        .output()
        .expect("failed to run mixed-segment absolute non-directory-root daglang modules");
    assert!(
        !mixed.status.success(),
        "mixed-segment absolute non-directory-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute non-directory-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute non-directory-root modules should fail"
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute non-directory-root modules stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&mixed.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_double_separator_single_file_root_matches_canonical_output() {
    let cwd = unique_temp_dir("modules_absolute_double_separator_single_file_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let file_path = cwd.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write .dag file");
    let absolute_double_separator = PathBuf::from(format!("{}//one.dag", cwd.display()));

    let double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_double_separator)
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator absolute single-file-root daglang modules");
    assert!(
        !double_separator.status.success(),
        "double-separator absolute single-file-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&file_path)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute single-file-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute single-file-root modules should fail"
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute single-file-root modules stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_absolute_mixed_segment_single_file_root_matches_canonical_output() {
    let cwd = unique_temp_dir("modules_absolute_mixed_segment_single_file_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let file_path = cwd.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write .dag file");
    let absolute_mixed = cwd.join(".").join("one.dag");

    let mixed = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_mixed)
        .current_dir(&cwd)
        .output()
        .expect("failed to run mixed-segment absolute single-file-root daglang modules");
    assert!(
        !mixed.status.success(),
        "mixed-segment absolute single-file-root modules should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("modules")
        .arg(&file_path)
        .current_dir(&cwd)
        .output()
        .expect("failed to run canonical absolute single-file-root daglang modules");
    assert!(
        !canonical.status.success(),
        "canonical absolute single-file-root modules should fail"
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute single-file-root modules stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&mixed.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_parent_segment_root_matches_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_root = workspace_root().join("dsl");

    let parent_segment = Command::new(daglang_bin())
        .arg("modules")
        .arg("../dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment-root daglang modules");
    assert!(
        parent_segment.status.success(),
        "parent-segment-root modules should succeed: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-root daglang modules from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute-root modules stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute-root modules stderr should match"
    );
}

#[test]
fn modules_command_parent_double_separator_root_matches_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_root = workspace_root().join("dsl");

    let parent_double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg("..//dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator-root daglang modules");
    assert!(
        parent_double_separator.status.success(),
        "parent-double-separator-root modules should succeed: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-root daglang modules from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_double_separator.stdout, absolute.stdout,
        "parent-double-separator and absolute-root modules stdout should match"
    );
    assert_eq!(
        parent_double_separator.stderr, absolute.stderr,
        "parent-double-separator and absolute-root modules stderr should match"
    );
}

#[test]
fn modules_command_parent_double_separator_trailing_slash_root_matches_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_root = workspace_root().join("dsl");

    let parent_double_separator_trailing = Command::new(daglang_bin())
        .arg("modules")
        .arg("..//dsl/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator-trailing-root daglang modules");
    assert!(
        parent_double_separator_trailing.status.success(),
        "parent-double-separator-trailing-root modules should succeed: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-root daglang modules from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_double_separator_trailing.stdout, absolute.stdout,
        "parent-double-separator-trailing and absolute-root modules stdout should match"
    );
    assert_eq!(
        parent_double_separator_trailing.stderr, absolute.stderr,
        "parent-double-separator-trailing and absolute-root modules stderr should match"
    );
}

#[test]
fn modules_command_parent_curdir_double_separator_root_matches_absolute_output() {
    let cwd = workspace_root().join("core");
    let absolute_root = workspace_root().join("dsl");

    let parent_curdir_double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg(".././dsl//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-double-separator-root daglang modules");
    assert!(
        parent_curdir_double_separator.status.success(),
        "parent-curdir-double-separator-root modules should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-root daglang modules from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_curdir_double_separator.stdout, absolute.stdout,
        "parent-curdir-double-separator and absolute-root modules stdout should match"
    );
    assert_eq!(
        parent_curdir_double_separator.stderr, absolute.stderr,
        "parent-curdir-double-separator and absolute-root modules stderr should match"
    );
}

#[test]
fn modules_command_curdir_segment_root_matches_plain_relative_output() {
    let cwd = workspace_root();

    let curdir_segment = Command::new(daglang_bin())
        .arg("modules")
        .arg("./dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-segment-root daglang modules");
    assert!(
        curdir_segment.status.success(),
        "curdir-segment-root modules should succeed: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative-root daglang modules");
    assert!(
        plain_relative.status.success(),
        "plain-relative-root modules should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        curdir_segment.stdout, plain_relative.stdout,
        "curdir-segment and plain-relative modules stdout should match"
    );
    assert_eq!(
        curdir_segment.stderr, plain_relative.stderr,
        "curdir-segment and plain-relative modules stderr should match"
    );
}

#[test]
fn modules_command_trailing_slash_root_matches_plain_relative_output() {
    let cwd = workspace_root();

    let trailing_slash = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash-root daglang modules");
    assert!(
        trailing_slash.status.success(),
        "trailing-slash-root modules should succeed: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative-root daglang modules");
    assert!(
        plain_relative.status.success(),
        "plain-relative-root modules should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        trailing_slash.stdout, plain_relative.stdout,
        "trailing-slash and plain-relative modules stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, plain_relative.stderr,
        "trailing-slash and plain-relative modules stderr should match"
    );
}

#[test]
fn modules_command_trailing_slash_missing_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("modules_trailing_slash_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");

    let trailing_slash = Command::new(daglang_bin())
        .arg("modules")
        .arg("missing_root/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash missing-root daglang modules");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash missing-root modules should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative missing-root daglang modules");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-root modules should fail"
    );

    assert_eq!(
        trailing_slash.stdout, plain_relative.stdout,
        "trailing-slash and plain-relative missing-root modules stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, plain_relative.stderr,
        "trailing-slash and plain-relative missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_trailing_slash_non_directory_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("modules_trailing_slash_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let trailing_slash = Command::new(daglang_bin())
        .arg("modules")
        .arg("input.txt/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash non-directory-root daglang modules");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash non-directory-root modules should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative non-directory-root daglang modules");
    assert!(
        !plain_relative.status.success(),
        "plain-relative non-directory-root modules should fail"
    );

    assert_eq!(
        trailing_slash.stdout, plain_relative.stdout,
        "trailing-slash and plain-relative non-directory-root modules stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, plain_relative.stderr,
        "trailing-slash and plain-relative non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_trailing_slash_single_file_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("modules_trailing_slash_single_file_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let file_path = cwd.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write .dag file");

    let trailing_slash = Command::new(daglang_bin())
        .arg("modules")
        .arg("one.dag/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run trailing-slash single-file-root daglang modules");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash single-file-root modules should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("one.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative single-file-root daglang modules");
    assert!(
        !plain_relative.status.success(),
        "plain-relative single-file-root modules should fail"
    );

    assert_eq!(
        trailing_slash.stdout, plain_relative.stdout,
        "trailing-slash and plain-relative single-file-root modules stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, plain_relative.stderr,
        "trailing-slash and plain-relative single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_mixed_segment_root_matches_plain_relative_output() {
    let cwd = workspace_root();

    let mixed_segment = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl/./std/..")
        .current_dir(&cwd)
        .output()
        .expect("failed to run mixed-segment-root daglang modules");
    assert!(
        mixed_segment.status.success(),
        "mixed-segment-root modules should succeed: {}",
        String::from_utf8_lossy(&mixed_segment.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative-root daglang modules");
    assert!(
        plain_relative.status.success(),
        "plain-relative-root modules should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        mixed_segment.stdout, plain_relative.stdout,
        "mixed-segment and plain-relative modules stdout should match"
    );
    assert_eq!(
        mixed_segment.stderr, plain_relative.stderr,
        "mixed-segment and plain-relative modules stderr should match"
    );
}

#[test]
fn modules_command_double_separator_root_matches_plain_relative_output() {
    let cwd = workspace_root();

    let double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator-root daglang modules");
    assert!(
        double_separator.status.success(),
        "double-separator-root modules should succeed: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative-root daglang modules");
    assert!(
        plain_relative.status.success(),
        "plain-relative-root modules should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        double_separator.stdout, plain_relative.stdout,
        "double-separator and plain-relative modules stdout should match"
    );
    assert_eq!(
        double_separator.stderr, plain_relative.stderr,
        "double-separator and plain-relative modules stderr should match"
    );
}

#[test]
fn modules_command_double_separator_missing_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("modules_double_separator_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");

    let double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg("missing_root//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator missing-root daglang modules");
    assert!(
        !double_separator.status.success(),
        "double-separator missing-root modules should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative missing-root daglang modules");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-root modules should fail"
    );

    assert_eq!(
        double_separator.stdout, plain_relative.stdout,
        "double-separator and plain-relative missing-root modules stdout should match"
    );
    assert_eq!(
        double_separator.stderr, plain_relative.stderr,
        "double-separator and plain-relative missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_double_separator_non_directory_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("modules_double_separator_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg("input.txt//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator non-directory-root daglang modules");
    assert!(
        !double_separator.status.success(),
        "double-separator non-directory-root modules should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative non-directory-root daglang modules");
    assert!(
        !plain_relative.status.success(),
        "plain-relative non-directory-root modules should fail"
    );

    assert_eq!(
        double_separator.stdout, plain_relative.stdout,
        "double-separator and plain-relative non-directory-root modules stdout should match"
    );
    assert_eq!(
        double_separator.stderr, plain_relative.stderr,
        "double-separator and plain-relative non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_double_separator_single_file_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("modules_double_separator_single_file_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let file_path = cwd.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write .dag file");

    let double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg("one.dag//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run double-separator single-file-root daglang modules");
    assert!(
        !double_separator.status.success(),
        "double-separator single-file-root modules should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("one.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative single-file-root daglang modules");
    assert!(
        !plain_relative.status.success(),
        "plain-relative single-file-root modules should fail"
    );

    assert_eq!(
        double_separator.stdout, plain_relative.stdout,
        "double-separator and plain-relative single-file-root modules stdout should match"
    );
    assert_eq!(
        double_separator.stderr, plain_relative.stderr,
        "double-separator and plain-relative single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_curdir_double_separator_root_matches_plain_relative_output() {
    let cwd = workspace_root();

    let curdir_double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg("./dsl//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-double-separator-root daglang modules");
    assert!(
        curdir_double_separator.status.success(),
        "curdir-double-separator-root modules should succeed: {}",
        String::from_utf8_lossy(&curdir_double_separator.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative-root daglang modules");
    assert!(
        plain_relative.status.success(),
        "plain-relative-root modules should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        curdir_double_separator.stdout, plain_relative.stdout,
        "curdir-double-separator and plain-relative modules stdout should match"
    );
    assert_eq!(
        curdir_double_separator.stderr, plain_relative.stderr,
        "curdir-double-separator and plain-relative modules stderr should match"
    );
}

#[test]
fn modules_command_curdir_double_separator_missing_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("modules_curdir_double_separator_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");

    let curdir_double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg("./missing_root//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-double-separator missing-root daglang modules");
    assert!(
        !curdir_double_separator.status.success(),
        "curdir-double-separator missing-root modules should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative missing-root daglang modules");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-root modules should fail"
    );

    assert_eq!(
        curdir_double_separator.stdout, plain_relative.stdout,
        "curdir-double-separator and plain-relative missing-root modules stdout should match"
    );
    assert_eq!(
        curdir_double_separator.stderr, plain_relative.stderr,
        "curdir-double-separator and plain-relative missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_double_separator.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_curdir_double_separator_non_directory_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("modules_curdir_double_separator_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let curdir_double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg("./input.txt//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-double-separator non-directory-root daglang modules");
    assert!(
        !curdir_double_separator.status.success(),
        "curdir-double-separator non-directory-root modules should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative non-directory-root daglang modules");
    assert!(
        !plain_relative.status.success(),
        "plain-relative non-directory-root modules should fail"
    );

    assert_eq!(
        curdir_double_separator.stdout, plain_relative.stdout,
        "curdir-double-separator and plain-relative non-directory-root modules stdout should match"
    );
    assert_eq!(
        curdir_double_separator.stderr, plain_relative.stderr,
        "curdir-double-separator and plain-relative non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_curdir_double_separator_single_file_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("modules_curdir_double_separator_single_file_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let file_path = cwd.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write .dag file");

    let curdir_double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg("./one.dag//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-double-separator single-file-root daglang modules");
    assert!(
        !curdir_double_separator.status.success(),
        "curdir-double-separator single-file-root modules should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("one.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative single-file-root daglang modules");
    assert!(
        !plain_relative.status.success(),
        "plain-relative single-file-root modules should fail"
    );

    assert_eq!(
        curdir_double_separator.stdout, plain_relative.stdout,
        "curdir-double-separator and plain-relative single-file-root modules stdout should match"
    );
    assert_eq!(
        curdir_double_separator.stderr, plain_relative.stderr,
        "curdir-double-separator and plain-relative single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_curdir_segment_missing_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("modules_curdir_segment_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_root = cwd.join("missing_root");

    let curdir_segment = Command::new(daglang_bin())
        .arg("modules")
        .arg("./missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-segment missing-root daglang modules");
    assert!(
        !curdir_segment.status.success(),
        "curdir-segment missing-root modules should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative missing-root daglang modules");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-root modules should fail"
    );

    assert_eq!(
        curdir_segment.stdout, plain_relative.stdout,
        "curdir-segment and plain-relative missing-root modules stdout should match"
    );
    assert_eq!(
        curdir_segment.stderr, plain_relative.stderr,
        "curdir-segment and plain-relative missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment.stderr).contains(&format!(
            "input root does not exist: {}",
            missing_root.display()
        )),
        "missing-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_curdir_segment_non_directory_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("modules_curdir_segment_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let curdir_segment = Command::new(daglang_bin())
        .arg("modules")
        .arg("./input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-segment non-directory-root daglang modules");
    assert!(
        !curdir_segment.status.success(),
        "curdir-segment non-directory-root modules should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative non-directory-root daglang modules");
    assert!(
        !plain_relative.status.success(),
        "plain-relative non-directory-root modules should fail"
    );

    assert_eq!(
        curdir_segment.stdout, plain_relative.stdout,
        "curdir-segment and plain-relative non-directory-root modules stdout should match"
    );
    assert_eq!(
        curdir_segment.stderr, plain_relative.stderr,
        "curdir-segment and plain-relative non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_curdir_segment_single_file_root_matches_plain_relative_output() {
    let cwd = unique_temp_dir("modules_curdir_segment_single_file_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let file_path = cwd.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write .dag file");

    let curdir_segment = Command::new(daglang_bin())
        .arg("modules")
        .arg("./one.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run curdir-segment single-file-root daglang modules");
    assert!(
        !curdir_segment.status.success(),
        "curdir-segment single-file-root modules should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("one.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative single-file-root daglang modules");
    assert!(
        !plain_relative.status.success(),
        "plain-relative single-file-root modules should fail"
    );

    assert_eq!(
        curdir_segment.stdout, plain_relative.stdout,
        "curdir-segment and plain-relative single-file-root modules stdout should match"
    );
    assert_eq!(
        curdir_segment.stderr, plain_relative.stderr,
        "curdir-segment and plain-relative single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_relative_and_absolute_missing_roots_are_equivalent() {
    let missing_relative = unique_name("modules_relative_absolute_missing_root");
    let cwd = workspace_root()
        .canonicalize()
        .unwrap_or_else(|_| workspace_root());

    let relative = Command::new(daglang_bin())
        .arg("modules")
        .arg(&missing_relative)
        .current_dir(&cwd)
        .output()
        .expect("failed to run relative missing-root daglang modules");
    assert!(
        !relative.status.success(),
        "relative missing-root modules should fail"
    );

    let absolute_root = cwd.join(&missing_relative);
    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute missing-root modules should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute missing-root modules stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative.stderr).contains(&format!(
            "input root does not exist: {}",
            absolute_root.display()
        )),
        "missing-root diagnostic should contain normalized absolute path: {}",
        String::from_utf8_lossy(&relative.stderr)
    );
}

#[test]
fn modules_command_parent_segment_missing_root_is_normalized_and_equivalent() {
    let parent = unique_temp_dir("modules_parent_segment_missing_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let absolute_root = parent.join("missing_root");

    let relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("../missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment relative missing-root daglang modules");
    assert!(
        !relative.status.success(),
        "parent-segment relative missing-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute missing-root modules should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "parent-segment relative and absolute missing-root modules stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "parent-segment relative and absolute missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative.stderr).contains(&format!(
            "input root does not exist: {}",
            absolute_root.display()
        )),
        "missing-root diagnostics should include normalized parent-segment path: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn modules_command_parent_double_separator_missing_root_is_normalized_and_equivalent() {
    let parent = unique_temp_dir("modules_parent_double_separator_missing_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let absolute_root = parent.join("missing_root");

    let parent_double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg("..//missing_root")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator relative missing-root daglang modules");
    assert!(
        !parent_double_separator.status.success(),
        "parent-double-separator relative missing-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute missing-root modules should fail"
    );

    assert_eq!(
        parent_double_separator.stdout, absolute.stdout,
        "parent-double-separator and absolute missing-root modules stdout should match"
    );
    assert_eq!(
        parent_double_separator.stderr, absolute.stderr,
        "parent-double-separator and absolute missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator.stderr).contains(&format!(
            "input root does not exist: {}",
            absolute_root.display()
        )),
        "missing-root diagnostics should include normalized parent-double-separator path: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn modules_command_parent_double_separator_trailing_slash_missing_root_is_normalized_and_equivalent() {
    let parent = unique_temp_dir("modules_parent_double_separator_trailing_slash_missing_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let absolute_root = parent.join("missing_root");

    let parent_double_separator_trailing = Command::new(daglang_bin())
        .arg("modules")
        .arg("..//missing_root/")
        .current_dir(&cwd)
        .output()
        .expect(
            "failed to run parent-double-separator-trailing relative missing-root daglang modules",
        );
    assert!(
        !parent_double_separator_trailing.status.success(),
        "parent-double-separator-trailing relative missing-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute missing-root modules should fail"
    );

    assert_eq!(
        parent_double_separator_trailing.stdout, absolute.stdout,
        "parent-double-separator-trailing and absolute missing-root modules stdout should match"
    );
    assert_eq!(
        parent_double_separator_trailing.stderr, absolute.stderr,
        "parent-double-separator-trailing and absolute missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr).contains(&format!(
            "input root does not exist: {}",
            absolute_root.display()
        )),
        "missing-root diagnostics should include normalized parent-double-separator-trailing path: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn modules_command_parent_curdir_double_separator_missing_root_is_normalized_and_equivalent() {
    let parent = unique_temp_dir("modules_parent_curdir_double_separator_missing_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let absolute_root = parent.join("missing_root");

    let parent_curdir_double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg(".././missing_root//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-double-separator relative missing-root daglang modules");
    assert!(
        !parent_curdir_double_separator.status.success(),
        "parent-curdir-double-separator relative missing-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_root)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute missing-root modules should fail"
    );

    assert_eq!(
        parent_curdir_double_separator.stdout, absolute.stdout,
        "parent-curdir-double-separator and absolute missing-root modules stdout should match"
    );
    assert_eq!(
        parent_curdir_double_separator.stderr, absolute.stderr,
        "parent-curdir-double-separator and absolute missing-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr).contains(&format!(
            "input root does not exist: {}",
            absolute_root.display()
        )),
        "missing-root diagnostics should include normalized parent-curdir-double-separator path: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn modules_command_parent_segment_non_directory_root_matches_absolute_output() {
    let parent = unique_temp_dir("modules_parent_segment_non_directory_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = parent.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let parent_segment = Command::new(daglang_bin())
        .arg("modules")
        .arg("../input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment non-directory-root daglang modules");
    assert!(
        !parent_segment.status.success(),
        "parent-segment non-directory-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute non-directory-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute non-directory-root modules should fail"
    );

    assert_eq!(
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute non-directory-root modules stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized parent-segment path: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn modules_command_parent_double_separator_non_directory_root_matches_absolute_output() {
    let parent = unique_temp_dir("modules_parent_double_separator_non_directory_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = parent.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let parent_double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg("..//input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator non-directory-root daglang modules");
    assert!(
        !parent_double_separator.status.success(),
        "parent-double-separator non-directory-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute non-directory-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute non-directory-root modules should fail"
    );

    assert_eq!(
        parent_double_separator.stdout, absolute.stdout,
        "parent-double-separator and absolute non-directory-root modules stdout should match"
    );
    assert_eq!(
        parent_double_separator.stderr, absolute.stderr,
        "parent-double-separator and absolute non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized parent-double-separator path: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn modules_command_parent_double_separator_trailing_slash_non_directory_root_matches_absolute_output()
{
    let parent = unique_temp_dir("modules_parent_double_separator_trailing_slash_non_directory_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = parent.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let parent_double_separator_trailing = Command::new(daglang_bin())
        .arg("modules")
        .arg("..//input.txt/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator-trailing non-directory-root daglang modules");
    assert!(
        !parent_double_separator_trailing.status.success(),
        "parent-double-separator-trailing non-directory-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute non-directory-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute non-directory-root modules should fail"
    );

    assert_eq!(
        parent_double_separator_trailing.stdout, absolute.stdout,
        "parent-double-separator-trailing and absolute non-directory-root modules stdout should match"
    );
    assert_eq!(
        parent_double_separator_trailing.stderr, absolute.stderr,
        "parent-double-separator-trailing and absolute non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized parent-double-separator-trailing path: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn modules_command_parent_curdir_double_separator_non_directory_root_matches_absolute_output() {
    let parent = unique_temp_dir("modules_parent_curdir_double_separator_non_directory_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let root_file = parent.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let parent_curdir_double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg(".././input.txt//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-double-separator non-directory-root daglang modules");
    assert!(
        !parent_curdir_double_separator.status.success(),
        "parent-curdir-double-separator non-directory-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute non-directory-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute non-directory-root modules should fail"
    );

    assert_eq!(
        parent_curdir_double_separator.stdout, absolute.stdout,
        "parent-curdir-double-separator and absolute non-directory-root modules stdout should match"
    );
    assert_eq!(
        parent_curdir_double_separator.stderr, absolute.stderr,
        "parent-curdir-double-separator and absolute non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include normalized parent-curdir-double-separator path: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn modules_command_parent_segment_single_file_root_matches_absolute_output() {
    let parent = unique_temp_dir("modules_parent_segment_single_file_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let file_path = parent.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write .dag file");

    let parent_segment = Command::new(daglang_bin())
        .arg("modules")
        .arg("../one.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment single-file-root daglang modules");
    assert!(
        !parent_segment.status.success(),
        "parent-segment single-file-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&file_path)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute single-file-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute single-file-root modules should fail"
    );

    assert_eq!(
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute single-file-root modules stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostics should include normalized parent-segment path: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn modules_command_parent_double_separator_single_file_root_matches_absolute_output() {
    let parent = unique_temp_dir("modules_parent_double_separator_single_file_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let file_path = parent.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write .dag file");

    let parent_double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg("..//one.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator single-file-root daglang modules");
    assert!(
        !parent_double_separator.status.success(),
        "parent-double-separator single-file-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&file_path)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute single-file-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute single-file-root modules should fail"
    );

    assert_eq!(
        parent_double_separator.stdout, absolute.stdout,
        "parent-double-separator and absolute single-file-root modules stdout should match"
    );
    assert_eq!(
        parent_double_separator.stderr, absolute.stderr,
        "parent-double-separator and absolute single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostics should include normalized parent-double-separator path: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn modules_command_parent_double_separator_trailing_slash_single_file_root_matches_absolute_output() {
    let parent = unique_temp_dir("modules_parent_double_separator_trailing_slash_single_file_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let file_path = parent.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write .dag file");

    let parent_double_separator_trailing = Command::new(daglang_bin())
        .arg("modules")
        .arg("..//one.dag/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator-trailing single-file-root daglang modules");
    assert!(
        !parent_double_separator_trailing.status.success(),
        "parent-double-separator-trailing single-file-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&file_path)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute single-file-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute single-file-root modules should fail"
    );

    assert_eq!(
        parent_double_separator_trailing.stdout, absolute.stdout,
        "parent-double-separator-trailing and absolute single-file-root modules stdout should match"
    );
    assert_eq!(
        parent_double_separator_trailing.stderr, absolute.stderr,
        "parent-double-separator-trailing and absolute single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostics should include normalized parent-double-separator-trailing path: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn modules_command_parent_curdir_double_separator_single_file_root_matches_absolute_output() {
    let parent = unique_temp_dir("modules_parent_curdir_double_separator_single_file_root");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let file_path = parent.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write .dag file");

    let parent_curdir_double_separator = Command::new(daglang_bin())
        .arg("modules")
        .arg(".././one.dag//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-double-separator single-file-root daglang modules");
    assert!(
        !parent_curdir_double_separator.status.success(),
        "parent-curdir-double-separator single-file-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&file_path)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute single-file-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute single-file-root modules should fail"
    );

    assert_eq!(
        parent_curdir_double_separator.stdout, absolute.stdout,
        "parent-curdir-double-separator and absolute single-file-root modules stdout should match"
    );
    assert_eq!(
        parent_curdir_double_separator.stderr, absolute.stderr,
        "parent-curdir-double-separator and absolute single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostics should include normalized parent-curdir-double-separator path: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn modules_command_relative_and_absolute_non_directory_roots_are_equivalent() {
    let cwd = unique_temp_dir("modules_relative_absolute_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp directory");
    let root_file = cwd.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("input.txt")
        .current_dir(&cwd)
        .output()
        .expect("failed to run relative non-directory-root daglang modules");
    assert!(
        !relative.status.success(),
        "relative non-directory-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute non-directory-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute non-directory-root modules should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute non-directory-root modules stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute non-directory-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative.stderr).contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostic should contain normalized absolute path: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp directory");
}

#[test]
fn viz_self_renders_pipeline_mermaid() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg("--self")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz --self");

    assert!(
        output.status.success(),
        "viz --self command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("flowchart TB"));
    assert!(stdout.contains("discover_files"));
    assert!(stdout.contains("report_modules"));
}

#[test]
fn viz_self_output_is_deterministic_for_same_input() {
    let first = Command::new(daglang_bin())
        .arg("viz")
        .arg("--self")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang viz --self");
    assert!(first.status.success(), "first viz --self run should succeed");

    let second = Command::new(daglang_bin())
        .arg("viz")
        .arg("--self")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang viz --self");
    assert!(second.status.success(), "second viz --self run should succeed");

    assert_eq!(
        first.stdout, second.stdout,
        "viz --self output should be deterministic across runs"
    );
}

#[test]
fn viz_self_contains_expected_pipeline_edge_labels() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg("--self")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz --self");

    assert!(output.status.success(), "viz --self should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("files:files"),
        "viz --self should include discover->parse files edge label"
    );
    assert!(
        stdout.contains("diagnostics:diagnostics"),
        "viz --self should include diagnostics flow edges"
    );
    assert!(
        stdout.contains("module_graph:module_graph"),
        "viz --self should include build->report module graph edge label"
    );
}

#[test]
fn viz_self_matches_expected_mermaid_snapshot() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg("--self")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz --self");

    assert!(output.status.success(), "viz --self should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, expected_viz_self_mermaid());
}

#[test]
fn check_command_reports_file_line_col_for_broken_file() {
    let broken_file = unique_temp_file("broken");
    std::fs::write(&broken_file, "module tmp.bad\nfn broken( -> String {\n  \"oops\"\n}\n")
        .expect("failed to create broken dag file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check for broken file");

    assert!(!output.status.success(), "broken file should fail check");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    assert!(
        stderr.contains(":2:12:"),
        "expected file:line:col in stderr, got: {stderr}"
    );

    std::fs::remove_file(broken_file).expect("failed to remove broken dag file");
}

#[cfg(unix)]
#[test]
fn check_command_accepts_symlink_single_file_target() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let real = root.join("real.dag");
    let link = root.join("link.dag");
    std::fs::write(&real, "module sample.real\nfn ok() -> Unit {}")
        .expect("failed to write real source");
    symlink(&real, &link).expect("failed to create symlinked target");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on symlinked target");

    assert!(
        output.status.success(),
        "check should succeed for symlinked single-file target: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, expected_check_success_stdout(1));
    assert!(
        output.stderr.is_empty(),
        "symlinked single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_symlink_and_real_invalid_single_file_targets_are_equivalent() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_real_invalid_single_file_equivalent");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let real = root.join("real.dag");
    let link = root.join("link.dag");
    std::fs::write(&real, "module sample.broken\nfn")
        .expect("failed to write malformed real source");
    symlink(&real, &link).expect("failed to create symlinked target");

    let real_output = Command::new(daglang_bin())
        .arg("check")
        .arg(&real)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on real malformed target");
    assert!(
        !real_output.status.success(),
        "real malformed single-file target should fail"
    );

    let link_output = Command::new(daglang_bin())
        .arg("check")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on symlinked malformed target");
    assert!(
        !link_output.status.success(),
        "symlinked malformed single-file target should fail"
    );

    assert_eq!(
        real_output.stdout, link_output.stdout,
        "real and symlink invalid target check stdout should match"
    );
    assert_eq!(
        real_output.stderr, link_output.stderr,
        "real and symlink invalid target check stderr should match"
    );
    let stderr = String::from_utf8_lossy(&real_output.stderr);
    assert!(
        stderr.contains("real.dag:2:3:"),
        "expected canonicalized parse diagnostic path in stderr: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_dangling_symlink_single_file_target_exits_nonzero() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_dangling_symlink_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let dangling_target = root.join("missing.dag");
    let dangling_link = root.join("broken.dag");
    symlink(&dangling_target, &dangling_link).expect("failed to create dangling symlink target");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&dangling_link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on dangling symlink target");

    assert!(
        !output.status.success(),
        "check should fail for dangling symlink single-file target"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("failed to canonicalize"));
    assert!(
        stderr.contains("broken.dag"),
        "dangling symlink single-file failure should include offending path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_relative_and_absolute_dangling_symlink_targets_are_equivalent() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_relative_absolute_dangling_symlink_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let dangling_target = root.join("missing.dag");
    let dangling_link = root.join("broken.dag");
    symlink(&dangling_target, &dangling_link).expect("failed to create dangling symlink target");

    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative dangling-target daglang check");
    assert!(
        !relative.status.success(),
        "relative dangling-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&dangling_link)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute dangling-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute dangling-target check should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute dangling-target check stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute dangling-target check stderr should match"
    );
    let stderr = String::from_utf8_lossy(&relative.stderr);
    assert!(
        stderr.contains(&format!(
            "failed to canonicalize {}",
            dangling_link.display()
        )),
        "dangling-target diagnostics should include normalized absolute path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_relative_and_absolute_missing_single_file_targets_are_equivalent() {
    let root = unique_temp_dir("check_relative_absolute_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_file = root.join("missing.dag");

    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative missing-target daglang check");
    assert!(
        !relative.status.success(),
        "relative missing-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute missing-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-target check should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute missing-target check stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute missing-target check stderr should match"
    );
    let stderr = String::from_utf8_lossy(&relative.stderr);
    assert!(
        stderr.contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_parent_segment_missing_single_file_target_is_normalized_and_equivalent() {
    let parent = unique_temp_dir("check_parent_segment_missing_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_file = parent.join("missing.dag");

    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("../missing.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment relative missing-target daglang check");
    assert!(
        !relative.status.success(),
        "parent-segment relative missing-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-target check should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "parent-segment relative and absolute missing-target check stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "parent-segment relative and absolute missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include normalized parent-segment path: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_double_separator_missing_single_file_target_is_normalized_and_equivalent() {
    let parent = unique_temp_dir("check_parent_double_separator_missing_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_file = parent.join("missing.dag");

    let parent_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("..//missing.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator relative missing-target daglang check");
    assert!(
        !parent_double_separator.status.success(),
        "parent-double-separator relative missing-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-target check should fail"
    );

    assert_eq!(
        parent_double_separator.stdout, absolute.stdout,
        "parent-double-separator and absolute missing-target check stdout should match"
    );
    assert_eq!(
        parent_double_separator.stderr, absolute.stderr,
        "parent-double-separator and absolute missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include normalized parent-double-separator path: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_double_separator_trailing_slash_missing_single_file_target_is_normalized_and_equivalent(
) {
    let parent = unique_temp_dir("check_parent_double_separator_trailing_slash_missing_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_file = parent.join("missing.dag");

    let parent_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg("..//missing.dag/")
        .current_dir(&cwd)
        .output()
        .expect(
            "failed to run parent-double-separator-trailing relative missing-target daglang check",
        );
    assert!(
        !parent_double_separator_trailing.status.success(),
        "parent-double-separator-trailing relative missing-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-target check should fail"
    );

    assert_eq!(
        parent_double_separator_trailing.stdout, absolute.stdout,
        "parent-double-separator-trailing and absolute missing-target check stdout should match"
    );
    assert_eq!(
        parent_double_separator_trailing.stderr, absolute.stderr,
        "parent-double-separator-trailing and absolute missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include normalized parent-double-separator-trailing path: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_curdir_double_separator_missing_single_file_target_is_normalized_and_equivalent(
) {
    let parent = unique_temp_dir("check_parent_curdir_double_separator_missing_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_file = parent.join("missing.dag");

    let parent_curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".././missing.dag//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-double-separator relative missing-target daglang check");
    assert!(
        !parent_curdir_double_separator.status.success(),
        "parent-curdir-double-separator relative missing-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-target check should fail"
    );

    assert_eq!(
        parent_curdir_double_separator.stdout, absolute.stdout,
        "parent-curdir-double-separator and absolute missing-target check stdout should match"
    );
    assert_eq!(
        parent_curdir_double_separator.stderr, absolute.stderr,
        "parent-curdir-double-separator and absolute missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include normalized parent-curdir-double-separator path: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_missing_single_file_exits_nonzero() {
    let missing_file = unique_temp_file("missing_single_file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check for missing file");

    assert!(
        !output.status.success(),
        "check should fail when target .dag file does not exist"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("pipeline error"),
        "missing single-file path should surface pipeline error: {stderr}"
    );
    assert!(
        stderr.contains(&missing_file.display().to_string()),
        "missing single-file error should include offending path: {stderr}"
    );
}

#[test]
fn check_command_missing_directory_exits_nonzero() {
    let missing_dir = unique_temp_dir("missing_dir");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_dir)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check for missing directory");

    assert!(
        !output.status.success(),
        "check should fail when input directory does not exist"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root does not exist"));
    assert!(
        stderr.contains(&missing_dir.display().to_string()),
        "missing-root error should include path context: {stderr}"
    );
}

#[test]
fn check_command_non_directory_root_exits_nonzero() {
    let root_file = unique_temp_file("check_non_directory_root")
        .with_extension("txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check for non-directory root");

    assert!(
        !output.status.success(),
        "check should fail when root path is not a directory"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root is not a directory"));
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "non-directory root error should include offending path: {stderr}"
    );

    std::fs::remove_file(root_file).expect("failed to cleanup root file");
}

#[test]
fn check_command_reports_lex_diagnostic_for_unknown_character() {
    let bad_file = unique_temp_file("lex_bad");
    std::fs::write(&bad_file, "module tmp.bad\n$\n").expect("failed to create bad dag file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&bad_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check for lex-bad file");

    assert!(!output.status.success(), "lex-bad file should fail check");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(":2:1:"),
        "expected lexical diagnostic line/column, got: {stderr}"
    );
    assert!(stderr.contains("unexpected character '$'"));

    std::fs::remove_file(bad_file).expect("failed to remove bad dag file");
}

#[test]
fn modules_command_reports_graph_diagnostics_without_failing() {
    let root = unique_temp_dir("modules_diag");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nimport missing.dep\nfn ok() -> Unit {}",
    )
    .expect("failed to write temp dag file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for temp dir");

    assert!(
        output.status.success(),
        "modules command should still succeed while reporting diagnostics"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Diagnostics:"));
    assert!(stdout.contains("unresolved import"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_reports_parse_diagnostics_without_failing() {
    let root = unique_temp_dir("modules_parse_diag");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("broken.dag"),
        "module sample.broken\nfn broken( -> Unit {\n",
    )
    .expect("failed to write malformed dag file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for malformed source");

    assert!(
        output.status.success(),
        "modules command should succeed while surfacing parse diagnostics"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Diagnostics:"));
    assert!(stdout.contains("broken.dag:2:12:"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_reports_lex_diagnostics_without_failing() {
    let root = unique_temp_dir("modules_lex_diag");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("broken.dag"), "module sample.broken\n$\n")
        .expect("failed to write malformed dag file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for malformed source");

    assert!(
        output.status.success(),
        "modules command should succeed while surfacing lex diagnostics"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Diagnostics:"));
    assert!(stdout.contains("broken.dag:2:1:"));
    assert!(stdout.contains("unexpected character '$'"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_sorts_lex_diagnostics_before_resolve_diagnostics() {
    let root = unique_temp_dir("modules_diag_kind_order");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("z_resolve.dag"),
        "module sample.resolve\nimport missing.dep\nfn ok() -> Unit {}",
    )
    .expect("failed to write resolve-error source");
    std::fs::write(root.join("a_lex.dag"), "module sample.lex\n$\n")
        .expect("failed to write lex-error source");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for mixed diagnostics");

    assert!(
        output.status.success(),
        "modules command should remain non-failing when diagnostics are present"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let diagnostics = stdout
        .split("Diagnostics:\n")
        .nth(1)
        .expect("expected diagnostics section in modules output");
    let first = diagnostics
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("expected at least one diagnostic line");
    assert!(
        first.contains("a_lex.dag"),
        "lex diagnostics should be sorted before resolve diagnostics: {stdout}"
    );
    assert!(stdout.contains("z_resolve.dag"));
    assert!(stdout.contains("unexpected character '$'"));
    assert!(stdout.contains("unresolved import"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_sorts_parse_diagnostics_before_resolve_diagnostics() {
    let root = unique_temp_dir("modules_parse_resolve_order");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("a_parse.dag"), "module sample.parse\nfn")
        .expect("failed to write parse-error source");
    std::fs::write(
        root.join("z_resolve.dag"),
        "module sample.resolve\nimport missing.dep\nfn ok() -> Unit {}",
    )
    .expect("failed to write resolve-error source");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for mixed parse/resolve diagnostics");

    assert!(
        output.status.success(),
        "modules command should remain non-failing when diagnostics are present"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let diagnostics = stdout
        .split("Diagnostics:\n")
        .nth(1)
        .expect("expected diagnostics section in modules output");
    let first = diagnostics
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("expected at least one diagnostic line");
    assert!(
        first.contains("a_parse.dag"),
        "parse diagnostics should sort before resolve diagnostics: {stdout}"
    );
    assert!(stdout.contains("z_resolve.dag"));
    assert!(stdout.contains("unresolved import"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_reports_cycle_diagnostics_without_failing() {
    let root = unique_temp_dir("modules_cycle_diag");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("a.dag"),
        "module cycle.a\nimport cycle.b\nfn a() -> Unit {}",
    )
    .expect("failed to write cycle a");
    std::fs::write(
        root.join("b.dag"),
        "module cycle.b\nimport cycle.a\nfn b() -> Unit {}",
    )
    .expect("failed to write cycle b");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for cycle dir");

    assert!(
        output.status.success(),
        "modules command should still succeed while reporting cycle diagnostics"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Diagnostics:"));
    assert!(stdout.contains("cyclic dependencies detected"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_reports_duplicate_module_diagnostics_without_failing() {
    let root = unique_temp_dir("modules_duplicate_diag");
    std::fs::create_dir_all(root.join("a")).expect("failed to create temp dir a");
    std::fs::create_dir_all(root.join("b")).expect("failed to create temp dir b");
    std::fs::write(
        root.join("a/one.dag"),
        "module dup.mod\nfn one() -> Unit {}",
    )
    .expect("failed to write file one");
    std::fs::write(
        root.join("b/two.dag"),
        "module dup.mod\nfn two() -> Unit {}",
    )
    .expect("failed to write file two");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for duplicate dir");

    assert!(
        output.status.success(),
        "modules command should still succeed while reporting duplicate diagnostics"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Diagnostics:"));
    assert!(stdout.contains("duplicate module path"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_diagnostic_output_is_deterministic_for_same_input() {
    let root = unique_temp_dir("modules_diag_deterministic");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nimport missing.dep\nfn ok() -> Unit {}",
    )
    .expect("failed to write temp dag file");

    let first = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first modules command");
    assert!(first.status.success(), "first modules run should succeed");

    let second = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second modules command");
    assert!(second.status.success(), "second modules run should succeed");

    assert_eq!(
        first.stdout, second.stdout,
        "modules diagnostic output should be deterministic"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_single_file_mode_ignores_sibling_broken_files() {
    let root = unique_temp_dir("single_file_mode");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    let good_file = root.join("good.dag");
    std::fs::write(&good_file, "module sample.good\nfn ok() -> Unit {}")
        .expect("failed to write good file");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write broken sibling");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&good_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on single file");

    assert!(
        output.status.success(),
        "single-file check should succeed even with sibling broken file: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, expected_check_success_stdout(1));
    assert!(
        output.stderr.is_empty(),
        "single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_directory_mode_aggregates_multiple_file_diagnostics() {
    let root = unique_temp_dir("directory_mode_errors");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("broken_a.dag"), "module sample.a\nfn")
        .expect("failed to write broken_a");
    std::fs::write(root.join("broken_b.dag"), "module sample.b\nimport")
        .expect("failed to write broken_b");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on directory");

    assert!(
        !output.status.success(),
        "directory check should fail when multiple files are invalid"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    assert!(stderr.contains("broken_a.dag"));
    assert!(stderr.contains("broken_b.dag"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_directory_mode_outputs_deterministic_diagnostic_order() {
    let root = unique_temp_dir("directory_mode_order");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("z_broken.dag"), "module sample.z\nfn")
        .expect("failed to write z_broken");
    std::fs::write(root.join("a_broken.dag"), "module sample.a\nfn")
        .expect("failed to write a_broken");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on directory");

    assert!(
        !output.status.success(),
        "directory check should fail when files are invalid"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut lines = stderr.lines().filter(|line| !line.trim().is_empty());
    let first_line = lines.next().expect("expected at least one diagnostic line");
    assert!(
        first_line.contains("a_broken.dag"),
        "diagnostics should be deterministically sorted by path: {stderr}"
    );
    assert!(stderr.contains("z_broken.dag"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_diagnostic_output_is_deterministic_for_same_input() {
    let root = unique_temp_dir("check_output_deterministic");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("z_lex.dag"), "module sample.lex\n$\n")
        .expect("failed to write z_lex");
    std::fs::write(root.join("a_parse.dag"), "module sample.parse\nfn")
        .expect("failed to write a_parse");

    let first = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang check");
    assert!(!first.status.success(), "first check run should fail");

    let second = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang check");
    assert!(!second.status.success(), "second check run should fail");

    assert_eq!(
        first.stderr, second.stderr,
        "check diagnostics should be deterministic for identical inputs"
    );
    assert_eq!(
        first.stdout, second.stdout,
        "check stdout should be deterministic for identical inputs"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_sorts_lex_diagnostics_before_parse_diagnostics() {
    let root = unique_temp_dir("check_kind_order");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("a_parse.dag"), "module sample.parse\nfn")
        .expect("failed to write parse-error file");
    std::fs::write(root.join("z_lex.dag"), "module sample.lex\n$\n")
        .expect("failed to write lex-error file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on mixed-error directory");

    assert!(
        !output.status.success(),
        "check should fail when diagnostics are present"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let first_line = stderr
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("expected at least one diagnostic line");
    assert!(
        first_line.contains("z_lex.dag"),
        "lex diagnostics should sort before parse diagnostics: {stderr}"
    );
    assert!(stderr.contains("a_parse.dag"));
    assert!(stderr.contains("unexpected character '$'"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_empty_directory_succeeds_with_zero_files() {
    let root = unique_temp_dir("check_empty_dir");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on empty directory");

    assert!(
        output.status.success(),
        "check should succeed for an empty directory"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, expected_check_success_stdout(0));
    assert!(
        output.stderr.is_empty(),
        "empty-directory check should not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_ignores_non_dag_files_in_directory() {
    let root = unique_temp_dir("check_ignore_non_dag");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("notes.txt"), "module fake\n$").expect("failed to write txt file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on non-dag-only directory");

    assert!(
        output.status.success(),
        "check should ignore non-.dag files in directory mode"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OK: parsed 0 file(s)"),
        "non-.dag files should be ignored during check: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_ignores_non_dag_files_when_dag_files_exist() {
    let root = unique_temp_dir("check_ignore_non_dag_mixed");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("good.dag"), "module sample.good\nfn ok() -> Unit {}")
        .expect("failed to write dag file");
    std::fs::write(root.join("notes.txt"), "module fake\n$").expect("failed to write txt file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on mixed dag/non-dag directory");

    assert!(
        output.status.success(),
        "check should parse only .dag files and ignore non-.dag files"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OK: parsed 1 file(s)"),
        "expected only the .dag file to be parsed: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_does_not_run_resolve_stage_for_unresolved_imports() {
    let root = unique_temp_dir("check_parse_only_unresolved_import");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nimport missing.dep\nfn ok() -> Unit {}",
    )
    .expect("failed to write dag file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on unresolved-import source");

    assert!(
        output.status.success(),
        "check should parse successfully without resolve-stage diagnostics"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("OK: parsed 1 file(s)"));
    assert!(
        stderr.trim().is_empty(),
        "check should not emit resolve diagnostics: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_defaults_to_workspace_dsl_root() {
    let output = Command::new(daglang_bin())
        .arg("check")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check with default root");

    assert!(
        output.status.success(),
        "default check command should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("OK: parsed 42 file(s)"),
        "default check should parse full DSL corpus"
    );
}

#[test]
fn check_command_default_root_missing_in_cwd_exits_nonzero() {
    let cwd = unique_temp_dir("check_default_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_default_root = cwd.join("dsl");

    let output = Command::new(daglang_bin())
        .arg("check")
        .current_dir(&cwd)
        .output()
        .expect("failed to run daglang check with missing default root");

    assert!(
        !output.status.success(),
        "default check should fail when cwd lacks dsl/ root"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root does not exist"));
    assert!(
        stderr.contains(&missing_default_root.display().to_string()),
        "default-root check error should include resolved cwd/dsl path: {stderr}"
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_default_root_non_directory_in_cwd_exits_nonzero() {
    let cwd = unique_temp_dir("check_default_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let non_directory_default_root = cwd.join("dsl");
    std::fs::write(&non_directory_default_root, "not a directory")
        .expect("failed to create non-directory default root");

    let output = Command::new(daglang_bin())
        .arg("check")
        .current_dir(&cwd)
        .output()
        .expect("failed to run daglang check with non-directory default root");

    assert!(
        !output.status.success(),
        "default check should fail when cwd/dsl exists as a file"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root is not a directory"));
    assert!(
        stderr.contains(&non_directory_default_root.display().to_string()),
        "default-root check non-directory error should include resolved cwd/dsl path: {stderr}"
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_defaults_to_workspace_dsl_root() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules with default root");

    assert!(
        output.status.success(),
        "default modules command should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Discovered modules:"));
    let reported_modules = reported_modules_sorted(&stdout);
    let expected_modules: Vec<String> = expected_dsl_modules_sorted()
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        reported_modules, expected_modules,
        "default modules command should report the complete dsl corpus"
    );
}

#[test]
fn modules_command_default_root_missing_in_cwd_exits_nonzero() {
    let cwd = unique_temp_dir("modules_default_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_default_root = cwd.join("dsl");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .current_dir(&cwd)
        .output()
        .expect("failed to run daglang modules with missing default root");

    assert!(
        !output.status.success(),
        "default modules should fail when cwd lacks dsl/ root"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root does not exist"));
    assert!(
        stderr.contains(&missing_default_root.display().to_string()),
        "default-root modules error should include resolved cwd/dsl path: {stderr}"
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_default_root_non_directory_in_cwd_exits_nonzero() {
    let cwd = unique_temp_dir("modules_default_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let non_directory_default_root = cwd.join("dsl");
    std::fs::write(&non_directory_default_root, "not a directory")
        .expect("failed to create non-directory default root");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .current_dir(&cwd)
        .output()
        .expect("failed to run daglang modules with non-directory default root");

    assert!(
        !output.status.success(),
        "default modules should fail when cwd/dsl exists as a file"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root is not a directory"));
    assert!(
        stderr.contains(&non_directory_default_root.display().to_string()),
        "default-root modules non-directory error should include resolved cwd/dsl path: {stderr}"
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_output_is_deterministic_for_same_input() {
    let first = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang modules");
    assert!(first.status.success(), "first modules run should succeed");

    let second = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang modules");
    assert!(second.status.success(), "second modules run should succeed");

    assert_eq!(
        first.stdout, second.stdout,
        "modules output should be deterministic for identical inputs"
    );
}

#[cfg(unix)]
#[test]
fn check_command_accepts_symlink_root_directory() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_root");
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir_all(&real).expect("failed to create real root");
    std::fs::write(real.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write source");
    symlink(&real, &link).expect("failed to create root symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on symlink root");

    assert!(
        output.status.success(),
        "check should succeed for symlink root: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OK: parsed 1 file(s)"),
        "symlink-root check should parse exactly one file: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_handles_directory_symlink_cycle_root() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_cycle_root");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested root");
    std::fs::write(nested.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write source");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on symlink-cycle root");

    assert!(
        output.status.success(),
        "check should succeed for directory symlink cycle root: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OK: parsed 1 file(s)"),
        "symlink-cycle root should parse exactly one file: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_symlink_alias_root_order_is_deterministic_for_errors() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_alias_root_order_errors");
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir_all(&real).expect("failed to create real root");
    std::fs::write(real.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write invalid source");
    symlink(&real, &link).expect("failed to create root symlink");

    let first = Command::new(daglang_bin())
        .arg("check")
        .arg(&real)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang check");
    let second = Command::new(daglang_bin())
        .arg("check")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang check");

    assert!(!first.status.success(), "first run should fail");
    assert!(!second.status.success(), "second run should fail");
    assert_eq!(
        first.stderr, second.stderr,
        "check diagnostics should match for real/symlink root aliases"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_symlink_alias_root_order_is_deterministic_for_success() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_alias_root_order_success");
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir_all(&real).expect("failed to create real root");
    std::fs::write(real.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write source");
    symlink(&real, &link).expect("failed to create root symlink");

    let first = Command::new(daglang_bin())
        .arg("check")
        .arg(&real)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang check");
    let second = Command::new(daglang_bin())
        .arg("check")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang check");

    assert!(first.status.success(), "first run should succeed");
    assert!(second.status.success(), "second run should succeed");
    assert_eq!(
        first.stdout, second.stdout,
        "check output should match for real/symlink root aliases"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_deduplicates_parse_errors_in_directory_symlink_cycle_root() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_cycle_parse_errors");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested root");
    std::fs::write(nested.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write invalid source");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on symlink-cycle parse-error root");

    assert!(
        !output.status.success(),
        "check should fail when the symlink-cycle root contains parse errors"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let broken_hits = stderr.matches("broken.dag:").count();
    assert_eq!(
        broken_hits, 1,
        "symlink-cycle traversal should not duplicate parse diagnostics: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_output_is_deterministic_in_directory_symlink_cycle_with_errors() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_cycle_deterministic_errors");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested root");
    std::fs::write(nested.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write invalid source");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let first = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang check on symlink-cycle parse-error root");
    let second = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang check on symlink-cycle parse-error root");

    assert!(!first.status.success(), "first run should fail");
    assert!(!second.status.success(), "second run should fail");
    assert_eq!(
        first.stderr, second.stderr,
        "check stderr should be deterministic under symlink-cycle parse-error input"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_symlink_root_without_module_decl_uses_path_fallback() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_symlink_root_module_fallback");
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir_all(real.join("nested")).expect("failed to create nested root");
    std::fs::write(real.join("nested/no_module.dag"), "fn ok() -> Unit {}")
        .expect("failed to write source");
    symlink(&real, &link).expect("failed to create root symlink");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on symlink root");

    assert!(
        output.status.success(),
        "modules should succeed for symlink root: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("nested.no_module"),
        "symlink-root module fallback should render path-derived module: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_symlink_alias_root_order_is_deterministic_for_success() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_symlink_alias_root_order_success");
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir_all(&real).expect("failed to create real root");
    std::fs::write(real.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write source");
    symlink(&real, &link).expect("failed to create root symlink");

    let first = Command::new(daglang_bin())
        .arg("modules")
        .arg(&real)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang modules");
    let second = Command::new(daglang_bin())
        .arg("modules")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang modules");

    assert!(first.status.success(), "first run should succeed");
    assert!(second.status.success(), "second run should succeed");
    assert_eq!(
        first.stdout, second.stdout,
        "modules output should match for real/symlink root aliases"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_symlink_alias_root_order_is_deterministic_for_errors() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_symlink_alias_root_order_errors");
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir_all(&real).expect("failed to create real root");
    std::fs::write(real.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write invalid source");
    symlink(&real, &link).expect("failed to create root symlink");

    let first = Command::new(daglang_bin())
        .arg("modules")
        .arg(&real)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang modules");
    let second = Command::new(daglang_bin())
        .arg("modules")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang modules");

    assert!(first.status.success(), "first run should succeed");
    assert!(second.status.success(), "second run should succeed");
    assert_eq!(
        first.stdout, second.stdout,
        "modules diagnostics/report should match for real/symlink root aliases"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_deduplicates_parse_errors_in_directory_symlink_cycle_root() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_symlink_cycle_parse_errors");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested root");
    std::fs::write(nested.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write invalid source");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on symlink-cycle parse-error root");

    assert!(
        output.status.success(),
        "modules should still succeed while reporting parse diagnostics"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let broken_hits = stdout.matches("broken.dag:").count();
    assert_eq!(
        broken_hits, 1,
        "symlink-cycle traversal should not duplicate parse diagnostics in modules output: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_output_is_deterministic_in_directory_symlink_cycle_with_errors() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_symlink_cycle_deterministic_errors");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested root");
    std::fs::write(nested.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write invalid source");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let first = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang modules on symlink-cycle parse-error root");
    let second = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang modules on symlink-cycle parse-error root");

    assert!(first.status.success(), "first run should succeed");
    assert!(second.status.success(), "second run should succeed");
    assert_eq!(
        first.stdout, second.stdout,
        "modules stdout should be deterministic under symlink-cycle parse-error input"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_handles_directory_symlink_cycle_root() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_symlink_cycle_root");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested root");
    std::fs::write(nested.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write source");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on symlink-cycle root");

    assert!(
        output.status.success(),
        "modules should succeed for directory symlink cycle root: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("sample.main"),
        "symlink-cycle root should still render discovered module: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_dangling_dag_symlink_in_root_exits_nonzero() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_dangling_symlink_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let dangling_target = root.join("missing.dag");
    let dangling_link = root.join("broken.dag");
    symlink(&dangling_target, &dangling_link).expect("failed to create dangling symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on dangling symlink root");

    assert!(
        !output.status.success(),
        "check should fail when root contains dangling .dag symlink"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("failed to canonicalize"));
    assert!(
        stderr.contains("broken.dag"),
        "dangling symlink failure should include offending path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_dangling_dag_symlink_in_root_exits_nonzero() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_dangling_symlink_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let dangling_target = root.join("missing.dag");
    let dangling_link = root.join("broken.dag");
    symlink(&dangling_target, &dangling_link).expect("failed to create dangling symlink");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on dangling symlink root");

    assert!(
        !output.status.success(),
        "modules should fail when root contains dangling .dag symlink"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("failed to canonicalize"));
    assert!(
        stderr.contains("broken.dag"),
        "dangling symlink failure should include offending path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn modules_command_empty_directory_succeeds_without_diagnostics() {
    let root = unique_temp_dir("modules_empty_dir");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on empty directory");

    assert!(
        output.status.success(),
        "modules should succeed for an empty directory"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Discovered modules:"),
        "modules output should include summary header"
    );
    assert!(
        !stdout.contains("Diagnostics:"),
        "empty directory should not produce diagnostics: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_ignores_non_dag_files_in_directory() {
    let root = unique_temp_dir("modules_ignore_non_dag");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("notes.txt"), "module fake\n$").expect("failed to write txt file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on non-dag-only directory");

    assert!(
        output.status.success(),
        "modules should ignore non-.dag files in directory mode"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Discovered modules:"),
        "modules output should include summary header"
    );
    assert!(
        !stdout.contains("Diagnostics:"),
        "non-.dag files should not produce diagnostics: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_ignores_non_dag_files_when_dag_files_exist() {
    let root = unique_temp_dir("modules_ignore_non_dag_mixed");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write dag file");
    std::fs::write(root.join("notes.txt"), "module fake\n$").expect("failed to write txt file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on mixed dag/non-dag directory");

    assert!(
        output.status.success(),
        "modules should report only discovered .dag modules"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sample.main"));
    assert!(
        !stdout.contains("Diagnostics:"),
        "ignored non-.dag files should not produce diagnostics: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_missing_directory_exits_nonzero() {
    let missing_dir = unique_temp_dir("modules_missing_dir");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&missing_dir)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for missing directory");

    assert!(
        !output.status.success(),
        "modules should fail when input directory does not exist"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root does not exist"));
    assert!(
        stderr.contains(&missing_dir.display().to_string()),
        "missing-root error should include path context: {stderr}"
    );
}

#[test]
fn modules_command_non_directory_root_exits_nonzero() {
    let root_file = unique_temp_file("modules_non_directory_root")
        .with_extension("txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for non-directory root");

    assert!(
        !output.status.success(),
        "modules should fail when root path is not a directory"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root is not a directory"));
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "non-directory root error should include offending path: {stderr}"
    );

    std::fs::remove_file(root_file).expect("failed to cleanup root file");
}

#[test]
fn modules_command_single_dag_file_path_exits_nonzero() {
    let file_path = unique_temp_file("modules_single_file_root");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to create .dag file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&file_path)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for single-file path");

    assert!(
        !output.status.success(),
        "modules should fail when given a file path instead of directory root"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root is not a directory"));
    assert!(
        stderr.contains(&file_path.display().to_string()),
        "single-file-root error should include offending path: {stderr}"
    );

    std::fs::remove_file(file_path).expect("failed to cleanup .dag file");
}

#[test]
fn modules_command_relative_and_absolute_single_file_roots_are_equivalent() {
    let root = unique_temp_dir("modules_relative_absolute_single_file_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let file_path = root.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write temp dag file");

    let relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("one.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative single-file-root daglang modules");
    assert!(
        !relative.status.success(),
        "relative single-file-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&file_path)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute single-file-root daglang modules");
    assert!(
        !absolute.status.success(),
        "absolute single-file-root modules should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute single-file-root modules stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute single-file-root modules stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative.stderr).contains(&format!(
            "input root is not a directory: {}",
            file_path.display()
        )),
        "single-file-root diagnostic should contain normalized absolute path: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn unknown_command_exits_nonzero_with_message() {
    let output = Command::new(daglang_bin())
        .arg("unknown-cmd")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang with unknown command");

    assert!(
        !output.status.success(),
        "unknown command should fail with non-zero exit code"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown command"));
}

#[test]
fn compile_family_commands_execute_real_pipeline_paths() {
    for command in ["expand", "manifest", "compile"] {
        let output = Command::new(daglang_bin())
            .arg(command)
            .arg("dsl/tools/makegen.dag")
            .current_dir(workspace_root())
            .output()
            .unwrap_or_else(|err| panic!("failed to run {command}: {err}"));
        assert!(
            output.status.success(),
            "{command} should execute successfully for makegen fixture"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("TODO"),
            "{command} should no longer emit TODO placeholder output: {stderr}"
        );
        assert!(
            !output.stdout.is_empty(),
            "{command} should emit meaningful stdout output"
        );
    }
}

#[test]
fn viz_without_self_emits_compiled_mermaid_graph() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg("dsl/tools/makegen.dag")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz");

    assert!(
        output.status.success(),
        "viz should compile and render mermaid output"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("flowchart TB"),
        "viz mermaid output should include flowchart header: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("TODO"),
        "viz should not emit TODO placeholder output: {stderr}"
    );
}
