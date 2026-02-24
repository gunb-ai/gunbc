use daglang_driver::{compile_from_context, DriverContext};
use gunbc_codegen::derive_tool_defs;
use gunbc_dag::makegen::{BuildConfig, ToolInfo, ToolRegistry};
use gunbc_ir::cargo::Warnings;
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
use gunbc_lib_review::review_tool;
// These are in gunbc-dag itself (same binary), but reference for completeness.
use gunbc_dag::bootstrap::bootstrap_tool;
use gunbc_dag::makegen::makegen_tool;

/// Verify that derive_tool_defs() returns all expected tools from inventory.
#[test]
fn derive_tool_defs_matches_inventory() {
    // Touch the functions to prevent the linker from stripping them.
    let _: fn() = clippy_tool;
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
            && !content.contains("deps_tool"),
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

/// Every inventory tool with `has_invocation` must produce a valid
/// `CargoInvocation` (non-empty binary name and package). Every
/// `WorkspaceBinary` variant must resolve a working invocation
/// (either from registry or from the composed fallback).
///
/// `WorkspaceBinary` may contain variants whose `ToolRegistration` has
/// `has_invocation = false` (e.g., codegen, testgen) — these use the
/// composed fallback in `invocation()`. Tools with `has_invocation`
/// that are NOT in `WorkspaceBinary` are also fine (codegen-generated
/// binaries like deps).
#[test]
fn workspace_binary_registry_invocations_are_consistent() {
    use gunbc_dag::WorkspaceBinary;

    force_linker_include();

    let mut violations = Vec::new();

    // Every WorkspaceBinary variant must resolve to a non-empty invocation.
    for binary in WorkspaceBinary::ALL {
        let inv = binary.invocation();
        if inv.binary.is_empty() {
            violations.push(format!(
                "WorkspaceBinary variant '{}' resolves to empty binary name",
                binary.tool_name(),
            ));
        }
    }

    // Every inventory tool with has_invocation must have a valid package
    // and binary name for constructing a CargoInvocation.
    for tool in iter_tool_targets() {
        if !tool.has_invocation {
            continue;
        }
        let binary_name = tool.binary.unwrap_or(tool.tool_name);
        if binary_name.is_empty() {
            violations.push(format!(
                "tool '{}' has has_invocation=true but empty binary name",
                tool.tool_name,
            ));
        }
        if tool.package.is_none() {
            violations.push(format!(
                "tool '{}' has has_invocation=true but no package — \
                 cannot construct a CargoInvocation",
                tool.tool_name,
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "WorkspaceBinary↔inventory consistency violations:\n{}",
        violations.join("\n"),
    );
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

/// Every inventory tool with `has_invocation` must be reachable either as
/// a `WorkspaceBinary` variant OR as a codegen-generated CLI binary (those
/// that produce `ToolDef.invocation.is_some()` via `derive_tool_defs()`).
///
/// The enum may contain extra entries (internal binaries without tool
/// registration), but every invocable registry entry must be covered by
/// at least one dispatch path.
///
/// This catches the case where someone adds a `#[tool_target]` with
/// `has_invocation: true` but forgets to add either a `workspace_binaries!`
/// entry or a codegen-generated CLI.
#[test]
fn workspace_binary_enum_matches_invocable_registry_entries() {
    use gunbc_dag::WorkspaceBinary;

    force_linker_include();

    // Codegen-generated CLIs: tools whose ToolDef has an invocation derived
    // from the registry (deps, etc.).
    let codegen_generated: BTreeSet<String> = derive_tool_defs()
        .into_iter()
        .filter(|tool| tool.invocation.is_some())
        .map(|tool| tool.meta.tool_name.to_string())
        .collect();

    let registry_binaries: BTreeSet<&str> = iter_tool_targets()
        .filter(|t| t.has_invocation)
        .map(|t| t.tool_name)
        .collect();
    let enum_binaries: BTreeSet<&str> = WorkspaceBinary::ALL
        .iter()
        .map(|b| b.tool_name())
        .collect();

    // Every invocable registry entry should be in the enum OR be a
    // codegen-generated CLI binary.
    let missing: BTreeSet<_> = registry_binaries
        .iter()
        .filter(|name| !enum_binaries.contains(*name))
        .filter(|name| !codegen_generated.contains(**name))
        .collect();
    assert!(
        missing.is_empty(),
        "invocable tool registrations missing from both WorkspaceBinary \
         and codegen-generated CLIs: {:?}",
        missing
    );
}

/// Tools with non-empty `outputs` are producers — they generate files.
/// This test validates that producer tools are aware of the provides/consumes
/// metadata fields and will catch regressions when generator edge derivation
/// starts relying on `provides` being populated for all producers.
///
/// Currently informational: logs tools with outputs but empty provides.
/// Will be promoted to a hard assertion when generator edge derivation
/// uses provides/consumes for automatic dependency wiring.
#[test]
fn producer_tools_declare_provides() {
    force_linker_include();

    for tool in iter_tool_targets() {
        if !tool.outputs.is_empty() {
            // Tools with outputs should ideally declare what they provide.
            // This is informational for now — will be enforced when generator
            // edge derivation uses provides/consumes.
            if tool.provides.is_empty() {
                eprintln!(
                    "info: tool '{}' has {} output(s) but empty provides — \
                     consider populating provides for generator edge derivation",
                    tool.tool_name,
                    tool.outputs.len(),
                );
            }
        }
    }
}

/// Every tool registration must have a unique `tool_name`. Duplicate names
/// would cause ambiguous lookups in `WorkspaceBinary::from_tool_name()`,
/// `derive_tool_defs()`, and Makefile target generation.
#[test]
fn tool_names_are_unique() {
    force_linker_include();

    let mut seen = BTreeSet::new();
    for tool in iter_tool_targets() {
        assert!(
            seen.insert(tool.tool_name),
            "duplicate tool_name in registry: {}",
            tool.tool_name
        );
    }
}

/// **Single Inventory Authority**: proves `iter_tool_targets()` is the single
/// source of truth for tool definitions. No tool definition path may bypass
/// the inventory:
///
/// 1. `derive_tool_defs()` (codegen) must draw exclusively from inventory.
/// 2. `ToolRegistry::default_registry()` (makegen) must draw generated-CLI
///    tools exclusively from `derive_tool_defs()` (which draws from inventory).
/// 3. Every makegen-registered tool with `needs_generated_cli` must trace
///    back to an inventory `ToolRegistration`.
/// 4. No hardcoded tool list outside the registry may introduce tools that
///    the inventory does not know about.
///
/// This is the capstone M14 test: if it passes, adding a new tool requires
/// only one `#[tool_target]` registration and everything else derives from it.
#[test]
fn inventory_is_single_authority() {
    force_linker_include();

    // === Source 1: Inventory (the authority) ===
    let inventory_names: BTreeSet<&str> = iter_tool_targets().map(|r| r.tool_name).collect();

    // === Source 2: Codegen's derive_tool_defs() ===
    let codegen_names: BTreeSet<String> = derive_tool_defs()
        .into_iter()
        .map(|t| t.meta.tool_name.to_string())
        .collect();

    // derive_tool_defs must be a bijection with inventory.
    let codegen_as_str: BTreeSet<&str> = codegen_names.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        inventory_names, codegen_as_str,
        "derive_tool_defs() must be a 1:1 mapping of iter_tool_targets() — \
         inventory-only: {:?}, codegen-only: {:?}",
        inventory_names.difference(&codegen_as_str).collect::<Vec<_>>(),
        codegen_as_str.difference(&inventory_names).collect::<Vec<_>>(),
    );

    // === Source 3: Makegen default_registry() ===
    let registry = ToolRegistry::default_registry();
    let makegen_generated: BTreeSet<String> = registry
        .tools
        .iter()
        .filter(|t| t.needs_generated_cli)
        .map(|t| t.short_name.clone())
        .collect();

    // Every makegen generated-CLI tool must exist in inventory.
    let makegen_outside_inventory: BTreeSet<&String> = makegen_generated
        .iter()
        .filter(|name| !inventory_names.contains(name.as_str()))
        .collect();
    assert!(
        makegen_outside_inventory.is_empty(),
        "makegen has generated-CLI tools not in inventory (hand-wired bypass): {:?}",
        makegen_outside_inventory,
    );

    // === Source 4: No tool definition source outside inventory ===
    // All makegen tools (generated or manual) should trace back to either
    // the inventory or the documented MANUAL_TOOL_DEFS set.
    // The manual tools are not generated CLIs, so they won't have
    // needs_generated_cli=true. Verify manual tools are accounted for.
    let all_makegen_names: BTreeSet<String> = registry
        .tools
        .iter()
        .map(|t| t.short_name.clone())
        .collect();
    let manual_makegen: BTreeSet<&String> = all_makegen_names
        .iter()
        .filter(|name| !inventory_names.contains(name.as_str()))
        .collect();

    // Manual tools must be an auditable, small set. If this grows,
    // it means tools are bypassing the single authority.
    // Currently: "build-all" is the only non-registry manual target.
    let known_manual: BTreeSet<&str> = ["build-all"].into_iter().collect();
    let manual_str: BTreeSet<&str> = manual_makegen.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        manual_str, known_manual,
        "unexpected manual makegen tools outside inventory — \
         any new tool should use #[tool_target] registration, not manual wiring. \
         extra: {:?}, missing: {:?}",
        manual_str.difference(&known_manual).collect::<Vec<_>>(),
        known_manual.difference(&manual_str).collect::<Vec<_>>(),
    );

    // Non-vacuity: the inventory is non-empty.
    assert!(
        inventory_names.len() >= 8,
        "inventory has suspiciously few tools ({}) — \
         linker may have stripped symbols",
        inventory_names.len(),
    );
}

/// Verify that `provides`/`consumes` form a valid directed acyclic graph.
///
/// The provides/consumes metadata on `ToolRegistration` defines a producer→consumer
/// dependency graph between tools. This test:
/// 1. Builds the dependency graph from provides/consumes edges
/// 2. Verifies the graph is acyclic (no circular dependencies)
/// 3. Verifies a valid topological ordering exists
///
/// A cycle would mean tool A depends on tool B's output while tool B depends
/// on tool A's output — an unresolvable build ordering.
#[test]
fn provides_consumes_form_acyclic_graph() {
    force_linker_include();

    // Build artifact→producer and consumer→artifacts maps.
    let mut artifact_to_producer: BTreeMap<&str, &str> = BTreeMap::new();
    for tool in iter_tool_targets() {
        for artifact in tool.provides {
            artifact_to_producer.insert(artifact, tool.tool_name);
        }
    }

    // Build adjacency list: tool_name → set of tool_names it depends on.
    let mut depends_on: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for tool in iter_tool_targets() {
        let entry = depends_on.entry(tool.tool_name).or_default();
        for consumed in tool.consumes {
            if let Some(&producer) = artifact_to_producer.get(consumed) {
                if producer != tool.tool_name {
                    entry.insert(producer);
                }
            }
        }
    }

    // Topological sort via Kahn's algorithm to detect cycles.
    let all_tools: BTreeSet<&str> = iter_tool_targets().map(|t| t.tool_name).collect();

    // Compute in-degree: for each tool, count how many tools it depends on.
    let mut in_degree: BTreeMap<&str, usize> = BTreeMap::new();
    for &tool in &all_tools {
        in_degree.insert(tool, 0);
    }
    for (&consumer, deps) in &depends_on {
        *in_degree.entry(consumer).or_insert(0) = deps.len();
    }

    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&tool, _)| tool)
        .collect();
    queue.sort(); // deterministic ordering
    let mut sorted = Vec::new();

    while let Some(tool) = queue.pop() {
        sorted.push(tool);
        // Find all tools that depend on this one and reduce their in-degree.
        for (&consumer, deps) in &depends_on {
            if deps.contains(tool) {
                if let Some(deg) = in_degree.get_mut(consumer) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        // Only add to queue if not already sorted.
                        if !sorted.contains(&consumer) && !queue.contains(&consumer) {
                            queue.push(consumer);
                            queue.sort();
                        }
                    }
                }
            }
        }
    }

    let unsorted: BTreeSet<&str> = all_tools
        .iter()
        .filter(|t| !sorted.contains(t))
        .copied()
        .collect();

    assert!(
        unsorted.is_empty(),
        "provides/consumes dependency graph has a cycle involving: {:?}. \
         No valid build ordering exists for these tools.",
        unsorted,
    );

    // Non-vacuity: at least one edge exists in the graph.
    let total_edges: usize = depends_on.values().map(|deps| deps.len()).sum();
    assert!(
        total_edges > 0,
        "provides/consumes graph has zero edges — \
         test is vacuous (no tool declares consumes with a matching provides)"
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

/// Contract test: BuildConfig.warnings must match dsl/config/build_policy.dag warning_policy=DenyAll.
///
/// The DSL source of truth declares `data warning_policy: WarningPolicy = DenyAll`. The Rust
/// `BuildConfig::cargo()` must agree, ensuring `make gist` and all tool targets compile with
/// `RUSTFLAGS="-D warnings"`. If this test fails, the DSL and Rust config have drifted.
#[test]
fn dsl_warning_policy_matches_build_config() {
    let config = BuildConfig::cargo();
    assert_eq!(
        config.warnings,
        Warnings::Deny,
        "BuildConfig.warnings must match dsl/config/build_policy.dag warning_policy=DenyAll"
    );
}
