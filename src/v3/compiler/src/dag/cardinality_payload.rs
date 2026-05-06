use super::{CardinalityBound, DeclarationId};

/// Payload for [`super::TypeConnective::Cardinality`].
///
/// Construction is restricted to `crate::dag::builder` (`alloc_cardinality_decl`)
/// and `bootstrap_*_generated` modules under `dag`
/// (`CardinalityPayload::new_unchecked_bypassing_idempotence`). Any call-site
/// using the constructor by name is, by name, declaring that it is *not* going
/// through `alloc_cardinality_decl`'s nested-`AtMostOne` idempotence rule —
/// either the caller has already applied the rule (e.g. `alloc_cardinality_decl`
/// itself, `type_connective_cardinality`), or the caller is a generated/test
/// surface intentionally minting a raw payload.
#[derive(Debug, Clone)]
pub struct CardinalityPayload {
    element: DeclarationId,
    bound: CardinalityBound,
}

impl CardinalityPayload {
    /// Bypass-discipline visible at call-site (T-ImpossibleBugs nested-optional
    /// codegen-bypass closure, Path B / Director #828): callers MUST either
    /// have already applied the `cardinality_idempotent_target` rule, or be a
    /// generated/test surface.
    pub(in crate::dag) fn new_unchecked_bypassing_idempotence(
        element: DeclarationId,
        bound: CardinalityBound,
    ) -> Self {
        Self { element, bound }
    }

    pub fn element(&self) -> DeclarationId {
        self.element
    }

    pub fn bound(&self) -> CardinalityBound {
        self.bound
    }
}
