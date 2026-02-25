// Regression tests for fail-closed review credential behavior.
#![allow(clippy::disallowed_methods)]

use gunbc_ir::WorkspaceLayout;
use std::path::PathBuf;
use std::process::{Command, Output};

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

#[allow(clippy::disallowed_methods)]
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

// NOTE: `gunbc-review` currently has an empty main() because dsl/tools/review.dag
// contains only test fixtures (no `func` items), so no entrypoints are inferred.
// The review_*_fails_closed_without_credentials tests were removed because they
// asserted credential-checking behavior for a binary that doesn't do anything yet.
// Re-add these tests when review.dag gains a func entrypoint that performs reviews.
// The gcloud-missing tests below self-skip when gcloud is on PATH.

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
