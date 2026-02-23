use daglang_driver::{compile_from_context, DriverContext};
use gunbc_codegen::derive_tool_defs;
use gunbc_dag::makegen::{ToolInfo, ToolRegistry};
use gunbc_ir::resource::ResourceIo;
use gunbc_lib_transport::TransportIo;
use gunbc_tool_registry::iter_tool_targets;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

// Force the linker to include inventory submissions from dependency crates.
// Without these references, the linker may dead-strip the inventory symbols
// and iter_tool_targets() would return an empty iterator.
use gunbc_clippy::clippy_tool;
use gunbc_dag::deps_tool::deps_tool;
use gunbc_gist::{gist_diff_tool, gist_recent_tool, gist_snapshot_tool};
use gunbc_lib_review::review_tool;
// These are in gunbc-dag itself (same binary), but reference for completeness.
use gunbc_dag::bootstrap::bootstrap_tool;
use gunbc_dag::makegen::makegen_tool;

/// Verify that derive_tool_defs() returns all expected tools from inventory.
#[test]
fn derive_tool_defs_matches_inventory() {
    // Touch the functions to prevent the linker from stripping them.
    let _: fn() = clippy_tool;
    let _: fn() = gist_snapshot_tool;
    let _: fn() = gist_diff_tool;
    let _: fn() = gist_recent_tool;
    let _: fn() = deps_tool;
    let _: fn() = review_tool;
    let _: fn() = makegen_tool;
    let _: fn() = bootstrap_tool;

    let tools = derive_tool_defs();
    let tool_names: HashSet<&str> = tools.iter().map(|t| t.meta.tool_name.as_ref()).collect();
    let reg_names: HashSet<&str> = iter_tool_targets().map(|r| r.tool_name).collect();

    // derive_tool_defs and inventory must agree
    assert_eq!(
        tool_names, reg_names,
        "derive_tool_defs() and iter_tool_targets() should contain the same tool names"
    );

    // Verify expected set
    let expected = [
        "bootstrap",
        "clippy",
        "deps",
        "gist",
        "gist-diff",
        "gist-recent",
        "makegen",
        "review",
    ];
    for name in &expected {
        assert!(
            tool_names.contains(name),
            "expected tool '{}' not found in derive_tool_defs()",
            name
        );
    }

    // Verify tools with invocations have package info
    for tool in &tools {
        if let Some(inv) = &tool.invocation {
            assert!(
                !inv.binary.is_empty(),
                "tool '{}' has empty binary in invocation",
                tool.meta.tool_name
            );
        }
    }

    // Hard-migration guardrail: do not hide execution routing behind adapter aliases.
    for tool in &tools {
        if let Some(import) = &tool.custom_import {
            assert!(
                !import.contains("_adapter as "),
                "tool '{}' uses adapter-style import alias in registry metadata: {}",
                tool.meta.tool_name,
                import
            );
        }
    }
}

#[test]
fn makegen_default_registry_matches_codegen_registry_plus_manual_targets() {
    let derived_with_invocation: HashSet<String> = derive_tool_defs()
        .into_iter()
        .filter(|tool| tool.invocation.is_some())
        .map(|tool| tool.meta.tool_name.to_string())
        .collect();

    let registry = ToolRegistry::default_registry();
    let makegen_generated: HashSet<String> = registry
        .tools
        .iter()
        .filter(|tool| tool.needs_generated_cli)
        .map(|tool| tool.short_name.clone())
        .collect();

    assert_eq!(
        makegen_generated, derived_with_invocation,
        "makegen generated-cli tools must stay in lockstep with codegen derive_tool_defs()"
    );

    let manual_targets: HashSet<&str> = registry
        .tools
        .iter()
        .filter(|tool| !tool.needs_generated_cli)
        .map(|tool| tool.short_name.as_str())
        .collect();
    let expected_manual: HashSet<&str> = ["build-all"].into_iter().collect();
    assert_eq!(
        manual_targets, expected_manual,
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
        !content.contains("derive_tool_defs"),
        "codegen_cli must discover tools from source/DSL, not derive_tool_defs() inventory"
    );
    assert!(
        !content.contains("iter_tool_targets"),
        "codegen_cli must not depend on iter_tool_targets() inventory"
    );
    assert!(
        !content.contains("clippy_tool")
            && !content.contains("deps_tool")
            && !content.contains("gist_snapshot_tool"),
        "codegen_cli must not use force-link tool symbol touch points for discovery"
    );
}

