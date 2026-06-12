//! T-30 — hollow-alias structural gate (bootstrap mirror).
//!
//! **Authority:** `docs/modeling-discipline.md` — **Practice 8 — Fact-bundle
//! modeling**, #### *Interim floor: the hollow-alias discriminator* (landed on
//! `main` with **#3226** / `77b9e7d72`; Practice 9 tightened with **#3234** /
//! `125fc88c8` — **diff `origin/main:docs/modeling-discipline.md` if this mirror
//! drifts**). This module is **not** the integer width/signedness worked example;
//! that is one motivating instance only (manager brief T-30 / CORE).
//!
//! **P2-staging `.dag` nominal witness:** `src/v4/std/fact_density.dag` — `SourceSpecReadFact`
//! is a **body-less nominal** (same *shape* as `Symbol` / `Hash` in `std/node.dag`):
//! not a `Nat` alias, not an unconstrained product. Per **INVARIANTS §P2** / **Practice 5**,
//! this file is **staging** until a **generated** `.dag` consumer reads it — it is **not**
//! a landed substrate primitive; this Rust module is the **interim authority** for the
//! Practice-8 hollow predicate. **Dissolution / richer `Node` payload:** same **T-30** bundle as
//! `INVARIANTS.md` §P5(b) on this path — former `src/v4/TASKS.md` T-30 interim mirror paragraph (ledger deleted) +
//! **`src/v4/DECISIONS.md`** T-30 `fact_density.dag` encoding note (generated `.dag` gate / bootstrap
//! bridge), not a separate prose-only trigger.
//! M1(2.8) rejects block-bodied `.dag` functions that walk `Node` with `match` in the
//! user declaration range (`lower.rs` — `reject_user_unparsed_scaffolds`), so the
//! gate logic is mirrored here as a **pure** harness until those bodies ship in `.dag`.
//!
//! **Practice-4 🟢/🟡 lines below:** harness-side §4 color receipts on this **P5(b)** path
//! (`INVARIANTS.md` §P5(b) names the file). The repo has **no** CI step today that blindly
//! greps these emojis across `src/v3/compiler/src/**/*.rs` as if every hit were a user `.dag`
//! coproduct tag; if such automation is added, **exclude** paths with an explicit P5(b) row so
//! interim hand-Rust mirrors are not misclassified.
//!
//! ## Interim-floor hollow predicate (three prongs + exemption)
//!
//! A declaration is **hollow** when **all three** hold on the harness projection (fail
//! closed ⇒ [`HollowAliasGateOutcome::Rejected`]), matching the numbered list in
//! `modeling-discipline.md` (~lines 389–399):
//!
//! 1. **Bare alias** (`type X = Y`) or a single-field wrapper that adds no field of its own
//!    — [`HollowDeclarationSite::bare_alias_or_empty_wrapper`].
//! 2. **External spec primitive** — a language / format / framework **spec** names the
//!    subject and states facts about it — [`ModeledSubject::ExternalSpecPrimitive`].
//! 3. **No coincidence evidence** — no `src/v4/DECISIONS.md` entry proving the alias
//!    endpoints coincide, cited from the modeling file by at most a **one-line tag**
//!    (Practice 9). The harness field [`HollowDeclarationSite::coincidence_evidence`]
//!    is the **structural projection** of that prong for test IR (evidence present vs absent).
//!
//! **Exempt:** kernel-ambient atoms — `Bool`, `Char`, and the other **irreducible**
//! substrate atoms (`src/v4/STRUCTURE.md` §Kernel-ambient types) — are legitimately atomic.
//! [`ModeledSubject::KernelAmbientAtom`] short-circuits before the three-part `AND`.

#![cfg_attr(not(test), allow(dead_code))]
// T-30 P5(b): no in-crate production consumer yet — `allow(dead_code)` dissolves with the
// generated `.dag` gate per `INVARIANTS.md` §P5(b) on this path.

// Practice 4 (coproduct checkpoint, `docs/modeling-discipline.md` §4):
// 🟢 GREEN — terminal two-variant harness mirror of `std/diagnostic.dag` `Outcome<Bool>`
// pass/fail tokens (`Produced` / `Rejected`); no third semantic axis at this boundary.
// Ledger: P5(b) interim Rust mirror only — the *verdict shape* stays this coproduct; a generated
// `.dag` checker does not add a third outcome variant here.
// These 🟢/🟡 lines are **harness-side §4 receipts** (dissolution + color), not a claim that
// arbitrary Rust enums are user `.dag` coproducts — scope stays the hand-built T-30 mirror.
/// Fail-closed outcome analogue to `std/diagnostic.dag`'s `Outcome<Bool>`
/// success token (`Produced { value: true }` in the `.dag` spelling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HollowAliasGateOutcome {
    Produced,
    Rejected,
}

