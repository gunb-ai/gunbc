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
//
// Structural refactor — M0.6 (review response):
//
//   Port.value_type: Option<TypeShape> was a state-space bug. It
//   conflated three states (Uninferred, Resolved, Unresolved) into a
//   two-valued type, so the invariant
//     value_type == None  iff  diagnostics.contains(port.id())
//   was a runtime biconditional maintained by API convention, not a
//   structural guarantee. An Uninferred port looked like an Unresolved
//   port (both were None) with no diagnostic, which is exactly the
//   illegal state the invariant forbids.
//
//   Fix: Port.state is now a three-state PortState enum. The illegal
//   states are unrepresentable — Uninferred, Resolved(T), Unresolved
//   are mutually exclusive by type. The biconditional becomes:
//     state == Unresolved  iff  diagnostics.contains(port.id())
//   and Uninferred is a transitional construction state the post-infer
//   sweep drives to Resolved or Unresolved before compile_to_dag
//   returns.
//
//   Same commit: SourceSpan now lives on every Behavior variant
//   structurally. The node_spans side table is deleted — spans are
//   facts that flow forward through lowering into the DAG, not
//   metadata to be looked up externally. Same shape as v2's provenance
//   reconstruction bug (drop the fact, reconstruct later); we fix it
//   by carrying the fact forward.

use std::collections::HashMap;

use crate::diagnostics::{Diagnostic, DiagnosticTable, SourceSpan};
use crate::types::TypeShape;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl NodeId {
    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortId(u32);

/// Literal value carried by a Value node. One variant per primitive
/// type the substrate understands.
///
/// **Dissolution receipt: DEFERRED — dissolves with the primitive
/// substrate refactor (first M1 task).** LiteralValue is one of the
/// three parallel-representation scaffolds flagged in the M0
/// retrospective (see `src/v3/M0_RETROSPECTIVE.md` §"The primitive
/// substrate gap"). The right shape is `{ declaration: DeclarationId,
/// data: LiteralData }` where the declaration points at the
/// primitive type's entry in the Dag's Declaration table and the
/// data is a carrier for the concrete value. Until that refactor
/// lands, this enum is parallel to `Prim` / `TypeShape::Primitive`
/// and dissolves at the same time.
///
/// Trigger: when the Declaration table exists on Dag and primitive
/// types are registered there. See ROADMAP.md §M1 steps (1)–(2).
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

/// Three-state port type. Illegal combinations of "has a type" and
/// "has a diagnostic" are unrepresentable by type:
///
///   - `Uninferred`: port exists but inference has not run on it yet.
///     Transitional state during DAG construction and fixpoint
///     iteration. The post-infer sweep drives every port to Resolved
///     or Unresolved before compile_to_dag returns.
///   - `Resolved(TypeShape)`: inference (or lowering from a
///     declaration) has committed to a type. No diagnostic.
///   - `Unresolved`: inference or lowering detected a failure and
///     called `Dag::mark_unresolved`. A diagnostic exists in the
///     DiagnosticTable keyed by this port's id.
///
/// Biconditional (checked by the invariant audit test):
///   state == Unresolved  iff  diagnostics.contains(port.id())
///
/// **Dissolution receipt: TERMINAL.** PortState is substrate, not an
/// annotation. The three states are mutually exclusive structural
/// states of a port — collapsing them into a flatter form (e.g.,
/// `Option<TypeShape>` plus convention, which is what M0.1–M0.5 had)
/// would make the biconditional behavioral instead of structural,
/// which is precisely what M0.6's refactor moved away from. Adding
/// a 4th variant would require a structural reason — e.g., a genuine
/// new state a port can occupy — not a convenience for consumers.
/// Per INVARIANTS.md "No annotation mechanisms at any layer," this
/// enum is NOT an attribute system; its variants are load-bearing
/// for the fail-closed (C-8) invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortState {
    Uninferred,
    Resolved(TypeShape),
    Unresolved,
}

/// A Port carries a typed value forward in time.
///
/// `state` has a single authoritative location: the Port struct
/// stored in Dag.ports. There are no stale copies — behaviors hold
/// PortId references, not embedded Ports.
///
/// `state` is module-private. The only code paths that can write it
/// are `Dag::alloc_port` (births a port as `Uninferred`),
/// `Dag::set_port_type` (`Uninferred` → `Resolved` during inference
/// or from declared annotations), and `Dag::mark_unresolved` (any →
/// `Unresolved`, atomically with a diagnostic — the only path that
/// ever sets `Unresolved`). No public mutator exists. Guardrail G5.
#[derive(Debug, Clone)]
pub struct Port {
    id: PortId,
    pub(super) state: PortState,
    pub produced_by: Option<NodeId>,
}

