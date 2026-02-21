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

mod common;
use common::fixture::CliTestContext;

fn run_review_without_credentials(provider: &str) -> Output {
    let ctx = CliTestContext::new(provider, review_bin());
    let missing_adc_path = ctx.join("missing-adc.json");

    ctx.command()
        .arg("-r")
        .arg(".")
        .arg("--provider")
        .arg(provider)
        .current_dir(workspace_root())
        .env("GOOGLE_APPLICATION_CREDENTIALS", &missing_adc_path)
        .env_remove("OPENAI_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("GUNBC_CREDENTIAL_POLICY_JSON")
        .output()
        .expect("run gunbc-review")
}

fn gcloud_available() -> bool {
    Command::new("bash")
        .arg("-lc")
        .arg("command -v gcloud >/dev/null 2>&1")
        .status()
        .is_ok_and(|status| status.success())
}

fn run_review_with_adc_only(provider: &str) -> Output {
    let ctx = CliTestContext::new(&format!("{provider}_adc_only"), review_bin());
    let adc_path = ctx.join(".config/gcloud/application_default_credentials.json");
    ctx.write_file(
        &adc_path,
        r#"{"type":"authorized_user","client_id":"test","client_secret":"test","refresh_token":"test-refresh-token"}"#,
    );

    ctx.command()
        .arg("-r")
        .arg(".")
        .arg("--provider")
        .arg(provider)
        .current_dir(workspace_root())
        .env_remove("OPENAI_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("GUNBC_CREDENTIAL_POLICY_JSON")
        .output()
        .expect("run gunbc-review with adc-only fixture")
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

#[test]
fn review_anthropic_reports_actionable_error_when_gcloud_cli_missing() {
    if gcloud_available() {
        eprintln!("SKIP: gcloud available on PATH; missing-gcloud scenario not reproducible");
        return;
    }
    let output = run_review_with_adc_only("anthropic");
    assert!(
        !output.status.success(),
        "anthropic review should fail closed when gcloud CLI is missing"
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
    assert!(
        combined.contains("provider: anthropic"),
        "failure output should preserve provider context: {combined}"
    );
}
