// The v3 substrate: five L1 behaviors, Ports, a Declaration table, and the Dag container.
//
// Two coordinated substrates share one container:
//
//   - Computation substrate: five L1 behaviors (Value, Transform, Branch, Loop, Bind)
//     wiring PortIds into a DAG of runtime operations. M0-terminal.
//   - Type substrate: a Declaration table indexed by DeclarationId, each declaration
//     carrying one of six TypeConnective variants (Atom, Conj, Disj, Arrow, Cardinality,
//     Instantiation). M1(2.5)-terminal until the convergence note in M1_DESIGN.md §1
//     reopens substrate extension.
//
// The two substrates reference each other by typed ID, not by name. Transform.target
// is a DeclarationId into the Declaration table; ArrowBody::UserDefined holds a NodeId
// back into the computation substrate. There is no name-based dispatch at the substrate
// layer — operators like `+` resolve to the `add` field of an inhabited algebra
// declaration during inference (via M1_DESIGN §8.9), not at parse time.
//
// Dissolution receipt — M0.3 (UPDATED at M1(2.5)):
//
//   Checkpoint C2 RESOLVED by deletion and renewed. TransformRule no longer exists;
//   Transform is Apply(DeclarationId): one shape that covers operators, primitives,
//   and user functions uniformly. The target is a typed edge into the Declaration
//   table. C2 is a NEGATIVE guardrail: a new Transform variant triggers the C2 stop
//   signal (feedback_checkpoint_dissolution_default, feedback_coproduct_dissolution).
//
//   Define (function definition) is a BIND whose params: Vec<PortId> is non-empty.
//   The body is a sub-DAG whose return port is the Bind's value. No TransformRule
//   variant.
//
// Dissolution receipt — M1(2.5):
//
//   The primitive substrate parallel-representation debt flagged in M0.3
//   (LiteralValue, FunctionRef { name: String }, primitive_signature lookup table,
//   Dag.signatures HashMap) dissolves here. Primitives are now Declarations in the
//   Declaration table, reached by DeclarationId from Transform.target. Operators
//   dispatch via inhabitance walks, not string matching. See M1_DESIGN.md §2, §3,
//   §5, §8.9.
//
// Structural refactor — M0.6 (preserved):
//
//   Port.state is a three-state PortState enum, not Option<TypeShape>. The biconditional
//     state == Unresolved  iff  diagnostics.contains(port.id())
//   holds by construction. Uninferred is the transitional state; the post-infer sweep
//   drives every port to Resolved or Unresolved before compile_to_dag returns.
//
//   SourceSpan lives on every Behavior and every Declaration structurally. Spans flow
//   forward through lowering; no side tables, no reconstruction.

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclarationId(u32);

impl DeclarationId {
    fn index(self) -> usize {
        self.0 as usize
    }

    pub fn raw(self) -> u32 {
        self.0
    }
}

/// A type-system declaration. The unit of the type substrate.
///
/// Every named declaration (primitive, algebra, user type, type alias) lives in
/// `Dag.declarations` under a stable DeclarationId. Anonymous declarations (the
/// inner types of Cardinality bounds, Arrow inputs, etc.) also live here; only the
/// `name` field distinguishes them.
///
/// `meta_tag` and `inhabits` are separate edges with distinct semantics per
/// M1_DESIGN.md §Q0 (and PR #444's refinement):
/// - `meta_tag`: "this Conj's shape is constrained by the linked meta-type
///   declaration." Used for value construction (records, services, transports).
///   Empty across the M1(2.5) bootstrap set.
/// - `inhabits`: "this declaration additionally satisfies the linked algebra's
///   laws." Used for secondary algebra inhabitance on declarations with their
///   own primary structure. Empty across the M1(2.5) bootstrap set.
#[derive(Debug, Clone)]
pub struct Declaration {
    pub id: DeclarationId,
    pub name: Option<String>,
    pub connective: TypeConnective,
    pub meta_tag: Option<DeclarationId>,
    pub inhabits: Option<DeclarationId>,
    pub span: SourceSpan,
}

/// The six-variant type substrate. Terminal at M1(2.5). Extension requires the
/// C1-class stop signal in M1_DESIGN.md §8.10 / THESIS.md §"Substrate extension"
/// — all four dissolution patterns must be attempted before adding a 7th variant.
#[derive(Debug, Clone)]
pub enum TypeConnective {
    /// Irreducible leaf. See AtomPayload.
    Atom(AtomPayload),
    /// Labeled product — logical AND. All children present together.
    Conj { children: Vec<Field> },
    /// Labeled coproduct — logical OR. Exactly one variant active.
    Disj { variants: Vec<Field> },
    /// Function type — directional flow from inputs to an output. `body` covers
    /// user sub-DAGs, extdeps-declared realizations, and the Pending bootstrap
    /// state. See M1_DESIGN.md §Q7.
    Arrow {
        inputs: Vec<DeclarationId>,
        output: DeclarationId,
        body: ArrowBody,
    },
    /// Repetition over an element type with a bound. Unifies v2's Required/Optional
    /// with list cardinality.
    Cardinality {
        element: DeclarationId,
        bound: CardinalityBound,
    },
    /// Specialization of a parameterized template with concrete template arguments.
    /// For pure aliases like `Int = OrderedRing<Word64>`, Int's connective IS
    /// Instantiation directly — inhabitance collapses into this form. See
    /// M1_DESIGN.md §Q0, §Q1.
    Instantiation {
        template: DeclarationId,
        arguments: Vec<TemplateArgument>,
    },
}

