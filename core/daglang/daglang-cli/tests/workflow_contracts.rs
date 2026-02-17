// Test infrastructure: filesystem access for golden fixtures
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct WorkflowFixture {
    scenario: &'static str,
    module: &'static str,
    fixture_file: &'static str,
}

const WORKFLOW_FIXTURES: &[WorkflowFixture] = &[
    WorkflowFixture {
        scenario: "S1",
        module: "tools.makegen",
        fixture_file: "s1_makegen.json",
    },
    WorkflowFixture {
        scenario: "S2",
        module: "cloud.gcp.credential",
        fixture_file: "s2_credential_chain_gcp.json",
    },
    WorkflowFixture {
        scenario: "S3",
        module: "tools.bootstrap",
        fixture_file: "s3_tool_install_upsert.json",
    },
    WorkflowFixture {
        scenario: "S4",
        module: "tools.gist",
        fixture_file: "s4_gist_snapshot.json",
    },
    WorkflowFixture {
        scenario: "S5",
        module: "pipelines.ci",
        fixture_file: "s5_ci_pipeline.json",
    },
    WorkflowFixture {
        scenario: "S6",
        module: "examples.abstract_services",
        fixture_file: "s6_llm_review.json",
    },
    WorkflowFixture {
        scenario: "W-credential-aws",
        module: "cloud.aws.credential",
        fixture_file: "w_credential_aws.json",
    },
    WorkflowFixture {
        scenario: "W-credential-azure",
        module: "cloud.azure.credential",
        fixture_file: "w_credential_azure.json",
    },
    WorkflowFixture {
        scenario: "W-clippy",
        module: "tools.clippy",
        fixture_file: "w_clippy.json",
    },
    WorkflowFixture {
        scenario: "W-deps",
        module: "tools.deps",
        fixture_file: "w_deps.json",
    },
    WorkflowFixture {
        scenario: "W-auth",
        module: "services.shell",
        fixture_file: "w_auth.json",
    },
    WorkflowFixture {
        scenario: "W-contract-aws",
        module: "infra.aws.resources",
        fixture_file: "w_contract_aws_resources.json",
    },
    WorkflowFixture {
        scenario: "W-contract-azure",
        module: "infra.azure.resources",
        fixture_file: "w_contract_azure_resources.json",
    },
    WorkflowFixture {
        scenario: "S8",
        module: "infra.core",
        fixture_file: "s8_infra_bootstrap.json",
    },
    WorkflowFixture {
        scenario: "S9",
        module: "examples.deployment",
        fixture_file: "s9_cross_cloud_deployment.json",
    },
];

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/workflow_fixtures")
}

fn daglang_bin() -> &'static str {
    env!("CARGO_BIN_EXE_daglang")
}

fn normalize_module_path(path: &str) -> String {
    let mut normalized_components = Path::new(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    if let Some(dsl_index) = normalized_components
        .iter()
        .position(|component| component.eq_ignore_ascii_case("dsl"))
    {
        normalized_components.drain(..=dsl_index);
        return normalized_components.join("/");
    }
    normalized_components.join("/")
}

fn normalize_module_fixture(module: &Value) -> Value {
    let module_name = module
        .get("module")
        .and_then(Value::as_str)
        .expect("module field should be a string");
    let path = module
        .get("path")
        .and_then(Value::as_str)
        .expect("path field should be a string");
    let items = module
        .get("items")
        .and_then(Value::as_u64)
        .expect("items field should be an integer");
    let dependencies = module
        .get("dependencies")
        .and_then(Value::as_array)
        .expect("dependencies field should be an array")
        .iter()
        .map(|dep| {
            dep.as_str()
                .expect("dependency entries should be strings")
                .to_string()
        })
        .collect::<Vec<_>>();
    json!({
        "module": module_name,
        "path": normalize_module_path(path),
        "items": items,
        "dependencies": dependencies,
    })
}

fn expected_module_snapshot(fixture: &Value) -> Value {
    let module_name = fixture
        .get("module")
        .and_then(Value::as_str)
        .expect("fixture module field should be a string");
    let path = fixture
        .get("path")
        .and_then(Value::as_str)
        .expect("fixture path field should be a string");
    let items = fixture
        .get("items")
        .and_then(Value::as_u64)
        .expect("fixture items field should be an integer");
    let dependencies = fixture
        .get("dependencies")
        .and_then(Value::as_array)
        .expect("fixture dependencies field should be an array")
        .iter()
        .map(|dep| {
            dep.as_str()
                .expect("fixture dependency entries should be strings")
                .to_string()
        })
        .collect::<Vec<_>>();
    json!({
        "module": module_name,
        "path": path,
        "items": items,
        "dependencies": dependencies,
    })
}

fn classify_command_status(output: &Output) -> &'static str {
    if output.status.success() {
        return "success";
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stderr}\n{stdout}");
    if combined.contains("typecheck errors") {
        return "typecheck_error";
    }
    if combined.contains("lower error") {
        return "lower_error";
    }
    "error"
}

