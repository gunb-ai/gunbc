# M1 Design Note: Substrate Rework

**Status:** design-first spec for M1(2.5). Answers load-bearing representation
choices before any code changes. Paired with `THESIS.md` §"The substrate: two
coordinated shapes" and §"Epistemic stacking."

**Supersedes:** PR #442's `DeclKind::{Type, Function}` flat shape. That work
(M1(1) + M1(2)) is wiped in full — no rescue plan. It was ~30 minutes of
implementation and the infrastructure scaffolds are useful for reference but
not for salvage. Convergence note from 2026-04-15 classifies "flat `DeclKind`
is too narrow" as settled.

## 0. Purpose

This document is the oracle M1 rework executes against. It:

1. Names what infrastructure survives the wipe vs. what is replaced.
2. Commits the concrete Rust data model for the two-substrate shape committed
   in THESIS.md.
3. Answers six load-bearing representation questions that determine whether
   M1 rework is one PR or three.
4. Walks `dsl/std/algebra.dag` and a minimal `dsl/extdeps/shell.dag` subset
   onto the data model to verify the substrate test passes.
5. Gives a focused file-by-file implementation plan.
6. Declares the M1(2.5) scope boundary — what is explicitly not in scope.

If any decision in this note is wrong, it's fixed HERE before code changes.
Design-first is the working discipline.

## 1. Convergence reference

The project converged at 2026-04-15 on:

- **Settled:** epistemic stacking is load-bearing; flat `DeclKind::{Type,
  Function}` is too narrow; substrate test is "host `dsl/std/algebra.dag`
  + a synthetic nested-domain model analogous to `shell.Exec.Run`
  with no new DeclKind variants";
  design-first before implementation; omni-emission is the long-term goal.
- **Open (this doc pins them down):** two-substrate concrete shape,
  Declaration struct, PR #442 infrastructure inventory.
- **Deferred:** five L1 behaviors as patterns over the type substrate
  (unification); further reduction to three primitives (Atom/Conj/Disj only);
  `std/meta.dag` as a first-class mechanism; cost lens + single emitter.

This design note executes on the Open items only.

## 2. PR #442 wipe: what goes, what stays

**Deleted entirely:**

- `DeclKind::{Type, Function}` enum in `src/v3/compiler/src/dag.rs`.
- `primitive_signature` / `Signature` hardcoded lookup table in `src/v3/compiler/src/infer.rs`.
- `FunctionRef { name: String }` (name-based function dispatch) — replaced
  by `DeclarationId`-based references in the new connective shape.
- `LiteralValue` enum with flat Int/Bool/String variants — replaced by
  `AtomPayload::Literal`.
- Per-primitive function declarations in `dsl/std/core.dag` (e.g., `fn
  std::int::add(a: Int, b: Int) -> Int`). These are exactly the
  parallel-representation pattern the thesis rejects. Replaced by
  inhabitance-derived signatures walking `std/algebra.dag`.
- The `bootstrap_primitives` function body that hand-registers Int/Bool/
  String and operators in Rust. Replaced by parsing `std/algebra.dag` at
  `Dag::new()`.

**Survives the wipe (keep and extend):**

- `DeclarationId` newtype — stable integer IDs over the declaration table.
- `Dag::new()` bootstrap entry point and declaration table mechanism.
  Contents change; shape stays.
- Parser extensions for nested declarations, keyword paths, optional `fn`
  bodies. Needed for the new shape's richer syntax.
- Name resolution hooks (identifier → DeclarationId).
- **Five L1 behaviors** (`Value`, `Transform`, `Branch`, `Loop`, `Bind`)
  in the computation substrate — unchanged from M0. Validated by M0 under
  three reviewer rounds; not revisited.
- `PortState` enum (`Uninferred | Resolved(TypeShape) | Unresolved`).
- `DiagnosticTable` and the fail-closed compile boundary (`Err(CompileError::
  Semantic(Dag))`).
- `SourceSpan` tracking structurally on every Node.

**Unaffected:**

- Tokenizer (`src/v3/compiler/src/tokenize.rs`) — minor updates only.
- Lens architecture (`lens_depth.rs`, `lens_provenance.rs`) — read
  interfaces will need light updates to walk connectives instead of
  `DeclKind` variants, but the lens-as-read-only-observer pattern stands.

## 3. Declaration data model

