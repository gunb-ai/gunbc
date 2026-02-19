//! Credential policy loading and intent binding helpers.

use gunbc_ir::transport::{
    CloudProviderKind, CloudRuntimeKind, CredentialIntent, CredentialPolicySpec,
    ImpersonationPolicy, VersionSelector,
};
use std::path::Path;

/// Env var containing inline JSON `CredentialPolicySpec`.
pub const ENV_CREDENTIAL_POLICY_JSON: &str = "GUNBC_CREDENTIAL_POLICY_JSON";
/// Env var containing path to JSON policy file.
pub const ENV_CREDENTIAL_POLICY_PATH: &str = "GUNBC_CREDENTIAL_POLICY_PATH";
/// Env var selecting policy profile.
pub const ENV_CREDENTIAL_POLICY_PROFILE: &str = "GUNBC_CREDENTIAL_POLICY_PROFILE";

/// Policy-bound credential intent with strategy metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundCredentialIntent {
    pub intent: CredentialIntent,
    pub policy_provider: Option<CloudProviderKind>,
    pub policy_runtime: Option<CloudRuntimeKind>,
    pub impersonation: Option<ImpersonationPolicy>,
    pub version_selector: Option<VersionSelector>,
}

impl BoundCredentialIntent {
    fn fallback(intent: CredentialIntent) -> Self {
        Self {
            intent,
            policy_provider: None,
            policy_runtime: None,
            impersonation: None,
            version_selector: None,
        }
    }
}

/// Map credential policy impersonation mode to an allow/deny decision.
pub fn policy_allows_impersonation(policy: Option<&ImpersonationPolicy>) -> bool {
    !matches!(policy, Some(ImpersonationPolicy::Never))
}

/// Bind a fallback credential intent through configured credential policy.
///
/// When no policy source is configured, returns the fallback intent unchanged.
/// When policy is configured, profile + intent resolution errors are surfaced.
pub fn bind_credential_intent_policy(
    intent_key: &str,
    fallback: &CredentialIntent,
) -> Result<BoundCredentialIntent, String> {
    let Some(spec) = load_policy_from_env()? else {
        return Ok(BoundCredentialIntent::fallback(fallback.clone()));
    };

    let profile = env_nonempty(ENV_CREDENTIAL_POLICY_PROFILE)
        .or_else(|| spec.default_profile.clone())
        .ok_or_else(|| {
            format!(
                "credential policy is configured but no profile selected; set {} or default_profile",
                ENV_CREDENTIAL_POLICY_PROFILE
            )
        })?;

    let resolved = spec
        .resolve_intent_policy(&profile, intent_key)
        .map_err(|e| e.to_string())?;

    let mut intent = fallback.clone();
    if let Some(secret) = resolved.secret.as_ref() {
        intent.secret_name = Some(secret.name.clone());
    }
    if !resolved.required_scopes.is_empty() {
        intent.required_scopes = resolved.required_scopes.clone();
    }

    Ok(BoundCredentialIntent {
        intent,
        policy_provider: resolved.provider,
        policy_runtime: resolved.runtime,
        impersonation: resolved.impersonation,
        version_selector: resolved.version_selector,
    })
}

#[allow(clippy::disallowed_methods)] // Credential policy loader reads policy from file-backed profiles.
fn load_policy_from_env() -> Result<Option<CredentialPolicySpec>, String> {
    if let Some(raw_json) = env_nonempty(ENV_CREDENTIAL_POLICY_JSON) {
        let spec = serde_json::from_str::<CredentialPolicySpec>(&raw_json)
            .map_err(|e| format!("{ENV_CREDENTIAL_POLICY_JSON} must be valid JSON: {e}"))?;
        return Ok(Some(spec));
    }

    if let Some(path) = env_nonempty(ENV_CREDENTIAL_POLICY_PATH) {
        let path_ref = Path::new(path.as_str());
        let raw = std::fs::read_to_string(path_ref).map_err(|e| {
            format!(
                "failed to read credential policy file '{}': {}",
                path_ref.display(),
                e
            )
        })?;
        let spec = serde_json::from_str::<CredentialPolicySpec>(&raw)
            .map_err(|e| format!("credential policy file '{}' is invalid JSON: {e}", path))?;
        return Ok(Some(spec));
    }

    Ok(None)
}

fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    #[test]
    fn bind_credential_intent_policy_returns_fallback_when_unconfigured() {
        with_env_lock(|| {
            let fallback = CredentialIntent::new("github", "github", "bearer")
                .with_secret_name("github-token")
                .with_required_scopes(["gist:write"])
                .with_interactive_allowed(true);

            let bound =
                bind_credential_intent_policy("github.gist.create", &fallback).expect("fallback");
            assert_eq!(bound.intent, fallback);
            assert!(bound.policy_provider.is_none());
            assert!(bound.policy_runtime.is_none());
        });
    }

    #[test]
    fn bind_credential_intent_policy_applies_secret_and_scopes() {
        with_env_lock(|| {
            std::env::set_var(
                ENV_CREDENTIAL_POLICY_JSON,
                sample_policy_json("prod-github-token"),
            );
            std::env::set_var(ENV_CREDENTIAL_POLICY_PROFILE, "prod");

            let fallback = CredentialIntent::new("github", "github", "bearer")
                .with_secret_name("github-token")
                .with_required_scopes(["gist:write"])
                .with_interactive_allowed(true);

            let bound = bind_credential_intent_policy("github.gist.create", &fallback)
                .expect("policy bind should resolve");
            assert_eq!(
                bound.intent.secret_name.as_deref(),
                Some("prod-github-token")
            );
            assert_eq!(bound.intent.required_scopes, vec!["gist:write".to_string()]);
            assert_eq!(bound.policy_provider, Some(CloudProviderKind::Gcp));
            assert_eq!(bound.policy_runtime, Some(CloudRuntimeKind::GitHubActions));
        });
    }

    #[test]
    fn bind_credential_intent_policy_errors_when_profile_missing() {
        with_env_lock(|| {
            std::env::set_var(
                ENV_CREDENTIAL_POLICY_JSON,
                sample_policy_json("prod-github-token"),
            );

            let fallback = CredentialIntent::new("github", "github", "bearer")
                .with_secret_name("github-token")
                .with_required_scopes(["gist:write"])
                .with_interactive_allowed(true);
            let err = bind_credential_intent_policy("github.gist.create", &fallback)
                .expect_err("profile selection should be required");
            assert!(err.contains("no profile selected"));
        });
    }

    #[test]
    fn policy_allows_impersonation_maps_modes() {
        assert!(policy_allows_impersonation(None));
        assert!(!policy_allows_impersonation(Some(
            &ImpersonationPolicy::Never
        )));
        assert!(policy_allows_impersonation(Some(
            &ImpersonationPolicy::IfConfigured {
                service_account: None
            }
        )));
        assert!(policy_allows_impersonation(Some(
            &ImpersonationPolicy::Always {
                service_account: "svc@p.iam.gserviceaccount.com".to_string()
            }
        )));
    }

    fn sample_policy_json(secret_name: &str) -> String {
        serde_json::json!({
            "version": 0,
            "profiles": [
                {
                    "name": "prod",
                    "defaults": {
                        "provider": "Gcp",
                        "runtime": "GitHubActions"
                    },
                    "intents": [
                        {
                            "intent": "github.gist.create",
                            "secret": {
                                "name": secret_name
                            },
                            "required_scopes": ["gist:write"]
                        }
                    ]
                }
            ]
        })
        .to_string()
    }

    fn with_env_lock<F>(f: F)
    where
        F: FnOnce() + std::panic::UnwindSafe,
    {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        clear_policy_env();
        let result = std::panic::catch_unwind(f);
        clear_policy_env();
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }

    fn clear_policy_env() {
        for key in [
            ENV_CREDENTIAL_POLICY_JSON,
            ENV_CREDENTIAL_POLICY_PATH,
            ENV_CREDENTIAL_POLICY_PROFILE,
        ] {
            std::env::remove_var(key);
        }
    }
}
