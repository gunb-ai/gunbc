> Part of: [THESIS.md](../THESIS.md) > **Correctness dimensions** + **Binding unification**
> See also: [dag-vocabulary-reconciliation.md](dag-vocabulary-reconciliation.md),
> [binding-unification-design.md](binding-unification-design.md),
> [dimensions-design.md](../src/v2/dimensions-design.md)

# Proposed .dag modeling: edges, bindings, dimensions

This document proposes the structural `.dag` types for the DAG edge
vocabulary, binding unification (7 → 2), and correctness dimensions
(provenance, ownership). These are **proposals** — the types will
land in `dsl/std/` when the corresponding implementation work begins.

The key insight: **bindings are not a separate concept from edges.**
A binding is a name attached to a DAG edge. The binding's form
(parameter vs let-binding) is a projection of the edge's value-flow
kind. All downstream concepts (provenance, ownership, access shape)
derive from the edge vocabulary, not from independent classifications.

See [dag-vocabulary-reconciliation.md](dag-vocabulary-reconciliation.md)
for the full accounting of existing concepts and conflicts.

---

## 0. DAG edge vocabulary (`std/edge.dag` — proposed new file)

This is the **foundation** from which everything else derives. The
DAG today has Node but no explicit Edge type. Edges are implicit in
Node fields. This proposal makes them explicit.

External authority: directed acyclic graphs (graph theory). An edge
is an ordered pair (source, target) with a kind that classifies the
relationship.

### Structural edges

```dag
// StructuralEdge: which Node field this relationship occupies.
// Replaces the implicit field-position classification.
//
// Concept DAG attachment:
//   constructors.dag (Product, Coproduct)
//     └── edge.dag — edges connect Products/Coproducts in the DAG
type StructuralEdge
  = Containment      // children: structural sub-nodes (AND/OR branches)
  | InputPort        // params: values flowing in from outside scope
  | ComputationBody  // body: how this node computes its value
  | ExecutionBinding // transport: external execution strategy
  | ResourceDep      // uses: resources/services this node depends on
  | TypeAssertion    // type_annotation: declared type constraint
  | Metadata         // properties: non-structural annotations
```

`NodeFieldRole` (currently in 00_core.dag) becomes a derivable
function, not an independent classification:

```dag
fn structural_descent_role(edge: StructuralEdge) -> NodeFieldRole {
  match edge {
    Containment      => ChildrenListField
    InputPort        => SubValueField
    ComputationBody  => SubValueField
    ExecutionBinding => MetadataField
    ResourceDep      => MetadataField
    TypeAssertion    => MetadataField
    Metadata         => MetadataField
  }
}
```

### Value-flow edges

```dag
// ValueFlow: how a value moves through a DAG edge.
//
// Every binding introduces a name. The value behind that name
// arrives via one of these two flows. This is the DAG-native
// concept that BindingForm projects from.
//
// External authority: lambda calculus — abstraction (boundary
// crossing) and let-binding (local naming).
type ValueFlow
  = BoundaryCrossing   // value flows across a scope boundary
  | LocalNaming        // value named within current scope
```

### Boundary context (for BoundaryCrossing edges)

```dag
// BoundaryContext: what the provider side contributes across
// a scope boundary. This is the DAG-native concept that
// CallerContext projects from.
//
// This is a property of the CALL SITE edge, not the binding
// at the target. A lambda parameter reads whatever the caller
// provides — it doesn't need to know the context.
type BoundaryContext
  = CollectionElement  // fold/map: element is sub-value of collection
  | TreeChild          // descend: child is strict sub-value of parent
  | ContractProvided   // higher-order: callee contract specifies
  | DirectPass         // plain argument: value = caller's value
  | Unspecified        // no context available (fail-closed to bottom)
```

### Naming shape (for LocalNaming edges)

```dag
// NamingShape: how a locally-named value relates to its source
// expression. This is the DAG-native concept that AccessShape
// projects from.
type NamingShape
  = DirectAlias        // let x = y — same value, new name
  | FieldProjection    // let x = y.field — structural descent into source
  | ElementExtraction  // let x = elem of collection — iteration
  | ArithmeticStep     // let x = n - 1 — arithmetic descent
  | FreshConstruction  // let x = { ... } — newly constructed value
```