```rust
// src/v3/compiler/src/dag.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclarationId(u32);

#[derive(Debug, Clone)]
pub struct Declaration {
    pub id: DeclarationId,
    pub name: Option<String>,
    pub connective: TypeConnective,
    /// Optional additional inhabitance edge. For Instantiation-connective
    /// declarations, inhabits is typically None — the Instantiation IS the
    /// inhabitance. For declarations with their own primary structure
    /// (Conj/Disj/Atom) that additionally inhabit an algebra (e.g.,
    /// a user-defined Money record inhabiting a Currency lattice),
    /// inhabits carries that secondary relation.
    pub inhabits: Option<DeclarationId>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum TypeConnective {
    /// Irreducible leaf. See AtomPayload.
    Atom(AtomPayload),

    /// Labeled product — logical AND. All children present together.
    Conj { children: Vec<Field> },

    /// Labeled coproduct — logical OR. Exactly one variant active.
    Disj { variants: Vec<Field> },

    /// Function type — directional flow from inputs to an output.
    /// Function type — directional flow from inputs to an output.
    /// `body` is an ArrowBody enum covering user-defined sub-DAGs,
    /// extdeps-declared realizations, and the Pending bootstrap
    /// state. See Q7 for the full semantics.
    Arrow {
        inputs: Vec<DeclarationId>,
        output: DeclarationId,
        body: ArrowBody,
    },

    /// Repetition over an element type with a bound. Unifies v2's
    /// Required/Optional with list cardinality.
    Cardinality {
        element: DeclarationId,
        bound: CardinalityBound,
    },

    /// Specialization of a parameterized template declaration with
    /// concrete template arguments. Matches C++ template instantiation:
    /// `template` is the parameterized declaration (analogous to
    /// `template<typename T> class X { ... }`), `arguments` is the
    /// list of concrete bindings (analogous to the `X<int>` site).
    /// For pure aliases like `Int = OrderedRing<Word64>`, Int's
    /// connective IS Instantiation directly — inhabitance collapses
    /// into this form. See also Declaration.inhabits for the rare
    /// secondary case.
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
pub enum ArrowBody {
    /// User function. NodeId is the root of a sub-DAG of L1
    /// behavior nodes in Dag::nodes.
    UserDefined(NodeId),
    /// Primitive / intrinsic whose realization is declared in
    /// an extdeps language spec. DeclarationId points at the
    /// realization declaration, reached by a typed edge — not
    /// by name lookup. See Q7.
    ExternalRealization(DeclarationId),
    /// Bootstrap state at M1(2.5): static grounding is complete
    /// via inhabitance, but no realization declaration is
    /// loaded yet. Scaffolded state — CI ratchet (§8.10)
    /// enforces that Pending resolves to UserDefined or
    /// ExternalRealization before M3 completes.
    Pending,
}

#[derive(Debug, Clone)]
pub enum AtomPayload {
    /// A literal bit pattern carried at the type level.
    Literal(LiteralBits),
    /// An identifier reference. String pre-resolution, DeclarationId
    /// post-resolution. Stored as a string so pre/post is a bit flag,
    /// not a variant split.
    Identifier { name: String, resolved: Option<DeclarationId> },
    /// A type parameter declaration slot. Appears as a child of a
    /// parameterized declaration; referenced from inside the body by
    /// other Identifier atoms that resolve to this slot.
    TypeParam(String),
    /// A source span reference for metadata-only atoms.
    Span(SourceSpanId),
}

#[derive(Debug, Clone)]
pub enum LiteralBits {
    Int(i64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone)]
pub enum CardinalityBound {
    /// Exact count. Required fields use Exact(1); fixed-size
    /// arrays use Exact(n); argv literals use Exact(3).
    Exact(u32),
    /// Zero or one. The distinct shape for optional fields
    /// (`T?` in surface syntax). Keeps 0..1 structurally
    /// distinguishable from 0..n.
    AtMostOne,
    /// Zero or more. The shape for `List<T>`, used anywhere
    /// the count is unbounded.
    Unbounded,
    // Range / AtLeast(n) / AtMost(n) deferred to M1+; neither
    // algebra.dag nor minimal shell.dag needs them. AtMostOne
    // is included from the start because Optional is a
    // distinct shape from List and conflating them would let
    // illegal states (a "List-typed optional") be representable.
}

#[derive(Debug, Clone)]
pub struct TemplateArgument {
    /// The template parameter being bound. References a TypeParam
    /// Atom declared as a child of the template. In C++ terms:
    /// the `T` in `template<typename T>`.
    pub parameter: DeclarationId,
    /// The concrete type the parameter is bound to. In C++ terms:
    /// the `int` in `X<int>`.
    pub value: DeclarationId,
}
```

Estimated line count: ~120 lines of Rust for the data model + helper impls.
Concrete implementation may grow this slightly for accessor ergonomics.

## 4. Load-bearing design questions — answered

### Q0: What counts as an Instantiation (vs a Conj, vs inhabitance)?

**Answer: Instantiation is ONLY for type parameterization. Value
construction and inhabitance witness use different mechanisms.**

Earlier drafts of the thesis used "Instance" as a catch-all for
three different structural relationships. The reviewer flagged
this as a coproduct hiding in prose — the Rust struct layout
can't accommodate all three without becoming a three-variant
coproduct itself. The resolution is to commit each to a specific
mechanism:

**Case 1: Type parameterization** (e.g., `Int64 = OrderedRing<Word64>`,
`Monoid<Int>`) uses the `Instantiation` connective. Binds type
parameters (Atoms with `TypeParam` payload) to concrete type
declarations. Substitution is lazy at walk time via a substitution
stack. Matches C++ `template<typename T> class X { ... }` →
`X<int>` exactly.

**Case 2: Value construction** (e.g., `transport shell { argv:
["sh", "-lc", "{script}"] }`, `Order { customer_id: "x", items:
[...] }`) is **NOT** Instantiation. It is a plain `Conj` whose
children are bound to specific values, with an optional
`inhabits` edge on the Declaration pointing at the meta-type the
Conj satisfies:

```
Node: Run_transport
  connective: Conj
  children: [
    Field { label: "argv", ty: argv_literal_node }
  ]
  inhabits: Some(transport_shell_meta_ref)   // tag, not Instantiation
```

The compiler type-checks this Conj against `transport_shell_meta`'s
declared field shape via a structural check — no Instantiation
machinery involved. The inhabits tag answers "what shape should
this Conj satisfy?"; the Conj's children answer "what values do
the fields hold?"

**Case 3: Inhabitance witness** (e.g., `Int inhabits OrderedRing`)
for pure aliases like `Int = OrderedRing<Word64>` is **the
Instantiation itself** — Int's declaration has connective
`Instantiation { template: OrderedRing, bindings: [T := Word64] }`
directly. There is NO separate witness edge. Inhabitance for
pure aliases COLLAPSES into the Instantiation.

The optional `Declaration.inhabits` slot (see Q2 below) is
reserved for the rare case of a declaration with primary
structure AND additional inhabitance — e.g., a hypothetical
`Money` record that also inhabits a `CurrencyLattice` algebra.
At M1(2.5) this slot stays empty for every declaration we'll
encounter in algebra.dag and shell.dag. It exists as forward
compatibility without substrate complexity.

**Result: one Instantiation connective with unambiguous
semantics (type parameterization only).** Value construction
stays in the Conj family, inhabitance-for-aliases collapses into
Instantiation naturally, and the optional `inhabits` edge covers
the rare secondary case. No three-variant coproduct, no hidden
overload.

### Q1: What does Instantiation actually hold structurally?

**Answer: lazy substitution with shared templates.**