// Practice 4 (coproduct checkpoint, `docs/modeling-discipline.md` §4):
// 🟡 YELLOW — harness IR for Practice 8 condition (2); substrate should own this classification
// when the T-30 hollow-alias gate is generated from `.dag` / walks structural `Node`.
// scaffold: dissolve when T-30 `.dag` checker + bootstrap bridge land (`INVARIANTS.md` P5(b)).
/// Classification of **what** the declaration models for condition (2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeledSubject {
    /// Kernel-ambient atom — **exempt** from the hollow three-prong `AND` (Practice 8 interim).
    ///
    /// **Authoritative membership** is `src/v4/STRUCTURE.md` **Kernel-ambient types** (same
    /// section the module-level doc cites). The substrate names there (`String`, `Int`, …) are
    /// the spec anchor; this enum variant is **harness / test IR** only until the gate walks
    /// structural `Node` — when wiring to syntax, keep the exempt set aligned with that doc, not
    /// with ad hoc renames in tests.
    KernelAmbientAtom,
    /// Internal `std` / compiler substrate — not an external spec primitive for (2).
    InternalStdCarrier,
    /// External spec names a primitive with its own facts (Practice 8 (2)).
    ExternalSpecPrimitive,
}

/// One declaration site’s Practice-8 predicate inputs (structural carrier).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HollowDeclarationSite {
    /// Condition (1): bare `type X = Y` or single-field wrapper that adds no own field.
    pub bare_alias_or_empty_wrapper: bool,
    /// Condition (2): what the declaration claims to model.
    pub modeled_subject: ModeledSubject,
    /// Prong (3): **true** when coincidence evidence is in scope (a `src/v4/DECISIONS.md`
    /// proof of coincide, cited per Practice 9); **false** supplies the third conjunct for hollow.
    pub coincidence_evidence: bool,
}

// Practice 4 (coproduct checkpoint, `docs/modeling-discipline.md` §4):
// 🟡 YELLOW — interim harness AST discriminator (`Declaration` vs `Group`) until the Practice-8
// gate walks structural `Node` in the substrate (same T-30 sunset as `ModeledSubject`).
// scaffold: dissolve when T-30 `.dag` checker + bootstrap bridge land (`INVARIANTS.md` P5(b)).
/// Classifier for the **minimal** tree the T-30 harness walks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HollowGateKind {
    /// A type/alias declaration site carrying Practice-8 inputs.
    Declaration(HollowDeclarationSite),
    /// Transparent grouping node (module, section) — recurse into [`HollowGateNode::children`].
    Group,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HollowGateNode {
    pub kind: HollowGateKind,
    pub children: Vec<HollowGateNode>,
}

/// Pure gate entry — Practice 8 structural `Node` analogue (test / harness IR).
pub fn hollow_alias_gate(root: &HollowGateNode) -> HollowAliasGateOutcome {
    if hollow_alias_violation_present(root) {
        HollowAliasGateOutcome::Rejected
    } else {
        HollowAliasGateOutcome::Produced
    }
}

fn hollow_alias_violation_present(n: &HollowGateNode) -> bool {
    if let HollowGateKind::Declaration(site) = &n.kind {
        if declaration_site_is_hollow(site) {
            return true;
        }
    }
    n.children.iter().any(hollow_alias_violation_present)
}

fn declaration_site_is_hollow(site: &HollowDeclarationSite) -> bool {
    if matches!(site.modeled_subject, ModeledSubject::KernelAmbientAtom) {
        return false;
    }
    site.bare_alias_or_empty_wrapper
        && matches!(site.modeled_subject, ModeledSubject::ExternalSpecPrimitive)
        && !site.coincidence_evidence
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decl(site: HollowDeclarationSite) -> HollowGateNode {
        HollowGateNode {
            kind: HollowGateKind::Declaration(site),
            children: Vec::new(),
        }
    }

    fn group(children: Vec<HollowGateNode>) -> HollowGateNode {
        HollowGateNode {
            kind: HollowGateKind::Group,
            children,
        }
    }

    fn site(bare: bool, subj: ModeledSubject, evidence: bool) -> HollowDeclarationSite {
        HollowDeclarationSite {
            bare_alias_or_empty_wrapper: bare,
            modeled_subject: subj,
            coincidence_evidence: evidence,
        }
    }

    #[test]
    fn rejects_bare_external_primitive_without_evidence() {
        let hollow = decl(site(true, ModeledSubject::ExternalSpecPrimitive, false));
        assert_eq!(hollow_alias_gate(&hollow), HollowAliasGateOutcome::Rejected);
    }

    #[test]
    fn accepts_when_not_bare_alias() {
        let ok = decl(site(false, ModeledSubject::ExternalSpecPrimitive, false));
        assert_eq!(hollow_alias_gate(&ok), HollowAliasGateOutcome::Produced);
    }

    #[test]
    fn accepts_bare_external_with_coincidence_evidence() {
        let ok = decl(site(true, ModeledSubject::ExternalSpecPrimitive, true));
        assert_eq!(hollow_alias_gate(&ok), HollowAliasGateOutcome::Produced);
    }

    #[test]
    fn accepts_bare_kernel_ambient_without_evidence() {
        let ok = decl(site(true, ModeledSubject::KernelAmbientAtom, false));
        assert_eq!(hollow_alias_gate(&ok), HollowAliasGateOutcome::Produced);
    }

    #[test]
    fn accepts_bare_internal_std_without_evidence() {
        let ok = decl(site(true, ModeledSubject::InternalStdCarrier, false));
        assert_eq!(hollow_alias_gate(&ok), HollowAliasGateOutcome::Produced);
    }

    #[test]
    fn rejects_nested_hollow_under_group() {
        let root = group(vec![decl(site(
            true,
            ModeledSubject::ExternalSpecPrimitive,
            false,
        ))]);
        assert_eq!(hollow_alias_gate(&root), HollowAliasGateOutcome::Rejected);
    }
}
