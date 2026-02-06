use glob::glob;
use std::fs;
use std::path::Path;

#[test]
fn all_mock_specs_are_registered() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let pattern = format!("{}/**/graph_mock.rs", root.display());

    let mut missing = Vec::new();

    for entry in glob(&pattern).expect("glob pattern should be valid") {
        let path = entry.expect("glob entry should be valid");
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/") || path_str.contains("/buck-out/") {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
        let lines: Vec<&str> = content.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("pub fn")
                && trimmed.contains("mock_spec")
                && trimmed.contains("-> MockSpec")
            {
                let mut has_attr = false;
                let start = idx.saturating_sub(16);
                for j in start..idx {
                    if lines[j].contains("testgen_target") {
                        has_attr = true;
                        break;
                    }
                }
                if !has_attr {
                    missing.push(format!(
                        "{}:{}: {}",
                        path.display(),
                        idx + 1,
                        trimmed
                    ));
                }
            }
        }
    }

    assert!(
        missing.is_empty(),
        "MockSpec functions missing #[testgen_target] annotation:\n{}",
        missing.join("\n")
    );
}
