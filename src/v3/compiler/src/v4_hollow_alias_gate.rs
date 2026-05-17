//! T-30 — hollow-alias / fact-density structural gate (bootstrap mirror).
//!
//! Substrate authority for the **nominal witness type** lives in
//! `src/v4/std/fact_density.dag` (`SourceSpecReadFact`, currently staged as a
//! nominal alias per that file's resolver NOTE — migrate to `Node` payload).
//! M1(2.8) currently rejects block-bodied `.dag` functions that walk `Node` with `match` in the
//! user declaration range (`lower.rs` — `reject_user_unparsed_scaffolds`).
//! This module is the hermetic **pure** mirror of the intended gate until
//! those bodies can ship in `.dag` form.
//!
//! Contract (lockstep with `fact_density.dag` header):
//! - **Hollow:** a type `Instantiation` spine with exactly two **bare Atom**
//!   positional children and **no** `SourceSpecReadFact` witness anywhere in
//!   that subtree (D2 bare `type LangX = StdY` anti-pattern).
//! - **Pass:** anything else, including trees that carry a witness root.

/// Fail-closed outcome analogue to `std/diagnostic.dag`'s `Outcome<Bool>`
/// success token (`Produced { value: true }` in the `.dag` spelling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HollowAliasGateOutcome {
    Produced,
    Rejected,
}

/// Classifier for the **minimal** structural slice the T-30 proxy inspects.
///
/// This is not the full v4 `Node` substrate; it is an isomorphic test /
/// integration harness carrier. Mapping from real `Node` trees is a T-1 /
/// normalize concern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HollowGateKind {
    /// `TypeNode` + `Atom`.
    TypeAtom,
    /// `TypeNode` + `Instantiation` (children are positional targets in order).
    TypeInstantiation,
    /// A subtree rooted at a `SourceSpecReadFact` / `SourceSpecRead` Disj witness.
    SourceSpecReadWitnessRoot,
    /// Any other type or computation node — does not participate in the proxy spine.
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HollowGateNode {
    pub kind: HollowGateKind,
    pub children: Vec<HollowGateNode>,
}

/// Pure gate entry — the T-30 `Node → Outcome` contract (test / harness IR).
pub fn hollow_alias_gate(root: &HollowGateNode) -> HollowAliasGateOutcome {
    if hollow_alias_violation_present(root) {
        HollowAliasGateOutcome::Rejected
    } else {
        HollowAliasGateOutcome::Produced
    }
}

fn hollow_alias_violation_present(n: &HollowGateNode) -> bool {
    if is_hollow_instantiation_candidate(n) {
        return true;
    }
    n.children.iter().any(hollow_alias_violation_present)
}

fn is_hollow_instantiation_candidate(n: &HollowGateNode) -> bool {
    if n.kind != HollowGateKind::TypeInstantiation {
        return false;
    }
    if n.children.len() != 2 {
        return false;
    }
    if !n
        .children
        .iter()
        .all(|c| c.kind == HollowGateKind::TypeAtom)
    {
        return false;
    }
    if subtree_contains_spec_read_fact(n) {
        return false;
    }
    true
}

fn subtree_contains_spec_read_fact(n: &HollowGateNode) -> bool {
    if node_roots_source_spec_read_fact(n) {
        return true;
    }
    n.children.iter().any(subtree_contains_spec_read_fact)
}

fn node_roots_source_spec_read_fact(n: &HollowGateNode) -> bool {
    n.kind == HollowGateKind::SourceSpecReadWitnessRoot
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom() -> HollowGateNode {
        HollowGateNode {
            kind: HollowGateKind::TypeAtom,
            children: Vec::new(),
        }
    }

    fn inst(children: Vec<HollowGateNode>) -> HollowGateNode {
        HollowGateNode {
            kind: HollowGateKind::TypeInstantiation,
            children,
        }
    }

    fn witness() -> HollowGateNode {
        HollowGateNode {
            kind: HollowGateKind::SourceSpecReadWitnessRoot,
            children: vec![atom()],
        }
    }

    #[test]
    fn rejects_two_atom_instantiation_without_witness() {
        let hollow = inst(vec![atom(), atom()]);
        assert_eq!(hollow_alias_gate(&hollow), HollowAliasGateOutcome::Rejected);
    }

    #[test]
    fn accepts_when_subtree_contains_witness() {
        let ok = inst(vec![atom(), witness()]);
        assert_eq!(hollow_alias_gate(&ok), HollowAliasGateOutcome::Produced);
    }

    #[test]
    fn accepts_instantiation_when_inner_spine_is_not_hollow() {
        let inner_ok = inst(vec![atom(), witness()]);
        let ok = inst(vec![atom(), inner_ok]);
        assert_eq!(hollow_alias_gate(&ok), HollowAliasGateOutcome::Produced);
    }

    #[test]
    fn accepts_atom_only() {
        let ok = atom();
        assert_eq!(hollow_alias_gate(&ok), HollowAliasGateOutcome::Produced);
    }

    #[test]
    fn rejects_nested_hollow_under_wrapper() {
        let root = HollowGateNode {
            kind: HollowGateKind::Other,
            children: vec![inst(vec![atom(), atom()])],
        };
        assert_eq!(hollow_alias_gate(&root), HollowAliasGateOutcome::Rejected);
    }
}
