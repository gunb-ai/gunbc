//! Wave-1 §11.7 operator **kill-criterion instrumentation** — per-PR affected-set analytics.
//!
//! This is an *ops measurement* artifact, NOT load-bearing CI substrate. It answers the operator
//! Tier-1 de-risk question (PM dispatch 2026-06-02):
//!
//!   On what fraction of real PRs does the affected-set actually let us skip expensive work
//!   (especially the `ci_floor` v2→v4 bootstrap path), and how many wall-clock minutes does that
//!   save versus a full run?
//!
//! There is no pre-coded threshold in-tree: the deliverable is per-PR JSON the operator aggregates
//! to decide whether to flip `ci_floor` from "always run" to affected-set-gated.
//!
//! Distinct from the Wave-3 modeled `CiSelectionReceipt` (`src/v4/workflow/ci.dag`, PR #4224): that
//! is the selection-receipt *substrate* (Shadow/Active, modeled-vs-host parity witness, path to
//! TestClaim projection). This receipt is an ops timing ledger emitted *alongside* it, never inside
//! `CiSelectionReceipt`. The shared input is the affected-set partition over `git diff`.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::CiComponentAffected;

/// The six CI "bankruptcy buckets" (docs/design-ci-bankruptcy-rebuild.md), in canonical order.
/// `release_distribution_only` is a refinement of `release_distribution`, not a separate bucket, so
/// it is reported as a flag on the receipt rather than as a seventh component.
pub const COMPONENT_BUCKETS: [&str; 6] = [
    "v2",
    "v3",
    "v4",
    "testclaim_corpus",
    "workflow_policy",
    "release_distribution",
];

/// JSON schema version for the emitted artifact. Bump on any field add/rename so aggregation
/// tooling can branch on shape.
pub const SCHEMA_VERSION: u32 = 1;

/// Provisional baseline for a full (un-skipped) CI run, in wall-clock minutes.
///
/// `0.0` means "baseline unset": `saved_minutes` is then reported as `0.0` and is NOT meaningful.
/// The operator supplies the real `ci_floor` p50 via `--estimated-full-run-minutes` (PM: ping
/// silent-crane-669 for the rolling median). We deliberately do not invent a constant here — an
/// overstated baseline would manufacture phantom savings in the aggregate.
pub const BASELINE_FULL_RUN_MINUTES_UNSET: f64 = 0.0;

/// Per-PR affected-set CI instrumentation receipt (serialized to a workflow JSON artifact).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AffectedSetCiReceipt {
    pub schema_version: u32,
    /// `git diff --name-only` paths for the PR (empty when the diff read failed; see `fail_closed`).
    pub changed_paths: Vec<String>,
    /// Buckets the affected-set selects (would run). Subset of `COMPONENT_BUCKETS`.
    pub selected_components: Vec<String>,
    /// Buckets the affected-set skips (would NOT run). Complement of `selected_components`.
    pub skipped_components: Vec<String>,
    /// True when `release_distribution` is the only triggered bucket (RELEASE §5 fast-path).
    pub release_distribution_only: bool,
    /// Would the `ci_floor` v2→v4 bootstrap path (v2 build + bootstrap viability + M1 emit probe)
    /// be required? Mirrors `ci_v4_bootstrap_gate_result_skip_guard_if` and
    /// `M1RustEmitProbeCommand` job masks in `src/v4/workflow/ci.dag` (needs-closure pulls
    /// `v2_compile_src_v4` for testclaim/workflow_policy/release_distribution, not only v4).
    pub bootstrap_required: bool,
    /// Number of TestClaims the node-frontier selection would run. v1: 0 (claim selection not wired
    /// into this ops path yet — it stays shadow in `CiSelectionReceipt`). TODO: populate from the
    /// eval harness frontier intersection once Wave-3 active selection lands.
    pub selected_claim_count: u32,
    /// Observed wall-clock per CI job id, in seconds (from GHA job/step timings). Empty in v1 when
    /// timings are not supplied to the emitter.
    pub wall_clock_by_job: BTreeMap<String, u64>,
    /// Operator baseline for a full run (minutes). See `BASELINE_FULL_RUN_MINUTES_UNSET`.
    pub estimated_full_run_minutes: f64,
    /// Observed wall-clock for this run (minutes). 0.0 when not supplied.
    pub actual_run_minutes: f64,
    /// Structurally derived skip savings (minutes). `0.0` when `fail_closed`, when
    /// `bootstrap_required` (no ci_floor bootstrap skip opportunity), when the baseline is unset,
    /// or when `actual_run_minutes` is unobserved — never from runtime variance alone on paths
    /// that would still run bootstrap.
    pub saved_minutes: f64,
    /// True when the `git diff` read failed and the affected-set fell back to the fail-closed
    /// superset (all components). Skip-rate aggregation must exclude these rows.
    pub fail_closed: bool,
}