`Instantiation { template: DeclarationId, arguments: Vec<TemplateArgument> }`.
Using C++ vocabulary: `template` is the parameterized declaration
being instantiated (the `template<typename T> class X { ... }`
side); `arguments` is the list of template arguments (the
`X<int>` side). Each `TemplateArgument` pairs a template
parameter (a TypeParam Atom in the template declaration) with a
concrete type reference.

The template is shared; substitution happens at walk time, not at
declaration time. Int and UInt both reference the same OrderedRing
template, with different arguments. Each walker carries a
substitution stack and resolves parameter references on demand.

**Why lazy over eager.** Eager substitution (copying the template and
replacing parameter atoms with their values) is simpler for a first pass
but:

1. Memory waste — each instantiation allocates a full copy of the template.
2. Sharing loss — the structural fact "Int and UInt both inhabit the same
   algebra" is not visible in the Node graph.
3. Diagnostic degradation — when inference fails on `Int.add`, the error
   context points at an anonymous substituted copy instead of "OrderedRing's
   `add` specialized with T := Word64."

Lazy substitution preserves all three properties at the cost of a small
substitution-stack overhead during walks. That overhead is O(declaration
depth), not O(user program size).

**Memoization is a lens concern, not substrate.** If walking the same
Instantiation repeatedly becomes hot, lenses can cache — but that's a
lens-level optimization, not a substrate decision.

### Q2: How is `inhabits` realized?

**Answer: Instantiation is the primary realization; the secondary `inhabits`
edge is a slot held open for forward compatibility.**

For pure aliases like `Int = OrderedRing<Word64>`, Int's Node has
`connective: Instantiation { template: OrderedRing, bindings: [T := Word64] }`.
There is NO separate "Int declaration with an inhabits edge pointing at
an Instantiation Node." The collapse is honest because Int has no structure
of its own beyond the inhabitance — it's a pure alias.

`Declaration.inhabits: Option<DeclarationId>` is reserved for the rarer
case where a declaration has its own primary structure AND inhabits
something additional. Example (not in scope for M1(2.5)):

```dag
type Money {
  value: Float
  currency: Currency
}
// Money primarily is a Conj, AND additionally inhabits CurrencyLattice
```

In that case Money's primary connective is Conj, and `inhabits` holds an
optional reference to the additional semantic layer. At M1(2.5) this slot
stays empty for every declaration in `algebra.dag` and `shell.dag`; it
exists for forward compatibility without adding substrate complexity.

**No separate "kind" field beyond the connective enum.** The connective
variant IS the kind.

### Q3: How are type parameters (`<T>`) represented at the substrate level?

**Answer: Type parameters are Atom declarations with a TypeParam payload.
No new connective.**

A parameterized declaration lists its parameters as children of its Conj.
The children whose target's connective is `Atom(TypeParam(_))` are the
parameters; the rest are the declaration's "real" content. Example:

```
Declaration: Magma
  connective: Conj {
    children: [
      Field { label: "T",  ty: DeclarationId(T_param) },
      Field { label: "op", ty: DeclarationId(op_arrow) },
    ]
  }

Declaration: T_param
  connective: Atom(TypeParam("T"))

Declaration: op_arrow
  connective: Arrow {
    inputs:  [DeclarationId(T_ref), DeclarationId(T_ref)],
    output:  DeclarationId(T_ref),
    body:    None,  // algebra field, no body
  }

Declaration: T_ref
  connective: Atom(Identifier { name: "T", resolved: Some(T_param) })
```

`T_param` is the parameter's declaration site; `T_ref` is a reference
to it from inside the body. The parser produces both; name resolution
links identifier atoms to their target declarations.

The distinction between "parameter child" and "field child" is
determined by walking to the target and checking its connective — no
metadata on the `Field` edge, no separate parameter list. A walker that
wants to enumerate parameters filters `children` to those whose target
is `Atom(TypeParam(_))`.

**No seventh connective.** Type parameters reuse the Atom primitive with
a specialized payload.

### Q4: Where does parameter-to-value substitution actually happen?

**Answer: Lazy at walk time via a substitution stack.**

```rust
struct SubstStack {
    frames: Vec<Vec<TemplateArgument>>,
}

impl SubstStack {
    fn lookup(&self, param: DeclarationId) -> Option<DeclarationId> {
        // Walk frames top-down, return first matching binding.
        for frame in self.frames.iter().rev() {
            for binding in frame {
                if binding.parameter == param {
                    return Some(binding.value);
                }
            }
        }
        None
    }

    fn push(&mut self, bindings: Vec<TemplateArgument>) {
        self.frames.push(bindings);
    }

    fn pop(&mut self) {
        self.frames.pop();
    }
}

fn walk(dag: &Dag, id: DeclarationId, subst: &mut SubstStack) -> /* result */ {
    let decl = dag.declaration(id);
    match &decl.connective {
        TypeConnective::Atom(AtomPayload::Identifier { resolved: Some(tgt), .. }) => {
            // If the target is a TypeParam and we have a binding for it,
            // follow the binding instead of the param.
            if let Some(bound) = subst.lookup(*tgt) {
                walk(dag, bound, subst)
            } else {
                walk(dag, *tgt, subst)
            }
        }
        TypeConnective::Instantiation { template, bindings } => {
            subst.push(bindings.clone());
            let result = walk(dag, *template, subst);
            subst.pop();
            result
        }
        TypeConnective::Conj { children } => {
            for field in children {
                walk(dag, field.ty, subst);
            }
            // ...
        }
        // ... similar for Disj, Arrow, Cardinality, Atom
    }
}
```

Inference and emission both carry their own `SubstStack` during walks.
The substrate stores no substitution state; every walker pushes/pops as
it enters/exits Instantiation nodes.

**Why not lowering-time substitution?** Lowering-time would produce a
fully-specialized tree for each Instantiation, losing sharing and diagnostic
provenance. See Q1 rationale.

### Q5: Cardinality's scope — does it subsume v2's Required/Optional?

**Answer: Yes, with three distinct bound variants at M1(2.5).**

v2's `constructors.dag` has `Required | Optional` as a flat enum
on binding sites. In v3's substrate, all presence/repetition
shapes become variants of `CardinalityBound`:

