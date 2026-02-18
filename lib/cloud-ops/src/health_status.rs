//! Status/health checks for infra bootstrap prerequisites.

use crate::infra_spec::InfraSpec;
use crate::login_flow::inspect_login_flow;
use crate::project_spec::SecretStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthCheckItem {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthCheckReport {
    pub overall_ok: bool,
    pub items: Vec<HealthCheckItem>,
}

/// Evaluate local/operator-facing health for one infra environment.
///
/// Checks:
/// - auth: ADC presence + refresh token readability
/// - projects: project identifiers are non-empty
/// - service accounts: configured SA emails are syntactically valid
/// - secrets: at least one active secret in the environment spec
pub fn evaluate_health(spec: &InfraSpec) -> HealthCheckReport {
    let login = inspect_login_flow(spec);

    let auth_ok = login.adc_exists && login.adc_has_refresh_token;
    let auth_detail = if auth_ok {
        format!("ADC ready at {}", login.adc_path)
    } else if !login.adc_exists {
        format!("ADC missing at {}", login.adc_path)
    } else {
        format!("ADC present at {} but refresh_token is missing", login.adc_path)
    };

    let projects_ok = !spec.config.project.trim().is_empty()
        && !spec.config.secrets_project.trim().is_empty();
    let project_detail = if projects_ok {
        format!(
            "project={} secrets_project={}",
            spec.config.project, spec.config.secrets_project
        )
    } else {
        "project identifiers must be non-empty".to_string()
    };

    let valid_service_accounts = spec
        .service_accounts
        .iter()
        .map(|sa| sa.email(spec.config.secrets_project))
        .filter(|email| email.contains('@') && email.ends_with(".iam.gserviceaccount.com"))
        .count();
    let service_accounts_ok =
        !spec.service_accounts.is_empty() && valid_service_accounts == spec.service_accounts.len();
    let service_accounts_detail = if service_accounts_ok {
        format!("{} configured service account(s)", spec.service_accounts.len())
    } else {
        format!(
            "invalid service account definitions: {} valid out of {}",
            valid_service_accounts,
            spec.service_accounts.len()
        )
    };

    let active_secrets = spec
        .secrets
        .iter()
        .filter(|secret| secret.status == SecretStatus::Active)
        .count();
    let secrets_ok = active_secrets > 0;
    let secrets_detail = if secrets_ok {
        format!("{} active secret(s)", active_secrets)
    } else {
        "no active secrets configured".to_string()
    };

    let items = vec![
        HealthCheckItem {
            name: "auth".to_string(),
            ok: auth_ok,
            detail: auth_detail,
        },
        HealthCheckItem {
            name: "projects".to_string(),
            ok: projects_ok,
            detail: project_detail,
        },
        HealthCheckItem {
            name: "service_accounts".to_string(),
            ok: service_accounts_ok,
            detail: service_accounts_detail,
        },
        HealthCheckItem {
            name: "secrets".to_string(),
            ok: secrets_ok,
            detail: secrets_detail,
        },
    ];

    let overall_ok = items.iter().all(|item| item.ok);
    HealthCheckReport { overall_ok, items }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra_spec::DEV_SPEC;
    use crate::project_spec::{SecretSpec, ServiceAccountSpec};

    #[test]
    fn health_report_covers_expected_sections() {
        let report = evaluate_health(&DEV_SPEC);
        let names: Vec<&str> = report.items.iter().map(|item| item.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["auth", "projects", "service_accounts", "secrets"]
        );
    }

    #[test]
    fn health_report_fails_when_service_accounts_or_secrets_missing() {
        static EMPTY_SERVICE_ACCOUNTS: &[ServiceAccountSpec] = &[];
        static EMPTY_SECRETS: &[SecretSpec] = &[];

        let spec = InfraSpec {
            environment: "dev",
            config: DEV_SPEC.config.clone(),
            service_accounts: EMPTY_SERVICE_ACCOUNTS,
            secrets: EMPTY_SECRETS,
            wif: DEV_SPEC.wif.clone(),
        };

        let report = evaluate_health(&spec);
        assert!(!report.overall_ok);
        assert!(report
            .items
            .iter()
            .any(|item| item.name == "service_accounts" && !item.ok));
        assert!(report
            .items
            .iter()
            .any(|item| item.name == "secrets" && !item.ok));
    }
}
