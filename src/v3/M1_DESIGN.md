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
  + a real domain model like `shell.Exec.Run` with no new DeclKind variants";
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
    /// Optional additional inhabitance edge. For Instance-connective
    /// declarations, inhabits is typically None — the Instance IS the
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
    /// Body references the computation substrate for user functions,
    /// or is None for primitives (realized by extdeps/ language specs
    /// at emission time).
    Arrow {
        inputs: Vec<DeclarationId>,
        output: DeclarationId,
        body: Option<NodeId>,
    },

    /// Repetition over an element type with a bound. Unifies v2's
    /// Required/Optional with list cardinality.
    Cardinality {
        element: DeclarationId,
        bound: CardinalityBound,
    },

    /// Instantiation of a parameterized declaration with concrete
    /// parameter bindings. The primary realization of inhabitance.
    Instance {
        template: DeclarationId,
        bindings: Vec<ParameterBinding>,
    },
}

#[derive(Debug, Clone)]
pub struct Field {
    pub label: String,
    pub ty: DeclarationId,
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
    /// Required (exactly 1), also Cardinality(3) for fixed-size.
    Exact(u32),
    /// Optional (0..1), also List<T>.
    Unbounded,
    // Range / AtMost / AtLeast deferred to M1+; not needed for
    // algebra.dag or minimal shell.dag.
}

#[derive(Debug, Clone)]
pub struct ParameterBinding {
    /// The parameter being bound. References a TypeParam Atom in
    /// the template.
    pub parameter: DeclarationId,
    /// What it's bound to.
    pub value: DeclarationId,
}
```

Estimated line count: ~120 lines of Rust for the data model + helper impls.
Concrete implementation may grow this slightly for accessor ergonomics.

## 4. Six load-bearing design questions — answered

### Q1: What does Instance actually hold structurally?

**Answer: lazy substitution with shared templates.**

`Instance { template: DeclarationId, bindings: Vec<ParameterBinding> }`.
The template is shared; substitution happens at walk time, not at
declaration time. Int and UInt both reference the same OrderedRing
template, with different parameter bindings. Each walker carries a
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
Instance repeatedly becomes hot, lenses can cache — but that's a
lens-level optimization, not a substrate decision.

### Q2: How is `inhabits` realized?

**Answer: Instance is the primary realization; the secondary `inhabits`
edge is a slot held open for forward compatibility.**

For pure aliases like `Int = OrderedRing<Word64>`, Int's Node has
`connective: Instance { template: OrderedRing, bindings: [T := Word64] }`.
There is NO separate "Int declaration with an inhabits edge pointing at
an Instance Node." The collapse is honest because Int has no structure
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
    frames: Vec<Vec<ParameterBinding>>,
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

    fn push(&mut self, bindings: Vec<ParameterBinding>) {
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
        TypeConnective::Instance { template, bindings } => {
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
it enters/exits Instance nodes.

**Why not lowering-time substitution?** Lowering-time would produce a
fully-specialized tree for each Instance, losing sharing and diagnostic
provenance. See Q1 rationale.

### Q5: Cardinality's scope — does it subsume v2's Required/Optional?

**Answer: Yes. Cardinality unifies repetition and presence.**

v2's `constructors.dag` has `Required | Optional` as a flat enum on
binding sites. In v3's substrate:

- `Required` = `Cardinality { element: T, bound: Exact(1) }`.
- `Optional` = `Cardinality { element: T, bound: Unbounded }` with
  upper-bound 1 (or a dedicated `AtMost(1)` variant if needed).
- `List<T>` = `Cardinality { element: T, bound: Unbounded }`.
- `[T; n]` fixed array = `Cardinality { element: T, bound: Exact(n) }`.
- `argv: ["sh", "-lc", "{script}"]` = `Cardinality { element: String, bound: Exact(3) }`
  with children inlined (implementation detail).

M1(2.5) commits to two bound variants: `Exact(u32)` and `Unbounded`.
Range / AtMost / AtLeast are deferred until a real oracle needs them
(neither algebra.dag nor minimal shell.dag does).

**`Optional` is not a separate primitive.** It's `Cardinality` with a
specific bound. The surface syntax `field: T?` lowers to
`Cardinality { element: T, bound: AtMost(1) }`; when `AtMost` is added
later, this lowering is trivial.

### Q6: How do Arrow bodies cross into the computation substrate?

**Answer: Co-allocation in a single `Dag::nodes` table. NodeId
namespace is shared between type declarations and computation nodes.**

The M0 Dag struct already holds both `Node`s (L1 behaviors) and
declarations in related tables. M1(2.5) maintains this layout:

```rust
pub struct Dag {
    // Unified node table. Type declarations and computation Nodes
    // share the same NodeId space. Distinguish by connective
    // (declarations) vs behavior (computation nodes).
    declarations: Vec<Declaration>,
    nodes: Vec<Node>,              // L1 behavior nodes, unchanged from M0
    ports: Vec<Port>,              // unchanged from M0