- `Required` = `Cardinality { element: T, bound: Exact(1) }`.
- `Optional` (`T?`) = `Cardinality { element: T, bound: AtMostOne }`.
- `List<T>` = `Cardinality { element: T, bound: Unbounded }`.
- `[T; n]` fixed array = `Cardinality { element: T, bound: Exact(n) }`.
- `argv: ["sh", "-lc", "{script}"]` = `Cardinality { element: String, bound: Exact(3) }`
  with children inlined (implementation detail).

**M1(2.5) commits to three bound variants: `Exact(u32)`,
`AtMostOne`, and `Unbounded`.** Range / AtLeast(n) / AtMost(n)
are deferred until a real oracle needs them (neither
`algebra.dag` nor minimal `shell.dag` does).

**Why AtMostOne is in scope from the start.** Collapsing
Optional into Unbounded (as earlier drafts proposed) lets
"0..1" and "0..n" be represented by the same shape — which
means a `T?` value and a `List<T>` value would be
structurally indistinguishable. That's an illegal-states-
unrepresentable violation: the substrate would admit shapes
("a List where we forgot the upper bound was 1") that should
not exist. `AtMostOne` as a dedicated variant keeps the
distinction structural.

**Optional syntax IS in scope for M1(2.5)** — `T?` parses to
`AtMostOne`. No deferral.

### Q6: How do Arrow bodies cross into the computation substrate?

**Answer: Separate tables with separate ID types, referenced by
typed edges. `DeclarationId` and `NodeId` are distinct newtypes;
`declarations` and `nodes` are distinct tables on `Dag`.**

The two substrates live in separate tables of the same Dag
struct. Each has its own stable ID type. Crossings between them
are **typed edges** (DeclarationId from one side, NodeId from
the other), not shared ID spaces.

```rust
pub struct Dag {
    // Type substrate — declarations (Conj/Disj/Arrow/etc. shapes).
    declarations: Vec<Declaration>,
    // Computation substrate — L1 behavior nodes, unchanged from M0.
    nodes: Vec<Node>,
    // Ports carry typed values through the computation graph.
    ports: Vec<Port>,

    // Name resolution, diagnostics, etc.
    named: HashMap<String, DeclarationId>,
    diagnostics: DiagnosticTable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclarationId(u32);   // index into Dag::declarations

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);          // index into Dag::nodes
```

**Two typed crossings connect the substrates:**

1. **`Transform::target: FunctionRef(DeclarationId)`** — a
   Transform node in the computation substrate holds a
   `FunctionRef` wrapping a `DeclarationId`. The DeclarationId
   points at an Arrow declaration in the type substrate, which
   has the function's signature. Inference uses this crossing
   to type-check the Transform.

2. **`Arrow::body: ArrowBody`** — an Arrow declaration in the
   type substrate holds an `ArrowBody` tag (see §3's data model
   and Q7). For user functions the variant is
   `UserDefined(NodeId)` pointing at the root of the function's
   computation sub-DAG. For primitives whose realization lives
   in an extdeps language spec the variant is
   `ExternalRealization(DeclarationId)` pointing at the realization
   declaration. For algebras at M1(2.5) bootstrap (before extdeps
   language specs are loaded) the variant is `Pending`, a
   scaffolded state with a CI ratchet enforcing resolution to
   one of the other two variants before M3 completes. All three
   variants are typed edges; no name-based lookup. See Q7.

**Keeping `DeclarationId` and `NodeId` as separate newtypes
prevents cross-substrate ID mixing bugs.** A function that
expects a `NodeId` can't be passed a `DeclarationId`; the type
system catches the error at compile time. The "two substrates,
two meeting points" framing is preserved both conceptually AND
in the type system — the two ID types are the structural
evidence that the substrates are distinct.

### Q7: How do primitive Arrows express their realization?

**Answer: via a typed `realization` edge on Arrow, not by name
lookup. `body` and `realization` are two separate fields on
Arrow. Exactly one of them must be present post-bootstrap.**

Earlier drafts of this section said "primitive Arrows have
`body: None`, and the emitter recovers the realization by name
from extdeps." That resurrected exactly the kind of name-based
special-case lookup the thesis forbids: a concept whose
realization is reached via a hardcoded path outside the substrate
is an ungrounded concept (the escape from the epistemic chain).
Fix: primitive Arrows point at a **realization declaration**
via a typed edge in the same way user functions point at a body.

Revised Arrow shape:

```rust
TypeConnective::Arrow {
    inputs: Vec<DeclarationId>,
    output: DeclarationId,
    /// Either the realization is defined in user source (as a
    /// sub-DAG of computation nodes) OR it's declared in an
    /// extdeps/ language spec (as a Declaration that the
    /// language spec references). Exactly one variant is
    /// present post-bootstrap; both None at M1(2.5) for
    /// algebras whose extdeps realization hasn't been
    /// declared yet.
    body: ArrowBody,
}

#[derive(Debug, Clone)]
pub enum ArrowBody {
    /// User function. The referenced NodeId is the root of a
    /// computation sub-DAG (Value/Transform/Branch/Loop/Bind).
    UserDefined(NodeId),
    /// Realization declared in a language spec. The
    /// DeclarationId points at a Realization declaration in
    /// dsl/extdeps/languages/<target>.dag. The language spec
    /// declaration is an ordinary Declaration in the type
    /// substrate, reached by a typed edge like any other
    /// declaration. Inference can follow the edge to the
    /// realization and check that the declared signature
    /// matches.
    ExternalRealization(DeclarationId),
    /// Bootstrap state: neither user body nor extdeps
    /// realization declared yet. Legal ONLY during M1(2.5)
    /// bootstrap while std/algebra.dag is loaded without
    /// extdeps/languages/*.dag yet. Inference must accept
    /// body: Pending for primitive Arrows whose signatures
    /// resolve via inhabitance. A post-bootstrap sweep
    /// ratchets "no Pending remains once extdeps are wired."
    Pending,
}
```

