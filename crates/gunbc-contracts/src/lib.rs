use gunbc_ir::{NodeId, PortName, TypeId};

/// A block contract declares a node's typed I/O interface.
#[derive(Debug, Clone)]
pub struct BlockContract {
    pub id: String,
    pub inputs: Vec<PortContract>,
    pub outputs: Vec<PortContract>,
}

/// A port contract declares a single input or output port.
#[derive(Debug, Clone)]
pub struct PortContract {
    pub name: PortName,
    pub type_id: TypeId,
    pub optional: bool,
    pub guard: Option<String>,
}

/// A pattern contract declares the topology of a SubDAG (e.g., Upsert).
#[derive(Debug, Clone)]
pub struct PatternContract {
    pub name: String,
    pub slots: Vec<SlotContract>,
    pub edges: Vec<EdgeContract>,
    pub export_slot: NodeId,
}

/// A slot in a pattern — references a BlockContract by id.
#[derive(Debug, Clone)]
pub struct SlotContract {
    pub node_id: NodeId,
    pub block_id: String,
}

/// An edge in a pattern contract — connects output of one slot to input of another.
#[derive(Debug, Clone)]
pub struct EdgeContract {
    pub from_node: NodeId,
    pub from_port: PortName,
    pub to_node: NodeId,
    pub to_port: PortName,
}

/// A pattern decision declaration.
#[derive(Debug, Clone)]
pub struct PatternDecisionContract {
    /// The SubDag node this decision applies to.
    pub node: NodeId,
    pub pattern: String,
    pub decision: DecisionContract,
}

#[derive(Debug, Clone)]
pub enum DecisionContract {
    Instantiated,
    NotApplicable { reason: String },
}
