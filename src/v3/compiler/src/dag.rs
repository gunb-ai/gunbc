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

use crate::diagnostics::{Diagnostic, DiagnosticTable, SourceSpan};
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
    /// Behaviors in construction order. Load-bearing invariant:
    /// each node in `nodes` comes after all nodes it depends on
    /// (ports it reads). Lowering emits children before parents,
    /// so a forward walk through this vector visits dependencies
    /// before dependents. Inference relies on this for a single
    /// forward pass; any pass that reorders nodes (future
    /// graph-rewriting) must preserve the topological order or
    /// the invariant breaks silently.
    nodes: Vec<Behavior>,
    ports: HashMap<PortId, Port>,
    diagnostics: DiagnosticTable,
    signatures: HashMap<String, Signature>,
    /// Source spans for declared type annotations, keyed by the
    /// port whose type the annotation declared. Populated during
    /// lowering; consulted by inference when a declared type
    /// conflicts with an inferred type so the resulting diagnostic
    /// points back at the user's annotation, not at some derived
    /// location.
    annotation_spans: HashMap<PortId, SourceSpan>,
    /// Source spans for each DAG node, keyed by NodeId. Populated
    /// during lowering from the SurfaceExpr's span so that
    /// decide-level failures in infer (arity mismatch, unknown
    /// function, etc.) can produce diagnostics that point at the
    /// actual call site rather than a synthetic "<inferred>" span.
    /// Fail-closed invariant C-8: every failure gets a real span.
    node_spans: HashMap<NodeId, SourceSpan>,
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
            annotation_spans: HashMap::new(),
            node_spans: HashMap::new(),
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

    /// Record a declared type annotation's span for a port. Used
    /// when inference detects a conflict between the annotation and
    /// the inferred type — the resulting diagnostic points back at
    /// the annotation's source location.
    pub(crate) fn record_annotation_span(&mut self, port: PortId, span: SourceSpan) {
        self.annotation_spans.insert(port, span);
    }

    pub(crate) fn annotation_span(&self, port: PortId) -> Option<&SourceSpan> {
        self.annotation_spans.get(&port)
    }

    /// Record the source span of a DAG node. Called by lowering for
    /// every behavior it creates, so infer can point diagnostics
    /// at the originating expression.
    pub(crate) fn record_node_span(&mut self, node: NodeId, span: SourceSpan) {
        self.node_spans.insert(node, span);
    }

    pub fn node_span(&self, node: NodeId) -> Option<&SourceSpan> {
        self.node_spans.get(&node)
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
    /// exclusively from Dag::mark_unresolved, which atomically also
    /// writes the diagnostic entry. Do not call from anywhere else.
    /// Linked-by-construction invariant:
    ///   port.value_type == None  iff  diagnostics.contains(port_id)
    fn clear_port_type(&mut self, id: PortId) {
        if let Some(port) = self.ports.get_mut(&id) {
            port.value_type = None;
        }
    }

    /// The enforced public API for transitioning a port from a
    /// typed state to Unresolved. Atomically: nulls the port's
    /// value_type AND records the diagnostic entry. There is NO
    /// other way to write `value_type = None` in the crate.
    ///
    /// The invariant holds by construction because:
    ///   1. `clear_port_type` is private to this module.
    ///   2. Only `mark_unresolved` calls it.
    ///   3. `mark_unresolved` always inserts a diagnostic.
    ///
    /// So a `None`-typed port always has a corresponding diagnostic.
    pub(crate) fn mark_unresolved(&mut self, port: PortId, diagnostic: Diagnostic) {
        self.clear_port_type(port);
        self.diagnostics.insert(port, diagnostic);
    }
}

impl Default for Dag {
    fn default() -> Self {
        Self::new()
    }
}
