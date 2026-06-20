//! Temporary scan: find live parse errors under src/v2/.
#![allow(dead_code)]

use crate::helpers::*;
use std::path::Path;

fn collect_dag_files(dir: &Path, out: &mut Vec<String>) {
    let entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .collect();
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, out);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            out.push(
                path.strip_prefix(workspace_root())
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }
}

#[test]
fn scan_v2_parse_temp() {
    let root = workspace_root().join("src/v2");
    let mut files = Vec::new();
    collect_dag_files(&root, &mut files);
    files.sort();
    assert!(!files.is_empty(), "no .dag files under src/v2/");

    let mut errors = Vec::new();
    for path in &files {
        let source = read_v2_file(path);
        let result = parse_source_named(path, &source);
        if let Some(ref err) = result.error {
            errors.push(format!(
                "{}: {}",
                path,
                v1_compiler::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
            ));
        }
    }

    if !errors.is_empty() {
        panic!(
            "src/v2 parse errors ({}):\n{}",
            errors.len(),
            errors
                .iter()
                .map(|e| format!("  {e}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
