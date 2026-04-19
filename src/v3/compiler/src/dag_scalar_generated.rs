// AUTO-GENERATED from `src/v3/std/substrate.dag`.
// Regenerate instead of hand-editing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralBits {
    Int(i64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardinalityBound {
    Exact(u32),
    AtMostOne,
    Unbounded,
}

#[derive(Debug, Clone)]
pub struct TemplateArgument {
    pub parameter: DeclarationId,
    pub value: DeclarationId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortState {
    Uninferred,
    Resolved(TypeShape),
    Unresolved,
}
