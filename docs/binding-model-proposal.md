> Part of: [THESIS.md](../THESIS.md) > **Correctness dimensions** + **Binding unification**
> See also: [dag-vocabulary-reconciliation.md](dag-vocabulary-reconciliation.md),
> [binding-unification-design.md](binding-unification-design.md),
> [dimensions-design.md](../src/v2/dimensions-design.md)

# Proposed modeling: edges, bindings, dimensions

The minimal edge vocabulary and its consequences for binding
unification and correctness dimensions. These are **proposals** —
types land in `dsl/std/` when implementation begins.

---

## The edge vocabulary already exists

A binding is a name attached to a DAG edge. The edge's value
relationship is classified by `SubValueRelation` (std/induction.dag),
which already exists and already does the right thing:

```dag
// std/induction.dag — the edge vocabulary
type SubValueRelation
  = StrictSubValue { field: InductiveField, factor: ShrinkFactor }
  | IteratedSubValue { field: InductiveField }
  | ArithmeticDescent { param: String, factor: ShrinkFactor }
  | PreservedValue
  | SubValueUnknown
```

This type classifies how a target value relates to a source value
along an edge. It already has a BoundedLattice (meet/join), a
composition function (`compose_sub_value`), and is carried on
`TypeBinding.provenance`.

The only genuinely new edge concept is `UsageEdge` — what happens
to the value at each use site (orthogonal to SVR):

```dag
// Proposed — belongs in std/ownership.dag
type UsageEdge
  = Consumed   // moved / ownership transferred
  | Read       // read but not consumed
  | Projected  // accessed via field / index
  | Threaded   // passed through unchanged (fold accumulator)
```

Everything else — binding form, caller context, access shape —
is a derived view, not a separate type.

---

## Binding forms: derived from edge position + SVR

The 7 → 2 binding unification is not a new type. It's a question
about the edge:

**"Is the SVR provided by the caller, or derived from the expression?"**

- **Parameter** (caller provides SVR): the binding sits in `params`
  — a boundary-crossing position. fold passes `IteratedSubValue`,
  descend passes `StrictSubValue`, plain call passes
  `PreservedValue`. The parameter reads what it receives.

- **Let-binding** (expression determines SVR): the binding sits in
  `body`/`children` — a local position. The SVR is derived from
  the expression: field access → `StrictSubValue`, method call on
  collection → `IteratedSubValue`, arithmetic → `ArithmeticDescent`,
  construction → `SubValueUnknown`.

This distinction is derivable from which Node field the binding
occupies — a structural fact about position in the tree, not a
string comparison. The compiler already knows which field it's
processing when it walks `Node.params` vs `Node.body` vs
`Node.children`; the walk context carries this structurally.

### Surface metadata (optional, for diagnostics)

```dag
type BindingSurface
  = FunctionParam | LambdaParam | ForEachVariable
  | LetDeclaration | MatchArmBind | BlockLetBind
```

No downstream code branches on this. Exists for error messages
("in match arm for Foo") and idiomatic emission (emit for-each
as for-loop). Does not affect the edge model.

### Iteration contracts: which SVR the primitive provides

fold/descend/repeat are not special types — they're call sites
that provide specific SubValueRelations to their body parameters:

| Primitive | Parameter | SVR provided |
|-----------|-----------|-------------|
| `fold(coll, init, (acc, elem) => ...)` | `acc` | `PreservedValue` |
| `fold(coll, init, (acc, elem) => ...)` | `elem` | `IteratedSubValue` |
| `descend(node, (child) => ...)` | `child` | `StrictSubValue` |
| `repeat(n, init, (i, acc) => ...)` | `acc` | `PreservedValue` |
| `repeat(n, init, (i, acc) => ...)` | `i` | `SubValueUnknown` |
| `for x in coll { ... }` (desugars to fold) | `x` | `IteratedSubValue` |

No `IterationBinding` type needed. The contract is: "what SVR does
this call site attach to each parameter edge?"

---

## Dimension computation: SVR-keyed table

Every correctness dimension computes a value from the
SubValueRelation on the binding's edge:

| SubValueRelation | Provenance | Ownership | Descent |
|---|---|---|---|
| `PreservedValue` | PreservedValue | source | NonIncreasing |
| `StrictSubValue` | StrictSubValue | Borrowed | Strict |
| `IteratedSubValue` | IteratedSubValue | Owned | Strict |
| `ArithmeticDescent` | ArithmeticDescent | Owned | Strict |
| `SubValueUnknown` | SubValueUnknown | Owned (fresh) or Shared (unknown) | DescentUnknown |

One row per SVR variant. One column per dimension. Adding a
dimension = one column. Adding a value relationship = one row.

The boundary-crossing question determines WHO provides the SVR:
- Parameter: caller provides SVR (read from call site)
- Let-binding: derive SVR from expression

It does not change what the SVR IS or how dimensions compute from it.

This replaces the triple classification system in the compiler:
- `classify_binding_provenance` (04_infer.dag ~2616)
- `classify_let_value` / `classify_argument` (04_infer.dag ~2649-3104)
- `classify_body_provenance` (04_infer.dag ~3851)
- `classify_self_call_evidence` (complexity.dag ~2535)

All four reconstruct SubValueRelation from AST structure. With
SVR carried on edges, they become thin readers.

---

## Ownership as dimension

```dag
// Proposed — std/ownership.dag
type OwnershipKind = Owned | Borrowed | Shared

// BoundedLattice: bottom = Shared, top = Owned, meet = min
fn ownership_meet(a: OwnershipKind, b: OwnershipKind) -> OwnershipKind
fn ownership_join(a: OwnershipKind, b: OwnershipKind) -> OwnershipKind
data ownership_bottom: OwnershipKind = Shared
data ownership_top: OwnershipKind = Owned
```

Ownership is computed from SVR on the edge (per the table above)
plus `UsageEdge` at use sites (for fan-out / last-use analysis).

### Fold accumulator

Inside a fold body, the accumulator is `Owned` per iteration (the
fold primitive guarantees unique access). This is a property of the
fold's calling convention (`PreservedValue` SVR + `Threaded`
UsageEdge), not a separate `AccumulatorOwnership` type.

---

## Open design questions

1. **`StrictSubValue` ownership** — the table says `Borrowed`. But
   if the source struct is being moved (destructured), the field
   could be `Owned`. May need to condition on whether the source
   is consumed: `StrictSubValue + source Consumed → Owned`,
   `StrictSubValue + source Read → Borrowed`.

2. **`SubValueUnknown` ownership** — says `Owned (fresh) or Shared
   (unknown)`. These are different cases: a newly constructed value
   is Owned, but an unknown-provenance value should be Shared
   (conservative). May need to distinguish "fresh unknown" from
   "genuinely unknown." Or: `SubValueUnknown` always → `Shared`
   (bottom), and construction gets a different SVR variant.

3. **Repeat counter** — the counter `i` has `SubValueUnknown`
   because it's not a sub-value of any input. But it IS a known
   bounded value (0..n). Does this matter for any dimension, or
   is `SubValueUnknown` correct?

---

## Status

**Proposal.** Grounded in SubValueRelation as the existing edge
vocabulary. UsageEdge is the only new type. Binding forms derived
from edge position. Dimension table is SVR-keyed. 7 redundant
coproducts from earlier drafts eliminated.

See [dag-vocabulary-reconciliation.md](dag-vocabulary-reconciliation.md)
for the full type inventory and conflict catalog.
