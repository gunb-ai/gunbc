//! Freshness policy: deduced chain for repo freshness.
//!
//! Instead of every tool manually declaring freshness dependencies, this module
//! provides a single `check_and_plan_freshness()` function that:
//!
//! 1. Checks if the repo is fresh (fast manifest mtime check)
//! 2. If stale, returns a list of [`FreshnessStep`]s that must run
//! 3. After execution, `update_freshness_manifest()` records the new state
//!
//! The steps are the same as the old `run_lint_upsert` but modeled as
//! individual DAG nodes instead of a monolithic callback.

use crate::preflight::{self};
use crate::TransportIo;
use gunbc_exec::freshness::FRESHNESS_ACTIVE_ENV;
use gunbc_exec::FreshnessStep;
use gunbc_ir::resource::{load_manifest_default, save_manifest_default, ManagedResource};

/// Check repo freshness and return planned steps if stale.
///
/// Returns `None` if:
/// - `GUNBC_FRESHNESS_ACTIVE` is set (recursion prevention)
/// - The repo is already fresh (manifest check passes)
///
/// Returns `Some(steps)` if the repo needs freshening, where steps are
/// the sequential chain: codegen-dag → testgen → pragma → clippy → test-compile → release-check.
pub fn check_and_plan_freshness() -> Option<Vec<FreshnessStep>> {
    // Recursion prevention: if we're already inside a freshness context, skip.
    if std::env::var(FRESHNESS_ACTIVE_ENV).is_ok() {
        return None;
    }

    // Fast freshness check via resource manifest
    let io = TransportIo::new();
    let resource = preflight::lint_resource();

    let manifest = match load_manifest_default(&io) {
        Ok(m) => m,
        Err(_) => {
            // Can't load manifest — assume stale
            return Some(freshness_steps());
        }
    };

    let state = resource.check_state(&manifest, &io);
    if state.is_fresh() {
        return None;
    }

    Some(freshness_steps())
}

/// Update the freshness manifest after successful execution.
///
/// Call this after `execute_and_display` succeeds when freshness steps were
/// included in the DAG.
pub fn update_freshness_manifest() -> Result<(), String> {
    let io = TransportIo::new();
    let resource = preflight::lint_resource();
    let mut manifest =
        load_manifest_default(&io).map_err(|e| format!("freshness: manifest load failed: {e}"))?;

    preflight::upsert_lint_manifest_entry_pub(&io, &mut manifest, &resource)?;

    save_manifest_default(&io, &manifest)
        .map_err(|e| format!("freshness: manifest save failed: {e}"))?;

    Ok(())
}

/// Build the freshness step chain.
///
/// These are the same steps as the old `run_lint_upsert`, modeled as
/// individual commands:
///
/// 1. codegen-dag: generate code from DAG structures
/// 2. testgen: generate tests
/// 3. pragma: process pragma directives
/// 4. clippy: lint check (with auto-fix)
/// 5. test-compile: compile lib tests without running
/// 6. release-check: compile-check release bins for the workspace
fn freshness_steps() -> Vec<FreshnessStep> {
    vec![
        FreshnessStep {
            id: "codegen-dag".into(),
            command: vec![
                "cargo".into(),
                "run".into(),
                "-p".into(),
                "gunbc-dag".into(),
                "--bin".into(),
                "gunbc-codegen-dag".into(),
                "--".into(),
                "dag".into(),
            ],
        },
        FreshnessStep {
            id: "testgen".into(),
            command: vec![
                "cargo".into(),
                "run".into(),
                "-p".into(),
                "gunbc-dag".into(),
                "--bin".into(),
                "gunbc-testgen".into(),
                "--".into(),
                "dag".into(),
            ],
        },
        FreshnessStep {
            id: "pragma".into(),
            command: vec![
                "cargo".into(),
                "run".into(),
                "-p".into(),
                "gunbc-dag".into(),
                "--bin".into(),
                "gunbc-pragma".into(),
                "--".into(),
                "dag".into(),
            ],
        },
        FreshnessStep {
            id: "clippy".into(),
            command: vec![
                "cargo".into(),
                "clippy".into(),
                "--workspace".into(),
                "--".into(),
                "-D".into(),
                "warnings".into(),
            ],
        },
        FreshnessStep {
            id: "test-compile".into(),
            command: vec![
                "cargo".into(),
                "test".into(),
                "--workspace".into(),
                "--lib".into(),
                "--no-run".into(),
            ],
        },
        FreshnessStep {
            id: "release-check".into(),
            command: vec![
                "cargo".into(),
                "check".into(),
                "--workspace".into(),
                "--release".into(),
                "--bins".into(),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freshness_steps_ends_with_release_check_gate() {
        let steps = freshness_steps();
        let release_check = steps.last().expect("freshness chain should not be empty");
        assert_eq!(release_check.id, "release-check");
        assert_eq!(
            release_check.command,
            vec!["cargo", "check", "--workspace", "--release", "--bins"]
        );
    }
}