/// Partition the six buckets into (selected, skipped) per the component flags.
pub fn component_partition(flags: CiComponentAffected) -> (Vec<String>, Vec<String>) {
    let selected_of = |bucket: &str| -> bool {
        match bucket {
            "v2" => flags.v2,
            "v3" => flags.v3,
            "v4" => flags.v4,
            "testclaim_corpus" => flags.testclaim_corpus,
            "workflow_policy" => flags.workflow_policy,
            "release_distribution" => flags.release_distribution,
            _ => false,
        }
    };
    let mut selected = Vec::new();
    let mut skipped = Vec::new();
    for bucket in COMPONENT_BUCKETS {
        if selected_of(bucket) {
            selected.push(bucket.to_string());
        } else {
            skipped.push(bucket.to_string());
        }
    }
    (selected, skipped)
}

/// Whether the Wave-1 `ci_floor` bootstrap path would run under affected-set gating.
///
/// Authority: `ci_v4_bootstrap_gate_result_skip_guard_if` and `M1RustEmitProbeCommand` masks in
/// `src/v4/workflow/ci.dag` — v4 bootstrap + M1 fire when any of
/// `{v4, testclaim_corpus, workflow_policy, release_distribution}` is selected; `v2` bucket
/// additionally requires the v2 gunbc build. v3-only is out of scope for this floor slice.
pub fn bootstrap_required(flags: CiComponentAffected) -> bool {
    flags.v2
        || flags.v4
        || flags.testclaim_corpus
        || flags.workflow_policy
        || flags.release_distribution
}

/// Structurally derived skip savings for the kill-criterion aggregate.
///
/// Returns `0.0` (unknown / not applicable) unless **all** of:
/// - `!fail_closed` — diff read succeeded (superset rows must not enter savings aggregates),
/// - `!bootstrap_required` — the affected-set would skip the `ci_floor` bootstrap path,
/// - baseline is set and `actual_run_minutes > 0` (observed wall-clock).
///
/// When eligible, `max(0, estimated_full_run_minutes - actual_run_minutes)`. INVARIANTS P3:
/// bootstrap-required or fail-closed PRs must never report positive savings from runtime variance.
pub fn saved_minutes(
    estimated_full_run_minutes: f64,
    actual_run_minutes: f64,
    bootstrap_required: bool,
    fail_closed: bool,
) -> f64 {
    if fail_closed || bootstrap_required {
        return 0.0;
    }
    if estimated_full_run_minutes <= BASELINE_FULL_RUN_MINUTES_UNSET || actual_run_minutes <= 0.0 {
        return 0.0;
    }
    (estimated_full_run_minutes - actual_run_minutes).max(0.0)
}

