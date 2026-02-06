use glob::glob;
use gunbc_codegen::all_tools;
use gunbc_tool_registry::iter_tool_targets;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Force the linker to include inventory submissions from dependency crates.
// Without these references, the linker may dead-strip the inventory symbols
// and iter_tool_targets() would return an empty iterator.
use gunbc_deps::deps_tool;
use gunbc_gist::{gist_diff_tool, gist_recent_tool, gist_snapshot_tool};
use gunbc_lib_review::review_tool;
// These are in gunbc-dag itself (same binary), but reference for completeness.
use gunbc_dag::bootstrap::bootstrap_tool;
use gunbc_dag::makegen::makegen_tool;

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
        assert_eq!(
            reg.mock_spec_call.map(|s| s.to_string()),
            tool.meta.mock_spec_call,
            "mock_spec_call mismatch for '{}'",
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

/// Every #[tool_target] builder function has at least one #[testgen_target]
/// covering it in the same crate. This prevents adding a tool without test
/// generation coverage.
#[allow(clippy::disallowed_methods)] // Test reads source files to validate registration
#[test]
fn tool_targets_have_testgen_coverage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");

    // Collect tool_target builder names and their source locations.
    // We scan all .rs files for #[...tool_target(... builder = "NAME" ...)]
    // and extract the builder function name.
    let tool_builders = collect_tool_target_builders(root);
    assert!(
        !tool_builders.is_empty(),
        "found no #[tool_target] annotations — test infrastructure broken"
    );

    // Collect testgen_target builder calls from graph_mock.rs files.
    // Each testgen_target has a builder = "crate::some_fn(...)" — we extract
    // the function name from that expression.
    let testgen_builders = collect_testgen_builder_functions(root);
    assert!(
        !testgen_builders.is_empty(),
        "found no #[testgen_target] builder calls — test infrastructure broken"
    );

    // For each tool_target builder, verify at least one testgen_target
    // references the same function name in the same crate directory.
    let mut missing = Vec::new();
    for (builder_fn, crate_dir, source_loc) in &tool_builders {
        let has_testgen = testgen_builders.iter().any(|(testgen_fn, testgen_dir)| {
            testgen_fn == builder_fn && testgen_dir == crate_dir
        });
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
#[allow(clippy::disallowed_methods)] // Test reads source files to validate registration
fn collect_tool_target_builders(root: &Path) -> Vec<(String, String, String)> {
    let pattern = format!("{}/**/*.rs", root.display());
    let mut results = Vec::new();

    for entry in glob(&pattern).expect("glob") {
        let path = entry.expect("glob entry");
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/") || path_str.contains("/buck-out/") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();

        // Find tool_target attribute blocks and extract builder = "..."
        let mut in_tool_target = false;
        let mut attr_start = 0;
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("tool_target(") && !trimmed.starts_with("//") {
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
#[allow(clippy::disallowed_methods)] // Test reads source files to validate registration
fn collect_testgen_builder_functions(root: &Path) -> Vec<(String, String)> {
    let pattern = format!("{}/**/graph_mock.rs", root.display());
    let mut results = Vec::new();

    for entry in glob(&pattern).expect("glob") {
        let path = entry.expect("glob entry");
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/") || path_str.contains("/buck-out/") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let crate_dir = crate_dir_from_path(root, &path);

        // Find testgen_target attribute blocks and extract builder = "..."
        let mut in_testgen_target = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("testgen_target(") && !trimmed.starts_with("//") {
                in_testgen_target = true;
            }
            if in_testgen_target {
                if let Some(builder_call) = extract_attr_value(trimmed, "builder") {
                    // builder_call is like "crate::build_gist_graph(...).unwrap()"
                    // Extract the function name (last path segment before '(')
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
/// Handles both `key = "value"` and `key = r#"value"#` syntax.
fn extract_attr_value(line: &str, key: &str) -> Option<String> {
    // Try regular string: key = "value"
    let needle = format!("{} = \"", key);
    if let Some(start) = line.find(&needle) {
        let after = &line[start + needle.len()..];
        let end = after.find('"')?;
        return Some(after[..end].to_string());
    }
    // Try raw string: key = r#"value"#
    let raw_needle = format!("{} = r#\"", key);
    if let Some(start) = line.find(&raw_needle) {
        let after = &line[start + raw_needle.len()..];
        let end = after.find("\"#")?;
        return Some(after[..end].to_string());
    }
    None
}

/// Extract the function name from a builder call expression.
/// e.g. "crate::build_gist_graph(crate::GistMode::Snapshot, ...)" → "build_gist_graph"
/// e.g. "build_deps_graph" → "build_deps_graph"
/// e.g. "crate::build_deps_graph().unwrap()" → "build_deps_graph"
fn extract_fn_name_from_call(call: &str) -> String {
    // Strip everything from first '(' onward (removes args)
    let without_args = call.split('(').next().unwrap_or(call);
    // Strip crate:: or module:: prefixes (take last segment)
    without_args
        .rsplit("::")
        .next()
        .unwrap_or(without_args)
        .to_string()
}

/// Derive the crate directory (relative to workspace root) from a file path.
/// e.g. /root/lib/tools/gist/src/lib.rs → "lib/tools/gist"
/// e.g. /root/gunbc-dag/src/makegen/mod.rs → "gunbc-dag"
fn crate_dir_from_path(root: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(root).unwrap_or(file);
    // Walk up from the file to find the directory containing Cargo.toml
    let mut dir = relative.parent().unwrap_or(relative);
    loop {
        if root.join(dir).join("Cargo.toml").exists() {
            return dir.to_string_lossy().to_string();
        }
        match dir.parent() {
            Some(p) if !p.as_os_str().is_empty() => dir = p,
            _ => return relative
                .components()
                .next()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .unwrap_or_default(),
        }
    }
}
