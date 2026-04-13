> Part of: [THESIS.md](../THESIS.md) > **Architecture**
> See also: [binding-model-proposal.md](binding-model-proposal.md), [ROADMAP.md](../ROADMAP.md)

# DAG Vocabulary Reconciliation

Full accounting of the compiler/DAG modeling vocabulary. Every
concept, where it lives, what conflicts exist, and how to reconcile
into a coherent layered vocabulary rooted in the DAG's own edge types.

---

## The gap: no explicit edge vocabulary

The DAG has two substrate primitives: **Node** and **Edge**. But only
Node is modeled. Edges are implicit — they're whatever field a child
Node sits in:

```
Node.children: List<Node>      — structural containment
Node.params: List<Node>        — input ports
Node.uses: List<Node>          — dependencies
Node.body: Node?               — computation definition
Node.transport: Node?          — execution strategy
Node.properties: List<Node>    — metadata
Node.type_annotation: Node?    — type assertion
```

There is no `type Edge` anywhere. The relationship kind is determined
by WHICH FIELD the reference sits in, not by an explicit type on the
edge. This means every consumer that needs to know "what kind of
relationship is this" must reconstruct it from field position —
exactly the construct-discard-reconstruct pattern the thesis rejects.

`NodeFieldRole` (00_core.dag line 855) partially classifies fields,
but only for descent analysis — it says whether a field is recursive,
not what the relationship means:

```dag
type NodeFieldRole
  = ChildrenListField    // structural recursion carrier
  | SubValueField        // proper sub-value
  | MetadataField        // non-recursive
```

---

## Layer 0: Proposed DAG edge vocabulary (`std/edge.dag`)

Every relationship in the DAG should have an explicit edge kind.
The edge vocabulary has three orthogonal classifications:

### Structural edge kind (from Node fields)

What structural role does this edge play?

```dag
// StructuralEdge: which Node field this relationship occupies.
// This is the explicit version of the implicit field-position
// classification that every consumer currently reconstructs.
type StructuralEdge
  = Containment      // children: structural sub-nodes (AND/OR branches)
  | InputPort        // params: values flowing in from outside
  | ComputationBody  // body: how this node computes its value
  | ExecutionBinding // transport: external execution strategy
  | ResourceDep      // uses: resources/services this node depends on
  | TypeAssertion    // type_annotation: declared type constraint
  | Metadata         // properties: non-structural annotations
```

This replaces `NodeFieldRole` (which classifies by descent behavior)
with a classification by relationship semantics. Descent behavior
(`ChildrenListField`, `SubValueField`, `MetadataField`) becomes a
DERIVABLE property of the structural edge kind:

```
Containment    → ChildrenListField (always recursive)
InputPort      → SubValueField (always sub-value of parent scope)
ComputationBody → SubValueField (body is sub-value of function)
ExecutionBinding → MetadataField (not recursive structure)
ResourceDep    → MetadataField (dependency, not containment)
TypeAssertion  → MetadataField (annotation, not containment)
Metadata       → MetadataField
```

### Value-flow edge kind (from expression semantics)

How does a value flow through this edge? This is the concept that
`BindingForm`, `CallerContext`, and `AccessShape` from the binding
proposal all project from.

```dag
// ValueFlow: how a value moves through the DAG.
//
// Every binding introduces a name. The value behind that name
// arrives via one of these flows:
type ValueFlow
  = BoundaryCrossing   // value flows across a scope boundary (Parameter)
  | LocalNaming        // value named within current scope (LetBinding)

// BoundaryContext: for BoundaryCrossing edges, what the provider
// side contributes. This is the explicit version of what inference
// currently reconstructs from method names ("fold", "map", etc.).
type BoundaryContext
  = CollectionElement  // fold/map: element of iterated collection
  | TreeChild          // descend: structural child of tree node
  | ContractProvided   // higher-order: callee contract specifies
  | DirectPass         // plain argument: value = caller's value
  | Unspecified        // no context available (fail-closed)

// NamingShape: for LocalNaming edges, how the named value relates
// to its source expression. This is the explicit version of what
// classify_binding_provenance reconstructs from AST walking.
type NamingShape
  = DirectAlias        // let x = y — same value, new name
  | FieldProjection    // let x = y.field — structural descent
  | ElementExtraction  // let x = list element — iteration
  | ArithmeticStep     // let x = n - 1 — arithmetic descent
  | FreshConstruction  // let x = { ... } — new value
```

### Projections: how existing concepts derive from edges

With these edge kinds, every existing classification becomes a
projection rather than an independent concept:

| Existing concept | Defined in | Projects from |
|-----------------|-----------|---------------|
| `BindingForm = Parameter \| LetBinding` | (proposed) binding.dag | `ValueFlow` |
| `CallerContext` | (proposed) binding.dag | `BoundaryContext` |
| `AccessShape` | (proposed) binding.dag | `NamingShape` |
| `NodeFieldRole` | 00_core.dag | `StructuralEdge` (descent derivation) |
| `VarBindingKind` | 00_core.dag | `ValueFlow` + `StructuralEdge` |
| `FieldAccessStyle` | 00_core.dag | `NamingShape` (for FieldProjection cases) |

### Use-site classification (orthogonal to flow)

How a value is consumed at each reference point. This already
exists as `EdgeKind` in `src/v2/ownership.dag` but belongs in
the shared vocabulary:

```dag
// UsageEdge: what happens to the value at a use site.
// Orthogonal to ValueFlow (how the value arrived) and
// OwnershipKind (what the binding is allowed to do).
type UsageEdge
  = Consumed   // moved / ownership transferred (last use candidate)
  | Read       // read but not consumed (borrow candidate)
  | Projected  // accessed via field / index (structural borrow)
  | Threaded   // passed through unchanged (fold accumulator)
```

This unifies `EdgeKind` from `src/v2/ownership.dag` with the
proposed `UsageEdge` — same concept, one location.

---

## Known conflicts and their resolutions

### 1. Cardinality: `Optional` vs `CardOptional`

**Conflict:** `dsl/std/constructors.dag` line 75 declares
`Cardinality = Required | Optional`. `src/v2/00_core.dag` line 103
declares `Cardinality = Required | CardOptional`. Same concept,
different variant name. ~118 references across .dag + stage0.

**Resolution:** Rename `CardOptional` to `Optional` in 00_core.dag
and all consumers. This is a bootstrap-level rename (touches stage0)
and should be its own PR with a regen cycle.

**Why `CardOptional` exists:** Likely a collision avoidance during
bootstrap — `Optional` as a bare name may have conflicted with
the `Optional` type/wrapper elsewhere. The fix is to use the
qualified form or resolve the collision.

### 2. Connective vs Product/Coproduct

**Conflict:** `00_core.dag` line 101 defines
`Connective = Conj | Disj | NoConnective | Arrow`.
`dsl/std/constructors.dag` defines `Product` and `Coproduct` as
the canonical structural axioms. The dissolution path is noted
in constructors.dag comments (line 54-58): "when the compiler reads
Product/Coproduct from std declarations, the Connective enum
dissolves." 313 sites read `.connective` and match on `Conj`/`Disj`.

**Resolution:** Multi-step dissolution:
1. Import `Product`/`Coproduct` into 00_core.dag
2. Add a `connective_to_constructor` mapping function
3. Migrate consumers incrementally (313 sites)
4. Delete `Connective` when all consumers migrated

This is a large mechanical refactor. Track separately.

### 3. SizeExpr name collision

**Conflict:** Two different types named `SizeExpr`:
- `complexity.dag` line 97: 5-variant symbolic size algebra
  (`Const | Var | Len | Add | Max`), 18 references
- `04_infer.dag` line 2306: 2-variant descent tracking
  (`ParamSize | DividedSize`), 3 references

**Resolution:** Rename the smaller one. `04_infer.dag`'s `SizeExpr`
becomes `DescentSizeAlias` (it tracks proportional-shrink aliases
during descent annotation). 3 references to update + stage0 regen.

### 4. TextFile schema divergence

**Conflict:** `dsl/std/types.dag` defines
`TextFile { path: FilePath, document: Document }` (structured).
`src/v2/00_core.dag` line 258 defines
`TextFile { path: FilePath, content: String }` (flat string).

**Resolution:** The compiler's TextFile is an emission artifact
(flat string content). The std TextFile is a domain model
(structured document). These are different concepts that share a
name. Rename the compiler's to `EmittedFile` or similar.

### 5. VarBindingKind dissolution

**Not a conflict but a redundancy.** `VarBindingKind` in 00_core.dag
classifies variable references by scope level + syntax:

```dag
type VarBindingKind
  = LocalValueBinding       // local let/block let
  | FunctionValueBinding    // refers to a function
  | VariantValueBinding     // refers to a variant constructor
  | MatchBoundBinding       // bound in a match arm
```

With the edge vocabulary, this dissolves:
- `LocalValueBinding` → `ValueFlow = LocalNaming`
- `FunctionValueBinding` → `ValueFlow = BoundaryCrossing` (function
  is provided from enclosing scope)
