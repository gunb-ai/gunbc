use crate::types::{BehaviorKind, ToolId};

/// Metadata attached to every node.
#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub tool: ToolId,
    pub behavior: BehaviorKind,
}
