> Part of: [THESIS.md](../THESIS.md) > **Architecture**
> See also: [binding-model-proposal.md](binding-model-proposal.md), [ROADMAP.md](../ROADMAP.md)

# DAG Vocabulary Reconciliation

Full accounting of the compiler/DAG modeling vocabulary. Every
concept, where it lives, what conflicts exist, and the minimal
edge vocabulary needed.

---

## Key finding: SubValueRelation IS the edge vocabulary

The DAG edge vocabulary is not missing — it already exists as
`SubValueRelation` in `std/induction.dag`. It classifies how a
target value relates to a source value along an edge:

```dag
type SubValueRelation
  = StrictSubValue { field: InductiveField, factor: ShrinkFactor }
  | IteratedSubValue { field: InductiveField }
  | ArithmeticDescent { param: String, factor: ShrinkFactor }
  | PreservedValue
  | SubValueUnknown
```

This type already has a `BoundedLattice` (meet/join), composition
(`compose_sub_value`), and is carried on `TypeBinding.provenance`.
It serves as both the provenance dimension AND the structural edge
classification.

An earlier draft of this document proposed 9 new coproducts with
39 total variants (`StructuralEdge`, `ValueFlow`, `BoundaryContext`,
`NamingShape`, `IterationBinding`, `AccumulatorOwnership`, etc.).
On scrutiny, almost all of them were renamings of SubValueRelation:

| Proposed type | Is actually |
|---|---|
| `NamingShape.DirectAlias` | `SubValueRelation.PreservedValue` |
| `NamingShape.FieldProjection` | `SubValueRelation.StrictSubValue` |
| `NamingShape.ElementExtraction` | `SubValueRelation.IteratedSubValue` |
| `NamingShape.ArithmeticStep` | `SubValueRelation.ArithmeticDescent` |
| `NamingShape.FreshConstruction` | `SubValueRelation.SubValueUnknown` |
| `BoundaryContext.CollectionElement` | `SubValueRelation.IteratedSubValue` |
| `BoundaryContext.TreeChild` | `SubValueRelation.StrictSubValue` |
| `BoundaryContext.DirectPass` | `SubValueRelation.PreservedValue` |
| `BoundaryContext.Unspecified` | `SubValueRelation.SubValueUnknown` |
| `ValueFlow.BoundaryCrossing` | Derivable from Node field position |
| `ValueFlow.LocalNaming` | Derivable from Node field position |

The only genuinely new concept is `UsageEdge` — what happens to
the value at each use site. This is orthogonal to SubValueRelation
(which describes the relationship along the edge, not what the
consumer does with it).

---

## Minimal edge vocabulary: 2 concepts

### 1. SubValueRelation (already exists — `std/induction.dag`)

How the target value relates to the source value along an edge.
This is the provenance dimension AND the structural edge classifier.

Already modeled, already has lattice operations, already carried
on bindings. No new type needed.

### 2. UsageEdge (new — belongs in `std/ownership.dag`)

What happens to the value at each use site. Orthogonal to SVR.

```dag
type UsageEdge
  = Consumed   // moved / ownership transferred (last use candidate)
  | Read       // read but not consumed (borrow candidate)
  | Projected  // accessed via field / index (structural borrow)
  | Threaded   // passed through unchanged (fold accumulator)
```

Currently exists as `EdgeKind` in `src/v2/ownership.dag` (compiler
layer). Should move to `std/` as the shared vocabulary.

### Derivable (not separate types)

- **Boundary crossing vs local:** which Node field the child sits
  in. `params` = boundary crossing. `body`/`children` = local.
  A function on Node field position, not a type.
- **Caller context:** the `SubValueRelation` the call site attaches.
  fold passes `IteratedSubValue`, descend passes `StrictSubValue`,
  plain call passes `PreservedValue`. Not a separate type.
- **Access shape:** the `SubValueRelation` derived from the
  expression. Field access → `StrictSubValue`, iteration →
  `IteratedSubValue`, etc. Not a separate type.
- **Iteration binding:** fold attaches `IteratedSubValue` to element
  param and `PreservedValue` to accumulator. Not a separate type —
  it's which SVR the iteration primitive provides.

### Surface metadata (optional, for diagnostics only)

```dag
type BindingSurface
  = FunctionParam | LambdaParam | ForEachVariable
  | LetDeclaration | MatchArmBind | BlockLetBind
```

No downstream code branches on this. Exists for error messages and
idiomatic emission. Keep or kill — it doesn't affect the edge model.

---

## Dimension computation — SVR-keyed

Every correctness dimension computes from `SubValueRelation` on
the edge:

| SubValueRelation | Provenance | Ownership | Descent |
|---|---|---|---|
| `PreservedValue` | PreservedValue | source | NonIncreasing |
| `StrictSubValue` | StrictSubValue | Borrowed | Strict |
| `IteratedSubValue` | IteratedSubValue | Owned | Strict |
| `ArithmeticDescent` | ArithmeticDescent | Owned | Strict |
| `SubValueUnknown` | SubValueUnknown | Owned (fresh) or Shared (unknown) | DescentUnknown |

One row per SVR variant. One column per dimension. Adding a
dimension = adding a column. Adding a new value relationship =
adding a row. Cost of change: O(1).

The boundary-crossing question ("is this a parameter?") determines
WHO provides the SVR, not what the SVR is:
- Parameter: caller provides SVR (read from call site)
- Let-binding: derive SVR from the expression

This distinction is a function on Node field position, not a type.

---

## The accounting: what exists today

### Layer -1: Structural axioms (`dsl/std/constructors.dag`)

