//! Secret rotation planning + age checks.

use crate::project_spec::{RotationHandler, SecretSpec};
use std::time::{Duration, SystemTime};

/// Planned action for rotating a secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretRotationAction {
    Manual {
        instructions: String,
    },
    GitHubPat {
        instructions: String,
        required_scopes: Vec<String>,
    },
    Skip {
        reason: String,
    },
}

/// Evaluate a secret's age against an optional max-age policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretAgeCheck {
    Unbounded { age_days: u64 },
    Fresh { age_days: u64, max_age_days: u32 },
    Overdue { age_days: u64, max_age_days: u32 },
}

/// Plan rotation behavior for a secret based on its configured handler.
pub fn rotate_secret(secret: &SecretSpec) -> SecretRotationAction {
    match secret.rotation {
        RotationHandler::Manual => SecretRotationAction::Manual {
            instructions: format!(
                "Rotate secret '{}' manually in provider dashboard, then update '{}'.",
                secret.secret_id, secret.env_name
            ),
        },
        RotationHandler::GitHubPat => SecretRotationAction::GitHubPat {
            instructions: format!(
                "Generate a new GitHub PAT for '{}' and update secret '{}'.",
                secret.env_name, secret.secret_id
            ),
            required_scopes: secret.scopes.iter().map(|s| s.to_string()).collect(),
        },
        RotationHandler::ServiceAccountKey => SecretRotationAction::Manual {
            instructions: format!(
                "Create a new service account key for '{}' and rotate secret '{}'.",
                secret.env_name, secret.secret_id
            ),
        },
        RotationHandler::None => SecretRotationAction::Skip {
            reason: format!(
                "secret '{}' has rotation handler None; no rotation required",
                secret.secret_id
            ),
        },
    }
}

/// Check secret age against `max_age_days`.
pub fn check_secret_age(
    created_at: SystemTime,
    now: SystemTime,
    max_age_days: Option<u32>,
) -> SecretAgeCheck {
    let age_days = now
        .duration_since(created_at)
        .unwrap_or(Duration::ZERO)
        .as_secs()
        / 86_400;
    match max_age_days {
        None => SecretAgeCheck::Unbounded { age_days },
        Some(max) if age_days > max as u64 => SecretAgeCheck::Overdue {
            age_days,
            max_age_days: max,
        },
        Some(max) => SecretAgeCheck::Fresh {
            age_days,
            max_age_days: max,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_spec::{RotationHandler, SecretRequirement, SecretStatus};

    fn secret(
        rotation: RotationHandler,
        scopes: &'static [&'static str],
        max_age_days: Option<u32>,
    ) -> SecretSpec {
        SecretSpec {
            env_name: "GITHUB_TOKEN",
            secret_id: "github-token",
            requirement: SecretRequirement::Required,
            status: SecretStatus::Active,
            scopes,
            rotation,
            max_age_days,
        }
    }

    #[test]
    fn rotate_secret_manual_returns_instructions() {
        let action = rotate_secret(&secret(RotationHandler::Manual, &[], None));
        assert!(matches!(action, SecretRotationAction::Manual { .. }));
    }

    #[test]
    fn rotate_secret_github_pat_carries_scopes() {
        let action = rotate_secret(&secret(
            RotationHandler::GitHubPat,
            &["repo", "gist"],
            Some(90),
        ));
        match action {
            SecretRotationAction::GitHubPat {
                required_scopes, ..
            } => {
                assert_eq!(
                    required_scopes,
                    vec!["repo".to_string(), "gist".to_string()]
                );
            }
            other => panic!("expected GitHubPat action, got {other:?}"),
        }
    }

    #[test]
    fn rotate_secret_none_skips() {
        let action = rotate_secret(&secret(RotationHandler::None, &[], None));
        assert!(matches!(action, SecretRotationAction::Skip { .. }));
    }

    #[test]
    fn check_secret_age_reports_overdue_when_age_exceeds_limit() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100 * 86_400);
        let created = SystemTime::UNIX_EPOCH;
        assert!(matches!(
            check_secret_age(created, now, Some(90)),
            SecretAgeCheck::Overdue {
                age_days: 100,
                max_age_days: 90
            }
        ));
    }
}
