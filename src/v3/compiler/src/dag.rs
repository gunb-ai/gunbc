// The v3 substrate: the five L1 behaviors, Ports, the Declaration
// table, and the Dag container.
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
//   uniformly. Operators like `+` resolve to a FunctionRef pointing
//   at the std::int::add declaration at parse/lowering; the
//   substrate sees no operator-vs-call distinction. This is the
//   v3-vs-v2 disease prevention — v2's ExprData had 22 variants,
//   v3 has one-variant Transform.
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
//
// Primitive substrate restructuring — M1 task (1):
//
//   M0 scaffolded primitive types and operator signatures as parallel
//   Rust representations (`enum Prim`, `FunctionRef { name: String }`,
//   hardcoded `primitive_signature()` match table). Those were the
//   three `parallel-representation debt` items flagged in
//   `src/v3/M0_RETROSPECTIVE.md` §"The primitive substrate gap" —
//   honestly labelled scaffolds that nonetheless violated the
//   single-authority invariant because the canonical source for
//   "what is Int" and "what is add" lives in std/, not in the
//   compiler's Rust.
//
//   M1 task (1) replaces them with a unified Declaration table on
//   the Dag. Every named thing (types, functions, operations,
//   eventually algebras and effects) is a `Declaration` with a
//   `name` and a `DeclKind`. References are `DeclarationId`s wrapped
//   in the typed newtypes `TypeShape` and `FunctionRef`. Primitive
//   types and operators are pre-populated at `Dag::new` via the
//   `bootstrap_primitives` method. The SHAPE of the table does not
//   change between M1 task (1) and M1 task (2) — only the source of
//   the Declarations does (M1 task (2) replaces the bootstrap body
//   with a std/ parse pass).
//
//   LiteralValue dissolved in the same refactor: the M0 enum
//   `{ Int | Bool | String }` became `Literal { ty: TypeShape,
//   data: LiteralData }`, where `ty` is a typed reference to the
//   primitive's declaration and `data` is the raw carrier. Consumers
//   no longer dispatch on LiteralData variants to derive the type —
//   they read `literal.ty` directly.

use std::collections::HashMap;

