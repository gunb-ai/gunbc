// The v3 substrate: the five L1 behaviors, Ports, and the Dag container.
//
// This module owns the single source of truth for everything flowing
// through the compiler. Other modules (parse, lower, infer, lenses)
// read from here, and write via the narrow mutator API.
//
// Dissolution receipt — M0.3:
//
//   Checkpoint C2 RESOLVED by deletion. TransformRule no longer
//   exists. Transform is now Apply(FunctionRef): a single shape that
//   covers operators, primitive calls, and user function calls
//   uniformly. Operators like `+` resolve to FunctionRef("std::int::add")
//   at parse/lowering; the substrate sees no operator-vs-call
//   distinction. This is the v3-vs-v2 disease prevention — v2's
//   ExprData had 22 variants, v3 has one-variant Transform.
//
//   Checkpoint C2 is now a NEGATIVE guardrail: if you ever add a
//   variant to Transform, re-read feedback_checkpoint_dissolution_default
//   and feedback_coproduct_dissolution before proceeding.
//
//   Define (function definition) is a BIND whose params: Vec<PortId>
//   field is non-empty, NOT a TransformRule variant. The function body
//   is a sub-DAG whose return port is the Bind's value.

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

// A reference to a function — the target of every Transform.
//
// At M0, this is a symbolic path string. Primitive operators resolve
// to stable names like "std::int::add", "std::int::gt". User functions
// resolve to their own names ("count_down"). The primitive signatures
// are hardcoded in infer.rs at M0, migrating to std/ declarations in
// M1+ without changing this shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionRef {
    pub name: String,
}

impl FunctionRef {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
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
//   - Dag::set_port_type       (None -> Some during inference or
//                               from declared type annotations)
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
    pub target: FunctionRef,
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
    /// Parameter ports for function bindings. Empty for value
    /// bindings. A non-empty `params` is the structural distinction
    /// between a function definition and a value definition — no
    /// Optional marker, no coproduct, no type tag. See Checkpoint C2
    /// dissolution receipt at the top of this file.
    pub params: Vec<PortId>,
    pub scope: Option<NodeId>,
}

// Checkpoint C1.
//
// Five L1 behaviors from docs/v3-spec.md lines 22-218. The thesis
// claim is that these are terminal — the irreducible decomposition
// of computation. M0 validates the claim by trying to build against
// it.
//
// Dissolution-patterns check (all four attempted at M0.1):
//
//   - Pattern 1 (fact placement): fails. Every consumer dispatches
//     on "which behavior" first — cost adds for sequence, multiplies
//     for Loop, maxes for Branch. Scattering fields into side tables
//     recreates the dispatch in every lens.
//
//   - Pattern 2 (variant-is-data): fails. Value has `data`, Transform
//     has `inputs + target`, Branch has `paths`, Loop has `bound +
//     source + init + body`, Bind has `value + params`. Structurally
//     different shapes, not one shape with a tag.
//
//   - Pattern 3 (algebraic-form): confirms terminality.
//     Value    = Terminal intro
//     Transform= morphism application (Apply(FunctionRef))
//     Branch   = Coproduct elim
//     Loop     = well-founded recursion
//     Bind     = let-abstraction (with params for function defs)
//     Five different algebras, not one tag over one algebra.
//
//   - Pattern 4 (dimensional): fails. No common M-dim coordinate
//     space generates all five.
//
// STOP SIGNAL: wanting a 6th variant. The terminality assumption
// would be wrong, and the L1 spec needs revisiting. Pause and
// escalate rather than silently extending.
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

/// Declared signature of a user function. Populated during lowering
/// from annotation data, BEFORE the function body is lowered. This
/// breaks inference fixpoint cycles for recursion: when the body's
/// recursive Transform needs to know its own function's return type,
/// it reads this registry instead of re-deriving from the body
/// (which would be circular).
///
/// Primitives (`std::int::add`, etc.) are NOT in this registry — they
/// live in a hardcoded table inside infer.rs until M1 migrates them
/// to std/ declarations.
#[derive(Debug, Clone)]
pub struct Signature {
    pub params: Vec<TypeShape>,
    pub return_type: TypeShape,
}

pub struct Dag {
    nodes: Vec<Behavior>,
    ports: HashMap<PortId, Port>,
    diagnostics: DiagnosticTable,
    signatures: HashMap<String, Signature>,
    next_node_id: u32,
    next_port_id: u32,
}

impl Dag {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            ports: HashMap::new(),
            diagnostics: DiagnosticTable::new(),
            signatures: HashMap::new(),
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

    /// Look up a function definition (Bind with non-empty params)
    /// by name. Returns the BindNode for the most recent matching
    /// binding, or None if no such binding exists.
    pub fn lookup_function(&self, name: &str) -> Option<&BindNode> {
        self.nodes
            .iter()
            .rev()
            .filter_map(Behavior::as_bind)
            .find(|b| b.name == name && !b.params.is_empty())
    }

    /// Look up a declared signature by name. Consulted by inference
    /// when a Transform's target is a user function; cycles through
    /// this registry are how recursion avoids fixpoint traps.
    pub fn signature(&self, name: &str) -> Option<&Signature> {
        self.signatures.get(name)
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

    pub(crate) fn register_signature(&mut self, name: impl Into<String>, sig: Signature) {
        self.signatures.insert(name.into(), sig);
    }

    /// Upgrade a port from None to Some(type). Used by both
    /// inference (computed types) and lowering (declared type
    /// annotations from parse).
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
