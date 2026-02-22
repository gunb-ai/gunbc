use gunbc_ir::resource::ResourceIo;
use gunbc_lib_transport::TransportIo;
use gunbc_testgen_registry::iter_resource_tests;
use std::collections::HashSet;
use std::path::Path;

#[test]
fn public_zero_arg_graph_builders_are_resource_registered() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let io = TransportIo::new();

    let builders = collect_public_zero_arg_graph_builders(root, &io);
    assert!(
        !builders.is_empty(),
        "found no public zero-arg graph builders; coverage test is broken"
    );

    let registered = collect_non_skip_resource_builder_registrations(root, &io);
    assert!(
        !registered.is_empty(),
        "found no non-skip #[resource_test_target] registrations; coverage test is broken"
    );

    let mut missing = Vec::new();
    for (builder_fn, crate_dir, source_loc) in builders {
        let covered = registered
            .iter()
            .any(|(_, registered_fn)| registered_fn == &builder_fn);
        if !covered {
            missing.push(format!(
                "{}: builder '{}' (crate {}) is not covered by non-skip #[resource_test_target]",
                source_loc, builder_fn, crate_dir
            ));
        }
    }

    assert!(
        missing.is_empty(),
        "public zero-arg graph builders missing resource registry coverage:\n{}",
        missing.join("\n")
    );
}

#[test]
fn runtime_registry_contains_non_skip_resource_annotations() {
    // Touch representative symbols so linker keeps object files that contain
    // inventory submissions from graph + graph_mock modules.
    let _: fn() -> gunbc_test::MockSpec = gunbc_clippy::graph_mock::clippy_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_deps::graph_mock::deps_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_gist::graph_mock::gist_snapshot_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_gist::graph_mock::gist_diff_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_gist::graph_mock::gist_recent_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_lib_gcp_ops::graph_mock::gcp_github_mock_spec;
    let _: fn() -> gunbc_test::MockSpec =
        gunbc_lib_gcp_ops::graph_mock::gcp_github_upsert_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_lib_llm_ops::graph_mock::openai_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_lib_review::graph_mock::inline_review_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_lib_review::graph_mock::diff_review_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_dag::bootstrap::graph_mock::bootstrap_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_dag::ci::graph_mock::ci_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_dag::makegen::graph_mock::makegen_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_dag::pragma::graph_mock::pragma_mock_spec;
    let _: fn() = gunbc_dag::testgen_dag::testgen_dag_resource_target;
    let _: fn() -> gunbc_test::MockSpec =
        gunbc_dag::credential_lifecycle::github_credential_lifecycle_mock_spec;
    let _ = gunbc_dag::build_build_graph();
    let _ = gunbc_dag::build_codegen_graph();
    let _ = gunbc_dag::build_docgen_graph();

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let io = TransportIo::new();

    let expected = collect_non_skip_resource_target_names(root, &io);
    assert!(
        !expected.is_empty(),
        "found no non-skip #[resource_test_target] annotations; coverage test is broken"
    );

    let runtime: HashSet<&str> = iter_resource_tests().map(|def| def.name).collect();
    assert!(
        !runtime.is_empty(),
        "runtime iter_resource_tests() returned no entries"
    );

    let mut missing = Vec::new();
    for (name, source_loc) in expected {
        if !runtime.contains(name.as_str()) {
            missing.push(format!(
                "{}: resource target '{}' missing from runtime registry",
                source_loc, name
            ));
        }
    }

    assert!(
        missing.is_empty(),
        "non-skip #[resource_test_target] annotations missing at runtime:\n{}",
        missing.join("\n")
    );
}

fn collect_public_zero_arg_graph_builders(
    root: &Path,
    io: &TransportIo,
) -> Vec<(String, String, String)> {
    let pattern = format!("{}/**/*.rs", root.display());
    let mut results = Vec::new();

    let paths = match io.glob_paths(&pattern) {
        Ok(paths) => paths,
        Err(_) => return results,
    };

    for path in paths {
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/")
            || path_str.contains("/buck-out/")
            || !path_str.contains("/src/")
        {
            continue;
        }

        let content = match io.read_file(&path) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(c) => c,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("pub fn build_") {
                continue;
            }
            if !trimmed.contains("graph") {
                continue;
            }
            if !trimmed.contains("-> Dag<") && !trimmed.contains("-> Result<Dag<") {
                continue;
            }

            let rest = match trimmed.strip_prefix("pub fn ") {
                Some(rest) => rest,
                None => continue,
            };
            let paren = match rest.find('(') {
                Some(idx) => idx,
                None => continue,
            };
            let fn_name = rest[..paren].trim();
            let after = &rest[paren + 1..];
            let close = match after.find(')') {
                Some(idx) => idx,
                None => continue,
            };
            if !after[..close].trim().is_empty() {
                continue;
            }

            let crate_dir = crate_dir_from_path(root, &path);
            let source_loc = format!("{}:{}", path.display(), idx + 1);
            results.push((fn_name.to_string(), crate_dir, source_loc));
        }
    }

    results
}