**Why `Pending` is a scaffolded state, not a forever-legal one.**
At M1(2.5), std/algebra.dag declares `Magma<T>.op`, `Ring<T>.add`,
etc., as Arrows. Their *signatures* are well-defined (they
resolve through inhabitance via the substitution stack). Their
*realizations* — what these operations map onto in Rust / Python
/ Go — need `dsl/extdeps/languages/rust.dag` and friends to
declare the target mapping, and those files don't exist yet.
`Pending` names the state "this Arrow has a valid static
grounding via inhabitance, but no realization declaration has
been loaded." It is explicitly scaffolded: a CI-enforced
ratchet (see §8.10) will require `Pending` → either `UserDefined`
or `ExternalRealization` before M3 completes.

**This matches `body` for user functions.** A `UserDefined`
body is a `NodeId` into the computation substrate — a typed
edge, not a name lookup. An `ExternalRealization` is a
`DeclarationId` into another declaration in the type substrate
(in `dsl/extdeps/languages/<target>.dag`) — also a typed edge,
also not a name lookup. Both are structural; both are walkable
by any consumer without name resolution.

**Inference handling:**

- `UserDefined(node_id)`: type-check the sub-DAG body against
  the declared input/output types.
- `ExternalRealization(decl_id)`: walk the realization
  declaration, verify its declared signature is compatible
  with this Arrow's signature.
- `Pending`: accept as realization-pending; the signature
  type-checks via inhabitance. Emission of this Arrow
  requires the `Pending` to resolve before the emitter runs.

**For M1(2.5):** primitive Arrows in algebra.dag land with
`body: Pending`. Inference treats them as ok-for-signature-
checking but not-yet-emittable. The substrate test
`parse_std_algebra_and_walk_int_add` validates that inhabitance
walk produces the correct Arrow type even with Pending bodies.

**The "no concept is opaque" stop signal applies fully.** An
Arrow whose realization is reached via a hardcoded path outside
the substrate (e.g., a Rust function inside the compiler that
the emitter calls by name when it sees this Arrow) IS ungrounded
and is a thesis violation. The `ArrowBody` enum forces
realization to be either a sub-DAG or a typed reference to
another Declaration — both walkable, both structural, neither
an escape hatch.

## 5. `dsl/std/algebra.dag` mapping (substrate test oracle #1)

Walk the first five algebras onto the data model. The remaining
structures (CommutativeMonoid, Group, AbelianGroup, Semiring, Ring,
OrderedRing, Field, Lattice, BoundedLattice, BooleanAlgebra, FreeMonoid,
PartialFunction) follow the same pattern.

### Magma<T>

```
Magma:
  connective: Conj {
    children: [
      Field { label: "T",  ty: Magma_T_param   },
      Field { label: "op", ty: Magma_op_arrow  },
    ]
  }

Magma_T_param:
  connective: Atom(TypeParam("T"))

Magma_op_arrow:
  connective: Arrow {
    inputs:  [Magma_T_ref1, Magma_T_ref2],
    output:  Magma_T_ref3,
    body:    None,
  }

Magma_T_ref1, Magma_T_ref2, Magma_T_ref3:
  connective: Atom(Identifier { name: "T", resolved: Some(Magma_T_param) })
```

Six declarations: one Conj, one TypeParam Atom, one Arrow, three
Identifier Atoms. No new primitives. The identifier atoms could be
collapsed into shared references during interning; that's an
implementation-side optimization, not a substrate decision.

### Semigroup<T>

Semigroup adds associativity as a law — a property of the algebra, not
a declared field. In the substrate, associativity is NOT a child.
Semigroup's declaration is structurally identical to Magma's Conj
shape; the associativity property is asserted by inference rules that
inspect `inhabits` chains and apply rewrite rules.

**M1(2.5) does not implement law checking.** Semigroup's declaration
lands as a Conj with the same `op` child. Associativity becomes
relevant when algebraic-simplification lenses are built — deferred.

### Monoid<T>

```
Monoid:
  connective: Conj {
    children: [
      Field { label: "T",        ty: Monoid_T_param       },
      Field { label: "op",       ty: Monoid_op_arrow      },
      Field { label: "identity", ty: Monoid_identity_ref  },
    ]
  }

Monoid_T_param:  Atom(TypeParam("T"))
Monoid_op_arrow: Arrow { inputs: [T, T], output: T, body: Pending }
Monoid_identity_ref: Atom(Identifier { name: "T", resolved: Some(Monoid_T_param) })
```

`identity` is a value of type T, represented as an Atom reference to
the T parameter. At inference time, `identity` resolves to a field with
type T (and ultimately a concrete carrier when Monoid is instantiated).

### Ring<T>

```
Ring:
  connective: Conj {
    children: [
      Field { "T",      Ring_T_param      },
      Field { "add",    Ring_add_arrow    },  // Arrow(T, T, T)
      Field { "zero",   Ring_T_ref_zero   },
      Field { "negate", Ring_negate_arrow },  // Arrow(T, T)
      Field { "mul",    Ring_mul_arrow    },  // Arrow(T, T, T)
      Field { "one",    Ring_T_ref_one    },
    ]
  }
```

Six children — one parameter, four Arrow-valued fields (add, negate,
mul), and two Atom-valued fields (zero, one). Same pattern as Monoid,
just more fields.

### OrderedRing<T>

Extends Ring with `compare` (an Arrow returning an `Ordering` Disj),
`quotient`, `remainder`. Same shape pattern.

### Int — the inhabiting instance

```
Int:
  connective: Instantiation {
    template: OrderedRing,
    bindings: [
      TemplateArgument {
        parameter: OrderedRing_T_param,
        value:     Word64,
      }
    ]
  }
```

Walking `Int.add`:

1. Start at `Int` — `Instantiation` connective.
2. Push `[T := Word64]` onto substitution stack.
3. Walk to template `OrderedRing` — `Conj` connective.
4. Find child `"add"` — points at `OrderedRing.add` which is an Arrow
   declaration.
5. Walk the Arrow's inputs: each is an Atom(Identifier) resolving to
   `OrderedRing_T_param`.