fn combined_output(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    )
}

fn load_fixture(path: &Path) -> Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("invalid fixture json {}: {err}", path.display()))
}

fn unique_temp_dir(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "daglang_workflow_contracts_{label}_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ))
}

#[test]
fn workflow_module_fixtures_match_golden_snapshots() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules --format json");
    assert!(
        output.status.success(),
        "modules command should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let parsed: Value =
        serde_json::from_slice(&output.stdout).expect("modules json output should parse");
    let modules = parsed
        .get("modules")
        .and_then(Value::as_array)
        .expect("modules key should be an array");

    for fixture in WORKFLOW_FIXTURES {
        let module_entry = modules
            .iter()
            .find(|entry| entry.get("module").and_then(Value::as_str) == Some(fixture.module))
            .unwrap_or_else(|| {
                panic!(
                    "module `{}` for scenario {} missing from modules output",
                    fixture.module, fixture.scenario
                )
            });
        let actual = normalize_module_fixture(module_entry);
        let expected = load_fixture(&fixture_dir().join(fixture.fixture_file));
        let expected_snapshot = expected_module_snapshot(&expected);
        assert_eq!(
            actual, expected_snapshot,
            "workflow fixture mismatch for scenario {} module {}",
            fixture.scenario, fixture.module
        );
    }
}

#[test]
fn workflow_expand_contracts_match_golden_snapshots() {
    let root = workspace_root();
    for fixture in WORKFLOW_FIXTURES {
        let expected = load_fixture(&fixture_dir().join(fixture.fixture_file));
        let relative_path = expected
            .get("path")
            .and_then(Value::as_str)
            .expect("fixture path should be present");
        let expected_expand = expected
            .get("expand_contract")
            .expect("fixture should include expand_contract object");
        let expected_status = expected_expand
            .get("status")
            .and_then(Value::as_str)
            .expect("expand_contract.status should be a string");
        let output = Command::new(daglang_bin())
            .arg("expand")
            .arg(format!("dsl/{relative_path}"))
            .current_dir(&root)
            .output()
            .unwrap_or_else(|err| {
                panic!(
                    "failed to run daglang expand for scenario {} ({}): {err}",
                    fixture.scenario, relative_path
                )
            });
        let actual_status = classify_command_status(&output);
        assert_eq!(
            actual_status, expected_status,
            "unexpected expand status for scenario {} module {}",
            fixture.scenario, fixture.module
        );
        if let Some(expected_substring) = expected_expand
            .get("error_contains")
            .and_then(Value::as_str)
        {
            let output_text = combined_output(&output);
            assert!(
                output_text.contains(expected_substring),
                "expected expand output for scenario {} to contain `{expected_substring}`, got: {}",
                fixture.scenario,
                output_text
            );
        }
    }
}