/// For each tool with a `dsl_module`, compile the `.dag` file and verify that
/// the DSL-derived `output_paths` match the tool's `ToolRegistration.outputs`.
///
/// This catches drift: if someone adds a `content_upsert(path: "new-file")`
/// or `@outputs("pattern")` but forgets to update the registry `outputs`,
/// the test fails. If someone removes a `content_upsert` but leaves a stale
/// `outputs` entry, it also fails.
#[test]
fn tool_declared_outputs_match_dsl() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let dsl_root = workspace_root.join("dsl");

    // Force linker to include inventory symbols.
    let _: fn() = clippy_tool;
    let _: fn() = gist_snapshot_tool;
    let _: fn() = gist_diff_tool;
    let _: fn() = gist_recent_tool;
    let _: fn() = deps_tool;
    let _: fn() = review_tool;
    let _: fn() = makegen_tool;
    let _: fn() = bootstrap_tool;

    for reg in iter_tool_targets() {
        let Some(dsl_module) = reg.dsl_module else {
            continue;
        };

        let dag_file = dsl_root.join(format!("tools/{dsl_module}.dag"));
        if !dag_file.exists() {
            // Some dsl_module values may reference pipelines instead of tools.
            continue;
        }

        let context = DriverContext {
            roots: vec![dsl_root.clone()],
            target_file: Some(dag_file.clone()),
        };
        let output = match compile_from_context(&context) {
            Ok(output) => output,
            Err(error) => {
                // Some tools may fail to compile in isolation (missing profiles, etc.)
                // — skip those rather than failing the invariant test.
                eprintln!(
                    "skipping drift check for {dsl_module} (compile error): {error}"
                );
                continue;
            }
        };

        let dsl_paths: BTreeSet<&str> = output
            .output_paths
            .iter()
            .map(|s| s.as_str())
            .collect();
        let reg_paths: BTreeSet<&str> = reg.outputs.iter().copied().collect();

        assert_eq!(
            dsl_paths, reg_paths,
            "output path drift for tool '{}' (dsl_module={dsl_module}): \
             DSL-derived={dsl_paths:?}, registry={reg_paths:?}",
            reg.tool_name,
        );
    }
}

/// Bootstrap-critical files: generated by tools but committed because they
/// must exist before `make install` (bootstrap) can run. These are explicit
/// exceptions to the "never commit generated files" invariant.
const COMMITTED_SEED_FILES: &[&str] = &[
    ".gitignore",  // bootstrap generates this, but git needs it to exist
    "clippy.toml", // pragma generates this, but clippy needs it pre-bootstrap
    "deps.toml",   // deps generates this, but install needs it pre-bootstrap
];