6. Look up `OrderedRing_T_param` in substitution stack → find `Word64`.
7. Substitute each input/output with `Word64`.
8. Yield `Arrow { inputs: [Word64, Word64], output: Word64, body: Pending }`.
9. Pop substitution frame on exit.

**Pure structural walk. No hardcoded knowledge of Int, Ring, add, or
Word64.** The compiler doesn't know what "add" means — it just finds
the child labeled "add" and returns its Arrow type with parameters
substituted.

**Substrate test `parse_std_algebra_and_walk_int_add` passes** when
this walk produces `Arrow(Word64, Word64, Word64)` as the type of
`Int.add` from an empty compiler state parsing only `std/algebra.dag`
+ the `Int = OrderedRing<Word64>` declaration.

## 6. Nested-domain-model oracle #2 (synthetic)

**Scope note:** M1(2.5) does NOT aim for full fidelity against
`dsl/extdeps/shell.dag` as-is. That file has structure we are
not ready to host yet: a `transport_shell_template` with its own
declaration (which lives in `dsl/extdeps/transports/shell.dag` —
not currently declared in the project), `exit` branches (which
use Disj with payloads and need the sum-type parser extensions),
and `mock_response` tables (which need literal-record construction).

**Instead:** oracle #2 is a **synthetic nested-domain model**
that exercises the same substrate shapes `shell.Exec.Run` would
exercise, without inheriting v2's specific extdeps declarations
we haven't staged yet. The synthetic model is defined inline in
the M1(2.5) test source:

```dag
// Test-only declaration. Exercises the nested-domain shape
// (Conj-of-Conj-of-field-values-with-inhabits-tags) that shell.dag
// uses, without importing shell.dag itself.

type SyntheticService {
  operations: Conj  // labeled-operation container
}

type SyntheticOperation {
  input:     Conj   // labeled-field record
  output:    Conj
  arguments: List<String>
}

// An instance of the synthetic service — what the test parses.
service CmdExec: SyntheticService {
  operation Run: SyntheticOperation {
    input  { script: String }
    output { stdout: String; stderr: String; exit_code: Int }
    arguments: ["sh", "-lc", "{script}"]
  }
}
```

All nested Conj compositions, no new connectives, no references
to un-declared v2 files. This subset exercises:

- Nested Conj (service → operations → operation → input/output).
- `inhabits` tags on the service/operation Conjs pointing at the
  meta-type declarations (`SyntheticService`, `SyntheticOperation`).
- Cardinality with `Exact(3)` for the argument list literal.
- String literal Atoms as leaves.
- Value-construction via Conj-with-field-values (NOT Instantiation —
  see Q0 and BI-1).

Parsed into the substrate:

```
CmdExec:
  connective: Conj
  inhabits: Some(SyntheticService_decl)
  children: [
    Field { label: "operations", ty: CmdExec_operations }
  ]

CmdExec_operations:
  connective: Conj
  children: [
    Field { label: "Run", ty: CmdExec_Run }
  ]

CmdExec_Run:
  connective: Conj
  inhabits: Some(SyntheticOperation_decl)
  children: [
    Field { label: "input",     ty: Run_input     },
    Field { label: "output",    ty: Run_output    },
    Field { label: "arguments", ty: Run_args      },
  ]

Run_input:
  connective: Conj
  children: [
    Field { label: "script", ty: String_decl }
  ]

Run_output:
  connective: Conj
  children: [
    Field { label: "stdout",    ty: String_decl },
    Field { label: "stderr",    ty: String_decl },
    Field { label: "exit_code", ty: Int_decl    },
  ]

Run_args:
  connective: Cardinality {
    element: String_decl,
    bound:   Exact(3),
  }
```

Five levels of nesting:
1. service (Conj with inhabits)
2. operations container (Conj)
3. operation Run (Conj with inhabits)
4. input / output / arguments members (Conj or Cardinality)
5. scalar fields and literal atoms (Atom)

Every level maps onto the six-connective shape. **No new
connective variants needed.** The `inhabits` tag provides the
"this Conj satisfies this meta-type" semantics without any
special Service/Operation connective.

**Substrate test `parse_synthetic_service_all_layers` passes** when
parsing this subset produces the above Declaration tree with no
extension to the connective enum.

**Why synthetic rather than the real shell.dag:** full shell.dag
fidelity requires the extdeps/transports layer, sum-type payloads
(for `exit`), and literal record construction (for `mock_response`)
— all of which are genuine substrate exercises but add scope
beyond what's needed to prove the substrate shape is right. The
synthetic oracle tests the shapes that shell.dag uses without
inheriting its specific external dependencies. Full shell.dag
adoption is a follow-up milestone (M1(2.6) or M2) that lands
after extdeps/transports exists as a declared .dag subtree.

## 7. Implementation plan

Focused, small, one PR. No scope creep.

### File-by-file

