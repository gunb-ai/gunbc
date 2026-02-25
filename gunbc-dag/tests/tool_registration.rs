use daglang_driver::{compile_from_context, DriverContext};
use daglang_lower::LoweredOp;
use gunbc_dag::extern_impls::lookup_extern_impl;
use gunbc_dag::dsl_registry::discover_tool_defs_from_dsl;
use gunbc_dag::makegen::{BuildConfig, ToolInfo, ToolRegistry};
use gunbc_infra::workspace_model::{baseline_commit_policies, CommitReason};
use gunbc_ir::cargo::Warnings;
use gunbc_ir::node::NodeBody;
use gunbc_ir::resource::ResourceIo;
use gunbc_lib_transport::TransportIo;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

/// Helper: get DSL-derived tool defs (the single source of truth).
fn dsl_tools() -> Vec<gunbc_codegen::registry::ToolDef> {
    discover_tool_defs_from_dsl()
}

/// Verify that DSL discovery returns all expected tools.
#[test]
fn dsl_discovery_finds_expected_tools() {
    let tools = dsl_tools();
    let tool_names: HashSet<&str> = tools.iter().map(|t| t.meta.tool_name.as_ref()).collect();

    let expected = [
        "bootstrap",
        "deps",
        "gist",
        "gist-diff",
        "gist-recent",
        "makegen",
        "pragma",
        "testgen",
    ];
    for name in &expected {
        assert!(
            tool_names.contains(name),
            "expected tool '{}' not found in discover_tool_defs_from_dsl(). Got: {:?}",
            name,
            tool_names
        );
    }

    for tool in &tools {
        if let Some(inv) = &tool.invocation {
            assert!(
                !inv.binary.is_empty(),
                "tool '{}' has empty binary in invocation",
                tool.meta.tool_name
            );
        }
    }

    assert_eq!(
        tool_names.len(),
        tools.len(),
        "duplicate tool names in DSL discovery"
    );
}

#[test]
fn makegen_default_registry_matches_dsl_tools_plus_manual_targets() {
    let registry = ToolRegistry::default_registry();

    // Core workflow names that filter out DSL tool collisions.
    let core_names: HashSet<String> = registry
        .core_workflows
        .iter()
        .map(|w| w.name.clone())
        .collect();

    // Manual tools overridden to needs_generated_cli = false.
    let manual_names: HashSet<&str> = registry
        .tools
        .iter()
        .filter(|tool| !tool.needs_generated_cli)
        .map(|tool| tool.short_name.as_str())
        .collect();

    // DSL-derived tools with invocations, minus core workflow collisions
    // and manual overrides.
    let derived_with_invocation: HashSet<String> = dsl_tools()
        .into_iter()
        .filter(|tool| tool.invocation.is_some())
        .filter(|tool| !core_names.contains(tool.meta.tool_name.as_ref()))
        .filter(|tool| !manual_names.contains(tool.meta.tool_name.as_ref()))
        .map(|tool| tool.meta.tool_name.to_string())
        .collect();

    let makegen_generated: HashSet<String> = registry
        .tools
        .iter()
        .filter(|tool| tool.needs_generated_cli)
        .map(|tool| tool.short_name.clone())
        .collect();

    assert_eq!(
        makegen_generated, derived_with_invocation,
        "makegen generated-cli tools must stay in lockstep with DSL-derived tools \
         (excluding core workflow collisions and manual overrides)"
    );

    // Manual targets: tools with hand-written binaries (no generated CLI).
    let expected_manual: HashSet<&str> = HashSet::from(["pragma"]);
    assert_eq!(
        manual_names, expected_manual,
        "makegen manual targets must stay explicit and auditable"
    );
}

