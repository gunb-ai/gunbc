use super::{CardinalityBound, DeclarationId};

/// Payload for [`super::TypeConnective::Cardinality`].
///
/// Construction is restricted to `crate::dag::builder` (`alloc_cardinality_decl`)
/// and `bootstrap_*_generated` modules under `dag` (`CardinalityPayload::new_unchecked`).
#[derive(Debug, Clone)]
pub struct CardinalityPayload {
    element: DeclarationId,
    bound: CardinalityBound,
}

impl CardinalityPayload {
    pub(in crate::dag) fn new_unchecked(element: DeclarationId, bound: CardinalityBound) -> Self {
        Self { element, bound }
    }

    pub fn element(&self) -> DeclarationId {
        self.element
    }

    pub fn bound(&self) -> CardinalityBound {
        self.bound
    }
}
