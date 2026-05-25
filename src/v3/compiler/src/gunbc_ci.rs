//! `gunbc.ci` workflow gate selection — affected-set dispatch substrate.
//!
//! **Authority** (Phase B): `docs/briefs/r3-wave1-s6-slice7-affected-set-impl-worker.md`;
//! fail-closed + obligation closure: `docs/design-t-wad-slice-7-binary-shim-affected-set-selection-canvas.md` §3–§4;
//! carrier shape: `dsl/gunbc/ci.dag` (`CIWorkflowDag`, `CIGateEdge`: `from` is a prerequisite of `to`).
//!
//! This module is intentionally **pure**: every decision is a function of
//! [`CiWorkflowDagInput`] + [`CiWorkflowDiff`] only. Git diffs, path-regex, env,
//! and PR #2713 lens receipts are upstream responsibilities; they must be
//! mapped into [`CiWorkflowDiff`] (or a superset-equivalent touch set) before
//! calling [`select_affected_gates`].
//!
//! **Verifier ratchet witness (Phase C scaffolding)** — `docs/design-ci-workflow-substrate-shape-2026-05-12.md`
//! S5 + prequeue §5.2: monotone **inclusion** under enlarging the touched-id
//! set (smaller touch ⊆ larger touch ⇒ selected subset ⊆ selected superset).
//! See [`selection_subset_under_touch_set_growth`].
//!
//! **Expansion semantics:** [`select_affected_gates`] walks each `CIGateEdge` **symmetrically**
//! (treat `from → to` as an undirected adjacency) to fixpoint, so a downstream obligation
//! pulls in its sibling prerequisites (e.g. touching `lint` selects `l1-ratchet`, which
//! forces `tests` because both parents are prerequisites of `l1-ratchet`). The returned
//! order is a **topological sort** of that vertex set on the directed prerequisite subgraph.
//!
//! **Global carrier validity:** before expanding any touch seed, [`select_affected_gates`] requires
//! the **entire** directed prerequisite graph on [`CiWorkflowDagInput::gates`] to be acyclic. A cycle
//! confined to an unselected weak component must still fail closed (P3), not succeed in “narrow” mode.

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

/// Structural mirror of `CIGate.id` in `dsl/gunbc/ci.dag`.
pub type CiGateId = String;

/// Structural mirror of `CIGate` in `dsl/gunbc/ci.dag`.
///
/// [`select_affected_gates`] consults only [`CiGateMeta::id`]. [`CiGateMeta::blocking`] is carried
/// for **carrier parity** with the DSL record so BinaryShim / dispatch wiring can map rows without
/// a forked struct; merge-blocking vs advisory semantics stay **outside** this selection substrate
/// until a consumer reads the field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CiGateMeta {
    pub id: CiGateId,
    pub blocking: bool,
}

/// Structural mirror of `CIWorkflowDag`: gate roster + prerequisite edges.
///
/// Each edge `(from, to)` means **`from` must complete before `to`** (matches
/// `CIGateEdge { from, to }` in `ci.dag`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CiWorkflowDagInput {
    pub gates: Vec<CiGateMeta>,
    /// Prerequisite edges: `from` runs before `to`.
    pub edges: Vec<(CiGateId, CiGateId)>,
}

// Practice 4 (coproduct checkpoint, `docs/modeling-discipline.md` §4):
// 🟢 GREEN — terminal upstream-policy boundary for gate-id touch semantics before
// `select_affected_gates`: `TouchAll` is the conservative in-DAG superset seed;
// `TouchedGates` is the explicit finite set upstream mapped from lens/git facts.
// Ledger: no third authority here — path-regex, env, and PR #2713 receipts stay
// outside this module (Slice 7 brief + affected-set selection canvas §1).
/// Diff-against-base input projected to **gate ids** (single authority at this boundary).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CiWorkflowDiff {
    /// Every gate id in [`CiWorkflowDagInput::gates`] is treated as directly touched.
    TouchAll,
    /// Upstream bundle names gate ids with proven or assumed direct impact.
    TouchedGates(BTreeSet<CiGateId>),
}