#[test]
fn codegen_cli_discovery_avoids_tool_registry_inventory() {
    let codegen_cli = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("bin")
        .join("codegen_cli.rs");
    let io = TransportIo::new();
    let content = String::from_utf8(
        io.read_file(&codegen_cli)
            .expect("codegen_cli source should be readable"),
    )
    .expect("codegen_cli source should be UTF-8");

    assert!(
        !content.contains("iter_tool_targets"),
        "codegen_cli must not depend on iter_tool_targets() inventory"
    );
    assert!(
        !content.contains("clippy_tool") && !content.contains("deps_tool"),
        "codegen_cli must not use force-link tool symbol touch points for discovery"
    );
}

fn dsl_compile_context() -> (PathBuf, PathBuf) {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let dsl_root = workspace_root.join("dsl");
    (workspace_root, dsl_root)
}

#[allow(clippy::disallowed_methods)]
fn discover_dsl_tool_stems() -> Vec<String> {
    let (_workspace_root, dsl_root) = dsl_compile_context();
    let tools_dir = dsl_root.join("tools");
    let mut stems = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&tools_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "dag") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    stems.push(stem.to_string());
                }
            }
        }
    }
    stems.sort();
    stems
}

#[test]
fn dsl_tool_modules_compile() {
    let (_workspace_root, dsl_root) = dsl_compile_context();
    let mut compile_errors = Vec::new();

    for stem in discover_dsl_tool_stems() {
        let dag_file = dsl_root.join(format!("tools/{stem}.dag"));
        let context = DriverContext {
            roots: vec![dsl_root.clone()],
            target_file: Some(dag_file),
        };
        if let Err(error) = compile_from_context(&context) {
            compile_errors.push(format!("dsl/tools/{stem}.dag: {error}"));
        }
    }

    assert!(
        compile_errors.is_empty(),
        "DSL tool modules failed to compile:\n{}",
        compile_errors.join("\n"),
    );
}

#[test]
fn tool_declared_outputs_match_dsl_compilation() {
    let (_workspace_root, dsl_root) = dsl_compile_context();
    let tools = dsl_tools();

    for tool in &tools {
        if tool.meta.tool_name == "testgen" {
            continue;
        }

        let stem = tool.meta.tool_name.replace('-', "_");
        let dag_file = dsl_root.join(format!("tools/{stem}.dag"));
        if !dag_file.exists() {
            continue;
        }

        let context = DriverContext {
            roots: vec![dsl_root.clone()],
            target_file: Some(dag_file),
        };
        let output = compile_from_context(&context).unwrap_or_else(|e| {
            panic!(
                "tool '{}' (dsl/tools/{stem}.dag) should compile: {e}",
                tool.meta.tool_name,
            )
        });

        let dsl_paths: BTreeSet<&str> = output.output_paths.iter().map(|s| s.as_str()).collect();
        let tool_paths: BTreeSet<&str> = tool.outputs.iter().map(|s| s.as_str()).collect();

        assert_eq!(
            dsl_paths, tool_paths,
            "output path drift for tool '{}': compiled={dsl_paths:?}, discovery={tool_paths:?}",
            tool.meta.tool_name,
        );
    }
}

/// Bootstrap seed files: generated but committed. Derived from the single
/// source of truth in `baseline_commit_policies()` (workspace_model).
fn committed_seed_files() -> Vec<&'static str> {
    baseline_commit_policies()
        .into_iter()
        .filter(|p| p.reason == CommitReason::BootstrapSeed)
        .map(|p| p.pattern)
        .collect()
}

