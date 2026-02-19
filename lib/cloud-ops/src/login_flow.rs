//! Login diagnostics and local env bootstrap helpers.

use crate::infra_spec::InfraSpec;
use crate::project_spec::SecretStatus;
use std::path::{Path, PathBuf};

/// Result of local login diagnostics for one environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginDiagnostics {
    pub environment: String,
    pub adc_path: String,
    pub adc_exists: bool,
    pub adc_has_refresh_token: bool,
    pub impersonation_service_account: String,
    pub impersonation_ready: bool,
    pub recommendations: Vec<String>,
    pub direnv_template: String,
}

/// Inspect local login prerequisites for a given infra spec.
pub fn inspect_login_flow(spec: &InfraSpec) -> LoginDiagnostics {
    let adc_path = resolve_adc_path();
    let (adc_exists, adc_has_refresh_token, adc_error) = check_adc_file(&adc_path);

    let impersonation_service_account = std::env::var("GCP_SECRETS_IMPERSONATE_SA")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            spec.service_accounts
                .first()
                .map(|sa| sa.email(spec.config.secrets_project))
                .unwrap_or_default()
        });
    let impersonation_ready = !impersonation_service_account.trim().is_empty();

    let mut recommendations = Vec::new();
    if !adc_exists {
        recommendations.push(
            "ADC credentials are missing; run `gcloud auth application-default login`.".to_string(),
        );
    } else if !adc_has_refresh_token {
        recommendations.push(
            "ADC is present but missing refresh_token; rerun `gcloud auth application-default login`."
                .to_string(),
        );
    }
    if let Some(error) = adc_error {
        recommendations.push(format!("ADC parse warning: {}", error));
    }
    if !impersonation_ready {
        recommendations.push(
            "No impersonation target detected; set `GCP_SECRETS_IMPERSONATE_SA` or provide a service account in the spec."
                .to_string(),
        );
    }

    LoginDiagnostics {
        environment: spec.environment.to_string(),
        adc_path: adc_path.display().to_string(),
        adc_exists,
        adc_has_refresh_token,
        impersonation_service_account: impersonation_service_account.clone(),
        impersonation_ready,
        recommendations,
        direnv_template: render_direnv_template(spec, &impersonation_service_account),
    }
}

fn resolve_adc_path() -> PathBuf {
    if let Ok(explicit) = std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".config/gcloud/application_default_credentials.json")
}

#[allow(clippy::disallowed_methods)] // Bootstrap diagnostic: reads local ADC credential file
fn check_adc_file(path: &Path) -> (bool, bool, Option<String>) {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return (false, false, None),
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(json) => json,
        Err(error) => return (true, false, Some(format!("invalid JSON: {}", error))),
    };
    let has_refresh_token = json
        .get("refresh_token")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());
    (true, has_refresh_token, None)
}

fn render_direnv_template(spec: &InfraSpec, impersonation_sa: &str) -> String {
    let mut lines = Vec::new();
    lines.push("# .envrc snippet for gunbc local cloud auth".to_string());
    lines.push("export CLOUD_PROVIDER='gcp'".to_string());
    lines.push("export CLOUD_RUNTIME='local'".to_string());
    lines.push(format!(
        "export GCP_SECRETS_PROJECT='{}'",
        spec.config.secrets_project
    ));
    lines.push(format!(
        "export GCP_SECRETS_PREFIX='{}'",
        spec.config.secrets_prefix
    ));
    lines.push(format!(
        "export GCP_WIF_PROVIDER='{}'",
        spec.wif.provider_resource_name()
    ));
    if !impersonation_sa.trim().is_empty() {
        lines.push(format!(
            "export GCP_SECRETS_IMPERSONATE_SA='{}'",
            impersonation_sa
        ));
    }
    lines.push(String::new());
    lines.push("# Optional local overrides for secret values".to_string());
    for secret in spec
        .secrets
        .iter()
        .filter(|secret| secret.status == SecretStatus::Active)
    {
        lines.push(format!("# export {}='<value>'", secret.env_name));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra_spec::DEV_SPEC;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn check_adc_file_reports_missing() {
        let path = PathBuf::from("/tmp/nonexistent-adc.json");
        let (exists, has_refresh_token, warning) = check_adc_file(&path);
        assert!(!exists);
        assert!(!has_refresh_token);
        assert!(warning.is_none());
    }

    #[test]
    #[allow(clippy::disallowed_methods)] // Test fixture: writes/cleans temp file
    fn check_adc_file_detects_refresh_token() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("gunbc-adc-{unique}.json"));
        std::fs::write(&path, r#"{"type":"authorized_user","refresh_token":"tok"}"#)
            .expect("should write temp ADC fixture");

        let (exists, has_refresh_token, warning) = check_adc_file(&path);
        assert!(exists);
        assert!(has_refresh_token);
        assert!(warning.is_none());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn inspect_login_flow_renders_direnv_template() {
        let diagnostics = inspect_login_flow(&DEV_SPEC);
        assert!(diagnostics
            .direnv_template
            .contains("export GCP_SECRETS_PROJECT='gunbai-secrets'"));
        assert!(diagnostics
            .direnv_template
            .contains("export GCP_SECRETS_PREFIX='dev-'"));
        assert!(diagnostics
            .direnv_template
            .contains("export GCP_WIF_PROVIDER='projects/314501921854/locations/global/workloadIdentityPools/github-pool/providers/github'"));
    }
}