### Use-site edges

```dag
// UsageEdge: what happens to the value at each reference point.
// Orthogonal to ValueFlow (how the value arrived).
//
// Unifies EdgeKind from src/v2/ownership.dag with the shared
// vocabulary — same concept, one location.
type UsageEdge
  = Consumed   // moved / ownership transferred (last use candidate)
  | Read       // read but not consumed (borrow candidate)
  | Projected  // accessed via field / index (structural borrow)
  | Threaded   // passed through unchanged (fold accumulator)
```

---

## 1. Binding forms — projections of edge vocabulary

`BindingForm`, `CallerContext`, and `AccessShape` from the original
binding-unification analysis are NOT independent types. They are
**projections** of the edge vocabulary:

| Binding concept | Projects from | Relationship |
|----------------|---------------|-------------|
| `BindingForm = Parameter \| LetBinding` | `ValueFlow` | `Parameter ≡ BoundaryCrossing`, `LetBinding ≡ LocalNaming` |
| `CallerContext` (5 variants) | `BoundaryContext` | Same concept, same variants |
| `AccessShape` (5 variants) | `NamingShape` | Same concept, same variants |

Whether the implementation uses the edge-vocabulary names directly
(`ValueFlow`, `BoundaryContext`, `NamingShape`) or keeps the
binding-specific projections (`BindingForm`, `CallerContext`,
`AccessShape`) as aliases is an implementation choice. The key
requirement: **one authority, not parallel classifications.**

### Surface syntax (metadata, not branching)

```dag
// BindingSurface: the original syntactic form.
// Downstream code does NOT branch on this.
type BindingSurface
  = FunctionParam      // → BoundaryCrossing
  | LambdaParam        // → BoundaryCrossing
  | ForEachVariable    // → BoundaryCrossing (desugars to fold)
  | LetDeclaration     // → LocalNaming
  | MatchArmBind       // → LocalNaming (desugars to destructure)
  | BlockLetBind       // → LocalNaming

fn value_flow(surface: BindingSurface) -> ValueFlow {
  match surface {
    FunctionParam   => BoundaryCrossing
    LambdaParam     => BoundaryCrossing
    ForEachVariable => BoundaryCrossing
    LetDeclaration  => LocalNaming
    MatchArmBind    => LocalNaming
    BlockLetBind    => LocalNaming
  }
}
```

### Iteration binding contracts

```dag
// IterationBinding: how fold/descend/repeat create BoundaryCrossing
// edges into their bodies.
type IterationBinding
  = FoldAccumulator  { cardinality: Cardinality }
  | FoldElement
  | DescendChild
  | RepeatCounter
  | RepeatAccumulator

fn iteration_boundary_context(ib: IterationBinding) -> BoundaryContext {
  match ib {
    FoldAccumulator { cardinality: _ } => DirectPass
    FoldElement                        => CollectionElement
    DescendChild                       => TreeChild
    RepeatCounter                      => Unspecified
    RepeatAccumulator                  => DirectPass
  }
}
```

### VarBindingKind dissolution

`VarBindingKind` (00_core.dag) dissolves into `ValueFlow`:
- `LocalValueBinding` → `LocalNaming`
- `FunctionValueBinding` → `BoundaryCrossing`
- `VariantValueBinding` → not a value flow (constructor reference)
- `MatchBoundBinding` → `LocalNaming` + `NamingShape = FieldProjection`

---

## 2. Dimension computation — keyed on edges

Every correctness dimension follows the same pattern, keyed on the
edge vocabulary:

### The generic interface

```
For a dimension D with BoundedLattice<D>:

  BoundaryCrossing edge:
    D(binding) = what the caller provides via BoundaryContext
    The binding reads what it receives.

  LocalNaming edge:
    D(binding) = compose_D(source_D, naming_shape)
    The composition is dimension-specific.

  When composition fails: D = D.bottom (fail-closed).
```

