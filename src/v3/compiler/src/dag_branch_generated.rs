// AUTO-GENERATED from `src/v3/std/substrate.dag`.
// Regenerate instead of hand-editing.

#[derive(Debug, Clone)]
pub enum BranchPattern {
    UnresolvedVariant {
        name: String,
        span: SourceSpan,
    },
    ResolvedVariant(DeclarationId),
}

#[derive(Debug, Clone)]
pub struct PayloadBinding {
    pub binding_name: String,
    pub payload_port: PortId,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub body: NodeId,
    pub output: PortId,
    pub pattern: BranchPattern,
    pub binding: Option<PayloadBinding>,
}
