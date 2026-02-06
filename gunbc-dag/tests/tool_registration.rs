use gunbc_codegen::all_tools;
use gunbc_tool_registry::iter_tool_targets;
use std::collections::HashMap;

// Force the linker to include inventory submissions from dependency crates.
// Without these references, the linker may dead-strip the inventory symbols
// and iter_tool_targets() would return an empty iterator.
use gunbc_gist::{gist_diff_tool, gist_recent_tool, gist_snapshot_tool};
use gunbc_deps::deps_tool;
use gunbc_lib_review::review_tool;
// These are in gunbc-dag itself (same binary), but reference for completeness.
use gunbc_dag::makegen::makegen_tool;
use gunbc_dag::bootstrap::bootstrap_tool;

#[test]
fn tool_registrations_match_all_tools() {
    // Touch the functions to prevent the linker from stripping them.
    let _: fn() = gist_snapshot_tool;
    let _: fn() = gist_diff_tool;
    let _: fn() = gist_recent_tool;
    let _: fn() = deps_tool;
    let _: fn() = review_tool;
    let _: fn() = makegen_tool;
    let _: fn() = bootstrap_tool;

    let all = all_tools();
    let regs: HashMap<&str, _> = iter_tool_targets().map(|r| (r.tool_name, r)).collect();

    // Every all_tools entry has a matching registration
    for tool in &all {
        let reg = regs.get(tool.meta.tool_name.as_str()).unwrap_or_else(|| {
            panic!(
                "tool '{}' in all_tools() has no #[tool_target] annotation",
                tool.meta.tool_name
            )
        });
        assert_eq!(
            reg.crate_name, tool.meta.crate_name,
            "crate_name mismatch for '{}'",
            tool.meta.tool_name
        );
        assert_eq!(
            reg.graph_builder_call, tool.meta.graph_builder_call,
            "graph_builder_call mismatch for '{}'",
            tool.meta.tool_name
        );
        assert_eq!(
            reg.returns_result, tool.meta.returns_result,
            "returns_result mismatch for '{}'",
            tool.meta.tool_name
        );
    }

    // Every registration has a matching all_tools entry
    let tool_names: Vec<&str> = all.iter().map(|t| t.meta.tool_name.as_str()).collect();
    for reg in iter_tool_targets() {
        assert!(
            tool_names.contains(&reg.tool_name),
            "tool '{}' has #[tool_target] but is missing from all_tools()",
            reg.tool_name
        );
    }
}