### Per-dimension composition table

| NamingShape | Provenance (SVR) | Ownership | Descent | Effect |
|-------------|-----------------|-----------|---------|--------|
| DirectAlias | source | source | source | source |
| FieldProjection | compose_sub_value | Borrowed | structural_descent | source |
| ElementExtraction | IteratedSubValue | Owned | IteratedSubValue | source |
| ArithmeticStep | ArithmeticDescent | Owned | ArithmeticDescent | Pure |
| FreshConstruction | SubValueUnknown | Owned | DescentUnknown | from body |

| BoundaryContext | Provenance (SVR) | Ownership | Descent |
|-----------------|-----------------|-----------|---------|
| CollectionElement | IteratedSubValue | Owned | IteratedSubValue |
| TreeChild | StrictSubValue | Owned | Strict |
| ContractProvided | from DeclaredFuncSig | from contract | from contract |
| DirectPass | PreservedValue | Owned (caller transfers) | NonIncreasing |
| Unspecified | SubValueUnknown | Shared (bottom) | DescentUnknown |

This is the SINGLE TABLE that replaces per-dimension, per-syntax-form
code paths in the compiler. Adding a new dimension means adding one
column. Adding a new NamingShape or BoundaryContext means adding one
row. The cost of change is O(1).

### Provenance (SubValueRelation) — already exists

`std/induction.dag` already defines SubValueRelation and the
composition function `compose_sub_value`. The table above documents
how to call it keyed on the edge vocabulary. This replaces:
- `classify_binding_provenance` (04_infer.dag ~2616)
- `classify_let_value` / `classify_argument` (04_infer.dag ~2649-3104)
- `classify_body_provenance` (04_infer.dag ~3851)
- `classify_self_call_evidence` (complexity.dag ~2535)

### Ownership (OwnershipKind) — proposed

```dag
// OwnershipKind: how exclusively a binding holds its value.
// BoundedLattice: bottom = Shared, top = Owned, meet = min.
type OwnershipKind = Owned | Borrowed | Shared

fn ownership_at_naming(source: OwnershipKind, shape: NamingShape) -> OwnershipKind {
  match shape {
    DirectAlias       => source
    FieldProjection   => Borrowed
    ElementExtraction => Owned
    ArithmeticStep    => Owned
    FreshConstruction => Owned
  }
}
```

### Fold accumulator contract

```dag
type AccumulatorOwnership
  = ThreadedOwned    // fold guarantees unique access per iteration
  | ThreadedShared   // cannot guarantee (aliased in body)
```

---

## Open design questions

1. **FieldProjection ownership** — proposed as `Borrowed`
   unconditionally. But if the source struct is being moved
   (destructured), the field could be `Owned`. May need:
   `FieldProjection + Owned source → Owned` (destructuring move)
   vs `FieldProjection + Borrowed source → Borrowed` (borrow).

2. **RepeatCounter boundary context** — proposed as `Unspecified`.
   The counter is a known bounded arithmetic sequence (0..n).
   Should this be a new variant, or is `Unspecified` correct
   because the counter is not a sub-value of any input?

3. **ElementExtraction ownership** — proposed as `Owned`
   unconditionally. But if the collection is borrowed, the element
   may also be borrowed.

4. **Edge vocabulary as types vs metadata** — should `StructuralEdge`
   be a field on Node (explicit edge kind per child), or remain
   implicit in the field position? Explicit is cleaner but changes
   the Node type. Implicit preserves backward compatibility but
   requires the projection function.

---

## Status

**Proposal.** Grounded in the DAG edge vocabulary from
[dag-vocabulary-reconciliation.md](dag-vocabulary-reconciliation.md).
Types land in `dsl/std/` when implementation begins. The
reconciliation doc tracks all known conflicts and their resolutions.

Next steps:
- Theme 1: implement edge vocabulary → binding unification → CX gate
- Conflict PRs: Cardinality, SizeExpr, TextFile (independent)
- Connective dissolution (313 sites, tracked separately)