// Practice 4 (coproduct checkpoint, `docs/modeling-discipline.md` §4):
// 🟢 GREEN — terminal typed failure surface for [`select_affected_gates`]: malformed `CIWorkflowDag`
// carrier vs non-acyclic directed prerequisites (full roster graph and/or selected subset; no silent success).
// Ledger: (1) fact placement — not split across consumers; this `Result` boundary owns the `Err`
// coproduct. (2) variant-is-data — rejected; payloads differ structurally. (3) algebraic — not
// `std/` refs. (4) dimensional — not one decomposed axis space; each variant is an irreducible
// outcome at the `dsl/gunbc/ci.dag` structural mirror until `.dag`-owned dispatch absorbs diagnostics.
/// [`select_affected_gates`] failed: malformed `CIWorkflowDag` carrier or non-acyclic prerequisites.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CiAffectedGatesError {
    /// The same `CIGate.id` appears more than once in [`CiWorkflowDagInput::gates`] (malformed carrier; must not collapse into a `HashSet` silently).
    DuplicateGateRosterId { id: CiGateId },
    /// `CIGateEdge` names a gate id not present in [`CiWorkflowDagInput::gates`] (violates single roster authority in `dsl/gunbc/ci.dag`).
    UnknownEdgeEndpoint { from: CiGateId, to: CiGateId },
    /// The directed prerequisite graph is not a DAG: either the **full** roster graph from
    /// [`CiWorkflowDagInput::edges`] contains a cycle, or the selected vertex set’s induced subgraph
    /// does (broken indegree accounting).
    NonAcyclicPrerequisiteGraph,
}

/// On success, returns gate ids to execute: the **connected component** of the seed under the symmetric
/// closure of prerequisite edges (iterate `from ↔ to` adjacency to fixpoint), then
/// **topologically sorted** on the directed subgraph (prerequisites first).
///
/// **Fail-closed** (`design-t-wad-slice-7-binary-shim-affected-set-selection-canvas.md` §3):
/// any touched id not present in `dag.gates` forces **all** gates listed in `dag.gates`
/// into the plan (superset within this DAG).
///
/// **Empty diff** (`TouchedGates` empty): returns `Ok` of an **empty** plan — callers map that
/// to “no merge-blocking re-verify” vs “full superset” per repository policy **outside**
/// this function (this module does not read policy carriers).
///
/// **Malformed carrier:** duplicate [`CiGateMeta::id`] values in [`CiWorkflowDagInput::gates`]
/// yield [`CiAffectedGatesError::DuplicateGateRosterId`] (roster must be unique before any `HashSet`
/// view so gates are not dropped silently). Any edge whose `from` or `to` is absent from that roster
/// yields [`CiAffectedGatesError::UnknownEdgeEndpoint`]. Any directed cycle in the **full**
/// prerequisite graph on the roster (not only the touch-selected component) yields
/// [`CiAffectedGatesError::NonAcyclicPrerequisiteGraph`] instead of returning `Ok` in narrow mode.
pub fn select_affected_gates(
    dag: &CiWorkflowDagInput,
    diff: &CiWorkflowDiff,
) -> Result<Vec<CiGateId>, CiAffectedGatesError> {
    validate_unique_gate_roster(dag)?;
    let known: HashSet<&str> = dag.gates.iter().map(|g| g.id.as_str()).collect();
    for (from, to) in &dag.edges {
        if !known.contains(from.as_str()) || !known.contains(to.as_str()) {
            return Err(CiAffectedGatesError::UnknownEdgeEndpoint {
                from: from.clone(),
                to: to.clone(),
            });
        }
    }

    // Fail-closed on the whole carrier: a cycle in an unselected component must not yield `Ok`
    // for a disjoint touch seed (codex / P3).
    topo_sort_subset(dag, &known)?;

    let force_superset = match diff {
        CiWorkflowDiff::TouchAll => false,
        CiWorkflowDiff::TouchedGates(ids) => ids.iter().any(|id| !known.contains(id.as_str())),
    };

    let seed: HashSet<&str> = if force_superset {
        known.clone()
    } else {
        match diff {
            CiWorkflowDiff::TouchAll => known.clone(),
            CiWorkflowDiff::TouchedGates(ids) => ids.iter().map(|s| s.as_str()).collect(),
        }
    };

    if seed.is_empty() {
        return Ok(Vec::new());
    }

    // Symmetric edge closure: expand along `from ↔ to` until stable, then topo-sort
    // the induced set on directed prerequisite edges.
    let mut selected: HashSet<&str> = seed.clone();
    loop {
        let before = selected.len();
        for (from, to) in &dag.edges {
            let from = from.as_str();
            let to = to.as_str();
            if selected.contains(from) {
                selected.insert(to);
            }
            if selected.contains(to) {
                selected.insert(from);
            }
        }
        if selected.len() == before {
            break;
        }
    }

    topo_sort_subset(dag, &selected)
}