| File | Action | Est. time |
|---|---|---|
| `src/v3/compiler/src/dag.rs` | Replace `DeclKind::{Type, Function}` with the `TypeConnective` enum + `Declaration` / `Field` / `AtomPayload` / `CardinalityBound` / `TemplateArgument` data model from §3. Keep `DeclarationId`, `Dag`, bootstrap entry. | ~1.5h |
| `src/v3/compiler/src/tokenize.rs` | Minor — add `<` and `>` as type-parameter delimiters, `|` as top-level sum-type separator, `fn(...)` arrow-type prefix if needed. | ~30m |
| `src/v3/compiler/src/parse.rs` | Extensions per §8: `type Foo<T> { ... }` into Conj with TypeParam children (§8.2/8.3); `type Foo = Bar<X>` into Instantiation (§Q0); `type Name = A \| B(payload)` into Disj with payload variants (§8.7); `fn(A, B) -> C` as type-position expression (§8.8); infix operators as Conj-with-function-label (§8.9). Drop the flat `fn std::int::add(...)` primitive-signature shortcut. | ~4h |
| `src/v3/compiler/src/lower.rs` | Two-pass lowering: (1) collect declaration names into symbol table; (2) resolve Identifier atoms and build the connective tree (§8.1). Handle substitution scopes for parameterized declarations. | ~2h |
| `src/v3/compiler/src/infer.rs` | Replace old `DeclKind` dispatch with connective-pattern walks. Implement `SubstStack` per §8.4 (`Vec<Vec<TemplateArgument>>`). Lazy substitution at walk time. Handle `ArrowBody::Pending` as realization-pending (signature type-checks via inhabitance, body-checking skipped) per §Q7. Handle `ArrowBody::ExternalRealization(decl_id)` by walking the realization declaration and verifying signature compatibility. Inference resolves `Int.add` by walking the inhabits chain through OrderedRing and substituting. | ~3h |
| `src/v3/compiler/tests/m0_acceptance.rs` | Update test helpers to build connective nodes. Semantics unchanged; all 40 M0 tests should pass against the new shape. | ~1h |
| `dsl/std/core.dag` | DELETE per-primitive function declarations (`fn std::int::add` etc.). They become inhabitance-derived via OrderedRing. | ~5m |
| `dsl/std/logic.dag` | VERIFY or CREATE. Parse Bit = Disj{On, Off} and True/False Atoms per §8.6. Stub file if v2 doesn't have one. | ~15m |
| `dsl/std/bit.dag` | VERIFY Word8/16/32/64 declarations work against the new parser. | ~15m |
| `dsl/std/algebra.dag` | VERIFY it parses into the new substrate. No file content changes — the file's shape is already right; just bring it into the bootstrap path. | ~15m |
| `dsl/std/types.dag` | VERIFY Int = OrderedRing<Word64>, Bool = Classical, String = FreeMonoid<Char> all parse as Instantiation declarations. | ~15m |
| `src/v3/compiler/src/bootstrap.rs` | NEW or UPDATE. Parse logic.dag → bit.dag → algebra.dag → types.dag in order at `Dag::new()` time (§8.6). | ~1h |
| `src/v3/compiler/tests/m1_substrate_test.rs` | NEW. Two test cases: `parse_std_algebra_and_walk_int_add` and `parse_synthetic_service_all_layers`. | ~1.5h |

**Total estimate: 14–16 hours.** Scope expanded from original
10–12h estimate after the review pass identified additional
parser work (operator dispatch, Arrow-in-type-position, sum types
with payloads) and bootstrap reordering. Still a single focused
PR — the work is tightly coupled and splitting it would leave the
tree in an incoherent state between PRs.

### Acceptance gates (CI must show all green)

1. **`cargo test -p v3-compiler`** — all 40 M0 tests pass unchanged.
2. **`parse_std_algebra_and_walk_int_add`** — parse `std/algebra.dag`,
   declare `Int = OrderedRing<Word64>`, walk `Int.add`, assert the
   result is `Arrow { inputs: [Word64, Word64], output: Word64 }`.
3. **`parse_synthetic_service_all_layers`** — parse the synthetic
   nested-domain model from §6, assert the Declaration tree
   structure matches. Tests the same substrate shapes shell.dag
   would exercise without requiring v2's extdeps/transports
   declarations to exist yet.
4. **`cargo clippy -p v3-compiler --all-targets -- -D warnings`** —
   clean.
5. **Substrate audit:** grep `TypeConnective::` variants in the M1
   code; confirm only the six variants from §3 exist. No
   `DeclKind::Service`, `DeclKind::Operation`, etc. variants were
   added. Extension stop signal holds.

### Explicit non-goals for M1(2.5)

- **Cost lens.** Deferred to M1(3). The substrate is ready for it
  after M1(2.5) lands; the lens is a separate PR.
- **Single Rust emitter.** Deferred to M1(4).
- **Unification (five behaviors → patterns over type substrate).**
  Deferred per convergence note. Revisit after M1(2.5) ships.
- **Three-primitive reduction (Atom/Conj/Disj only).** Deferred per
  convergence note.
- **`dsl/std/meta.dag` first-class named patterns.** Deferred until a
  second consumer appears.
- **`exit` and `mock_response` in shell.dag.** Follow-up — adds Disj
  complexity that's not needed to verify the substrate test.
- **Law verification (associativity, commutativity, etc.).** Deferred
  to the algebraic-simplification lens work.
- **Omni-emission projection rules.** Deferred until emitter.
- **Interpreter.** Deferred.
- **Sum types beyond minimal Disj parsing.** M1(2.5) needs to parse
  `type Result<T, E> = Ok(T) | Err(E)` but does not need to check
  exhaustiveness against Branch cases at inference time (M0's current
  exhaustiveness check still works against the new shape).

## 8. Sub-decisions committed

Small-but-real decisions that affect implementation mechanics.
These were open questions in earlier drafts; post-review they are
committed answers. If any is wrong, fix it here before code.

### 8.1: Identifier resolution timing — **lowering-time**

Resolve all `Atom(Identifier { resolved: None, .. })` to
`Some(DeclarationId)` during `lower.rs`. Post-lowering, every
Identifier Atom carries its target reference. Simpler; matches
M0's single-pass approach. Forward references are handled by a
two-pass lowering (first pass collects declaration names; second
pass resolves identifiers against the name table).

### 8.2: Parameter child enumeration — **filter by target connective**

A walker enumerates parameters by filtering `Conj::children` to
those whose target Declaration has connective
`Atom(TypeParam(_))`. No metadata on the `Field` edge. The walk
cost is negligible compared to inference, and not adding metadata
keeps the substrate minimal.

### 8.3: Shared parameter Atom identity — **one Atom per parameter, referenced multiply**

In `type Monoid<T> { op: fn(T, T) -> T; identity: T }`, there are
four structural references to `T`. They all resolve to **one**
Atom Node — the `T` parameter declaration. References are via
`DeclarationId`, not owned subtrees. This means the substrate is
technically a **DAG of declarations**, not a tree — one Atom can
be pointed at by many edges.

**Why**: substitution touches one site. Diagnostic provenance
points back to the same parameter declaration regardless of
which reference triggered the error. Matches the "single source
of truth" principle — T is one thing, referenced four times, not
four independent T-looking atoms.