#[derive(Debug, Clone)]
pub struct Field {
    pub label: String,
    pub ty: DeclarationId,
}

#[derive(Debug, Clone)]
pub enum AtomPayload {
    /// A literal bit pattern carried at the type level (e.g., the 3 in
    /// `Cardinality::Exact(3)`, a default value in a type declaration).
    /// Computation-side literals live in ValueNode.data, not here.
    Literal(LiteralBits),
    /// An identifier reference. `resolved` is None after parse/lower pass 1 and
    /// Some(_) after pass 2. Stored as a nullable slot rather than a variant
    /// split so the bit flag is structural, not semantic.
    Identifier {
        name: String,
        resolved: Option<DeclarationId>,
    },
    /// A type parameter declaration slot. Appears as a child of a parameterized
    /// declaration; referenced from inside the body by Identifier atoms that
    /// resolve to this slot's DeclarationId. Per M1_DESIGN.md §8.3, a single
    /// TypeParam Atom is shared across all references in the body — the
    /// substrate is a DAG of declarations, not a tree.
    TypeParam(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralBits {
    Int(i64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardinalityBound {
    /// Exact count. Required = Exact(1); fixed-size arrays = Exact(n); argv
    /// literals = Exact(3).
    Exact(u32),
    /// Zero or one. Distinct from Unbounded so that `T?` and `List<T>` are
    /// structurally unrepresentable as the same thing.
    AtMostOne,
    /// Zero or more. `List<T>` and friends.
    Unbounded,
}

#[derive(Debug, Clone)]
pub struct TemplateArgument {
    /// The template parameter being bound. References a TypeParam Atom declared
    /// as a child of the template.
    pub parameter: DeclarationId,
    /// The concrete type the parameter binds to.
    pub value: DeclarationId,
}

#[derive(Debug, Clone)]
pub enum ArrowBody {
    /// User-defined function. NodeId is the root of a sub-DAG of L1 behavior
    /// nodes in `Dag.nodes`. Inference walks the sub-DAG and checks the body
    /// against the declared inputs/output.
    UserDefined(NodeId),
    /// Primitive whose realization is declared in an extdeps language spec.
    /// DeclarationId points at the realization declaration via a typed edge;
    /// inference verifies signature compatibility.
    ExternalRealization(DeclarationId),
    /// Bootstrap state. Signature type-checks via inhabitance; body-walking is
    /// skipped. Scaffolded at M1(2.5) — the CI ratchet in M1_DESIGN.md §8.10
    /// (deferred to M1(3)) enforces Pending → UserDefined | ExternalRealization
    /// before M3.
    Pending,
}

/// Three-state port type. Illegal combinations of "has a type" and "has a
/// diagnostic" are unrepresentable by type:
///
///   - `Uninferred`: port exists but inference has not run on it yet.
///     Transitional state during DAG construction and fixpoint iteration. The
///     post-infer sweep drives every port to Resolved or Unresolved before
///     compile_to_dag returns.
///   - `Resolved(TypeShape)`: inference (or lowering from a declaration) has
///     committed to a type.
///   - `Unresolved`: inference or lowering detected a failure and called
///     `Dag::mark_unresolved`. A diagnostic exists in the DiagnosticTable
///     keyed by this port's id.
///
/// Biconditional (checked by the invariant audit test):
///   state == Unresolved  iff  diagnostics.contains(port.id())
///
/// **Dissolution receipt: TERMINAL.** PortState is substrate, not an annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortState {
    Uninferred,
    Resolved(TypeShape),
    Unresolved,
}

/// A Port carries a typed value forward in time.
///
/// `state` has a single authoritative location: the Port struct stored in
/// Dag.ports. There are no stale copies — behaviors hold PortId references, not
/// embedded Ports.
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

    /// Backward-compat accessor: returns `Some(&TypeShape)` for Resolved ports,
    /// `None` for Uninferred or Unresolved. Prefer `state()` when you need to
    /// distinguish the three cases.
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
    pub data: LiteralBits,
    pub output: PortId,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct TransformNode {
    pub id: NodeId,
    /// The declaration this transform invokes. Resolved to a DeclarationId at
    /// lowering time via two-pass identifier resolution (M1_DESIGN.md §8.1).
    /// Inference walks `dag.declaration(target)` to recover the Arrow signature.
    pub target: DeclarationId,
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
    /// Parameter ports for function bindings. Empty for value bindings. A
    /// non-empty `params` is the structural distinction between a function
    /// definition and a value definition — no Optional marker, no coproduct,
    /// no type tag. See the C2 dissolution receipt at the top of this file.
    pub params: Vec<PortId>,
    pub span: SourceSpan,
}

// Checkpoint C1.
//
// Five L1 behaviors from docs/v3-spec.md lines 22-218. The thesis claim is that
// these are terminal — the irreducible decomposition of computation. M0
// validated the claim; M1(2.5) did not add a 6th variant.
//
// Dissolution-patterns check (all four attempted at M0.1, preserved at M1(2.5)):
//
//   - Pattern 1 (fact placement): fails. Every consumer dispatches on behavior
//     first — cost adds for sequence, multiplies for Loop, maxes for Branch.
//   - Pattern 2 (variant-is-data): fails. Value has `data`, Transform has
//     `inputs + target`, Branch has `paths`, Loop has `bound + source + init +
//     body`, Bind has `value + params`. Structurally different, not one shape
//     with a tag.
//   - Pattern 3 (algebraic-form): confirms terminality.
//       Value    = Terminal intro
//       Transform= morphism application (Apply(DeclarationId))
//       Branch   = Coproduct elim
//       Loop     = well-founded recursion
//       Bind     = let-abstraction (with params for function defs)
//   - Pattern 4 (dimensional): fails. No common M-dim coordinate space generates
//     all five.
//
// STOP SIGNAL: wanting a 6th variant. Pause and escalate rather than silently
// extending.
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

#[derive(Debug)]
pub struct Dag {
    /// Behaviors in construction order. NodeId(k) lives at `nodes[k]`; a forward
    /// walk visits dependencies before dependents (load-bearing for inference).
    nodes: Vec<Behavior>,
    /// Declarations in allocation order. DeclarationId(k) lives at
    /// `declarations[k]` by the same invariant.
    declarations: Vec<Declaration>,
    ports: HashMap<PortId, Port>,
    diagnostics: DiagnosticTable,
    next_node_id: u32,
    next_declaration_id: u32,
    next_port_id: u32,
}

impl Dag {
    pub fn new() -> Self {
        let mut dag = Self {
            nodes: Vec::new(),
            declarations: Vec::new(),
            ports: HashMap::new(),
            diagnostics: DiagnosticTable::new(),
            next_node_id: 0,
            next_declaration_id: 0,
            next_port_id: 0,
        };
        crate::bootstrap::bootstrap(&mut dag);
        dag
    }

    pub fn nodes(&self) -> &[Behavior] {
        &self.nodes
    }

    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    /// O(1) lookup by NodeId. Relies on the dense-sequential allocation invariant.
    pub fn node(&self, id: NodeId) -> &Behavior {
        let node = &self.nodes[id.index()];
        debug_assert_eq!(
            node.id(),
            id,
            "Dag::node: NodeId desync — topological invariant broken"
        );
        node
    }

    /// O(1) lookup by DeclarationId. Same dense-sequential invariant as nodes.
    pub fn declaration(&self, id: DeclarationId) -> &Declaration {
        let decl = &self.declarations[id.index()];
        debug_assert_eq!(
            decl.id, id,
            "Dag::declaration: DeclarationId desync — allocation invariant broken"
        );
        decl
    }

    /// Find a top-level declaration by name. Used by the two-pass lowering symbol
    /// table and by inference when walking named algebra fields. Linear scan
    /// (declaration counts at M1(2.5) are ~50); becomes a HashMap if hot.
    pub fn declaration_by_name(&self, name: &str) -> Option<&Declaration> {
        self.declarations
            .iter()
            .find(|d| d.name.as_deref() == Some(name))
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

    pub(crate) fn alloc_declaration_id(&mut self) -> DeclarationId {
        let id = DeclarationId(self.next_declaration_id);
        self.next_declaration_id += 1;
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

    pub(crate) fn push_declaration(&mut self, declaration: Declaration) {
        debug_assert_eq!(
            declaration.id.index(),
            self.declarations.len(),
            "push_declaration out of sequence — allocation invariant broken"
        );
        self.declarations.push(declaration);
    }

    /// Mutable access to a declaration, scoped to lowering's second pass where
    /// Identifier atoms get their `resolved: Some(_)` slot filled in. No
    /// external mutator; resolution is an internal phase of lowering.
    pub(crate) fn declaration_mut(&mut self, id: DeclarationId) -> &mut Declaration {
        &mut self.declarations[id.index()]
    }

    /// Transition a port from Uninferred to Resolved. Idempotent when the new
    /// type matches the existing Resolved type. Skips Unresolved ports — once
    /// Unresolved, stays so (otherwise the biconditional breaks).
    pub(crate) fn set_port_type(&mut self, id: PortId, ty: TypeShape) {
        if let Some(port) = self.ports.get_mut(&id) {
            if matches!(port.state, PortState::Unresolved) {
                return;
            }
            port.state = PortState::Resolved(ty);
        }
    }

    /// Enforced public API for transitioning a port to Unresolved. Atomically:
    /// marks the port AND records the diagnostic. There is NO other way to
    /// construct an Unresolved port state.
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
