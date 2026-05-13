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
}