    // Name resolution, diagnostics, etc. unchanged.
    named: HashMap<String, DeclarationId>,
    diagnostics: DiagnosticTable,
}
```

`Arrow::body: Option<NodeId>` points into `Dag::nodes` for user
functions. For primitives, `body` is None, and the emitter / interpreter
resolves the primitive implementation by name from `extdeps/` when
emitting.

`Transform::target: FunctionRef(DeclarationId)` points at the declaration
table. Inference uses the declaration's Arrow signature to type-check
the Transform; emission walks from the Transform through the
FunctionRef to the Arrow to its body.

**The "two substrates, two meeting points" framing holds logically.**
Physically, the two substrates live in adjacent tables of the same Dag
struct. No serialization or storage split is needed. "Two substrates"
is a conceptual separation, not a memory-layout one.

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
Monoid_op_arrow: Arrow { inputs: [T, T], output: T, body: None }
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
  connective: Instance {
    template: OrderedRing,
    bindings: [
      ParameterBinding {
        parameter: OrderedRing_T_param,
        value:     Word64,
      }
    ]
  }
```

Walking `Int.add`:

1. Start at `Int` — `Instance` connective.
2. Push `[T := Word64]` onto substitution stack.
3. Walk to template `OrderedRing` — `Conj` connective.
4. Find child `"add"` — points at `OrderedRing.add` which is an Arrow
   declaration.
5. Walk the Arrow's inputs: each is an Atom(Identifier) resolving to
   `OrderedRing_T_param`.
6. Look up `OrderedRing_T_param` in substitution stack → find `Word64`.
7. Substitute each input/output with `Word64`.
8. Yield `Arrow { inputs: [Word64, Word64], output: Word64, body: None }`.
9. Pop substitution frame on exit.

**Pure structural walk. No hardcoded knowledge of Int, Ring, add, or
Word64.** The compiler doesn't know what "add" means — it just finds
the child labeled "add" and returns its Arrow type with parameters
substituted.

**Substrate test `parse_std_algebra_and_walk_int_add` passes** when
this walk produces `Arrow(Word64, Word64, Word64)` as the type of
`Int.add` from an empty compiler state parsing only `std/algebra.dag`
+ the `Int = OrderedRing<Word64>` declaration.

## 6. `dsl/extdeps/shell.dag` minimal mapping (substrate test oracle #2)

The minimal subset for M1(2.5): `service shell.Exec { operation Run {
input { script: String }; output { ... }; transport shell { argv: [...] } } }`.
Drop `exit` and `mock_response` (they add Disj and nested Instance
complexity; add them in a follow-up).