#[test]
fn workflow_obligations_contracts_match_golden_snapshots() {
    let root = workspace_root();
    for fixture in WORKFLOW_FIXTURES {
        let expected = load_fixture(&fixture_dir().join(fixture.fixture_file));
        let relative_path = expected
            .get("path")
            .and_then(Value::as_str)
            .expect("fixture path should be present");
        let expected_obligations = expected
            .get("obligations_contract")
            .expect("fixture should include obligations_contract object");
        let expected_status = expected_obligations
            .get("status")
            .and_then(Value::as_str)
            .expect("obligations_contract.status should be a string");
        let output = Command::new(daglang_bin())
            .arg("obligations")
            .arg(format!("dsl/{relative_path}"))
            .arg("--format")
            .arg("json")
            .current_dir(&root)
            .output()
            .unwrap_or_else(|err| {
                panic!(
                    "failed to run daglang obligations for scenario {} ({}): {err}",
                    fixture.scenario, relative_path
                )
            });
        let actual_status = classify_command_status(&output);
        assert_eq!(
            actual_status, expected_status,
            "unexpected obligations status for scenario {} module {}",
            fixture.scenario, fixture.module
        );
        if actual_status == "success" {
            let expected_json = expected_obligations
                .get("expected_json")
                .expect("successful obligations contract should include expected_json");
            let actual_json: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
                panic!(
                    "obligations json for scenario {} should parse: {err}",
                    fixture.scenario
                )
            });
            assert_eq!(
                actual_json,
                expected_json.clone(),
                "obligations json mismatch for scenario {} module {}",
                fixture.scenario, fixture.module
            );
        }
        if let Some(expected_substring) = expected_obligations
            .get("error_contains")
            .and_then(Value::as_str)
        {
            let output_text = combined_output(&output);
            assert!(
                output_text.contains(expected_substring),
                "expected obligations output for scenario {} to contain `{expected_substring}`, got: {}",
                fixture.scenario,
                output_text
            );
        }
    }
}

#[test]
fn workflow_fixture_registry_has_unique_entries_and_matrix_coverage() {
    let mut scenarios = HashSet::new();
    let mut modules = HashSet::new();
    let mut fixture_files = HashSet::new();
    for fixture in WORKFLOW_FIXTURES {
        assert!(
            scenarios.insert(fixture.scenario),
            "duplicate scenario key in workflow fixture registry: {}",
            fixture.scenario
        );
        assert!(
            modules.insert(fixture.module),
            "duplicate module in workflow fixture registry: {}",
            fixture.module
        );
        assert!(
            fixture_files.insert(fixture.fixture_file),
            "duplicate fixture file in workflow fixture registry: {}",
            fixture.fixture_file
        );
    }
    assert!(
        WORKFLOW_FIXTURES.len() >= 11,
        "expected workflow fixture coverage for matrix rows, got {}",
        WORKFLOW_FIXTURES.len()
    );
}

#[test]
fn workflow_expand_negative_fixture_typecheck_error_contract() {
    let fixture = load_fixture(&fixture_dir().join("w_negative_typecheck.json"));
    let source = fixture
        .get("source")
        .and_then(Value::as_str)
        .expect("negative fixture should include source");
    let expected_expand = fixture
        .get("expand_contract")
        .expect("negative fixture should include expand_contract");
    let expected_status = expected_expand
        .get("status")
        .and_then(Value::as_str)
        .expect("negative fixture expand_contract.status should be a string");
    let expected_substring = expected_expand
        .get("error_contains")
        .and_then(Value::as_str)
        .expect("negative fixture should include error_contains");

    let temp_dir = unique_temp_dir("negative_expand");
    std::fs::create_dir_all(&temp_dir).expect("failed to create temp dir");
    let dag_file = temp_dir.join("negative_typecheck.dag");
    std::fs::write(&dag_file, source).expect("failed to write negative fixture dag file");

    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(&dag_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand for negative fixture");
    let actual_status = classify_command_status(&output);
    assert_eq!(
        actual_status, expected_status,
        "negative fixture status should match expected expand status"
    );
    let output_text = combined_output(&output);
    assert!(
        output_text.contains(expected_substring),
        "expected negative fixture output to contain `{expected_substring}`, got: {}",
        output_text
    );

    std::fs::remove_dir_all(&temp_dir).expect("failed to remove temp dir");
}