fn validate_unique_gate_roster(dag: &CiWorkflowDagInput) -> Result<(), CiAffectedGatesError> {
    let mut seen: HashSet<&str> = HashSet::new();
    for g in &dag.gates {
        if !seen.insert(g.id.as_str()) {
            return Err(CiAffectedGatesError::DuplicateGateRosterId { id: g.id.clone() });
        }
    }
    Ok(())
}

/// Set inclusion witness for ratchet tests.
///
/// Returns whether `select_affected_gates(dag, narrow)` ⊆ `select_affected_gates(dag, wide)` as sets
/// when **both** calls succeed; if either returns `Err`, returns `false`.
/// Intended use: both arguments are [`CiWorkflowDiff::TouchedGates`] with `narrow_ids ⊆ wide_ids`, or
/// `wide` is [`CiWorkflowDiff::TouchAll`] (superset selection). Other pairings are not a refinement
/// partial order and may return `false` even when both plans are individually valid.
pub fn selection_subset_under_touch_set_growth(
    dag: &CiWorkflowDagInput,
    narrow: &CiWorkflowDiff,
    wide: &CiWorkflowDiff,
) -> bool {
    match (
        select_affected_gates(dag, narrow),
        select_affected_gates(dag, wide),
    ) {
        (Ok(a), Ok(b)) => {
            let a: BTreeSet<_> = a.into_iter().collect();
            let b: BTreeSet<_> = b.into_iter().collect();
            a.is_subset(&b)
        }
        _ => false,
    }
}

/// **Layer 1** gate-id touch seed for [`select_affected_gates_for_binary_shim`].
///
/// Canvas `design-t-wad-slice-7-binary-shim-affected-set-selection-canvas.md` §1.1
/// steps 3–5 and §4 require the **runner** to consume PR #2713 structured output, join
/// it with `CIWorkflowDag` / `TestClaim` metadata, and **then** emit executable work.
/// This struct is **not** that receipt: it only carries `CIGate.id` strings (and a
/// narrowing flag) **after** any `NodeRef` → gate-id mapping has happened elsewhere.
///
/// When `narrowing_available` is `false`, selection fails closed to the full gate
/// roster ([`CiWorkflowDiff::TouchAll`]; canvas §3: unknown dimension / missing
/// receipt / unbuildable DAG pair).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CiBinaryShimAffectedSetReceipt {
    /// `false` forces [`CiWorkflowDiff::TouchAll`] regardless of `proven_direct_gate_touches`.
    pub narrowing_available: bool,
    /// Gate ids proven directly touched **after** upstream mapping (seed before symmetric expansion).
    pub proven_direct_gate_touches: BTreeSet<CiGateId>,
}

/// Layer 1 entry: map a **gate-id-only** [`CiBinaryShimAffectedSetReceipt`] into [`select_affected_gates`].
pub fn select_affected_gates_for_binary_shim(
    dag: &CiWorkflowDagInput,
    receipt: &CiBinaryShimAffectedSetReceipt,
) -> Result<Vec<CiGateId>, CiAffectedGatesError> {
    let diff = if receipt.narrowing_available {
        CiWorkflowDiff::TouchedGates(receipt.proven_direct_gate_touches.clone())
    } else {
        CiWorkflowDiff::TouchAll
    };
    select_affected_gates(dag, &diff)
}

/// Structural mirror of `v4.lens.subsumption.DiffId` for Lens-CI rerun evidence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DissolutionDiffId {
    pub id: String,
}

/// Structural mirror of `v4.lens.subsumption.TestClaimId` for Lens-CI rerun evidence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DissolutionTestClaimId {
    pub id: String,
}

/// Structural mirror of `v4.lens.subsumption.ProducerStageId` for Lens-CI row evidence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DissolutionProducerStageId {
    pub id: String,
}

impl From<&str> for DissolutionDiffId {
    fn from(id: &str) -> Self {
        Self { id: id.into() }
    }
}

impl From<&str> for DissolutionTestClaimId {
    fn from(id: &str) -> Self {
        Self { id: id.into() }
    }
}

impl From<&str> for DissolutionProducerStageId {
    fn from(id: &str) -> Self {
        Self { id: id.into() }
    }
}

/// Structural mirror of `v4.lens.subsumption.SubsumptionVerification`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubsumptionVerificationRuntime {
    MechanicalReverification {
        test_claim: DissolutionTestClaimId,
    },
    ProducerStageDerivation {
        derivation_path: Vec<DissolutionProducerStageId>,
    },
}