#[test]
#[allow(clippy::disallowed_methods)]
fn no_generated_files_committed() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let tools = dsl_tools();
    let mut violations = Vec::new();

    for tool in &tools {
        for pattern in &tool.outputs {
            if committed_seed_files().contains(&pattern.as_str()) {
                continue;
            }
            let output = std::process::Command::new("git")
                .args(["ls-files", pattern])
                .current_dir(&workspace_root)
                .output();
            let Ok(output) = output else { continue };
            let stdout = String::from_utf8_lossy(&output.stdout);
            let tracked: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
            if !tracked.is_empty() {
                violations.push(format!(
                    "tool '{}' output '{}' matches tracked files: {:?}",
                    tool.meta.tool_name, pattern, tracked
                ));
            }
        }
    }

    assert!(violations.is_empty(), "generated files committed:\n{}", violations.join("\n"));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn all_tool_outputs_gitignored() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let tools = dsl_tools();
    let mut not_ignored = Vec::new();

    for tool in &tools {
        for pattern in &tool.outputs {
            if pattern.contains('*') || pattern.contains('?') { continue; }
            if committed_seed_files().contains(&pattern.as_str()) { continue; }
            let status = std::process::Command::new("git")
                .args(["check-ignore", "-q", pattern])
                .current_dir(&workspace_root)
                .status();
            let Ok(status) = status else { continue };
            if !status.success() {
                not_ignored.push(format!(
                    "tool '{}' output '{}' is not gitignored",
                    tool.meta.tool_name, pattern
                ));
            }
        }
    }

    assert!(not_ignored.is_empty(), "tool outputs not gitignored:\n{}", not_ignored.join("\n"));
}

// ============================================================================
// DSL Single Authority
// ============================================================================

#[test]
fn dsl_is_single_authority() {
    let dsl_names: BTreeSet<String> = dsl_tools()
        .into_iter()
        .map(|t| t.meta.tool_name.to_string())
        .collect();

    let registry = ToolRegistry::default_registry();
    let makegen_generated: BTreeSet<String> = registry
        .tools
        .iter()
        .filter(|t| t.needs_generated_cli)
        .map(|t| t.short_name.clone())
        .collect();

    let outside_dsl: BTreeSet<&String> = makegen_generated
        .iter()
        .filter(|name| !dsl_names.contains(name.as_str()))
        .collect();
    assert!(
        outside_dsl.is_empty(),
        "makegen has generated-CLI tools not in DSL discovery: {:?}",
        outside_dsl,
    );

    assert!(dsl_names.len() >= 8, "DSL discovery too few tools ({})", dsl_names.len());
}

#[test]
fn workspace_binary_invocations_are_consistent() {
    use gunbc_dag::WorkspaceBinary;

    let mut violations = Vec::new();
    for binary in WorkspaceBinary::ALL {
        let inv = binary.invocation();
        if inv.binary.is_empty() {
            violations.push(format!("WorkspaceBinary '{}' has empty binary", binary.tool_name()));
        }
    }

    assert!(violations.is_empty(), "violations:\n{}", violations.join("\n"));
}

#[test]
fn workspace_binary_enum_covers_dsl_tools() {
    use gunbc_dag::WorkspaceBinary;

    // Tools that exist as DSL entrypoints but are intentionally NOT in
    // the WorkspaceBinary dispatch enum. Includes newly-inferred entrypoints
    // that haven't been promoted to standalone CLI binaries yet.
    let non_workspace_dispatch: BTreeSet<&str> = BTreeSet::from([
        "build-all",
        "clippy-lint",
        "codegen-ensure",
        "deps",
        "deps-generate",
        "docgen",
        "review",
    ]);

    let enum_binaries: BTreeSet<&str> = WorkspaceBinary::ALL.iter().map(|b| b.tool_name()).collect();
    let dsl_invocable: BTreeSet<String> = dsl_tools()
        .into_iter()
        .filter(|tool| tool.invocation.is_some())
        .map(|tool| tool.meta.tool_name.to_string())
        .collect();

    let missing: BTreeSet<&String> = dsl_invocable
        .iter()
        .filter(|name| !enum_binaries.contains(name.as_str()))
        .filter(|name| !non_workspace_dispatch.contains(name.as_str()))
        .collect();
    assert!(missing.is_empty(), "DSL tools missing from WorkspaceBinary: {:?}", missing);
}

// ============================================================================
// ToolDef -> ToolInfo Roundtrip Contract Tests
// ============================================================================