impl Port {
    pub fn id(&self) -> PortId {
        self.id
    }

    pub fn state(&self) -> &PortState {
        &self.state
    }

    /// Backward-compat accessor: returns `Some(&TypeShape)` for
    /// Resolved ports, `None` for Uninferred or Unresolved. Prefer
    /// `state()` when you need to distinguish the three cases.
    pub fn value_type(&self) -> Option<&TypeShape> {
        match &self.state {
            PortState::Resolved(ty) => Some(ty),
            PortState::Uninferred | PortState::Unresolved => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValueNode {
    pub id: NodeId,
    pub data: LiteralValue,
    pub output: PortId,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct TransformNode {
    pub id: NodeId,
    pub target: FunctionRef,
    pub inputs: Vec<PortId>,
    pub output: PortId,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct BranchNode {
    pub id: NodeId,
    pub input: PortId,
    pub paths: Vec<Path>,
    pub output: PortId,
    pub span: SourceSpan,
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
    pub span: SourceSpan,
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
    pub span: SourceSpan,
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

    /// Source span for the expression that produced this node.
    /// Every behavior carries its own span structurally — no
    /// side table, no reconstruction.
    pub fn span(&self) -> &SourceSpan {
        match self {
            Behavior::Value(v) => &v.span,
            Behavior::Transform(t) => &t.span,
            Behavior::Branch(b) => &b.span,
            Behavior::Loop(l) => &l.span,
            Behavior::Bind(b) => &b.span,
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

#[derive(Debug)]
pub struct Dag {
    /// Behaviors in construction order. Load-bearing invariant:
    /// each node in `nodes` comes after all nodes it depends on
    /// (ports it reads). Lowering emits children before parents,
    /// so a forward walk through this vector visits dependencies
    /// before dependents. Inference relies on this for a single
    /// forward pass; any pass that reorders nodes (future
    /// graph-rewriting) must preserve the topological order or
    /// the invariant breaks silently.
    ///
    /// Additionally: NodeId(k) lives at `nodes[k]` because every
    /// alloc_node_id is immediately followed by push_node with the
    /// same id. `Dag::node` relies on this for O(1) lookup.
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

    /// O(1) lookup by NodeId. Relies on the dense-sequential
    /// allocation invariant documented on the `nodes` field.
    pub fn node(&self, id: NodeId) -> &Behavior {
        let node = &self.nodes[id.index()];
        debug_assert_eq!(
            node.id(),
            id,
            "Dag::node: NodeId desync — topological invariant broken"
        );
        node
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
                state: PortState::Uninferred,
                produced_by,
            },
        );
        id
    }

    pub(crate) fn push_node(&mut self, behavior: Behavior) {
        debug_assert_eq!(
            behavior.id().index(),
            self.nodes.len(),
            "push_node out of sequence — topological invariant broken"
        );
        self.nodes.push(behavior);
    }

    pub(crate) fn register_signature(&mut self, name: impl Into<String>, sig: Signature) {
        self.signatures.insert(name.into(), sig);
    }

    /// Transition a port from Uninferred to Resolved. Idempotent
    /// when the new type matches the existing Resolved type. Skips
    /// Unresolved ports — once a port is Unresolved, it stays so
    /// (otherwise the biconditional would break when inference
    /// cleared an earlier diagnostic by setting a new type).
    pub(crate) fn set_port_type(&mut self, id: PortId, ty: TypeShape) {
        if let Some(port) = self.ports.get_mut(&id) {
            if matches!(port.state, PortState::Unresolved) {
                return;
            }
            port.state = PortState::Resolved(ty);
        }
    }

    /// The enforced public API for transitioning a port to
    /// Unresolved. Atomically: marks the port AND records the
    /// diagnostic entry. There is NO other way to construct an
    /// Unresolved port state.
    ///
    /// Biconditional invariant, held by construction:
    ///   port.state == Unresolved  iff  diagnostics.contains(port_id)
    pub(crate) fn mark_unresolved(&mut self, port: PortId, diagnostic: Diagnostic) {
        if let Some(p) = self.ports.get_mut(&port) {
            p.state = PortState::Unresolved;
        }
        self.diagnostics.insert(port, diagnostic);
    }
}

impl Default for Dag {
    fn default() -> Self {
        Self::new()
    }
}
