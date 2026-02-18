//! Secret export rendering (shell/direnv integration helpers).

use crate::project_spec::{ProjectSpec, SecretRequirement, SecretStatus};
use std::collections::BTreeMap;

/// Result of rendering environment exports from fetched secrets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretExportResult {
    pub shell_exports: String,
    pub resolved: Vec<String>,
    pub missing_required: Vec<String>,
}

/// Render `export VAR=...` lines for one namespace from fetched secret values.
///
/// `fetched` keys should be prefixed secret IDs (e.g., `dev-github-token`).
pub fn render_direnv_exports(
    project_spec: &ProjectSpec,
    namespace: &str,
    fetched: &BTreeMap<String, String>,
) -> Result<SecretExportResult, String> {
    let ns = project_spec
        .namespace(namespace)
        .ok_or_else(|| format!("unknown namespace '{}'", namespace))?;

    let mut lines = Vec::new();
    let mut resolved = Vec::new();
    let mut missing_required = Vec::new();

    for secret in project_spec
        .secrets
        .iter()
        .filter(|s| s.status == SecretStatus::Active)
    {
        let prefixed = format!("{}{}", ns.secret_prefix(), secret.secret_id);
        if let Some(value) = fetched.get(&prefixed) {
            lines.push(format!(
                "export {}='{}'",
                secret.env_name,
                escape_single_quote(value)
            ));
            resolved.push(secret.env_name.to_string());
        } else if secret.requirement == SecretRequirement::Required {
            missing_required.push(secret.env_name.to_string());
        }
    }

    Ok(SecretExportResult {
        shell_exports: lines.join("\n"),
        resolved,
        missing_required,
    })
}

fn escape_single_quote(value: &str) -> String {
    value.replace('\'', "'\"'\"'")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_spec::GUNBAI_SECRETS;

    #[test]
    fn render_direnv_exports_renders_env_lines_for_resolved_values() {
        let mut fetched = BTreeMap::new();
        fetched.insert("dev-github-token".to_string(), "ghp_abc123".to_string());

        let exports =
            render_direnv_exports(&GUNBAI_SECRETS, "dev", &fetched).expect("render exports");
        assert!(exports
            .shell_exports
            .contains("export GITHUB_TOKEN='ghp_abc123'"));
        assert!(exports.missing_required.is_empty());
        assert_eq!(exports.resolved, vec!["GITHUB_TOKEN".to_string()]);
    }

    #[test]
    fn render_direnv_exports_handles_unknown_namespace() {
        let fetched = BTreeMap::new();
        let err = render_direnv_exports(&GUNBAI_SECRETS, "missing", &fetched)
            .expect_err("unknown namespace should fail");
        assert!(err.contains("unknown namespace"));
    }

    #[test]
    fn render_direnv_exports_escapes_single_quotes() {
        let mut fetched = BTreeMap::new();
        fetched.insert("dev-github-token".to_string(), "va'lue".to_string());
        let exports =
            render_direnv_exports(&GUNBAI_SECRETS, "dev", &fetched).expect("render exports");
        assert!(exports
            .shell_exports
            .contains("export GITHUB_TOKEN='va'\"'\"'lue'"));
    }
}
