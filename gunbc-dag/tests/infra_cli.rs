use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn infra_bin() -> &'static str {
    env!("CARGO_BIN_EXE_gunbc-infra")
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "gunbc_infra_cli_{label}_{}_{}",
        std::process::id(),
        nanos
    ))
}

#[test]
fn spec_command_emits_structured_json() {
    let output = Command::new(infra_bin())
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
    let output = Command::new(infra_bin())
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
    let temp_home = unique_temp_dir("status_missing_adc");
    std::fs::create_dir_all(&temp_home).expect("create temp HOME");

    let output = Command::new(infra_bin())
        .arg("status")
        .arg("--env")
        .arg("dev")
        .env("HOME", &temp_home)
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

    std::fs::remove_dir_all(temp_home).expect("cleanup temp HOME");
}

#[test]
fn status_command_succeeds_with_adc_refresh_token_in_isolated_home() {
    let temp_home = unique_temp_dir("status_with_adc");
    let adc_path = temp_home
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

    let output = Command::new(infra_bin())
        .arg("status")
        .arg("--env")
        .arg("dev")
        .env("HOME", &temp_home)
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

    std::fs::remove_dir_all(temp_home).expect("cleanup temp HOME");
}

#[test]
fn reconcile_command_fails_closed_when_status_unhealthy() {
    let temp_home = unique_temp_dir("reconcile_missing_adc");
    std::fs::create_dir_all(&temp_home).expect("create temp HOME");

    let output = Command::new(infra_bin())
        .arg("reconcile")
        .arg("--env")
        .arg("dev")
        .env("HOME", &temp_home)
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

    std::fs::remove_dir_all(temp_home).expect("cleanup temp HOME");
}

#[test]
fn reconcile_command_preview_succeeds_when_health_is_ready() {
    let temp_home = unique_temp_dir("reconcile_with_adc");
    let adc_path = temp_home
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

    let output = Command::new(infra_bin())
        .arg("reconcile")
        .arg("--env")
        .arg("dev")
        .env("HOME", &temp_home)
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

    std::fs::remove_dir_all(temp_home).expect("cleanup temp HOME");
}

#[test]
fn reconcile_execute_fails_closed_without_required_inputs() {
    let temp_home = unique_temp_dir("reconcile_execute_missing_inputs");
    let adc_path = temp_home
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

    let output = Command::new(infra_bin())
        .arg("reconcile")
        .arg("--env")
        .arg("dev")
        .arg("--execute")
        .env("HOME", &temp_home)
        .output()
        .expect("run infra reconcile execute command");
    assert!(
        !output.status.success(),
        "reconcile execute should fail closed without required mutating inputs"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing entrypoint input(s)"),
        "reconcile execute failure should explain required input contract: {stderr}"
    );

    std::fs::remove_dir_all(temp_home).expect("cleanup temp HOME");
}
