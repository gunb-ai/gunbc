use gunbc_ir::resource::ResourceIo;
use gunbc_lib_transport::TransportIo;
use gunbc_test::FermiCost;
use gunbc_testgen_registry::iter_dag_specs;
use std::path::Path;
// Force-link crates with live test target registrations.
use gunbc_lib_llm_ops as _;

#[derive(Debug, Clone)]
struct LiveSecretTarget {
    name: String,
    output_path: String,
    live_flow_tests: bool,
    live_fermi_cost: Option<FermiCost>,
    live_required: Vec<String>,
    live_required_any_of: Vec<Vec<String>>,
}

fn collect_live_secret_targets() -> Vec<LiveSecretTarget> {
    // Keep inventory submit objects from being stripped by the linker.
    let _: fn() -> gunbc_test::MockSpec =
        gunbc_dag::credential_lifecycle::github_credential_lifecycle_mock_spec;
    let _: fn() -> gunbc_test::MockSpec =
        gunbc_lib_llm_ops::graph_mock::credential_lifecycle_mock_spec;
    let _: fn() -> gunbc_test::MockSpec =
        gunbc_lib_llm_ops::graph_mock::credential_lifecycle_anthropic_mock_spec;

    let mut targets = Vec::new();
    for spec in iter_dag_specs() {
        let def = spec.to_def();
        let live_required = def.live_required.unwrap_or_default();
        let live_required_any_of = def.live_required_any_of.unwrap_or_default();
        if live_required.is_empty() && live_required_any_of.is_empty() {
            continue;
        }
        targets.push(LiveSecretTarget {
            name: def.name.into_owned(),
            output_path: def.output_path.into_owned(),
            live_flow_tests: def.live_flow_tests,
            live_fermi_cost: def.live_fermi_cost,
            live_required,
            live_required_any_of,
        });
    }
    targets.sort_by(|a, b| a.name.cmp(&b.name));
    targets
}

/// Testgen targets with `live_required` secrets must have live_fermi_cost > S.
///
/// The preflight test gate uses `GUNBC_TEST_MAX_COST=S` to skip expensive tests.
/// If a live test requires secrets but has cost <= S, it would run during
/// preflight and panic on missing secrets in CI (since the CI workflow does
/// not provide cloud credentials for the preflight sanity check).
#[test]
fn live_targets_with_secrets_have_cost_above_preflight_gate() {
    let targets = collect_live_secret_targets();
    assert!(
        !targets.is_empty(),
        "no live secret test targets were registered in this test binary"
    );
    let mut violations = Vec::new();
    for target in targets {
        if !target.live_flow_tests {
            violations.push(format!(
                "{}: has live_required metadata but live_flow_tests is disabled",
                target.name
            ));
        }
        match target.live_fermi_cost {
            Some(cost) if cost > FermiCost::S => {}
            Some(cost) => violations.push(format!(
                "{}: live_fermi={} (must be > S for preflight gate)",
                target.name,
                cost.as_str()
            )),
            None => violations.push(format!(
                "{}: missing explicit live_fermi (must be set and > S for preflight gate)",
                target.name
            )),
        };
    }

    assert!(
        violations.is_empty(),
        "testgen targets with live_required secrets need live_fermi > S \
         (preflight gate uses GUNBC_TEST_MAX_COST=S):\n{}",
        violations.join("\n")
    );
}

#[test]
#[allow(clippy::disallowed_methods)] // Test reads generated test files to verify guard presence
fn generated_live_tests_include_guard_for_required_secrets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let targets = collect_live_secret_targets();
    let mut violations = Vec::new();

    for target in targets {
        let generated = root.join(&target.output_path);
        let content = match std::fs::read_to_string(&generated) {
            Ok(content) => content,
            Err(err) => {
                violations.push(format!(
                    "{}: failed to read generated test file {}: {}",
                    target.name,
                    generated.display(),
                    err
                ));
                continue;
            }
        };

        if !content.contains("guard_test_with_env(") {
            violations.push(format!(
                "{}: generated tests missing guard_test_with_env in {}",
                target.name,
                generated.display()
            ));
        }

        for required in &target.live_required {
            let needle = format!("\"{required}\"");
            if !content.contains(&needle) {
                violations.push(format!(
                    "{}: generated tests missing required secret {} in {}",
                    target.name,
                    required,
                    generated.display()
                ));
            }
        }

        for group in &target.live_required_any_of {
            for required in group {
                let needle = format!("\"{required}\"");
                if !content.contains(&needle) {
                    violations.push(format!(
                        "{}: generated tests missing any-of secret {} in {}",
                        target.name,
                        required,
                        generated.display()
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "generated live tests must include env guards for required secrets:\n{}",
        violations.join("\n")
    );
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