use crate::diagnostics::{Diagnostic, DiagnosticTable, SourceSpan};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl NodeId {
    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortId(u32);

/// Stable index into the Dag's declaration table. Every named thing
/// (types, functions, operations, eventually algebras and effects)
/// is a [`Declaration`]; references to declarations are typed via
/// the newtypes [`TypeShape`] and [`FunctionRef`].
///
/// **Dissolution receipt: TERMINAL.** DeclarationId is the dissolution
/// target for both the M0 `Prim` enum and `FunctionRef { name: String }`
/// — it's what happens when you stop parallel-representing primitive
/// types and start consuming the canonical source. Index-based
/// references enable O(1) lookup, structural equality checks on
/// types, and uniform handling of primitive vs user-declared entities.
/// Collapsing further would require dissolving the declaration
/// *table*, which is exactly the substrate — not something to
/// dissolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclarationId(u32);

impl DeclarationId {
    fn index(self) -> usize {
        self.0 as usize
    }
}

/// A reference to a type declaration in the Dag's declaration table.
///
/// **Dissolution receipt — M1 task (1): REPLACES** the M0 enum
/// `TypeShape::Primitive(Prim)`. The M0 form was a single-variant
/// scaffold parallel-representing `Int`, `Bool`, `String` in Rust,
/// which violated single-authority (canonical source is std/).
/// TypeShape is now a newtype around [`DeclarationId`], pointing at
/// the real declaration in the Dag's table. Equality is O(1) integer
/// compare; all primitive types are first-class table entries.
///
/// STOP SIGNAL: if structural types (Product, Coproduct, Function)
/// arrive and TypeShape needs internal structure, pause and decide
/// whether:
///   (a) the structure belongs on the [`Declaration`] itself
///       (DeclKind::Type grows children), keeping TypeShape a pure
///       reference; or
///   (b) TypeShape grows an enum form with a Compound variant
///       carrying nested TypeShapes.
/// Option (a) is the preferred default — it keeps the "everything
/// named is in one table" property. Option (b) is the pattern to
/// resist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeShape(DeclarationId);

impl TypeShape {
    pub fn new(id: DeclarationId) -> Self {
        Self(id)
    }

    pub fn declaration(self) -> DeclarationId {
        self.0
    }
}

/// A reference to a function declaration in the Dag's declaration table.
///
/// **Dissolution receipt — M1 task (1): REPLACES** the M0 struct
/// `FunctionRef { name: String }`. The M0 form was name-based
/// dispatch (the pattern the modeling discipline explicitly rejects
/// — see `feedback_naming_is_aliasing.md`). FunctionRef is now a
/// newtype around [`DeclarationId`], so the compiler never resolves
/// a function by string lookup on the hot path. Operators resolve
/// to declarations at parse/lowering; user functions register
/// themselves during lowering; both end up as table entries
/// uniformly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionRef(DeclarationId);

impl FunctionRef {
    pub fn new(id: DeclarationId) -> Self {
        Self(id)
    }

    pub fn declaration(self) -> DeclarationId {
        self.0
    }
}

/// A literal value carried by a [`ValueNode`]: a typed reference to
/// the primitive type's declaration plus the raw carrier data.
///
/// **Dissolution receipt — M1 task (1): REPLACES** the M0 enum
/// `LiteralValue { Int(i64) | Bool(bool) | String(String) }`. The
/// M0 form was a flat enum parallel to `Prim` / `TypeShape::Primitive`
/// — consumers had to pattern-match `LiteralValue::Int(_) =>
/// Prim::Int` to derive the type, which is exactly the "re-derive
/// a fact that was already established upstream" disease v3 is
/// supposed to cure.
///
/// The new shape splits "what is the type" (`ty: TypeShape`,
/// pointing at the type's declaration) from "what is the carrier"
/// (`data: LiteralData`). Inference reads `literal.ty` directly;
/// no dispatch on LiteralData variants.
///
/// `LiteralData` still has one variant per primitive storage at
/// this stage (Int/Bool/String have genuinely different Rust
/// representations). Future work that adds compound literals
/// (records, tuples, containers) may grow the carrier; the key
/// point is that the variants are a storage concern, not a type
/// tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    pub ty: TypeShape,
    pub data: LiteralData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralData {
    Int(i64),
    Bool(bool),
    String(String),
}

/// Kinds of declaration. Each kind represents a distinct namespace
/// of named entity the compiler understands.
///
/// **Dissolution receipt: DEFERRED.** Two variants at M1 task (1):
/// `Type` (primitive atomic types, no internal structure at this
/// stage) and `Function` (parameter + return signatures plus an
/// optional body port for user functions). The dissolution target
/// is "Type and Function collapse into a single shape once std/
/// declarations give types real structural children" — i.e., both
/// become `Declaration { name, children, algebra, ... }`. That
/// requires Product/Coproduct/Function as first-class type
/// declarations, which is M1 task (2+) work.
///
/// Extension-by-one-variant (adding Algebra, Effect, Transport, ...)
/// is the deferred dissolution trigger: if a 4th variant is being
/// added without a deeper shape-collapse, pause and reassess. The
/// purpose of Declaration is to be the single shape for all named
/// entities; a proliferation of variants means the shape is wrong.
#[derive(Debug, Clone)]
pub enum DeclKind {
    /// A named type. At M1 task (1) this is purely nominal — the
    /// declaration's name distinguishes Int from Bool from String.
    /// Structural children (Product/Coproduct/Function) arrive with
    /// std/ parsing in M1 task (2+).
    Type,
    /// A function declaration — parameter types, return type, and
    /// an optional body port.
    ///
    /// `body_port: None` means the declaration is a primitive or
    /// extern function: no body to type-check, the declared
    /// signature is trusted unconditionally.
    ///
    /// `body_port: Some(port)` means the declaration is a user
    /// function whose body lowered to a sub-DAG. Call sites consult
    /// `dag.port(port).state()` to decide whether the declared
    /// signature is trustworthy — an Unresolved body_port means the
    /// body conflicted with the declared signature (caught at
    /// Loop/output reconciliation in infer) and the declared
    /// signature is no longer honest. This is the
    /// producer-fact-reaches-consumer path from M0.10.
    Function {
        params: Vec<TypeShape>,
        return_type: TypeShape,
        body_port: Option<PortId>,
    },
}

/// The single structural record for everything named in the
/// compiler. See [`DeclKind`] for the kinds.
#[derive(Debug, Clone)]
pub struct Declaration {
    pub name: String,
    pub kind: DeclKind,
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
    pub literal: Literal,
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
//   - Pattern 2 (variant-is-data): fails. Value has `literal`,
//     Transform has `inputs + target`, Branch has `paths`, Loop has
//     `bound + source + init + body`, Bind has `value + params`.
//     Structurally different shapes, not one shape with a tag.
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
    /// Declaration table. Every named entity (types, functions,
    /// operators, eventually algebras and effects) lives here.
    /// DeclarationId is a stable index; declarations are never
    /// reordered. See `register_declaration`.
    declarations: Vec<Declaration>,
    /// Name-to-id lookup for resolving type names and function
    /// targets during lowering. Later declarations with the same
    /// name override earlier ones in this map — the Vec still holds
    /// both, but only the latest is findable by name.
    decl_by_name: HashMap<String, DeclarationId>,
    next_node_id: u32,
    next_port_id: u32,
}

impl Dag {
    pub fn new() -> Self {
        let mut dag = Self {
            nodes: Vec::new(),
            ports: HashMap::new(),
            diagnostics: DiagnosticTable::new(),
            declarations: Vec::new(),
            decl_by_name: HashMap::new(),
            next_node_id: 0,
            next_port_id: 0,
        };
        dag.bootstrap_primitives();
        dag
    }

    /// Pre-populate the declaration table with the primitive types
    /// and primitive functions the substrate understands.
    ///
    /// **M1 task (1) boundary:** this function is the single
    /// authoritative source for primitives during M1 task (1).
    /// Shape is: register types first (Int, Bool, String), then
    /// register primitive functions that reference those types.
    /// M1 task (2) replaces the *body* of this function with a
    /// std/ parse pass — the shape of the declaration table does
    /// not change during that swap, only the source of the
    /// Declarations does. If M1 task (2) needs to change the table
    /// shape, something has gone wrong with this scaffolding.
    fn bootstrap_primitives(&mut self) {
        let int_id = self.register_declaration(Declaration {
            name: "Int".to_string(),
            kind: DeclKind::Type,
        });
        let bool_id = self.register_declaration(Declaration {
            name: "Bool".to_string(),
            kind: DeclKind::Type,
        });
        let _string_id = self.register_declaration(Declaration {
            name: "String".to_string(),
            kind: DeclKind::Type,
        });

        let int = TypeShape::new(int_id);
        let boolean = TypeShape::new(bool_id);

        // Arithmetic: (Int, Int) -> Int
        for name in [
            "std::int::add",
            "std::int::sub",
            "std::int::mul",
            "std::int::div",
        ] {
            self.register_declaration(Declaration {
                name: name.to_string(),
                kind: DeclKind::Function {
                    params: vec![int, int],
                    return_type: int,
                    body_port: None,
                },
            });
        }

        // Comparisons: (Int, Int) -> Bool
        for name in [
            "std::int::eq",
            "std::int::ne",
            "std::int::lt",
            "std::int::le",
            "std::int::gt",
            "std::int::ge",
        ] {
            self.register_declaration(Declaration {
                name: name.to_string(),
                kind: DeclKind::Function {
                    params: vec![int, int],
                    return_type: boolean,
                    body_port: None,
                },
            });
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

    /// O(1) lookup by DeclarationId. Every DeclarationId ever
    /// returned by `register_declaration` is valid for the life of
    /// the Dag — declarations are never removed or reordered.
    pub fn declaration(&self, id: DeclarationId) -> &Declaration {
        &self.declarations[id.index()]
    }

    /// Look up a declaration by name. Returns the most-recently
    /// registered declaration with this name, or None if no such
    /// declaration exists. Used during lowering to resolve type
    /// annotations and function-call targets from surface-level
    /// strings to typed references.
    pub fn declaration_by_name(&self, name: &str) -> Option<DeclarationId> {
        self.decl_by_name.get(name).copied()
    }

    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
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

    /// Register a new declaration and return its DeclarationId.
    /// The id is stable for the life of the Dag. If a declaration
    /// with the same name already exists, the new declaration
    /// overrides the old one in `decl_by_name` (both entries
    /// remain in the `declarations` Vec, but only the latest is
    /// findable by name).
    pub(crate) fn register_declaration(&mut self, decl: Declaration) -> DeclarationId {
        let id = DeclarationId(self.declarations.len() as u32);
        self.decl_by_name.insert(decl.name.clone(), id);
        self.declarations.push(decl);
        id
    }

    /// Attach a body port to an existing user-function declaration.
    /// Called from lowering after the Bind node for the function
    /// has been created, so the Bind's value port can be stored on
    /// the declaration. After this call, inference's Transform case
    /// consults `dag.declaration(id).kind`'s `body_port` to run
    /// the producer-fact-reaches-consumer check for call sites.
    ///
    /// Panics if the target declaration is not a Function. Debug-
    /// asserts that the body_port has not already been set
    /// (primitives never have a body_port; user functions register
    /// their declaration exactly once at lowering).
    pub(crate) fn set_function_body_port(&mut self, id: DeclarationId, port: PortId) {
        let decl = &mut self.declarations[id.index()];
        match &mut decl.kind {
            DeclKind::Function { body_port, .. } => {
                debug_assert!(
                    body_port.is_none(),
                    "set_function_body_port called twice for `{}`",
                    decl.name
                );
                *body_port = Some(port);
            }
            DeclKind::Type => panic!(
                "set_function_body_port on non-function declaration `{}`",
                decl.name
            ),
        }
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