- `Product` — all children hold (AND)
- `Coproduct` — one child holds (OR)
- `Cardinality = Required | Optional`

**CONFLICT:** `00_core.dag` line 103 defines
`Cardinality = Required | CardOptional` — different variant name.

### Layer 1: Foundational concepts (`dsl/std/`)

42 files, key vocabulary:

| File | Key types | Concept |
|------|-----------|---------|
| `algebra.dag` | `Lattice`, `BoundedLattice`, `Ordering` | Algebraic structures |
| `constructors.dag` | `Product`, `Coproduct`, `Cardinality` | Type constructors |
| `induction.dag` | `SubValueRelation`, `InductiveField`, `CostBound` | Provenance / edge vocabulary |
| `termination.dag` | `DescentEvidence`, `RankingDimension`, `TerminationProof` | Termination proofs |
| `computation.dag` | `IterationPrimitive`, `CallPattern`, `SizeBound` | Bounded iteration |
| `syntax.dag` | `BinOp`, `LiteralValue`, `AlgebraFieldKind` | Surface syntax |
| `effects.dag` | `EffectShape`, `KeySource`, `IdempotencyEvidence` | Effect algebra |
| `types.dag` | kernel types, refinements, `SourceSpan`, etc. | ~90 types |

### Layer 2: Compiler IR (`src/v2/00_core.dag`)

| Type | Kind | Role |
|------|------|------|
| `Node` | Product (18 fields) | Universal recursive DAG node |
| `Connective` | Coproduct | `Conj \| Disj \| NoConnective \| Arrow` |
| `ExprData` | Coproduct (22 variants) | Expression discriminator |
| `VarBindingKind` | Coproduct | `Local \| Function \| Variant \| MatchBound` |
| `CallSemantics` | Coproduct | `Plain \| Lookup` |
| `MethodSemantics` | Coproduct | `Plain \| Algebra \| Service` |
| `LambdaSemantics` | Product | Lambda param types |
| `MatchPattern` | Coproduct | `Bind \| Lit \| Variant \| Wildcard` |
| `InferredNode` | Coproduct | `Resolved \| CompilerError \| TypeVariable` |
| `NodeFieldRole` | Coproduct | `ChildrenList \| SubValue \| Metadata` |
| `CompilerDiagnostic` | Coproduct (16 variants) | Error types |

Plus string-keyed data tables: `expr_child_roles`,
`node_field_roles`, `function_size_effects`.

### Layer 3: Pipeline types (compiler-specific)

| File | Key types | Count |
|------|-----------|-------|
| `04_env.dag` | `TypeEnv`, `TypeBinding` | 2 |
| `04_infer.dag` | `InferScope`, `DescentContext`, + ~25 Result/Accum types | ~28 |
| `ownership.dag` | `EdgeKind`, `BindingUsage`, `OwnershipProof` | 7 |
| `complexity.dag` | `SizeExpr`, `CostExpr`, `ComplexitySummary` | ~22 |
| `04_emit_info.dag` | `EmitGraphInfo`, `TypeSummary` | 4 |
| `05_emit.dag` | `ExprCategory`, `FuncBodyShape` | ~12 |
| `compile.dag` | `SourceFile`, `PipelineResult` | 5 |

---

## Known conflicts

| Concept | Location A | Location B | Problem |
|---------|-----------|-----------|---------|
| Cardinality optional | `constructors.dag`: `Optional` | `00_core.dag`: `CardOptional` | Different variant name |
| Text file | `types.dag`: `TextFile { path, document }` | `00_core.dag`: `TextFile { path, content }` | Same name, different fields |
| Size expression | `complexity.dag`: `SizeExpr` (5 variants) | `04_infer.dag`: `SizeExpr` (2 variants) | Name collision |
| Edge/usage kind | `ownership.dag`: `EdgeKind` | (proposed) `UsageEdge` | Same concept, wrong layer |
| Connective | `00_core.dag`: `Conj \| Disj \| NoConnective \| Arrow` | `constructors.dag`: `Product`, `Coproduct` | Parallel representation |

### VarBindingKind dissolution

`VarBindingKind` (00_core.dag) classifies variable references:

```dag
type VarBindingKind
  = LocalValueBinding       // local let/block let
  | FunctionValueBinding    // refers to a function
  | VariantValueBinding     // refers to a variant constructor
  | MatchBoundBinding       // bound in a match arm
```

This dissolves once the boundary-crossing distinction is derived
from Node field position:
- `LocalValueBinding` → binding in `body`/`children` (local)
- `FunctionValueBinding` → binding crosses scope (parameter-like)
- `VariantValueBinding` → constructor reference (not a value flow)
- `MatchBoundBinding` → binding in `body` (local, destructuring)

---

## Reconciliation execution order

1. **Recognize SVR as edge vocabulary** — update docs (this PR)
2. **Add UsageEdge to std/** — the one new type
3. **Cardinality unification** — rename `CardOptional` → `Optional`
   (~118 sites + stage0 regen). Own PR.
4. **SizeExpr disambiguation** — rename infer's `SizeExpr` →
   `DescentSizeAlias`. Own PR.
5. **TextFile disambiguation** — rename compiler's → `EmittedFile`.
   Own PR.
6. **Connective dissolution** — migrate 313 sites. Tracked separately.
7. **VarBindingKind dissolution** — derive from field position.
   Depends on boundary-crossing function.

Items 2-5 are independent. Items 6-7 are larger and tracked separately.

---

## Status

**Proposal.** SVR recognized as the existing edge vocabulary.
UsageEdge identified as the only new concept. 5 conflicts documented
with resolutions. Redundant coproducts from earlier drafts killed.