/// Structural mirror of one `v4.lens.subsumption.DissolutionSubsumption` row.
///
/// `subsumed_fixes` mirrors the substrate `Set<DiffId>` shape as a
/// concrete enumerable `BTreeSet`: the predicate-shaped substrate set is
/// approximated by its membership extension here, so the host runtime can
/// iterate the fix set without enumerating the universe of `DiffId`s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DissolutionSubsumptionRuntimeRow {
    pub root_fix: DissolutionDiffId,
    pub subsumed_fixes: BTreeSet<DissolutionDiffId>,
    pub verification: SubsumptionVerificationRuntime,
}

/// Source authority for the post-root-fix TestClaim report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MechanicalReverificationReportAuthority {
    LensCiTestClaimRunner,
}

/// Typed execution status for the TestClaim rerun.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MechanicalReverificationExecution {
    Completed {
        report_authority: MechanicalReverificationReportAuthority,
    },
    Failed,
    NotRun,
    ReportUnavailable,
}

/// Lens-CI evidence from applying `root_fix` and rerunning the lens suite for one TestClaim row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MechanicalReverificationRun {
    pub test_claim: DissolutionTestClaimId,
    pub applied_root_fix: DissolutionDiffId,
    pub execution: MechanicalReverificationExecution,
    pub findings_before: BTreeSet<DissolutionDiffId>,
    pub findings_after: BTreeSet<DissolutionDiffId>,
}

/// Typed fail-closed outcome for `MechanicalReverification` row validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MechanicalReverificationError {
    NonMechanicalRow,
    TestClaimMismatch {
        expected: DissolutionTestClaimId,
        observed: DissolutionTestClaimId,
    },
    RootFixMismatch {
        expected: DissolutionDiffId,
        observed: DissolutionDiffId,
    },
    TestClaimExecutionNotAuthoritative {
        execution: MechanicalReverificationExecution,
    },
    SubsumedFixMissingBefore {
        fix: DissolutionDiffId,
    },
    SubsumedFixStillPresentAfter {
        fix: DissolutionDiffId,
    },
}

/// Verdict emitted by the Lens-CI `MechanicalReverification` runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MechanicalReverificationVerdict {
    Verified,
    NotVerified {
        diagnostic: MechanicalReverificationError,
    },
    Unverifiable {
        diagnostic: MechanicalReverificationError,
    },
}

/// Validate a `MechanicalReverification` row against one concrete rerun.
///
/// This is the Lens-CI host runtime for `v4.lens.subsumption` while v4 TestClaim execution is
/// still staged: CI supplies the observed pre/post fix sets, and this function validates the row
/// without reading comments or diagnostic prose.
pub fn verify_mechanical_reverification(
    row: &DissolutionSubsumptionRuntimeRow,
    run: &MechanicalReverificationRun,
) -> Result<(), MechanicalReverificationError> {
    let expected_claim = match &row.verification {
        SubsumptionVerificationRuntime::MechanicalReverification { test_claim } => test_claim,
        SubsumptionVerificationRuntime::ProducerStageDerivation { .. } => {
            return Err(MechanicalReverificationError::NonMechanicalRow);
        }
    };

    if expected_claim != &run.test_claim {
        return Err(MechanicalReverificationError::TestClaimMismatch {
            expected: expected_claim.clone(),
            observed: run.test_claim.clone(),
        });
    }
    if row.root_fix != run.applied_root_fix {
        return Err(MechanicalReverificationError::RootFixMismatch {
            expected: row.root_fix.clone(),
            observed: run.applied_root_fix.clone(),
        });
    }
    if run.execution
        != (MechanicalReverificationExecution::Completed {
            report_authority: MechanicalReverificationReportAuthority::LensCiTestClaimRunner,
        })
    {
        return Err(
            MechanicalReverificationError::TestClaimExecutionNotAuthoritative {
                execution: run.execution.clone(),
            },
        );
    }

    for fix in row.subsumed_fixes.iter() {
        if !run.findings_before.contains(fix) {
            return Err(MechanicalReverificationError::SubsumedFixMissingBefore {
                fix: fix.clone(),
            });
        }
        if run.findings_after.contains(fix) {
            return Err(
                MechanicalReverificationError::SubsumedFixStillPresentAfter { fix: fix.clone() },
            );
        }
    }

    Ok(())
}

/// Emit the Lens-CI verdict for one scaffolded `MechanicalReverification` rerun.
pub fn mechanical_reverification_verdict(
    row: &DissolutionSubsumptionRuntimeRow,
    run: &MechanicalReverificationRun,
) -> MechanicalReverificationVerdict {
    match verify_mechanical_reverification(row, run) {
        Ok(()) => MechanicalReverificationVerdict::Verified,
        Err(err @ MechanicalReverificationError::SubsumedFixStillPresentAfter { .. }) => {
            MechanicalReverificationVerdict::NotVerified { diagnostic: err }
        }
        Err(err) => MechanicalReverificationVerdict::Unverifiable { diagnostic: err },
    }
}