```
shell.Exec:
  connective: Conj {
    children: [
      Field { label: "operations", ty: shell_Exec_operations }
    ]
  }

shell_Exec_operations:
  connective: Conj {
    children: [
      Field { label: "Run", ty: shell_Exec_Run }
    ]
  }

shell_Exec_Run:
  connective: Conj {
    children: [
      Field { label: "input",     ty: Run_input     },
      Field { label: "output",    ty: Run_output    },
      Field { label: "transport", ty: Run_transport },
    ]
  }

Run_input:
  connective: Conj {
    children: [
      Field { label: "script", ty: String_decl }  // resolves to std/String
    ]
  }

Run_output:
  connective: Conj {
    children: [
      Field { label: "exit_code", ty: Int_decl    },
      Field { label: "success",   ty: Bool_decl   },
      Field { label: "stdout",    ty: String_decl },
      Field { label: "stderr",    ty: String_decl },
    ]
  }

Run_transport:
  connective: Instance {
    template: transport_shell_template,
    bindings: [
      ParameterBinding { parameter: argv_param, value: Run_argv_literal }
    ]
  }

Run_argv_literal:
  connective: Cardinality {
    element: String_decl,
    bound:   Exact(3),
  }
  // children are three String literal Atoms — inline literal construction
```

Six levels of nesting:
1. service (Conj)
2. operations container (Conj)
3. operation Run (Conj)
4. input / output / transport members (Conj or Instance)
5. scalar fields or argv (Atom or Cardinality)
6. literal payloads (Atom)

Every level maps onto the six-connective shape. **No new `DeclKind`
variant is needed for service, operation, input, output, or transport.**
They're all Conj/Instance/Atom. The `transport shell { ... }` syntax
lowers to an Instance of the `transport_shell_template` declaration,
which itself is a Conj declared in `dsl/extdeps/transports/shell.dag`
(deferred to emission work).

**Substrate test `parse_shell_exec_run_minimal` passes** when parsing
this subset produces the above Declaration tree with no extension to
the connective enum.

## 7. Implementation plan

Focused, small, one PR. No scope creep.

### File-by-file

| File | Action | Est. time |
|---|---|---|
| `src/v3/compiler/src/dag.rs` | Replace `DeclKind::{Type, Function}` with the `TypeConnective` enum + `Declaration` / `Field` / `AtomPayload` / `CardinalityBound` / `ParameterBinding` data model from §3. Keep `DeclarationId`, `Dag`, bootstrap entry. | ~1.5h |
| `src/v3/compiler/src/parse.rs` | Parse `type Foo<T> { ... }` into Conj with TypeParam children. Parse `type Foo = Bar<X>` into Instance. Parse `fn(A, B) -> C` into Arrow. Drop the flat `fn name(a, b) -> c` primitive-signature shortcut. | ~2.5h |
| `src/v3/compiler/src/lower.rs` | Lower surface syntax into the new connective shape. Handle name resolution for Identifier atoms. Build substitution scopes. | ~1.5h |
| `src/v3/compiler/src/infer.rs` | Replace the old `DeclKind` dispatch with connective-pattern walks. Implement `SubstStack`. Inference now resolves `Int.add` by walking inhabits → template → field → substitute. | ~2h |
| `src/v3/compiler/src/tokenize.rs` | Minor — add `<` and `>` as type-parameter delimiters if not already present. | ~15m |
| `src/v3/compiler/tests/m0_acceptance.rs` | Update test helpers to build connective nodes. Semantics unchanged; all 40 M0 tests should pass against the new shape. | ~1h |
| `dsl/std/core.dag` | DELETE per-primitive function declarations (`fn std::int::add` etc.). They become inhabitance-derived. | ~5m |
| `dsl/std/algebra.dag` | Parsed by the bootstrap. No file changes needed — the file already has the right shape; it just hasn't been the source of primitive operations before. | ~0m |
| `src/v3/compiler/tests/m1_substrate_test.rs` | NEW. Two test cases: `parse_std_algebra_and_walk_int_add` and `parse_shell_exec_run_minimal`. | ~1.5h |

