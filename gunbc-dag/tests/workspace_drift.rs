//! FC-P6-a / FC-P7-a: Workspace and build workflow drift tests.
//!
//! Verifies that DSL config data declarations match the actual workspace
//! state (Cargo.toml members) and that all config .dag files compile.

use daglang_driver::{compile_from_context, DriverContext};
use std::collections::BTreeSet;
use std::path::PathBuf;

fn dsl_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Verify that workspace.dag compiles successfully.
#[test]
fn workspace_dag_compiles() {
    let root = dsl_root();
    let dag_file = root.join("config/workspace.dag");
    let context = DriverContext {
        roots: vec![root],
        target_file: Some(dag_file),
    };
    compile_from_context(&context).expect("config/workspace.dag should compile");
}

/// Verify that build_workflows.dag compiles successfully.
#[test]
fn build_workflows_dag_compiles() {
    let root = dsl_root();
    let dag_file = root.join("config/build_workflows.dag");
    let context = DriverContext {
        roots: vec![root],
        target_file: Some(dag_file),
    };
    compile_from_context(&context).expect("config/build_workflows.dag should compile");
}

/// Verify that workspace.dag crate paths match Cargo.toml [workspace] members.
#[test]
#[allow(clippy::disallowed_methods)]
fn workspace_crate_paths_match_cargo_toml() {
    let ws_root = workspace_root();
    let cargo_toml = std::fs::read_to_string(ws_root.join("Cargo.toml"))
        .expect("workspace Cargo.toml should be readable");

    // Extract workspace members from Cargo.toml [workspace] members array.
    let cargo_members: BTreeSet<String> = {
        let mut in_members = false;
        let mut members = BTreeSet::new();
        for line in cargo_toml.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("members") && trimmed.contains('[') {
                in_members = true;
                continue;
            }
            if in_members {
                if trimmed == "]" || trimmed.starts_with(']') {
                    break;
                }
                // Skip comments.
                if trimmed.starts_with('#') || trimmed.is_empty() {
                    continue;
                }
                // Extract quoted path: "core/infra",
                let path =
                    trimmed.trim_matches(|c: char| c == '"' || c == ',' || c.is_whitespace());
                if !path.is_empty() {
                    members.insert(path.to_string());
                }
            }
        }
        members
    };

    // Extract paths from workspace.dag data declaration.
    let dag_content = std::fs::read_to_string(dsl_root().join("config/workspace.dag"))
        .expect("workspace.dag should be readable");

    let dag_paths: BTreeSet<String> = dag_content
        .lines()
        .map(|l| l.trim())
        .filter(|l| l.contains("path:"))
        .filter_map(|l| {
            let start = l.find("path:")? + 5;
            let rest = l[start..].trim();
            let rest = rest.trim_start_matches('"');
            let end = rest.find('"')?;
            Some(rest[..end].to_string())
        })
        .collect();

    let missing_in_dag: BTreeSet<&String> = cargo_members.difference(&dag_paths).collect();
    let extra_in_dag: BTreeSet<&String> = dag_paths.difference(&cargo_members).collect();

    let mut violations = Vec::new();
    if !missing_in_dag.is_empty() {
        violations.push(format!(
            "Cargo.toml members missing from workspace.dag: {:?}",
            missing_in_dag
        ));
    }
    if !extra_in_dag.is_empty() {
        violations.push(format!(
            "workspace.dag paths not in Cargo.toml: {:?}",
            extra_in_dag
        ));
    }

    assert!(
        violations.is_empty(),
        "workspace drift detected:\n{}",
        violations.join("\n"),
    );
}

/// Verify all config .dag files compile.
#[test]
#[allow(clippy::disallowed_methods)]
fn all_config_dag_files_compile() {
    let root = dsl_root();
    let config_dir = root.join("config");
    let mut errors = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&config_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "dag") {
                let context = DriverContext {
                    roots: vec![root.clone()],
                    target_file: Some(path.clone()),
                };
                if let Err(e) = compile_from_context(&context) {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    errors.push(format!("config/{name}: {e}"));
                }
            }
        }
    }

    assert!(
        errors.is_empty(),
        "config .dag files failed to compile:\n{}",
        errors.join("\n"),
    );
}