#[test]
fn repeatable_flags_survive_roundtrip() {
    let tools = dsl_tools();
    let mut checked = 0usize;

    for tool in &tools {
        let Some(info) = ToolInfo::from_tool_def(tool) else { continue };
        for ep in &tool.entrypoints {
            let Some(ref make_var) = ep.make_var else { continue };
            let param = info.entrypoints.iter().find(|p| p.make_var == *make_var)
                .unwrap_or_else(|| panic!("tool '{}' ep '{}' make_var='{}' not in ToolInfo",
                    tool.meta.tool_name, ep.port_name, make_var));

            if ep.cardinality.allows_many() {
                assert!(param.repeatable, "tool '{}' ep '{}': allows_many but not repeatable",
                    tool.meta.tool_name, ep.port_name);
            } else {
                assert!(!param.repeatable, "tool '{}' ep '{}': not allows_many but repeatable",
                    tool.meta.tool_name, ep.port_name);
            }
            checked += 1;
        }
    }

    assert!(checked > 0, "no entrypoints with make_var found");
}

#[test]
fn default_values_survive_roundtrip() {
    let tools = dsl_tools();
    let mut checked = 0usize;

    for tool in &tools {
        let Some(info) = ToolInfo::from_tool_def(tool) else { continue };
        for ep in &tool.entrypoints {
            let Some(ref make_var) = ep.make_var else { continue };
            let param = info.entrypoints.iter().find(|p| p.make_var == *make_var)
                .unwrap_or_else(|| panic!("tool '{}' ep '{}' make_var='{}' not in ToolInfo",
                    tool.meta.tool_name, ep.port_name, make_var));

            assert_eq!(ep.default_value, param.default,
                "tool '{}' ep '{}' (make_var={}): default mismatch",
                tool.meta.tool_name, ep.port_name, make_var);
            checked += 1;
        }
    }

    let has_default = tools.iter().any(|t| t.entrypoints.iter()
        .any(|ep| ep.make_var.is_some() && ep.default_value.is_some()));
    assert!(has_default, "no entrypoints with make_var AND default_value ({} checked)", checked);
}

#[test]
fn make_var_cli_flag_bijection() {
    let tools = dsl_tools();
    let mut violations = Vec::new();
    let mut checked_tools = 0usize;

    for tool in &tools {
        let Some(info) = ToolInfo::from_tool_def(tool) else { continue };

        let ep_make_vars: BTreeMap<&str, &str> = tool.entrypoints.iter()
            .filter_map(|ep| ep.make_var.as_deref().map(|mv| (mv, ep.port_name.as_str())))
            .collect();
        let param_make_vars: BTreeMap<&str, &str> = info.entrypoints.iter()
            .map(|p| (p.make_var.as_str(), p.port_name.as_str()))
            .collect();

        for (mv, pn) in &ep_make_vars {
            if !param_make_vars.contains_key(mv) {
                violations.push(format!("tool '{}': ep '{}' make_var='{}' missing from ToolInfo",
                    tool.meta.tool_name, pn, mv));
            }
        }
        for (mv, pn) in &param_make_vars {
            if !ep_make_vars.contains_key(mv) {
                violations.push(format!("tool '{}': param '{}' make_var='{}' missing from ep",
                    tool.meta.tool_name, pn, mv));
            }
        }
        for param in &info.entrypoints {
            if param.cli_flag.is_empty() {
                violations.push(format!("tool '{}': param '{}' empty cli_flag",
                    tool.meta.tool_name, param.port_name));
            }
            if !param.cli_flag.starts_with("--") {
                violations.push(format!("tool '{}': cli_flag='{}' missing '--'",
                    tool.meta.tool_name, param.cli_flag));
            }
        }
        if !ep_make_vars.is_empty() || !param_make_vars.is_empty() { checked_tools += 1; }
    }

    assert!(violations.is_empty(), "bijection violations:\n{}", violations.join("\n"));
    assert!(checked_tools > 0, "no tools with make_var entrypoints");
}