**Total estimate: 10–12 hours.** Single focused PR.

### Acceptance gates (CI must show all green)

1. **`cargo test -p v3-compiler`** — all 40 M0 tests pass unchanged.
2. **`parse_std_algebra_and_walk_int_add`** — parse `std/algebra.dag`,
   declare `Int = OrderedRing<Word64>`, walk `Int.add`, assert the
   result is `Arrow { inputs: [Word64, Word64], output: Word64 }`.
3. **`parse_shell_exec_run_minimal`** — parse the minimal shell.dag
   subset from §6, assert the Declaration tree structure matches.
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

## 8. Open sub-questions for pre-implementation review

These are small-but-real decisions that affect implementation
mechanics. Pinning them down before touching Rust is cheap; debating
them during code review is expensive.

### Q8.1: Identifier resolution timing

**Option A (lowering-time):** resolve all `Atom(Identifier { resolved:
None, .. })` to `Some(DeclarationId)` during `lower.rs`. Simpler; all
walks post-lowering see resolved references.

**Option B (lazy):** leave identifiers unresolved until inference
needs them. More like v2. Handles forward references naturally.

**Recommendation: Option A for M1(2.5).** Forward references are a
minor concern at this scale; lowering-time resolution simplifies
inference and matches M0's `Dag::new()` single-pass approach.

### Q8.2: Parameter child enumeration

**Option A:** a walker enumerates parameters by filtering `children`
for those whose target's connective is `Atom(TypeParam(_))`. No edge
metadata.

**Option B:** `Field` gains a `kind: FieldKind` field with variants
`Parameter | Content`. Cheaper enumeration, slightly more metadata.

**Recommendation: Option A.** The walk cost is negligible compared to
inference work, and not adding metadata keeps the substrate minimal.

### Q8.3: `SubstStack` data structure

**Option A:** `Vec<Vec<ParameterBinding>>` with linear lookup. Simple
but O(stack depth × bindings per frame) per lookup.

**Option B:** `Vec<HashMap<DeclarationId, DeclarationId>>` per frame.
O(1) lookup, more allocation overhead.

**Recommendation: Option A for M1(2.5).** Typical stack depth is 1–3,
typical bindings per frame is 1–2, so linear scan is ~6 comparisons
max — cheaper than hashing. Revisit if profiling shows it's hot.

### Q8.4: `Cardinality` parse syntax

**Option A:** `List<T>` parses to `Cardinality { element: T, bound:
Unbounded }`. `[T; 3]` parses to `Cardinality { element: T, bound:
Exact(3) }`. Surface syntax covers both.

**Option B:** only `List<T>` at M1(2.5), add fixed-size later.

**Recommendation: Option A.** `argv: ["sh", "-lc", "{script}"]` in
shell.dag needs Exact(3) for its argv literal. Both forms are one
parse rule each.

### Q8.5: Where to parse `std/algebra.dag` during bootstrap

**Option A:** parse at `Dag::new()` synchronously, before any user
source. Matches M0's `bootstrap_primitives` pattern.

**Option B:** parse lazily on first reference. More like real-world
compiler imports.

**Recommendation: Option A.** `std/algebra.dag` is the compiler's
primitive-operation source; it must be loaded before any user code
that uses arithmetic/comparison/etc. Synchronous bootstrap is the
simplest pattern.

## 9. What "done" looks like for M1(2.5)

- The five files in §7 are updated; the two test files are new.
- `cargo test -p v3-compiler` shows 40 M0 tests + 2 new substrate
  tests = 42 green.
- `cargo clippy` is clean.
- `src/v3/compiler/src/dag.rs` contains no `DeclKind::{Type, Function}`
  references. `TypeConnective` has exactly the six variants in §3.
- `Int.add` resolves through inhabitance (no hardcoded signature).
- `shell.Exec.Run` (minimal subset) parses and type-checks.
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