- `VariantValueBinding` → not a value flow — it's a constructor
  reference (structural, not a binding)
- `MatchBoundBinding` → `ValueFlow = LocalNaming` with
  `NamingShape = FieldProjection` (match arm is a destructuring let)

The 4-variant coproduct becomes 2 cases of `ValueFlow` plus
constructor references (a separate concept). This is the same
7→2 reduction applied to variable REFERENCES instead of variable
INTRODUCTIONS.

---

## Concept DAG: the full layering

```
Layer -1: Structural axioms
  constructors.dag: Product, Coproduct, Cardinality

Layer 0: DAG edge vocabulary (PROPOSED — std/edge.dag)
  StructuralEdge: what role this edge plays in the DAG
  ValueFlow: how values move through edges (BoundaryCrossing | LocalNaming)
  BoundaryContext: what the provider contributes across boundaries
  NamingShape: how a named value relates to its source
  UsageEdge: what happens to the value at each use site

Layer 1: Dimensions as lattices on edges (std/)
  algebra.dag:      BoundedLattice<T> (the generic mechanism)
  induction.dag:    SubValueRelation   (provenance dimension)
  termination.dag:  DescentEvidence    (descent dimension)
  effects.dag:      EffectShape        (effect dimension)
  ownership.dag:    OwnershipKind      (ownership dimension) [PROPOSED]

  Each dimension:
    DECLARE: lattice in std/ (the vocabulary)
    COMPUTE: at each edge, keyed on ValueFlow + NamingShape/BoundaryContext
    CARRY:   through the DAG on bindings or dimension tables
    ENFORCE: at consumption points (gates, diagnostics)

Layer 2: Compiler IR (src/v2/00_core.dag)
  Node: the universal recursive DAG node
  ExprData: 22 expression forms (surface syntax)
  Connective → dissolves into Product/Coproduct
  VarBindingKind → dissolves into ValueFlow projections
  NodeFieldRole → derived from StructuralEdge
  ChildRole, expr_child_roles, etc. → derived from typed edges

Layer 3: Pipeline types (compiler-specific)
  TypeEnv, TypeBinding, InferScope, EmitGraphInfo, etc.
  Reference Layer 0/1 vocabulary at boundaries.
  Internal fold/accum types stay compiler-local.
```

### Dimension computation keyed on edges

Every dimension follows the same pattern, keyed on the edge
vocabulary:

| ValueFlow | Key | Dimension value |
|-----------|-----|-----------------|
| BoundaryCrossing | BoundaryContext | Caller provides the dimension value |
| LocalNaming | NamingShape | Compose source dimension with shape |

Per-dimension composition for LocalNaming:

| NamingShape | Provenance | Ownership | Descent | Effect |
|-------------|-----------|-----------|---------|--------|
| DirectAlias | source | source | source | source |
| FieldProjection | compose_sub_value | Borrowed | structural_descent | source |
| ElementExtraction | IteratedSubValue | Owned | IteratedSubValue | source |
| ArithmeticStep | ArithmeticDescent | Owned | ArithmeticDescent | Pure |
| FreshConstruction | SubValueUnknown | Owned | DescentUnknown | from body |

This is the SINGLE TABLE that replaces per-dimension, per-syntax-form
code paths. Every cell is derivable from the edge vocabulary +
the dimension's lattice operations.

---

## Reconciliation execution order

Bottom-up, each a separate reviewable PR:

1. **DAG edge vocabulary proposal** — this document (DONE)
2. **Cardinality unification** — rename `CardOptional` → `Optional`
   (~118 sites + stage0 regen). Own PR.
3. **SizeExpr disambiguation** — rename infer's `SizeExpr` →
   `DescentSizeAlias` (~3 sites + stage0 regen). Own PR.
4. **TextFile disambiguation** — rename compiler's `TextFile` →
   `EmittedFile`. Own PR.
5. **Connective dissolution** — multi-step migration of 313 sites.
   Tracked separately.
6. **VarBindingKind dissolution** — replace with ValueFlow
   projections. Depends on edge vocabulary landing.
7. **String table dissolution** — derive `expr_child_roles`,
   `node_field_roles`, `function_size_effects` from typed edges.
   Depends on edge vocabulary landing.

Items 1-4 are independent and can proceed in parallel.
Items 5-7 depend on the edge vocabulary being implemented.

---

## Status

**Proposal.** This document is the accounting and reconciliation
plan. Edge vocabulary types will land in `dsl/std/edge.dag` when
implementation begins. Conflict resolutions (Cardinality, SizeExpr,
TextFile) are mechanical and can proceed as independent PRs.