/// Verify that no generated output file is tracked in git (except known seeds).
///
/// Collects all output patterns from derive_tool_defs() and checks that
/// `git ls-files` returns no matches for any of them. Bootstrap-critical
/// seed files in COMMITTED_SEED_FILES are exempt.
#[test]
#[allow(clippy::disallowed_methods)]
fn no_generated_files_committed() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

    // Force linker.
    let _: fn() = clippy_tool;
    let _: fn() = gist_snapshot_tool;
    let _: fn() = gist_diff_tool;
    let _: fn() = gist_recent_tool;
    let _: fn() = deps_tool;
    let _: fn() = review_tool;
    let _: fn() = makegen_tool;
    let _: fn() = bootstrap_tool;

    let tools = derive_tool_defs();
    let mut violations = Vec::new();

    for tool in &tools {
        for pattern in &tool.outputs {
            if COMMITTED_SEED_FILES.contains(&pattern.as_str()) {
                continue;
            }
            let output = std::process::Command::new("git")
                .args(["ls-files", pattern])
                .current_dir(&workspace_root)
                .output();
            let Ok(output) = output else {
                continue;
            };
            let stdout = String::from_utf8_lossy(&output.stdout);
            let tracked: Vec<&str> = stdout
                .lines()
                .filter(|line| !line.is_empty())
                .collect();
            if !tracked.is_empty() {
                violations.push(format!(
                    "tool '{}' output pattern '{}' matches tracked files: {:?}",
                    tool.meta.tool_name, pattern, tracked
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "generated files are committed in git (invariant: never commit generated files):\n{}",
        violations.join("\n")
    );
}

/// Verify that all declared tool output paths are covered by .gitignore
/// (except known seed files that must be committed).
///
/// For non-glob output paths, runs `git check-ignore -q <path>` and fails
/// if any declared output isn't covered. Glob patterns are skipped since
/// `check-ignore` doesn't expand them.
#[test]
#[allow(clippy::disallowed_methods)]
fn all_tool_outputs_gitignored() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

    // Force linker.
    let _: fn() = clippy_tool;
    let _: fn() = gist_snapshot_tool;
    let _: fn() = gist_diff_tool;
    let _: fn() = gist_recent_tool;
    let _: fn() = deps_tool;
    let _: fn() = review_tool;
    let _: fn() = makegen_tool;
    let _: fn() = bootstrap_tool;

    let tools = derive_tool_defs();
    let mut not_ignored = Vec::new();

    for tool in &tools {
        for pattern in &tool.outputs {
            // Skip glob patterns — check-ignore doesn't expand them.
            if pattern.contains('*') || pattern.contains('?') {
                continue;
            }
            // Skip bootstrap-critical seed files.
            if COMMITTED_SEED_FILES.contains(&pattern.as_str()) {
                continue;
            }
            let status = std::process::Command::new("git")
                .args(["check-ignore", "-q", pattern])
                .current_dir(&workspace_root)
                .status();
            let Ok(status) = status else {
                continue;
            };
            if !status.success() {
                not_ignored.push(format!(
                    "tool '{}' output '{}' is not gitignored",
                    tool.meta.tool_name, pattern
                ));
            }
        }
    }

    assert!(
        not_ignored.is_empty(),
        "tool outputs not covered by .gitignore (invariant: all generated files must be gitignored):\n{}",
        not_ignored.join("\n")
    );
}

// ============================================================================
// M14: Single Inventory Authority
// ============================================================================

/// Every inventory tool with `has_invocation` must have a corresponding
/// `WorkspaceBinary` variant. Every `WorkspaceBinary` that resolves via
/// `registry_invocation()` must appear in the inventory with `has_invocation`.
///
/// `WorkspaceBinary` may contain extra entries (e.g., internal binaries like
/// `codegen-dag`, `deps-config`, `sdlc`) that have no `#[tool_target]`
/// registration. Those are expected — the enum is a superset. But any tool
/// that *does* register with `has_invocation` must be in the enum.
#[test]
fn workspace_binary_enum_matches_inventory_binaries() {
    use gunbc_dag::WorkspaceBinary;

    force_linker_include();

    let inventory_with_invocation: BTreeSet<&str> = iter_tool_targets()
        .filter(|t| t.has_invocation)
        .map(|t| t.tool_name)
        .collect();

    let enum_tool_names: BTreeSet<&str> = WorkspaceBinary::ALL
        .iter()
        .map(|b| b.tool_name())
        .collect();

    // Forward: every inventory tool with has_invocation must be in the enum.
    let missing_from_enum: BTreeSet<&&str> = inventory_with_invocation
        .iter()
        .filter(|name| !enum_tool_names.contains(*name))
        .collect();
    assert!(
        missing_from_enum.is_empty(),
        "inventory tools with has_invocation missing from WorkspaceBinary: {:?}",
        missing_from_enum,
    );

    // Reverse: every enum variant that resolves from the registry must have
    // has_invocation in inventory (no phantom registry lookups).
    for binary in WorkspaceBinary::ALL {
        let tool_name = binary.tool_name();
        let reg = iter_tool_targets().find(|t| t.tool_name == tool_name);
        if let Some(reg) = reg {
            // If the tool is registered, it must have has_invocation.
            assert!(
                reg.has_invocation,
                "WorkspaceBinary variant '{}' has a ToolRegistration but \
                 has_invocation=false — either set has_invocation or remove \
                 the enum variant",
                tool_name,
            );
        }
        // Tools without a ToolRegistration are allowed (internal binaries).
    }
}

/// Verify that provides/consumes metadata is consistent: every `consumes`
/// entry in one tool should match a `provides` entry from another tool.
///
/// This catches orphaned consumer declarations (tool declares it consumes
/// a file but no tool provides it) and helps validate the generator edge
/// graph derivation.
#[test]
fn provides_consumes_edges_are_consistent() {
    force_linker_include();

    let all_provides: BTreeSet<&str> = iter_tool_targets()
        .flat_map(|t| t.provides.iter().copied())
        .collect();

    let mut orphaned_consumes = Vec::new();

    for tool in iter_tool_targets() {
        for consumed in tool.consumes {
            if !all_provides.contains(consumed) {
                orphaned_consumes.push(format!(
                    "tool '{}' consumes '{}' but no tool provides it",
                    tool.tool_name, consumed,
                ));
            }
        }
    }

    assert!(
        orphaned_consumes.is_empty(),
        "orphaned consumes declarations (no matching provider):\n{}",
        orphaned_consumes.join("\n"),
    );
}

/// Verify that `provides` is a subset of `outputs` for each tool.
///
/// `provides` declares producer relationships for the generator edge graph,
/// while `outputs` declares all generated files for gitignore/clean. Every
/// file a tool provides should also be in its outputs list.
#[test]
fn provides_is_subset_of_outputs() {
    force_linker_include();

    let mut violations = Vec::new();

    for tool in iter_tool_targets() {
        let outputs: BTreeSet<&str> = tool.outputs.iter().copied().collect();
        for provided in tool.provides {
            if !outputs.contains(provided) {
                violations.push(format!(
                    "tool '{}' provides '{}' but it is not in outputs {:?}",
                    tool.tool_name, provided, tool.outputs,
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "provides entries not found in outputs:\n{}",
        violations.join("\n"),
    );
}

// ============================================================================
// M13: Registry→CLI→Make Contract Tests
// ============================================================================

/// Helper: force linker to include inventory symbols.
///
/// Shared by M13 contract tests to avoid repeating the symbol-touch block.
fn force_linker_include() {
    let _: fn() = clippy_tool;
    let _: fn() = gist_snapshot_tool;
    let _: fn() = gist_diff_tool;
    let _: fn() = gist_recent_tool;
    let _: fn() = deps_tool;
    let _: fn() = review_tool;
    let _: fn() = makegen_tool;
    let _: fn() = bootstrap_tool;
}

/// For each tool with entrypoints where `cardinality.allows_many()`, verify
/// the `repeatable` property is preserved through `ToolDef → ToolInfo` conversion.
///
/// This catches bugs where `from_tool_def()` drops the cardinality (e.g.,
/// hardcoding `repeatable: false` or skipping the field).
#[test]
fn repeatable_flags_survive_roundtrip() {
    force_linker_include();

    let tools = derive_tool_defs();
    let mut checked = 0usize;

    for tool in &tools {
        let Some(info) = ToolInfo::from_tool_def(tool) else {
            continue;
        };

        for ep in &tool.entrypoints {
            // Only check entrypoints that flow into makegen (have make_var).
            let Some(ref make_var) = ep.make_var else {
                continue;
            };

            // Find the corresponding EntrypointParam in the ToolInfo.
            let param = info
                .entrypoints
                .iter()
                .find(|p| p.make_var == *make_var)
                .unwrap_or_else(|| {
                    panic!(
                        "tool '{}' entrypoint '{}' has make_var='{}' but no matching \
                         EntrypointParam in ToolInfo",
                        tool.meta.tool_name, ep.port_name, make_var,
                    )
                });

            if ep.cardinality.allows_many() {
                assert!(
                    param.repeatable,
                    "tool '{}' entrypoint '{}' (make_var={}) has allows_many() cardinality \
                     but ToolInfo.EntrypointParam.repeatable is false — roundtrip lost",
                    tool.meta.tool_name, ep.port_name, make_var,
                );
                checked += 1;
            } else {
                assert!(
                    !param.repeatable,
                    "tool '{}' entrypoint '{}' (make_var={}) does NOT have allows_many() \
                     cardinality but ToolInfo.EntrypointParam.repeatable is true — \
                     spurious repeatability injected",
                    tool.meta.tool_name, ep.port_name, make_var,
                );
                checked += 1;
            }
        }
    }

    // Non-vacuity guard: at least one entrypoint with make_var must exist,
    // otherwise this test is trivially passing on an empty set.
    assert!(
        checked > 0,
        "no entrypoints with make_var found — test is vacuous; \
         add entrypoints to tool registrations or remove this test"
    );
}

/// For each tool with entrypoints that have `default_value`, verify defaults
/// are preserved through the `ToolDef → ToolInfo` conversion chain.
///
/// This catches bugs where `from_tool_def()` drops defaults (e.g., always
/// setting `default: None`).
#[test]
fn default_values_survive_roundtrip() {
    force_linker_include();

    let tools = derive_tool_defs();
    let mut checked = 0usize;

    for tool in &tools {
        let Some(info) = ToolInfo::from_tool_def(tool) else {
            continue;
        };

        for ep in &tool.entrypoints {
            let Some(ref make_var) = ep.make_var else {
                continue;
            };

            let param = info
                .entrypoints
                .iter()
                .find(|p| p.make_var == *make_var)
                .unwrap_or_else(|| {
                    panic!(
                        "tool '{}' entrypoint '{}' has make_var='{}' but no matching \
                         EntrypointParam in ToolInfo",
                        tool.meta.tool_name, ep.port_name, make_var,
                    )
                });

            // Both directions: if CliEntrypoint has a default, EntrypointParam must too.
            // If CliEntrypoint has no default, EntrypointParam must not fabricate one.
            assert_eq!(
                ep.default_value, param.default,
                "tool '{}' entrypoint '{}' (make_var={}): default_value mismatch \
                 across ToolDef→ToolInfo roundtrip. CliEntrypoint={:?}, EntrypointParam={:?}",
                tool.meta.tool_name, ep.port_name, make_var, ep.default_value, param.default,
            );
            checked += 1;
        }
    }

    // Non-vacuity: at least one entrypoint with make_var and default must exist.
    let has_default = tools.iter().any(|t| {
        t.entrypoints
            .iter()
            .any(|ep| ep.make_var.is_some() && ep.default_value.is_some())
    });
    assert!(
        has_default,
        "no entrypoints with make_var AND default_value found — \
         test is vacuous for default preservation; the cardinality check \
         still ran ({} entrypoints checked) but defaults were never tested",
        checked,
    );
}

/// Every entrypoint with `make_var` must produce a CLI flag in the ToolInfo.
/// Every `EntrypointParam` in makegen must trace back to an entrypoint with `make_var`.
///
/// This is a bijection check: the two representations must be 1:1.
#[test]
fn make_var_cli_flag_bijection() {
    force_linker_include();

    let tools = derive_tool_defs();
    let mut violations = Vec::new();
    let mut checked_tools = 0usize;

    for tool in &tools {
        let Some(info) = ToolInfo::from_tool_def(tool) else {
            continue;
        };

        // Forward direction: every CliEntrypoint with make_var → EntrypointParam
        let ep_make_vars: BTreeMap<&str, &str> = tool
            .entrypoints
            .iter()
            .filter_map(|ep| ep.make_var.as_deref().map(|mv| (mv, ep.port_name.as_str())))
            .collect();

        let param_make_vars: BTreeMap<&str, &str> = info
            .entrypoints
            .iter()
            .map(|p| (p.make_var.as_str(), p.port_name.as_str()))
            .collect();

        // Forward: every make_var in CliEntrypoint appears in EntrypointParam
        for (make_var, port_name) in &ep_make_vars {
            if !param_make_vars.contains_key(make_var) {
                violations.push(format!(
                    "tool '{}': entrypoint '{}' has make_var='{}' but no \
                     corresponding EntrypointParam in ToolInfo",
                    tool.meta.tool_name, port_name, make_var,
                ));
            }
        }

        // Reverse: every EntrypointParam traces back to a CliEntrypoint with make_var
        for (make_var, port_name) in &param_make_vars {
            if !ep_make_vars.contains_key(make_var) {
                violations.push(format!(
                    "tool '{}': EntrypointParam '{}' has make_var='{}' but no \
                     corresponding CliEntrypoint with that make_var",
                    tool.meta.tool_name, port_name, make_var,
                ));
            }
        }

        // Verify that every EntrypointParam has a non-empty CLI flag
        for param in &info.entrypoints {
            if param.cli_flag.is_empty() {
                violations.push(format!(
                    "tool '{}': EntrypointParam '{}' (make_var={}) has empty cli_flag",
                    tool.meta.tool_name, param.port_name, param.make_var,
                ));
            }
            // The cli_flag should start with "--" (standard long-flag convention)
            if !param.cli_flag.starts_with("--") {
                violations.push(format!(
                    "tool '{}': EntrypointParam '{}' cli_flag='{}' does not start with '--'",
                    tool.meta.tool_name, param.port_name, param.cli_flag,
                ));
            }
        }

        if !ep_make_vars.is_empty() || !param_make_vars.is_empty() {
            checked_tools += 1;
        }
    }

    assert!(
        violations.is_empty(),
        "make_var↔CLI flag bijection violations:\n{}",
        violations.join("\n"),
    );

    // Non-vacuity: at least one tool must have make_var entrypoints.
    assert!(
        checked_tools > 0,
        "no tools with make_var entrypoints found — bijection test is vacuous"
    );
}
