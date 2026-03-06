//! gunbc-app CI module.
//!
//! CI-related utilities for the gunbc repo.

use gunbc_testgen_registry::iter_dag_specs;
use std::collections::BTreeSet;

/// Live-test secret env vars that must be exported in CI.
///
/// Derived from testgen target metadata (`live_required` and
/// `live_required_any_of`) to keep workflow secrets and test metadata in sync.
///
/// Note: `ACTIONS_ID_TOKEN_REQUEST_URL` and `ACTIONS_ID_TOKEN_REQUEST_TOKEN`
/// are automatically provided by GitHub Actions when `id-token: write` is
/// granted; they are excluded from repository-secret export lists.
pub fn ci_live_test_secrets() -> Vec<&'static str> {
    let mut secrets: BTreeSet<&'static str> = BTreeSet::new();

    for def in iter_dag_specs() {
        if let Some(required) = def.testgen.live_required {
            for &secret in required {
                if !is_github_actions_runtime_env(secret) {
                    secrets.insert(secret);
                }
            }
        }
        if let Some(any_of_groups) = def.testgen.live_required_any_of {
            for group in any_of_groups {
                for &secret in *group {
                    if !is_github_actions_runtime_env(secret) {
                        secrets.insert(secret);
                    }
                }
            }
        }
    }

    secrets.into_iter().collect()
}

fn is_github_actions_runtime_env(name: &str) -> bool {
    matches!(
        name,
        "ACTIONS_ID_TOKEN_REQUEST_URL" | "ACTIONS_ID_TOKEN_REQUEST_TOKEN"
    )
}
