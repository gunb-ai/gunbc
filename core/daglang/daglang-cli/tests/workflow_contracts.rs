// Test infrastructure: filesystem access for golden fixtures
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

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
    let marker = "/dsl/";
    path.split_once(marker)
        .map(|(_, suffix)| suffix.to_string())
        .unwrap_or_else(|| path.to_string())
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

fn load_fixture(path: &Path) -> Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("invalid fixture json {}: {err}", path.display()))
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
        assert_eq!(
            actual, expected,
            "workflow fixture mismatch for scenario {} module {}",
            fixture.scenario, fixture.module
        );
    }
}
