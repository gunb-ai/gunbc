// The v3 substrate: the five L1 behaviors, Ports, and the Dag container.
//
// This module owns the single source of truth for everything flowing
// through the compiler. Other modules (parse, lower, infer, lenses)
// read from here, and write via the narrow mutator API.

use std::collections::HashMap;

use crate::diagnostics::DiagnosticTable;
use crate::types::TypeShape;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortId(u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralValue {
    Int(i64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

// Checkpoint C2.
//
// Started at 1 variant for M0.1. Dissolution target per
// docs/v3-modeling-analysis.md §TransformRule is
// { AlgebraRef, IntroElim } — the structure should be read from
// std/ algebra declarations instead of a Rust enum. That requires
// std/ to declare intro/elim forms, which is M1 work.
//
// STOP SIGNAL: adding any variant to this enum. At that moment,
// pause and ask: (a) is std/ ready to dissolve, or (b) does the
// scaffold extend by one? Neither answer is wrong; making the
// decision is what matters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformRule {
    BinaryOp(BinOp),
}

// A Port carries a typed value forward in time.
//
// value_type has a single authoritative location: the Port struct
// stored in Dag.ports. There are no stale copies — behaviors hold
// PortId references, not embedded Ports.
//
// The value_type field is module-private. The only code paths that
// can write it are:
//   - Dag::alloc_port          (births a port with None)
//   - Dag::set_port_type       (None -> Some during inference)
//   - Dag::clear_port_type     (Some -> None, called ONLY from
//                               DiagnosticTable::mark_unresolved,
//                               which atomically writes a diagnostic)
// No public mutator exists. Guardrail G5.
#[derive(Debug, Clone)]
pub struct Port {
    id: PortId,
    pub(super) value_type: Option<TypeShape>,
    pub produced_by: Option<NodeId>,
}

impl Port {
    pub fn id(&self) -> PortId {
        self.id
    }

    pub fn value_type(&self) -> Option<&TypeShape> {
        self.value_type.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct ValueNode {
    pub id: NodeId,
    pub data: LiteralValue,
    pub output: PortId,
}

#[derive(Debug, Clone)]
pub struct TransformNode {
    pub id: NodeId,
    pub rule: TransformRule,
    pub inputs: Vec<PortId>,
    pub output: PortId,
}

#[derive(Debug, Clone)]
pub struct BranchNode {
    pub id: NodeId,
    pub input: PortId,
    pub paths: Vec<Path>,
    pub output: PortId,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub body: NodeId,
    pub output: PortId,
}

#[derive(Debug, Clone)]
pub struct LoopNode {
    pub id: NodeId,
    pub source: PortId,
    pub init: PortId,
    pub body: NodeId,
    pub bound: Bound,
    pub output: PortId,
}

#[derive(Debug, Clone)]
pub struct Bound {
    pub count: PortId,
}

#[derive(Debug, Clone)]
pub struct BindNode {
    pub id: NodeId,
    pub name: String,
    pub value: PortId,
    pub scope: Option<NodeId>,
}

// Checkpoint C1.
//
// Five L1 behaviors from docs/v3-spec.md lines 22-218. The thesis
// claim is that these are terminal — the irreducible decomposition
// of computation. M0 validates the claim by trying to build against
// it.
//
// Dissolution-patterns check (all four attempted):
//
//   - Pattern 1 (fact placement): fails. Every consumer dispatches
//     on "which behavior" first — cost adds for sequence, multiplies
//     for Loop, maxes for Branch (per spec). Scattering fields into
//     side tables would recreate the dispatch in every lens.
//
//   - Pattern 2 (variant-is-data): fails. Value has `data`,
//     Transform has `inputs + rule`, Branch has `paths`, Loop has
//     `bound + source + init + body`, Bind has no output. These are
//     structurally different shapes, not one shape with a tag.
//
//   - Pattern 3 (algebraic-form): confirms terminality.
//     Value    = Terminal intro
//     Transform= morphism application
//     Branch   = Coproduct elim
//     Loop     = well-founded recursion
//     Bind     = let-abstraction
//     Five different algebras, not one tag over one algebra. The
//     compression is BY algebra kind, not a coordinate.
//
//   - Pattern 4 (dimensional): fails. No common M-dim coordinate
//     space generates all five.
//
// STOP SIGNAL: wanting a 6th variant. If that happens, the
// terminality assumption is wrong and the L1 spec needs revisiting.
// Pause and escalate rather than silently extending.
#[derive(Debug, Clone)]
pub enum Behavior {
    Value(ValueNode),
    Transform(TransformNode),
    Branch(BranchNode),
    Loop(LoopNode),
    Bind(BindNode),
}

impl Behavior {
    pub fn id(&self) -> NodeId {
        match self {
            Behavior::Value(v) => v.id,
            Behavior::Transform(t) => t.id,
            Behavior::Branch(b) => b.id,
            Behavior::Loop(l) => l.id,
            Behavior::Bind(b) => b.id,
        }
    }

    pub fn as_value(&self) -> Option<&ValueNode> {
        if let Behavior::Value(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_transform(&self) -> Option<&TransformNode> {
        if let Behavior::Transform(t) = self {
            Some(t)
        } else {
            None
        }
    }

    pub fn as_branch(&self) -> Option<&BranchNode> {
        if let Behavior::Branch(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_loop(&self) -> Option<&LoopNode> {
        if let Behavior::Loop(l) = self {
            Some(l)
        } else {
            None
        }
    }

    pub fn as_bind(&self) -> Option<&BindNode> {
        if let Behavior::Bind(b) = self {
            Some(b)
        } else {
            None
        }
    }
}

pub struct Dag {
    nodes: Vec<Behavior>,
    ports: HashMap<PortId, Port>,
    diagnostics: DiagnosticTable,
    next_node_id: u32,
    next_port_id: u32,
}

impl Dag {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            ports: HashMap::new(),
            diagnostics: DiagnosticTable::new(),
            next_node_id: 0,
            next_port_id: 0,
        }
    }

    pub fn nodes(&self) -> &[Behavior] {
        &self.nodes
    }

    pub fn node(&self, id: NodeId) -> &Behavior {
        self.nodes
            .iter()
            .find(|n| n.id() == id)
            .expect("NodeId not in dag")
    }

    pub fn port(&self, id: PortId) -> &Port {
        self.ports.get(&id).expect("PortId not in dag")
    }

    pub fn all_ports(&self) -> impl Iterator<Item = &Port> {
        self.ports.values()
    }

    pub fn diagnostics(&self) -> &DiagnosticTable {
        &self.diagnostics
    }

    // --- crate-private mutators: construction + inference ---

    pub(crate) fn alloc_node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    pub(crate) fn alloc_port(&mut self, produced_by: Option<NodeId>) -> PortId {
        let id = PortId(self.next_port_id);
        self.next_port_id += 1;
        self.ports.insert(
            id,
            Port {
                id,
                value_type: None,
                produced_by,
            },
        );
        id
    }

    pub(crate) fn push_node(&mut self, behavior: Behavior) {
        self.nodes.push(behavior);
    }

    /// Inference: upgrade a port from None to Some(type).
    /// Never sets None. Guardrail: the None path is clear_port_type,
    /// which is only called from DiagnosticTable::mark_unresolved.
    pub(crate) fn set_port_type(&mut self, id: PortId, ty: TypeShape) {
        if let Some(port) = self.ports.get_mut(&id) {
            port.value_type = Some(ty);
        }
    }

    /// The ONLY code path that sets value_type to None. Called
    /// exclusively from DiagnosticTable::mark_unresolved, which
    /// atomically writes the diagnostic entry. Do not call from
    /// anywhere else. Linked-by-construction invariant:
    ///   port.value_type == None  iff  diagnostics.contains(port_id)
    pub(crate) fn clear_port_type(&mut self, id: PortId) {
        if let Some(port) = self.ports.get_mut(&id) {
            port.value_type = None;
        }
    }
}

impl Default for Dag {
    fn default() -> Self {
        Self::new()
    }
}