fn collect_non_skip_resource_builder_registrations(
    root: &Path,
    io: &TransportIo,
) -> HashSet<(String, String)> {
    let pattern = format!("{}/**/*.rs", root.display());
    let mut results = HashSet::new();

    let paths = match io.glob_paths(&pattern) {
        Ok(paths) => paths,
        Err(_) => return results,
    };

    for path in paths {
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/")
            || path_str.contains("/buck-out/")
            || !path_str.contains("/src/")
        {
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
        let lines: Vec<&str> = content.lines().collect();

        let mut in_attr = false;
        let mut attr_skip = false;
        let mut builder_expr: Option<String> = None;

        for line in lines {
            let trimmed = line.trim();
            if !in_attr {
                if trimmed.starts_with("#[") && trimmed.contains("resource_test_target(") {
                    in_attr = true;
                    attr_skip = has_skip_flag(trimmed);
                    builder_expr = extract_attr_value(trimmed, "builder");
                    if trimmed.contains(")]") {
                        if !attr_skip {
                            if let Some(expr) = builder_expr.as_ref() {
                                let fn_name = extract_fn_name_from_call(expr);
                                results.insert((crate_dir.clone(), fn_name));
                            }
                        }
                        in_attr = false;
                    }
                }
                continue;
            }

            if has_skip_flag(trimmed) {
                attr_skip = true;
            }
            if builder_expr.is_none() {
                builder_expr = extract_attr_value(trimmed, "builder");
            }

            if trimmed.contains(")]") {
                if !attr_skip {
                    if let Some(expr) = builder_expr.as_ref() {
                        let fn_name = extract_fn_name_from_call(expr);
                        results.insert((crate_dir.clone(), fn_name));
                    }
                }
                in_attr = false;
            }
        }
    }

    results
}

fn collect_non_skip_resource_target_names(root: &Path, io: &TransportIo) -> Vec<(String, String)> {
    let pattern = format!("{}/**/*.rs", root.display());
    let mut results = Vec::new();

    let paths = match io.glob_paths(&pattern) {
        Ok(paths) => paths,
        Err(_) => return results,
    };

    for path in paths {
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/")
            || path_str.contains("/buck-out/")
            || !path_str.contains("/src/")
        {
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

        let mut in_attr = false;
        let mut attr_skip = false;
        let mut name: Option<String> = None;
        let mut attr_start = 0usize;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !in_attr {
                if trimmed.starts_with("#[") && trimmed.contains("resource_test_target(") {
                    in_attr = true;
                    attr_start = idx;
                    attr_skip = has_skip_flag(trimmed);
                    name = extract_attr_value(trimmed, "name");
                    if trimmed.contains(")]") {
                        if !attr_skip {
                            if let Some(name) = name.as_ref() {
                                let source_loc = format!("{}:{}", path.display(), attr_start + 1);
                                results.push((name.clone(), source_loc));
                            }
                        }
                        in_attr = false;
                    }
                }
                continue;
            }

            if has_skip_flag(trimmed) {
                attr_skip = true;
            }
            if name.is_none() {
                name = extract_attr_value(trimmed, "name");
            }

            if trimmed.contains(")]") {
                if !attr_skip {
                    if let Some(name) = name.as_ref() {
                        let source_loc = format!("{}:{}", path.display(), attr_start + 1);
                        results.push((name.clone(), source_loc));
                    }
                }
                in_attr = false;
            }
        }
    }

    results
}

fn has_skip_flag(line: &str) -> bool {
    line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .any(|token| token == "skip")
}

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

fn extract_fn_name_from_call(call: &str) -> String {
    let without_args = call.split('(').next().unwrap_or(call);
    without_args
        .rsplit("::")
        .next()
        .unwrap_or(without_args)
        .to_string()
}

fn crate_dir_from_path(root: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(root).unwrap_or(file);
    let mut dir = relative.parent().unwrap_or(relative);
    loop {
        if root.join(dir).join("Cargo.toml").exists() {
            return dir.to_string_lossy().to_string();
        }
        match dir.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => dir = parent,
            _ => {
                return relative
                    .components()
                    .next()
                    .map(|component| component.as_os_str().to_string_lossy().to_string())
                    .unwrap_or_default();
            }
        }
    }
}