/// Build the receipt from the affected-set partition plus optional timing inputs.
#[allow(clippy::too_many_arguments)]
pub fn affected_set_ci_receipt(
    changed_paths: Vec<String>,
    flags: CiComponentAffected,
    fail_closed: bool,
    selected_claim_count: u32,
    wall_clock_by_job: BTreeMap<String, u64>,
    estimated_full_run_minutes: f64,
    actual_run_minutes: f64,
) -> AffectedSetCiReceipt {
    let (selected_components, skipped_components) = component_partition(flags);
    AffectedSetCiReceipt {
        schema_version: SCHEMA_VERSION,
        changed_paths,
        selected_components,
        skipped_components,
        release_distribution_only: flags.release_distribution_only,
        bootstrap_required: bootstrap_required(flags),
        selected_claim_count,
        wall_clock_by_job,
        estimated_full_run_minutes,
        actual_run_minutes,
        saved_minutes: saved_minutes(
            estimated_full_run_minutes,
            actual_run_minutes,
            bootstrap_required(flags),
            fail_closed,
        ),
        fail_closed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ci_component_affected_fail_closed, ci_component_affected_from_changed_paths};

    fn flags_for<'a>(paths: impl IntoIterator<Item = &'a str>) -> CiComponentAffected {
        ci_component_affected_from_changed_paths(paths)
    }

    #[test]
    fn docs_only_diff_skips_every_bucket() {
        let flags = flags_for(["docs/README.md"]);
        let (selected, skipped) = component_partition(flags);
        assert!(selected.is_empty());
        assert_eq!(skipped.len(), COMPONENT_BUCKETS.len());
        assert!(!bootstrap_required(flags));
    }

    #[test]
    fn v4_substrate_diff_requires_bootstrap() {
        let flags = flags_for(["src/v4/std/node.dag"]);
        assert!(bootstrap_required(flags));
        let (selected, _) = component_partition(flags);
        assert!(selected.contains(&"v4".to_string()));
    }

    #[test]
    fn claim_only_diff_requires_bootstrap_via_modeled_needs_closure() {
        // TestClaim corpus jobs need v2_compile_src_v4 + M1 (ci.dag needs-closure), even when v2/v4
        // bucket bits alone would be false.
        let flags = flags_for(["src/v4/test/claim/workflow/affected_set_ci_runner.dag"]);
        assert!(flags.testclaim_corpus);
        assert!(!flags.v2);
        assert!(!flags.v4);
        assert!(bootstrap_required(flags));
    }

    #[test]
    fn workflow_policy_only_diff_requires_bootstrap() {
        let flags = flags_for([".github/workflows/ci.yml"]);
        assert!(flags.workflow_policy);
        assert!(bootstrap_required(flags));
    }

    #[test]
    fn workspace_dep_change_requires_bootstrap_via_v2() {
        let flags = flags_for(["Cargo.lock"]);
        assert!(flags.v2);
        assert!(bootstrap_required(flags));
    }

    #[test]
    fn partition_is_a_complete_disjoint_cover() {
        let flags = flags_for(["src/v4/workflow/ci.dag"]);
        let (selected, skipped) = component_partition(flags);
        assert_eq!(selected.len() + skipped.len(), COMPONENT_BUCKETS.len());
        for bucket in COMPONENT_BUCKETS {
            let in_sel = selected.iter().any(|b| b == bucket);
            let in_skip = skipped.iter().any(|b| b == bucket);
            assert!(
                in_sel ^ in_skip,
                "{bucket} must be in exactly one partition"
            );
        }
    }

    #[test]
    fn fail_closed_superset_selects_all_buckets() {
        let flags = ci_component_affected_fail_closed();
        let (selected, skipped) = component_partition(flags);
        assert_eq!(selected.len(), COMPONENT_BUCKETS.len());
        assert!(skipped.is_empty());
        assert!(bootstrap_required(flags));
    }

    #[test]
    fn saved_minutes_clamps_and_handles_unset_baseline() {
        // Unset baseline → no phantom savings.
        assert_eq!(
            saved_minutes(BASELINE_FULL_RUN_MINUTES_UNSET, 12.0, false, false),
            0.0
        );
        // Faster than baseline, skip-eligible path → positive savings.
        assert_eq!(saved_minutes(40.0, 15.0, false, false), 25.0);
        // Slower than baseline (e.g. cold cache) → clamp to 0, never negative.
        assert_eq!(saved_minutes(40.0, 55.0, false, false), 0.0);
    }

    #[test]
    fn saved_minutes_is_zero_when_actual_unobserved_even_with_baseline() {
        // Codex #4271 regression: a provisional baseline with no observed runtime must NOT report
        // the full baseline as savings (fail-closed: unobserved actual ≠ instantaneous run).
        assert_eq!(saved_minutes(15.0, 0.0, false, false), 0.0);
        // Negative/garbage actual is likewise treated as unobserved.
        assert_eq!(saved_minutes(15.0, -3.0, false, false), 0.0);
    }

    #[test]
    fn saved_minutes_positive_only_on_skip_eligible_path_with_observed_timing() {
        let flags = flags_for(["docs/README.md"]);
        assert!(!bootstrap_required(flags));
        let receipt = affected_set_ci_receipt(
            vec!["docs/README.md".to_string()],
            flags,
            false,
            0,
            BTreeMap::new(),
            40.0,
            15.0,
        );
        assert_eq!(receipt.saved_minutes, 25.0);
    }

    #[test]
    fn saved_minutes_is_zero_when_bootstrap_required_or_fail_closed() {
        // Bootstrap-required PRs still run ci_floor today; baseline−actual is variance, not skip savings.
        assert_eq!(saved_minutes(40.0, 15.0, true, false), 0.0);
        // Fail-closed superset rows must not enter savings aggregates.
        assert_eq!(saved_minutes(40.0, 15.0, false, true), 0.0);
        assert_eq!(saved_minutes(40.0, 15.0, true, true), 0.0);
    }

    #[test]
    fn receipt_serializes_expected_keys() {
        let flags = flags_for(["src/v4/std/node.dag"]);
        let receipt = affected_set_ci_receipt(
            vec!["src/v4/std/node.dag".to_string()],
            flags,
            false,
            0,
            BTreeMap::new(),
            40.0,
            15.0,
        );
        let json = serde_json::to_value(&receipt).expect("serialize");
        for key in [
            "schema_version",
            "changed_paths",
            "selected_components",
            "skipped_components",
            "release_distribution_only",
            "bootstrap_required",
            "selected_claim_count",
            "wall_clock_by_job",
            "estimated_full_run_minutes",
            "actual_run_minutes",
            "saved_minutes",
            "fail_closed",
        ] {
            assert!(json.get(key).is_some(), "missing key {key}");
        }
        // v4 substrate diff ⇒ bootstrap_required; savings are structurally N/A (not skip-eligible).
        assert_eq!(receipt.saved_minutes, 0.0);
        assert!(receipt.bootstrap_required);
    }
}
