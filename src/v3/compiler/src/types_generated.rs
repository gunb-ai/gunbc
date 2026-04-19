// AUTO-GENERATED from `src/v3/std/substrate.dag`.
// Regenerate instead of hand-editing.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeShape {
    pub declaration: DeclarationId,
}

impl TypeShape {
    pub fn new(declaration: DeclarationId) -> Self {
        Self { declaration }
    }
}

impl From<DeclarationId> for TypeShape {
    fn from(declaration: DeclarationId) -> Self {
        Self { declaration }
    }
}
