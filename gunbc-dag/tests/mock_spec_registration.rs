use gunbc_ir::resource::ResourceIo;
use gunbc_lib_transport::TransportIo;
use std::path::Path;

#[test]
fn all_mock_specs_are_registered() {
    let io = TransportIo::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let pattern = format!("{}/**/graph_mock.rs", root.display());

    let mut missing = Vec::new();

    let paths = io
        .glob_paths(&pattern)
        .expect("glob pattern should be valid");

    for path in paths {
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/") || path_str.contains("/buck-out/") {
            continue;
        }

        let content = io
            .read_file(&path)
            .map_err(|e| format!("failed to read {}: {}", path.display(), e))
            .and_then(|bytes| String::from_utf8(bytes).map_err(|e| e.to_string()))
            .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
        let lines: Vec<&str> = content.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("pub fn")
                && trimmed.contains("mock_spec")
                && trimmed.contains("-> MockSpec")
            {
                let mut has_attr = false;
                // Attributes can be verbose (e.g., live test requirements), so
                // scan a wider window before the function signature.
                let start = idx.saturating_sub(64);
                for preceding_line in &lines[start..idx] {
                    if preceding_line.contains("testgen_target") {
                        has_attr = true;
                        break;
                    }
                }
                if !has_attr {
                    missing.push(format!("{}:{}: {}", path.display(), idx + 1, trimmed));
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
