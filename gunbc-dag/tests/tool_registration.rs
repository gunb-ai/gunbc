use gunbc_codegen::derive_tool_defs;
use gunbc_ir::resource::ResourceIo;
use gunbc_lib_transport::TransportIo;
use gunbc_tool_registry::iter_tool_targets;
use std::collections::HashSet;
use std::path::Path;

// Force the linker to include inventory submissions from dependency crates.
// Without these references, the linker may dead-strip the inventory symbols
// and iter_tool_targets() would return an empty iterator.
use gunbc_clippy::clippy_tool;
use gunbc_deps::deps_tool;
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

/// Every #[tool_target] builder function has at least one #[testgen_target]
/// covering it in the same crate. This prevents adding a tool without test
/// generation coverage.
#[test]
fn tool_targets_have_testgen_coverage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");

    // Collect tool_target builder names and their source locations.
    let tool_builders = collect_tool_target_builders(root);
    assert!(
        !tool_builders.is_empty(),
        "found no #[tool_target] annotations — test infrastructure broken"
    );

    // Collect testgen_target builder calls from graph_mock.rs files.
    let testgen_builders = collect_testgen_builder_functions(root);
    assert!(
        !testgen_builders.is_empty(),
        "found no #[testgen_target] builder calls — test infrastructure broken"
    );

    // For each tool_target builder, verify at least one testgen_target
    // references the same function name in the same crate directory.
    let mut missing = Vec::new();
    for (builder_fn, crate_dir, source_loc) in &tool_builders {
        let has_testgen = testgen_builders
            .iter()
            .any(|(testgen_fn, testgen_dir)| testgen_fn == builder_fn && testgen_dir == crate_dir);
        if !has_testgen {
            missing.push(format!(
                "{}: builder '{}' (crate {}) has no #[testgen_target] coverage",
                source_loc, builder_fn, crate_dir
            ));
        }
    }

    assert!(
        missing.is_empty(),
        "#[tool_target] builders without #[testgen_target] coverage:\n{}",
        missing.join("\n")
    );
}

/// Extract (builder_fn_name, crate_dir, source_location) from #[tool_target] annotations.
fn collect_tool_target_builders(root: &Path) -> Vec<(String, String, String)> {
    let io = TransportIo::new();
    let pattern = format!("{}/**/*.rs", root.display());
    let mut results = Vec::new();

    let paths = match io.glob_paths(&pattern) {
        Ok(p) => p,
        Err(_) => return results,
    };

    for path in paths {
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/") || path_str.contains("/buck-out/") {
            continue;
        }

        let content = match io.read_file(&path) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(c) => c,
                Err(_) => continue,
            },
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();

        // Find tool_target attribute blocks and extract builder = "..."
        let mut in_tool_target = false;
        let mut attr_start = 0;
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("#[") && trimmed.contains("tool_target(") {
                in_tool_target = true;
                attr_start = idx;
            }
            if in_tool_target {
                if let Some(builder) = extract_attr_value(trimmed, "builder") {
                    let crate_dir = crate_dir_from_path(root, &path);
                    let loc = format!("{}:{}", path.display(), attr_start + 1);
                    results.push((builder, crate_dir, loc));
                }
                // End of attribute (closing paren + bracket)
                if trimmed.contains(")]") {
                    in_tool_target = false;
                }
            }
        }
    }

    results
}

/// Extract builder function names from #[testgen_target] annotations in graph_mock.rs files.
/// Returns (function_name, crate_dir).
fn collect_testgen_builder_functions(root: &Path) -> Vec<(String, String)> {
    let io = TransportIo::new();
    let pattern = format!("{}/**/graph_mock.rs", root.display());
    let mut results = Vec::new();

    let paths = match io.glob_paths(&pattern) {
        Ok(p) => p,
        Err(_) => return results,
    };

    for path in paths {
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/") || path_str.contains("/buck-out/") {
            continue;
        }

        let content = match io.read_file(&path) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(c) => c,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        let crate_dir = crate_dir_from_path(root, &path);

        // Find testgen_target attribute blocks and extract builder = "..."
        let mut in_testgen_target = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#[") && trimmed.contains("testgen_target(") {
                in_testgen_target = true;
            }
            if in_testgen_target {
                if let Some(builder_call) = extract_attr_value(trimmed, "builder") {
                    let fn_name = extract_fn_name_from_call(&builder_call);
                    results.push((fn_name, crate_dir.clone()));
                }
                if trimmed.contains(")]") {
                    in_testgen_target = false;
                }
            }
        }
    }

    results
}

/// Extract a string value from an attribute key-value pair.
fn extract_attr_value(line: &str, key: &str) -> Option<String> {
    let needle = format!("{} = \"", key);
    if let Some(start) = line.find(&needle) {
        let after = &line[start + needle.len()..];
        let end = after.find('"')?;
        return Some(after[..end].to_string());
    }
    let raw_needle = format!("{} = r#\"", key);
    if let Some(start) = line.find(&raw_needle) {
        let after = &line[start + raw_needle.len()..];
        let end = after.find("\"#")?;
        return Some(after[..end].to_string());
    }
    None
}

/// Extract the function name from a builder call expression.
fn extract_fn_name_from_call(call: &str) -> String {
    let without_args = call.split('(').next().unwrap_or(call);
    without_args
        .rsplit("::")
        .next()
        .unwrap_or(without_args)
        .to_string()
}

/// Derive the crate directory (relative to workspace root) from a file path.
fn crate_dir_from_path(root: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(root).unwrap_or(file);
    let mut dir = relative.parent().unwrap_or(relative);
    loop {
        if root.join(dir).join("Cargo.toml").exists() {
            return dir.to_string_lossy().to_string();
        }
        match dir.parent() {
            Some(p) if !p.as_os_str().is_empty() => dir = p,
            _ => {
                return relative
                    .components()
                    .next()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .unwrap_or_default()
            }
        }
    }
}
