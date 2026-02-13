use gunbc_ir::resource::ResourceIo;
use gunbc_lib_transport::TransportIo;
use gunbc_test::FermiCost;
use std::path::Path;

/// Testgen targets with `live_required` secrets must have live_fermi_cost > S.
///
/// The preflight test gate uses `GUNBC_TEST_MAX_COST=S` to skip expensive tests.
/// If a live test requires secrets but has cost <= S, it would run during
/// preflight and panic on missing secrets in CI (since the CI workflow does
/// not provide cloud credentials for the preflight sanity check).
#[test]
fn live_targets_with_secrets_have_cost_above_preflight_gate() {
    // File-based check: scan testgen_target annotations for live_required
    // and verify they also have live_fermi above "S".
    let io = TransportIo::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let pattern = format!("{}/**/*.rs", root.display());

    let mut violations = Vec::new();

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

        // Find testgen_target attribute blocks with live_required.
        let mut i = 0;
        while i < lines.len() {
            let trimmed = lines[i].trim();
            // Only match attribute annotations, not macro definitions or comments.
            if trimmed.starts_with("#[") && trimmed.contains("testgen_target(") {
                // Scan the attribute block for live_required and live_fermi.
                let start = i;
                let mut has_live_required = false;
                let mut live_fermi: Option<FermiCost> = None;
                let mut is_skip = false;

                // Scan forward to find the end of the attribute + function.
                let mut j = i;
                while j < lines.len() {
                    let line = lines[j].trim();
                    if line.contains("skip") && j == start {
                        is_skip = true;
                    }
                    if line.contains("live_required(") || line.contains("live_required_any_of(") {
                        has_live_required = true;
                    }
                    if let Some(fermi_str) = extract_live_fermi(line) {
                        live_fermi = FermiCost::parse(fermi_str);
                    }
                    // End of attribute block: next line starts with `pub fn` or `fn`.
                    if j > start && (line.starts_with("pub fn") || line.starts_with("fn ")) {
                        break;
                    }
                    j += 1;
                }

                if !is_skip && has_live_required {
                    let cost = live_fermi.unwrap_or(FermiCost::S);
                    if cost <= FermiCost::S {
                        violations.push(format!(
                            "{}:{}: testgen_target has live_required secrets but live_fermi={} (must be > S for preflight gate)",
                            path.display(),
                            start + 1,
                            cost.as_str()
                        ));
                    }
                }

                i = j + 1;
            } else {
                i += 1;
            }
        }
    }

    assert!(
        violations.is_empty(),
        "testgen targets with live_required secrets need live_fermi > S \
         (preflight gate uses GUNBC_TEST_MAX_COST=S):\n{}",
        violations.join("\n")
    );
}

/// Extract the live_fermi value from a line like `live_fermi = "M"`.
fn extract_live_fermi(line: &str) -> Option<&str> {
    let needle = "live_fermi";
    let idx = line.find(needle)?;
    let rest = &line[idx + needle.len()..];
    let quote_start = rest.find('"')? + 1;
    let rest2 = &rest[quote_start..];
    let quote_end = rest2.find('"')?;
    Some(&rest2[..quote_end])
}

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
