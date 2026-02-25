#![allow(clippy::disallowed_methods)]
mod common;
use common::fixture::CliTestContext;

fn infra_bin() -> &'static str {
    env!("CARGO_BIN_EXE_gunbc-infra")
}

#[test]
fn spec_command_emits_structured_json() {
    let ctx = CliTestContext::new("infra", infra_bin());
    let output = ctx
        .command()
        .arg("spec")
        .arg("--env")
        .arg("dev")
        .output()
        .expect("run infra spec command");
    assert!(
        output.status.success(),
        "infra spec should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("infra spec output should be JSON");
    assert_eq!(payload["environment"], "dev");
    assert!(
        payload["service_accounts"]
            .as_array()
            .expect("service_accounts should be array")
            .iter()
            .any(|entry| entry["name"] == "gunbai-dev-secrets"),
        "infra spec should include configured service-account dependencies"
    );
    assert_eq!(payload["wif"]["pool_id"], "github-pool");
}

#[test]
fn plan_command_reports_runtime_dependency_targets() {
    let ctx = CliTestContext::new("infra", infra_bin());
    let output = ctx
        .command()
        .arg("plan")
        .arg("--env")
        .arg("dev")
        .output()
        .expect("run infra plan command");
    assert!(
        output.status.success(),
        "infra plan should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("secret:github-token"),
        "infra plan should include secret runtime dependency target: {stdout}"
    );
    assert!(
        stdout.contains("service-account:gunbai-dev-secrets"),
        "infra plan should include service-account runtime dependency target: {stdout}"
    );
    assert!(
        stdout.contains("wif:github-pool:github"),
        "infra plan should include wif runtime dependency target: {stdout}"
    );
}

#[test]
fn status_command_fails_closed_without_adc_in_isolated_home() {
    let ctx = CliTestContext::new("infra", infra_bin());

    let output = ctx
        .command()
        .arg("status")
        .arg("--env")
        .arg("dev")
        .output()
        .expect("run infra status command");
    assert!(
        !output.status.success(),
        "infra status should fail closed without adc in isolated HOME"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("[FAIL] auth"),
        "status output should include fail-closed auth section: {stdout}"
    );
    assert!(
        stdout.contains("overall: FAIL"),
        "status output should include failed overall health state: {stdout}"
    );
    assert!(
        stderr.contains("one or more health checks failed"),
        "status error should explain fail-closed health gate: {stderr}"
    );
}

#[test]
fn status_command_succeeds_with_adc_refresh_token_in_isolated_home() {
    let ctx = CliTestContext::new("infra", infra_bin());
    let adc_path = ctx
        .path()
        .to_path_buf()
        .join(".config")
        .join("gcloud")
        .join("application_default_credentials.json");
    if let Some(parent) = adc_path.parent() {
        std::fs::create_dir_all(parent).expect("create ADC parent directories");
    }
    std::fs::write(
        &adc_path,
        r#"{"type":"authorized_user","client_id":"test","client_secret":"test","refresh_token":"test-refresh-token"}"#,
    )
    .expect("write ADC fixture with refresh token");

    let output = ctx
        .command()
        .arg("status")
        .arg("--env")
        .arg("dev")
        .output()
        .expect("run infra status command");
    assert!(
        output.status.success(),
        "infra status should succeed with adc fixture: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[OK] auth"),
        "status output should include successful auth section: {stdout}"
    );
    assert!(
        stdout.contains("overall: OK"),
        "status output should include successful overall health state: {stdout}"
    );
}

#[test]
fn reconcile_command_fails_closed_when_status_unhealthy() {
    let ctx = CliTestContext::new("infra", infra_bin());

    let output = ctx
        .command()
        .arg("reconcile")
        .arg("--env")
        .arg("dev")
        .output()
        .expect("run infra reconcile command");
    assert!(
        !output.status.success(),
        "infra reconcile should fail closed when health checks are unhealthy"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot reconcile while infra health checks fail"),
        "reconcile should explain health-gated fail-closed behavior: {stderr}"
    );
}

#[test]
fn reconcile_command_preview_succeeds_when_health_is_ready() {
    let ctx = CliTestContext::new("infra", infra_bin());
    let adc_path = ctx
        .path()
        .to_path_buf()
        .join(".config")
        .join("gcloud")
        .join("application_default_credentials.json");
    if let Some(parent) = adc_path.parent() {
        std::fs::create_dir_all(parent).expect("create ADC parent directories");
    }
    std::fs::write(
        &adc_path,
        r#"{"type":"authorized_user","client_id":"test","client_secret":"test","refresh_token":"test-refresh-token"}"#,
    )
    .expect("write ADC fixture with refresh token");

    let output = ctx
        .command()
        .arg("reconcile")
        .arg("--env")
        .arg("dev")
        .output()
        .expect("run infra reconcile preview command");
    assert!(
        output.status.success(),
        "infra reconcile preview should succeed when health checks pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("infra reconcile preview"),
        "reconcile preview should print mode banner: {stdout}"
    );
    assert!(
        stdout.contains("service-account:gunbai-dev-secrets"),
        "reconcile preview should include runtime dependency targets from plan: {stdout}"
    );
}

// Tests for reconcile execute entrypoint input validation were removed:
// `run_compiled_infra_orchestration` provides all 7 entrypoint inputs
// (environment, runtime, spec_targets, target, skip, __deps, execute)
// from the InfraSpec, and `--input` CLI args aren't wired into that path.
// The removed tests asserted phantom contracts (missing inputs, invalid
// CloudSecretConfig json) that the current interface never triggers.