#[test]
fn dsl_warning_policy_matches_build_config() {
    let config = BuildConfig::cargo();
    assert_eq!(config.warnings, Warnings::Deny,
        "BuildConfig.warnings must match dsl/config/build_policy.dag warning_policy=DenyAll");
}

// ============================================================================
// DSL Passthrough Callable Visibility
// ============================================================================

fn is_passthrough_callable(module: &str, name: &str, has_service_metadata: bool) -> bool {
    if module == "std.resources" || module == "tools.infra" { return false; }
    if module.starts_with("services.") || module.starts_with("workspace.") { return false; }
    if name.starts_with("service_transport::") && (has_service_metadata || name.starts_with("service_transport::execute::")) {
        return false;
    }
    if lookup_extern_impl(module, name).is_some() { return false; }
    true
}

// Callables with fn_body (fn items) are evaluated by FnBodyDelegate, not passthrough.
// Only func/pattern items without fn_body appear here.
const ALLOWED_PASSTHROUGH_CALLABLES: &[&str] = &[
    "std.filesystem::is_text_readable",
    "std.patterns::acquire_subject_token",
    "std.patterns::classify_files",
    "std.patterns::content_upsert",
    "std.patterns::credential_chain",
    "std.patterns::ensure",
    "std.patterns::file_content_matches",
    "std.patterns::github_oidc",
    "std.patterns::iam_preflight_check",
    "std.patterns::local_auth",
    "std.patterns::metadata_oidc",
    "std.patterns::optional_impersonation",
    "std.patterns::read_binary_files",
    "std.patterns::read_text_files",
    "std.patterns::resource_provide::credential_chain::auth",
    "std.patterns::retry",
    "std.patterns::transaction",
    "std.patterns::upsert",
    "tools.bootstrap::bootstrap",
    "tools.build::build_all",
    "tools.clippy::clippy_lint",
    "tools.codegen::codegen",
    "tools.codegen::codegen_ensure",
    "tools.deps::deps_generate",
    "tools.deps::deps",
    "tools.design::generate_design",
    "tools.design::review_design",
    "tools.docgen::docgen",
    "tools.gist::gist_diff",
    "tools.gist::gist_recent",
    "tools.gist::gist",
    "tools.makegen::makegen",
    "tools.pragma::pragma",
    "tools.testgen::testgen",
];

#[test]
fn passthrough_callables_are_allowlisted() {
    let (_workspace_root, dsl_root) = dsl_compile_context();
    let mut found_passthrough: BTreeSet<String> = BTreeSet::new();

    for stem in discover_dsl_tool_stems() {
        let dag_file = dsl_root.join(format!("tools/{stem}.dag"));
        let context = DriverContext {
            roots: vec![dsl_root.clone()],
            target_file: Some(dag_file),
        };
        let output = match compile_from_context(&context) {
            Ok(output) => output,
            Err(_) => continue,
        };

        for node in &output.lowered_dag.nodes {
            if let NodeBody::Opaque(LoweredOp::Callable {
                module, name, service_metadata, fn_body, ..
            }) = &node.body
            {
                // Callables with fn_body are evaluated by FnBodyDelegate, not passthrough.
                if fn_body.is_some() {
                    continue;
                }
                if is_passthrough_callable(module, name, service_metadata.is_some()) {
                    found_passthrough.insert(format!("{module}::{name}"));
                }
            }
        }
    }

    let allowlist: BTreeSet<&str> = ALLOWED_PASSTHROUGH_CALLABLES.iter().copied().collect();

    let unexpected: BTreeSet<&String> = found_passthrough.iter()
        .filter(|key| !allowlist.contains(key.as_str()))
        .collect();
    assert!(unexpected.is_empty(), "passthrough callables not in allowlist:\n  {:?}", unexpected);

    let stale: BTreeSet<&&str> = allowlist.iter()
        .filter(|key| !found_passthrough.contains(**key))
        .collect();
    assert!(stale.is_empty(), "stale ALLOWED_PASSTHROUGH_CALLABLES:\n  {:?}", stale);
}
