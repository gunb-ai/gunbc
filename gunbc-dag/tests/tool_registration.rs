use gunbc_codegen::derive_tool_defs;
use gunbc_dag::makegen::ToolRegistry;
use gunbc_ir::resource::ResourceIo;
use gunbc_lib_transport::TransportIo;
use gunbc_tool_registry::iter_tool_targets;
use std::collections::HashSet;
use std::path::Path;

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
fn workspace_subdag_discovery_avoids_tool_registry_inventory() {
    let workspace_subdags = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("workspace")
        .join("subdags")
        .join("mod.rs");
    let io = TransportIo::new();
    let content = String::from_utf8(
        io.read_file(&workspace_subdags)
            .expect("workspace subdags module should be readable"),
    )
    .expect("workspace subdags module should be UTF-8");

    assert!(
        !content.contains("iter_tool_targets"),
        "workspace DAG discovery must source from DSL module discovery, not iter_tool_targets()"
    );
    assert!(
        !content.contains("gunbc_tool_registry"),
        "workspace DAG discovery must not import gunbc_tool_registry directly"
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