**Implementation**: `Field::ty: DeclarationId` is a pointer, not
an owned subtree. The Declaration table owns each declaration
once; edges are pointers.

### 8.4: `SubstStack` data structure — **Vec<Vec<TemplateArgument>>**

Simple linear-scan stack. Typical stack depth is 1–3, typical
arguments per frame is 1–2, so linear scan is ~6 comparisons max
— cheaper than hashing. Revisit to HashMap-per-frame only if
profiling shows it's hot.

### 8.5: `Cardinality` parse syntax — **both forms**

`List<T>` parses to `Cardinality { element: T, bound: Unbounded }`.
`[T; 3]` (or similar fixed-size syntax) parses to `Cardinality {
element: T, bound: Exact(3) }`. Surface syntax covers both.
`argv: ["sh", "-lc", "{script}"]` in shell.dag produces a
Cardinality literal with `Exact(3)` and inline String-Atom
children.

### 8.6: Bootstrap file order — **logic → bit → algebra → types → user**

`Dag::new()` parses in this order, synchronously, before any
user source:

1. **`dsl/std/logic.dag`** — Classical, Bit, True/False. If v2
   doesn't have this file yet, declare Bit inline in the
   compiler's bootstrap and parse a stub logic.dag containing
   only the Bit declaration. (Small open question: does v2's
   current logic.dag exist? If so, parse it. If not, create it.)
2. **`dsl/std/bit.dag`** — Word8/Word16/Word32/Word64 as finite
   Bit sequences. Built on top of Bit.
3. **`dsl/std/algebra.dag`** — Magma through OrderedRing, Field,
   Lattice, BoundedLattice, BooleanAlgebra, FreeMonoid,
   PartialFunction. The algebra hierarchy parsed in one pass.
4. **`dsl/std/types.dag`** — Int = OrderedRing<Word64>, Bool =
   Classical (Bit), String = FreeMonoid<Char>, List<T> =
   FreeMonoid<T>, etc. Inhabitance declarations.

Post-bootstrap, additional std/ files (iteration, containers,
graph, etc.) get parsed lazily when user code imports them.

This order matches MODELING.md's root-to-leaves layering: roots
(logic) → carriers (bit) → algebras → concrete types → user code.

### 8.7: Sum-type parser syntax — **`type Name = A | B(payload)`**

Matches v2. Unit variants (`Sunny`) and variants with payloads
(`Rainy { mm_per_hour: Float }` or `Ok(T)`). Payloads are either
named records (`{ label: type; ... }`) or positional tuples
(`(T)`). Parser extension: new surface syntax for top-level
disjunctions. `|` becomes a recognized top-level separator in
`type` items.

### 8.8: Arrow type expressions in type position — **`fn(T, T) -> T`**

Parser extension. Current v3 parser has `fn name(args) -> ret =
body` as an item-level function declaration. M1(2.5) adds
`fn(T1, T2, ..., Tn) -> Tr` as a type-position expression usable
inside field declarations and parameter lists. `type Monoid<T>
{ op: fn(T, T) -> T; identity: T }` needs this.

### 8.9: Operator dispatch path — **infix ops resolve via inhabitance**

Currently v3 M0 parses `1 + 2` as `Call { target: "std::int::add" }`
or similar — a hardcoded path to a primitive function declaration.
M1(2.5) removes `std::int::add` as a standalone declaration
(algebra.dag drives it via OrderedRing inhabitance). The parser
needs to produce a structure that the lowerer can resolve through
inhabitance:

- Parser option A: emit `Conj { function: "+", args: [lhs, rhs] }`
  with "+" as an identifier that resolves at inference time by
  walking the inferred type's inhabits chain to find an `add`
  field.
- Parser option B: emit a symbolic `InfixOp(Add, lhs, rhs)`
  variant and have the lowerer rewrite it to a function call
  against `inferred_type.inhabits.add`.

**Recommendation: option A**, as it unifies operator dispatch with
ordinary function-call dispatch. The operator "+" is just an
identifier; its resolution walks type inhabitance. The lowerer
handles this uniformly with other Transform nodes. Adds ~2h of
parser/lowerer work to M1(2.5) scope.

### 8.10: Substrate-extension enforcement — **deferred to M1(3)**

The "no seventh connective / no sixth behavior" stop signal is
currently prose-enforced. A mechanical enforcement mechanism (a
`SUBSTRATE_EXTENSION_AUDIT.md` file that any extension PR must
append to, checked in CI) is valuable but orthogonal to M1(2.5)'s
substrate rework. **Deferred to M1(3)** — implement alongside the
cost lens as a CI check that audits the `TypeConnective` and
`Behavior` enum variants against the audit file. ~1-2h of CI
scaffolding work.

## 9. What "done" looks like for M1(2.5)

- The five files in §7 are updated; the two test files are new.
- `cargo test -p v3-compiler` shows 40 M0 tests + 2 new substrate
  tests = 42 green.
- `cargo clippy` is clean.
- `src/v3/compiler/src/dag.rs` contains no `DeclKind::{Type, Function}`
  references. `TypeConnective` has exactly the six variants in §3.
- `Int.add` resolves through inhabitance (no hardcoded signature).
- Synthetic nested-domain model (§6) parses and type-checks.
  Exercises the same substrate shapes as `shell.Exec.Run` without
  requiring v2's extdeps/transports subtree to exist yet.
- PR description cites this design note; reviewers can verify
  against §3–§6 and the acceptance gates in §7.

No emitter, no cost lens, no unification, no three-primitive
reduction, no full shell.dag. Those come later.

## 10. Reference

- `THESIS.md` §"Epistemic stacking"
- `THESIS.md` §"The substrate: two coordinated shapes"
- `THESIS.md` §"Omni-emission" (for context on why the substrate test
  matters beyond algebra.dag)
- `MODELING.md` §"Composition layer" (four-layer architecture)
- `src/v3/M0_RETROSPECTIVE.md` (what M0 validated, carried forward
  unchanged)
- `docs/design-lineage.md` §"Stage 3" (v2's Node-tree type substrate)
- `feedback_compiler_is_dag_processor.md` (core-invariant memory)