fn topo_sort_subset(
    dag: &CiWorkflowDagInput,
    subset: &HashSet<&str>,
) -> Result<Vec<CiGateId>, CiAffectedGatesError> {
    let mut indegree: HashMap<&str, usize> = HashMap::new();
    for id in subset.iter().copied() {
        indegree.entry(id).or_insert(0);
    }
    for (from, to) in &dag.edges {
        let from = from.as_str();
        let to = to.as_str();
        if subset.contains(from) && subset.contains(to) {
            *indegree.entry(to).or_insert(0) += 1;
        }
    }

    let mut queue: VecDeque<&str> = indegree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(n, _)| *n)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let mut out: Vec<CiGateId> = Vec::new();
    while let Some(n) = queue.pop_front() {
        out.push(n.to_string());
        for (from, to) in &dag.edges {
            if from.as_str() != n {
                continue;
            }
            let to = to.as_str();
            if !subset.contains(to) {
                continue;
            }
            if let Some(d) = indegree.get_mut(to) {
                *d -= 1;
                if *d == 0 {
                    queue.push_back(to);
                }
            }
        }
    }

    if out.len() != subset.len() {
        return Err(CiAffectedGatesError::NonAcyclicPrerequisiteGraph);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diff_id(id: &str) -> DissolutionDiffId {
        DissolutionDiffId::from(id)
    }

    fn test_claim_id(id: &str) -> DissolutionTestClaimId {
        DissolutionTestClaimId::from(id)
    }

    fn producer_stage_id(id: &str) -> DissolutionProducerStageId {
        DissolutionProducerStageId::from(id)
    }

    fn demo_ci_dag() -> CiWorkflowDagInput {
        CiWorkflowDagInput {
            gates: vec![
                CiGateMeta {
                    id: "compile-gates".into(),
                    blocking: true,
                },
                CiGateMeta {
                    id: "lint".into(),
                    blocking: true,
                },
                CiGateMeta {
                    id: "tests".into(),
                    blocking: true,
                },
                CiGateMeta {
                    id: "l1-ratchet".into(),
                    blocking: true,
                },
            ],
            edges: vec![
                ("compile-gates".into(), "lint".into()),
                ("compile-gates".into(), "tests".into()),
                ("lint".into(), "l1-ratchet".into()),
                ("tests".into(), "l1-ratchet".into()),
            ],
        }
    }

    fn demo_mechanical_subsumption_row_fixture() -> DissolutionSubsumptionRuntimeRow {
        DissolutionSubsumptionRuntimeRow {
            root_fix: diff_id("demo_mechanical_root_fix"),
            subsumed_fixes: BTreeSet::from([
                diff_id("demo_mechanical_leaf_fix_a"),
                diff_id("demo_mechanical_leaf_fix_b"),
                diff_id("demo_mechanical_leaf_fix_c"),
            ]),
            verification: SubsumptionVerificationRuntime::MechanicalReverification {
                test_claim: test_claim_id("demo_mechanical_reverification_claim"),
            },
        }
    }

    fn demo_producer_stage_subsumption_row_fixture() -> DissolutionSubsumptionRuntimeRow {
        DissolutionSubsumptionRuntimeRow {
            root_fix: diff_id("demo_producer_stage_root_fix"),
            subsumed_fixes: BTreeSet::from([diff_id("demo_producer_stage_leaf_fix")]),
            verification: SubsumptionVerificationRuntime::ProducerStageDerivation {
                derivation_path: vec![producer_stage_id("demo_producer_stage")],
            },
        }
    }

    fn successful_demo_reverification() -> MechanicalReverificationRun {
        let row = demo_mechanical_subsumption_row_fixture();
        MechanicalReverificationRun {
            test_claim: test_claim_id("demo_mechanical_reverification_claim"),
            applied_root_fix: diff_id("demo_mechanical_root_fix"),
            execution: MechanicalReverificationExecution::Completed {
                report_authority: MechanicalReverificationReportAuthority::LensCiTestClaimRunner,
            },
            findings_before: row.subsumed_fixes.clone(),
            findings_after: BTreeSet::new(),
        }
    }

    #[test]
    fn select_affected_gates_empty_diff_returns_empty_plan() {
        let dag = demo_ci_dag();
        let diff = CiWorkflowDiff::TouchedGates(BTreeSet::new());
        assert_eq!(select_affected_gates(&dag, &diff), Ok(Vec::new()));
    }

    #[test]
    fn select_affected_gates_touch_all_runs_full_dag_in_topo_order() {
        let dag = demo_ci_dag();
        let got =
            select_affected_gates(&dag, &CiWorkflowDiff::TouchAll).expect("demo dag is valid");
        assert_eq!(
            got,
            vec![
                "compile-gates".to_string(),
                "lint".to_string(),
                "tests".to_string(),
                "l1-ratchet".to_string(),
            ]
        );
    }

    #[test]
    fn select_affected_gates_isolated_gate_no_edges() {
        let dag = CiWorkflowDagInput {
            gates: vec![CiGateMeta {
                id: "solo".into(),
                blocking: true,
            }],
            edges: vec![],
        };
        let diff = CiWorkflowDiff::TouchedGates(BTreeSet::from(["solo".into()]));
        assert_eq!(
            select_affected_gates(&dag, &diff),
            Ok(vec!["solo".to_string()])
        );
    }

    #[test]
    fn select_affected_gates_unknown_touch_id_forces_superset() {
        let dag = demo_ci_dag();
        let diff = CiWorkflowDiff::TouchedGates(BTreeSet::from(["no-such-gate".into()]));
        let got = select_affected_gates(&dag, &diff).expect("superset path is acyclic");
        let expect = select_affected_gates(&dag, &CiWorkflowDiff::TouchAll).expect("demo dag");
        assert_eq!(got, expect);
    }

    #[test]
    fn select_affected_gates_rejects_unknown_edge_endpoint() {
        let dag = CiWorkflowDagInput {
            gates: vec![CiGateMeta {
                id: "solo".into(),
                blocking: true,
            }],
            edges: vec![("solo".into(), "phantom".into())],
        };
        let diff = CiWorkflowDiff::TouchedGates(BTreeSet::from(["solo".into()]));
        assert_eq!(
            select_affected_gates(&dag, &diff),
            Err(CiAffectedGatesError::UnknownEdgeEndpoint {
                from: "solo".into(),
                to: "phantom".into(),
            })
        );
    }

    #[test]
    fn select_affected_gates_rejects_duplicate_gate_roster_ids() {
        let dag = CiWorkflowDagInput {
            gates: vec![
                CiGateMeta {
                    id: "dup".into(),
                    blocking: true,
                },
                CiGateMeta {
                    id: "dup".into(),
                    blocking: false,
                },
            ],
            edges: vec![],
        };
        assert_eq!(
            select_affected_gates(&dag, &CiWorkflowDiff::TouchAll),
            Err(CiAffectedGatesError::DuplicateGateRosterId { id: "dup".into() })
        );
    }

    #[test]
    fn select_affected_gates_rejects_prerequisite_cycle() {
        let dag = CiWorkflowDagInput {
            gates: vec![
                CiGateMeta {
                    id: "a".into(),
                    blocking: true,
                },
                CiGateMeta {
                    id: "b".into(),
                    blocking: true,
                },
            ],
            edges: vec![("a".into(), "b".into()), ("b".into(), "a".into())],
        };
        assert_eq!(
            select_affected_gates(&dag, &CiWorkflowDiff::TouchAll),
            Err(CiAffectedGatesError::NonAcyclicPrerequisiteGraph)
        );
    }

    #[test]
    fn select_affected_gates_rejects_global_cycle_even_when_touch_isolated() {
        let dag = CiWorkflowDagInput {
            gates: vec![
                CiGateMeta {
                    id: "a".into(),
                    blocking: true,
                },
                CiGateMeta {
                    id: "b".into(),
                    blocking: true,
                },
                CiGateMeta {
                    id: "c".into(),
                    blocking: true,
                },
            ],
            edges: vec![("a".into(), "b".into()), ("b".into(), "a".into())],
        };
        let diff = CiWorkflowDiff::TouchedGates(BTreeSet::from(["c".into()]));
        assert_eq!(
            select_affected_gates(&dag, &diff),
            Err(CiAffectedGatesError::NonAcyclicPrerequisiteGraph)
        );
    }

    #[test]
    fn select_affected_gates_empty_touch_rejects_when_global_graph_has_cycle() {
        let dag = CiWorkflowDagInput {
            gates: vec![
                CiGateMeta {
                    id: "a".into(),
                    blocking: true,
                },
                CiGateMeta {
                    id: "b".into(),
                    blocking: true,
                },
            ],
            edges: vec![("a".into(), "b".into()), ("b".into(), "a".into())],
        };
        let diff = CiWorkflowDiff::TouchedGates(BTreeSet::new());
        assert_eq!(
            select_affected_gates(&dag, &diff),
            Err(CiAffectedGatesError::NonAcyclicPrerequisiteGraph)
        );
    }

    #[test]
    fn select_affected_gates_lint_only_pulls_l1_and_shared_prereqs() {
        let dag = demo_ci_dag();
        let diff = CiWorkflowDiff::TouchedGates(BTreeSet::from(["lint".into()]));
        let got = select_affected_gates(&dag, &diff).expect("demo dag");
        let set: BTreeSet<_> = got.iter().cloned().collect();
        assert_eq!(
            set,
            BTreeSet::from([
                "compile-gates".into(),
                "lint".into(),
                "tests".into(),
                "l1-ratchet".into(),
            ])
        );
        assert_eq!(got.first().map(String::as_str), Some("compile-gates"));
    }

    #[test]
    fn selection_subset_under_touch_set_growth_holds_on_demo_dag() {
        let dag = demo_ci_dag();
        let narrow = CiWorkflowDiff::TouchedGates(BTreeSet::from(["lint".into()]));
        let wide = CiWorkflowDiff::TouchedGates(BTreeSet::from(["lint".into(), "tests".into()]));
        assert!(selection_subset_under_touch_set_growth(
            &dag, &narrow, &wide
        ));
    }

    #[test]
    fn selection_subset_empty_diff_is_subset_of_nonempty() {
        let dag = demo_ci_dag();
        let empty = CiWorkflowDiff::TouchedGates(BTreeSet::new());
        let wide = CiWorkflowDiff::TouchedGates(BTreeSet::from(["tests".into()]));
        assert!(selection_subset_under_touch_set_growth(&dag, &empty, &wide));
    }

    #[test]
    fn binary_shim_receipt_unknown_narrowing_matches_touch_all() {
        let dag = demo_ci_dag();
        let receipt = CiBinaryShimAffectedSetReceipt {
            narrowing_available: false,
            proven_direct_gate_touches: BTreeSet::from(["lint".into()]),
        };
        let got = select_affected_gates_for_binary_shim(&dag, &receipt).expect("valid dag");
        let expect = select_affected_gates(&dag, &CiWorkflowDiff::TouchAll).expect("valid dag");
        assert_eq!(got, expect);
    }

    #[test]
    fn binary_shim_receipt_narrow_lint_matches_touched_gates_lint() {
        let dag = demo_ci_dag();
        let receipt = CiBinaryShimAffectedSetReceipt {
            narrowing_available: true,
            proven_direct_gate_touches: BTreeSet::from(["lint".into()]),
        };
        let got = select_affected_gates_for_binary_shim(&dag, &receipt).expect("valid dag");
        let expect = select_affected_gates(
            &dag,
            &CiWorkflowDiff::TouchedGates(BTreeSet::from(["lint".into()])),
        )
        .expect("valid dag");
        assert_eq!(got, expect);
    }

    #[test]
    fn binary_shim_receipt_empty_narrow_seed_yields_empty_plan() {
        let dag = demo_ci_dag();
        let receipt = CiBinaryShimAffectedSetReceipt {
            narrowing_available: true,
            proven_direct_gate_touches: BTreeSet::new(),
        };
        assert_eq!(
            select_affected_gates_for_binary_shim(&dag, &receipt),
            Ok(Vec::new())
        );
    }

    #[test]
    fn mechanical_row_fixture_uses_nominal_runtime_ids() {
        let row = demo_mechanical_subsumption_row_fixture();
        let SubsumptionVerificationRuntime::MechanicalReverification { test_claim } =
            &row.verification
        else {
            panic!("demo subsumption row must use MechanicalReverification");
        };
        assert_eq!(
            test_claim,
            &test_claim_id("demo_mechanical_reverification_claim")
        );
        assert!(row
            .subsumed_fixes
            .contains(&diff_id("demo_mechanical_leaf_fix_a")));
        assert_eq!(row.subsumed_fixes.len(), 3);
    }

    #[test]
    fn producer_stage_rows_preserve_derivation_path() {
        let row = demo_producer_stage_subsumption_row_fixture();
        assert_eq!(
            row.verification,
            SubsumptionVerificationRuntime::ProducerStageDerivation {
                derivation_path: vec![producer_stage_id("demo_producer_stage")]
            }
        );
    }

    #[test]
    fn mechanical_reverification_accepts_when_subsumed_fixes_clear() {
        let row = demo_mechanical_subsumption_row_fixture();
        let run = successful_demo_reverification();
        assert_eq!(verify_mechanical_reverification(&row, &run), Ok(()));
        assert_eq!(
            mechanical_reverification_verdict(&row, &run),
            MechanicalReverificationVerdict::Verified
        );
    }

    #[test]
    fn mechanical_reverification_rejects_surviving_subsumed_fix() {
        let row = demo_mechanical_subsumption_row_fixture();
        let mut run = successful_demo_reverification();
        run.findings_after
            .insert(diff_id("demo_mechanical_leaf_fix_a"));
        assert_eq!(
            verify_mechanical_reverification(&row, &run),
            Err(
                MechanicalReverificationError::SubsumedFixStillPresentAfter {
                    fix: diff_id("demo_mechanical_leaf_fix_a")
                }
            )
        );
        assert_eq!(
            mechanical_reverification_verdict(&row, &run),
            MechanicalReverificationVerdict::NotVerified {
                diagnostic: MechanicalReverificationError::SubsumedFixStillPresentAfter {
                    fix: diff_id("demo_mechanical_leaf_fix_a")
                }
            }
        );
    }

    #[test]
    fn mechanical_reverification_rejects_unobserved_subsumed_fix() {
        let row = demo_mechanical_subsumption_row_fixture();
        let mut run = successful_demo_reverification();
        run.findings_before
            .remove(&diff_id("demo_mechanical_leaf_fix_c"));
        assert_eq!(
            verify_mechanical_reverification(&row, &run),
            Err(MechanicalReverificationError::SubsumedFixMissingBefore {
                fix: diff_id("demo_mechanical_leaf_fix_c")
            })
        );
        assert_eq!(
            mechanical_reverification_verdict(&row, &run),
            MechanicalReverificationVerdict::Unverifiable {
                diagnostic: MechanicalReverificationError::SubsumedFixMissingBefore {
                    fix: diff_id("demo_mechanical_leaf_fix_c")
                }
            }
        );
    }

    #[test]
    fn mechanical_reverification_rejects_unexecuted_claim() {
        let row = demo_mechanical_subsumption_row_fixture();
        let mut run = successful_demo_reverification();
        run.execution = MechanicalReverificationExecution::NotRun;
        assert_eq!(
            verify_mechanical_reverification(&row, &run),
            Err(
                MechanicalReverificationError::TestClaimExecutionNotAuthoritative {
                    execution: MechanicalReverificationExecution::NotRun
                }
            )
        );
        assert_eq!(
            mechanical_reverification_verdict(&row, &run),
            MechanicalReverificationVerdict::Unverifiable {
                diagnostic: MechanicalReverificationError::TestClaimExecutionNotAuthoritative {
                    execution: MechanicalReverificationExecution::NotRun
                }
            }
        );
    }

    #[test]
    fn mechanical_reverification_rejects_wrong_claim_or_root() {
        let row = demo_mechanical_subsumption_row_fixture();
        let mut wrong_claim = successful_demo_reverification();
        wrong_claim.test_claim = test_claim_id("other_claim");
        assert_eq!(
            verify_mechanical_reverification(&row, &wrong_claim),
            Err(MechanicalReverificationError::TestClaimMismatch {
                expected: test_claim_id("demo_mechanical_reverification_claim"),
                observed: test_claim_id("other_claim"),
            })
        );

        let mut wrong_root = successful_demo_reverification();
        wrong_root.applied_root_fix = diff_id("other_root");
        assert_eq!(
            verify_mechanical_reverification(&row, &wrong_root),
            Err(MechanicalReverificationError::RootFixMismatch {
                expected: diff_id("demo_mechanical_root_fix"),
                observed: diff_id("other_root"),
            })
        );
    }

    #[test]
    fn mechanical_reverification_rejects_producer_stage_rows() {
        let row = demo_producer_stage_subsumption_row_fixture();
        let run = MechanicalReverificationRun {
            test_claim: test_claim_id("demo_producer_stage_claim"),
            applied_root_fix: diff_id("demo_producer_stage_root_fix"),
            execution: MechanicalReverificationExecution::Completed {
                report_authority: MechanicalReverificationReportAuthority::LensCiTestClaimRunner,
            },
            findings_before: BTreeSet::from([diff_id("demo_producer_stage_leaf_fix")]),
            findings_after: BTreeSet::new(),
        };
        assert_eq!(
            verify_mechanical_reverification(&row, &run),
            Err(MechanicalReverificationError::NonMechanicalRow)
        );
    }
}
