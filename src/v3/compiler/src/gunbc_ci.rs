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

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

/// Structural mirror of `CIGate.id` in `dsl/gunbc/ci.dag`.
pub type CiGateId = String;

/// Mirror of `CIGate { blocking, … }` fields needed for selection.
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

/// Returns gate ids to execute: the **connected component** of the seed under the symmetric
/// closure of prerequisite edges (iterate `from ↔ to` adjacency to fixpoint), then
/// **topologically sorted** on the directed subgraph (prerequisites first).
///
/// **Fail-closed** (`design-t-wad-slice-7-binary-shim-affected-set-selection-canvas.md` §3):
/// any touched id not present in `dag.gates` forces **all** gates listed in `dag.gates`
/// into the plan (superset within this DAG).
///
/// **Empty diff** (`TouchedGates` empty): returns an **empty** plan — callers map that
/// to “no merge-blocking re-verify” vs “full superset” per repository policy **outside**
/// this function (this module does not read policy carriers).
pub fn select_affected_gates(dag: &CiWorkflowDagInput, diff: &CiWorkflowDiff) -> Vec<CiGateId> {
    let known: HashSet<&str> = dag.gates.iter().map(|g| g.id.as_str()).collect();
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
        return Vec::new();
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

/// Set inclusion witness for ratchet tests.
///
/// Returns whether `select_affected_gates(dag, narrow)` ⊆ `select_affected_gates(dag, wide)` as sets.
/// Intended use: both arguments are [`CiWorkflowDiff::TouchedGates`] with `narrow_ids ⊆ wide_ids`, or
/// `wide` is [`CiWorkflowDiff::TouchAll`] (superset selection). Other pairings are not a refinement
/// partial order and may return `false` even when both plans are individually valid.
pub fn selection_subset_under_touch_set_growth(
    dag: &CiWorkflowDagInput,
    narrow: &CiWorkflowDiff,
    wide: &CiWorkflowDiff,
) -> bool {
    let a: BTreeSet<_> = select_affected_gates(dag, narrow).into_iter().collect();
    let b: BTreeSet<_> = select_affected_gates(dag, wide).into_iter().collect();
    a.is_subset(&b)
}

fn topo_sort_subset(dag: &CiWorkflowDagInput, subset: &HashSet<&str>) -> Vec<CiGateId> {
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
        // Cycle or broken graph — fail-closed: deterministic id order of subset.
        let mut all: Vec<CiGateId> = subset.iter().map(|s| (*s).to_string()).collect();
        all.sort();
        return all;
    }
    out
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
        assert!(select_affected_gates(&dag, &diff).is_empty());
    }

    #[test]
    fn select_affected_gates_touch_all_runs_full_dag_in_topo_order() {
        let dag = demo_ci_dag();
        let got = select_affected_gates(&dag, &CiWorkflowDiff::TouchAll);
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
        assert_eq!(select_affected_gates(&dag, &diff), vec!["solo"]);
    }

    #[test]
    fn select_affected_gates_unknown_touch_id_forces_superset() {
        let dag = demo_ci_dag();
        let diff = CiWorkflowDiff::TouchedGates(BTreeSet::from(["no-such-gate".into()]));
        let got = select_affected_gates(&dag, &diff);
        let expect = select_affected_gates(&dag, &CiWorkflowDiff::TouchAll);
        assert_eq!(got, expect);
    }

    #[test]
    fn select_affected_gates_lint_only_pulls_l1_and_shared_prereqs() {
        let dag = demo_ci_dag();
        let diff = CiWorkflowDiff::TouchedGates(BTreeSet::from(["lint".into()]));
        let got = select_affected_gates(&dag, &diff);
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
