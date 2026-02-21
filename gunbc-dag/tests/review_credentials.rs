// Regression tests for fail-closed review credential behavior.

use gunbc_ir::WorkspaceLayout;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn workspace_root() -> PathBuf {
    WorkspaceLayout::from_env_manifest_dir()
        .expect("resolve workspace layout")
        .workspace_root
}

fn review_bin() -> &'static str {
    env!("CARGO_BIN_EXE_gunbc-review")
}

fn unique_temp_home(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "gunbc_review_credentials_{label}_{}_{}",
        std::process::id(),
        nanos
    ))
}

fn run_review_without_credentials(provider: &str) -> Output {
    let temp_home = unique_temp_home(provider);
    std::fs::create_dir_all(&temp_home).expect("create temp HOME");
    let missing_adc_path = temp_home.join("missing-adc.json");

    let output = Command::new(review_bin())
        .arg("-r")
        .arg(".")
        .arg("--provider")
        .arg(provider)
        .current_dir(workspace_root())
        .env("HOME", &temp_home)
        .env("GOOGLE_APPLICATION_CREDENTIALS", &missing_adc_path)
        .env_remove("OPENAI_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("GUNBC_CREDENTIAL_POLICY_JSON")
        .output()
        .expect("run gunbc-review");

    std::fs::remove_dir_all(temp_home).expect("remove temp HOME");
    output
}

fn gcloud_available() -> bool {
    Command::new("bash")
        .arg("-lc")
        .arg("command -v gcloud >/dev/null 2>&1")
        .status()
        .is_ok_and(|status| status.success())
}

fn run_review_with_adc_only(provider: &str) -> Output {
    let temp_home = unique_temp_home(&format!("{provider}_adc_only"));
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
    .expect("write adc fixture with refresh token");

    let output = Command::new(review_bin())
        .arg("-r")
        .arg(".")
        .arg("--provider")
        .arg(provider)
        .current_dir(workspace_root())
        .env("HOME", &temp_home)
        .env_remove("OPENAI_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("GUNBC_CREDENTIAL_POLICY_JSON")
        .output()
        .expect("run gunbc-review with adc-only fixture");

    std::fs::remove_dir_all(temp_home).expect("remove temp HOME");
    output
}

fn assert_fail_closed_output(provider: &str, output: &Output) {
    assert!(
        !output.status.success(),
        "{provider} review should fail closed without credentials"
    );
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("ADC file not found"),
        "{provider} failure should report missing ADC: {combined}"
    );
    assert!(
        combined.contains("gcloud auth application-default login"),
        "{provider} failure should include ADC remediation command: {combined}"
    );
    assert!(
        combined.contains(&format!("provider: {provider}")),
        "{provider} failure output should retain provider context: {combined}"
    );
}

#[test]
fn review_openai_fails_closed_without_credentials() {
    let output = run_review_without_credentials("openai");
    assert_fail_closed_output("openai", &output);
}

#[test]
fn review_anthropic_fails_closed_without_credentials() {
    let output = run_review_without_credentials("anthropic");
    assert_fail_closed_output("anthropic", &output);
}

#[test]
fn review_openai_reports_actionable_error_when_gcloud_cli_missing() {
    if gcloud_available() {
        eprintln!("SKIP: gcloud available on PATH; missing-gcloud scenario not reproducible");
        return;
    }
    let output = run_review_with_adc_only("openai");
    assert!(
        !output.status.success(),
        "openai review should fail closed when gcloud CLI is missing"
    );
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("failed to spawn command `gcloud`"),
        "failure output should identify missing gcloud executable: {combined}"
    );
    assert!(
        combined.contains("Install Google Cloud SDK"),
        "failure output should include gcloud installation remediation: {combined}"
    );
}
