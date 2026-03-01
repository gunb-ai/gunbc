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
/// the sequential chain: codegen → codegen-dag → testgen → pragma → clippy → test-compile → release-check.
pub fn check_and_plan_freshness() -> Option<Vec<FreshnessStep>> {
    check_and_plan_freshness_inner(freshness_steps)
}

/// Check repo freshness with only generation steps (codegen, testgen, pragma).
///
/// Use this for tools that already run their own build/clippy/test steps
/// (e.g., the CI binary runs Build+Clippy+Test via the build tool DAG).
/// Skips the redundant clippy, test-compile, and release-check freshness steps.
pub fn check_and_plan_generation_freshness() -> Option<Vec<FreshnessStep>> {
    check_and_plan_freshness_inner(generation_freshness_steps)
}

fn check_and_plan_freshness_inner(
    steps_fn: fn() -> Vec<FreshnessStep>,
) -> Option<Vec<FreshnessStep>> {
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
            return Some(steps_fn());
        }
    };

    let state = resource.check_state(&manifest, &io);
    if state.is_fresh() {
        return None;
    }

    Some(steps_fn())
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

/// Build the full freshness step chain (generation + build verification).
///
/// These are the same steps as the old `run_lint_upsert`, modeled as
/// individual commands:
///
/// 1. codegen: generate CLI entry points into target/codegen/bin/ (MUST be first —
///    codegen-dag/testgen/pragma are generated binaries whose source lives in
///    target/codegen/bin/; this step produces those files using the handwritten
///    gunbc-codegen binary)
/// 2. codegen-dag: generate code from DAG structures
/// 3. testgen: generate tests
/// 4. pragma: process pragma directives
/// 5. clippy: lint check (with auto-fix)
/// 6. test-compile: compile lib tests without running
/// 7. release-check: compile-check release bins for the workspace
fn freshness_steps() -> Vec<FreshnessStep> {
    let mut steps = generation_freshness_steps();
    steps.extend(build_verification_steps());
    steps
}

/// Generation-only freshness steps: codegen → codegen-dag → testgen → pragma.
///
/// Use when the caller already performs build/clippy/test (e.g., the CI binary
/// runs Build+Clippy+Test via the build tool DAG, making the build verification
/// steps redundant).
fn generation_freshness_steps() -> Vec<FreshnessStep> {
    vec![
        // codegen MUST run first: it generates target/codegen/bin/*/main.rs
        // that codegen-dag, testgen, and pragma need to compile.
        // Uses the handwritten gunbc-codegen binary (src/bin/codegen_cli.rs),
        // which skips freshness checks for the "codegen" subcommand.
        FreshnessStep {
            id: "codegen".into(),
            command: vec![
                "cargo".into(),
                "run".into(),
                "-p".into(),
                "gunbc-dag".into(),
                "--bin".into(),
                "gunbc-codegen".into(),
                "--".into(),
                "codegen".into(),
            ],
        },
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
                "codegen".into(),
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
            ],
        },
    ]
}

/// Build verification steps: clippy, test-compile, release-check.
///
/// These are separated from generation steps so that tools which already
/// run their own build/clippy/test (like the CI binary) can skip them.
fn build_verification_steps() -> Vec<FreshnessStep> {
    vec![
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

    #[test]
    fn generation_steps_exclude_build_verification() {
        let steps = generation_freshness_steps();
        let ids: Vec<&str> = steps.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["codegen", "codegen-dag", "testgen", "pragma"]);
        assert!(!ids.contains(&"clippy"));
        assert!(!ids.contains(&"test-compile"));
        assert!(!ids.contains(&"release-check"));
    }

    #[test]
    fn full_steps_include_all() {
        let steps = freshness_steps();
        let ids: Vec<&str> = steps.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["codegen", "codegen-dag", "testgen", "pragma", "clippy", "test-compile", "release-check"]
        );
    }
}
