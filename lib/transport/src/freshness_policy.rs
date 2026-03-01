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
//!
//! # Operation identity
//!
//! Build verification steps (clippy, test-compile, release-check) declare
//! `subsumes: Some(OperationKey { .. })` to express which service operation
//! they cover. This connects the freshness system to the domain model's
//! operation identity, enabling the general composition-level overlap
//! detection in `compose_with_freshness`.

use crate::preflight::{self};
use crate::TransportIo;
use gunbc_exec::freshness::FRESHNESS_ACTIVE_ENV;
use gunbc_exec::FreshnessStep;
use gunbc_ir::resource::{load_manifest_default, save_manifest_default, ManagedResource};
use gunbc_ir::OperationKey;

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
/// 5. clippy: lint check (subsumes cargo.Build.Clippy)
/// 6. test-compile: compile lib tests without running (subsumes cargo.Build.Test)
/// 7. release-check: compile-check release bins (subsumes cargo.Build.Build)
fn freshness_steps() -> Vec<FreshnessStep> {
    let mut steps = generation_freshness_steps();
    steps.extend(build_verification_steps());
    steps
}

/// Generation-only freshness steps: codegen → codegen-dag → testgen → pragma.
///
/// These don't subsume any service operation — they're code generation, not
/// build/lint/test. Safe to compose with any tool DAG.
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
            subsumes: None,
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
            subsumes: None,
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
            subsumes: None,
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
            subsumes: None,
        },
    ]
}

/// Build verification steps: clippy, test-compile, release-check.
///
/// Each step declares which service operation it subsumes via `OperationKey`.
/// This is derived from the domain model: `cargo.Build.Clippy`, `cargo.Build.Test`,
/// `cargo.Build.Build` are the canonical operation identities from `dsl/services/cargo.dag`.
///
/// When composed with a tool DAG that already contains these operations,
/// the C22 Deductive Redundancy Elimination system will detect the overlap
/// via idempotency fingerprints (see `docs/design/deductive-redundancy.md`).
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
            subsumes: Some(OperationKey::new("cargo.Build", "Clippy")),
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
            subsumes: Some(OperationKey::new("cargo.Build", "Test")),
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
            subsumes: Some(OperationKey::new("cargo.Build", "Build")),
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

    #[test]
    fn generation_steps_have_no_subsumes() {
        for step in generation_freshness_steps() {
            assert!(
                step.subsumes.is_none(),
                "generation step '{}' should not subsume any operation",
                step.id
            );
        }
    }

    #[test]
    fn build_verification_steps_all_have_subsumes() {
        for step in build_verification_steps() {
            assert!(
                step.subsumes.is_some(),
                "build verification step '{}' must declare its subsumes operation",
                step.id
            );
        }
    }

    #[test]
    fn build_verification_subsumes_match_cargo_service() {
        let steps = build_verification_steps();
        let keys: Vec<_> = steps
            .iter()
            .map(|s| s.subsumes.as_ref().unwrap().to_string())
            .collect();
        assert_eq!(
            keys,
            vec!["cargo.Build.Clippy", "cargo.Build.Test", "cargo.Build.Build"]
        );
    }
}
