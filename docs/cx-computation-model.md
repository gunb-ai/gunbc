# CX Computation Model: Migration Plan

Design doc for the computational concept DAG — the model that makes
CX-B through CX-E concrete.

## Core Model (approved 2026-04-04)

All computation desugars to three bounded iteration primitives:
**fold**, **descend**, **repeat**. No call pattern is rejected. The
compiler always succeeds in lowering; the question is not "does this
terminate?" but "what is its bound?"

Authoritative files:
- `dsl/std/iteration.dag` — declares the three primitives
- `dsl/std/computation.dag` — maps syntax to primitives (lowering table)
- `dsl/std/termination.dag` — proves termination via ranking functions

### The 7 system concepts

Everything the system understands:

1. **Node/DAG** — structural carrier (the ONLY recursive type)
2. **Product/Coproduct** — structural connectives
3. **Bit** — truth value, finite (2 inhabitants)
4. **Ring/Field/BooleanAlgebra/FreeMonoid** — value algebra
5. **fold/descend/repeat** — iteration (THE bottleneck)

Everything else is sugar. A developer writes `while`/`for`/`self`/`+`/`-`/`list.map`
and the compiler lowers it all to the above.

### CallPattern → LoweringTarget (exhaustive)

| CallPattern              | Lowers to      | Bound                 |
|--------------------------|----------------|-----------------------|
| ChildAccessorCall        | descend        | TreeSize              |
| CollectionShrinkCall     | fold           | CollectionSize        |
| ArithmeticDescentCall    | repeat         | ArithmeticParam       |
| ParserAdvanceCall        | fold           | CollectionSize(tokens)|
| WorklistDrainCall        | fold           | CollectionSize(set)   |
| FoldBodyCall             | (already fold) | (inherited)           |
| SameArgumentCall         | repeat         | Forever               |

No pattern is rejected. `SameArgumentCall → repeat(Forever)` is the
bounded truth principle: in a Bit/Word64 system, "always" = 2^63-1
iterations. True is a Bit, not infinity. INVARIANTS.md is aligned:
the lowering table is exhaustive and every call pattern has a bound.

### Cost = product of bounds

Nested computations multiply bounds:
```
fold(list, fn(acc, x) { descend(tree, ...) })
→ O(|list| × |tree| × per_node_cost)
```

This is the path to dissolving the recursive `CostExpr` type.

---

## What exists today

**`complexity.dag`** (~4300 lines) is a monolith with three interleaved
concerns:

1. **Type definitions** — CostExpr, SizeExpr, RecursionPattern,
   DescentEvidence, TerminationProof, etc.
2. **Classification heuristics** — 8 `classify_*` functions, 15+
   `is_*` probe functions, hardcoded field/method name strings
3. **Cost algebra** — simplify, format, normalize, bounded cost

**`dsl/std/primitives.dag`** has duplicate CostExpr/SizeExpr/Certainty
(skeleton types from an earlier design pass that diverged from the
working definitions in complexity.dag).

### Inventory of ad-hoc patterns

- **19 recursive self-call functions** in 4 categories: call-counting,
  descent evidence, parser analysis, SCC proof construction
- **8 classify_* heuristic functions**: `classify_recursion_pattern`,
  `classify_parser_scc_recursion_pattern`, `classify_scc_recursion_pattern`,
  `classify_self_call_evidence`, `classify_scc_call_progress`,
  `classify_arg_multidim`, `classify_list_method`, `classify_complexity`
- **10+ hardcoded field/method name strings**: `"children"`, `"expr_data"`,
  `"state"`, `"skip"`, `"first"`, `"last"`, `"fold"`, `"map"`, etc.
- **15+ fallback heuristic functions**: `is_*` probes that string-match
  instead of reading structural facts
- **230+ generated Rust references** to CostExpr/SizeExpr variants in stage0

---

## Migration phases

### Phase 1: Unify type definitions (low risk, high value)

Single source of truth for all proof/cost types.

| Current location | Type | Action |
|---|---|---|
| `primitives.dag:31` | Certainty (4 variants) | **Delete** — superseded by `complexity.dag:93` |
| `primitives.dag:33-48` | SizeExpr, CostExpr (skeleton) | **Delete** — superseded by `complexity.dag:58-83` |
| `complexity.dag:217-220` | RecursionPattern | **Rewire** — align to CallPattern → LoweringTarget |
| `complexity.dag:310-358` | DescentEvidence, TerminationProof, ProofEdge | **Move** imports to `termination.dag` |

Key change: `RecursionPattern` (3 variants) → `LoweringTarget`:
- `LinearRecursion { iteration_var }` → `LoweringTarget { Fold/Repeat, ... }`
- `DivideAndConquer { split_factor }` → `LoweringTarget { Descend, TreeSize, ... }`
- `UnresolvableRecursion { reason }` → **Delete** (no rejected patterns)

### Phase 2: Rewire classify functions to use CallPattern (medium risk)

Only 3 of 8 `classify_*` functions need to change — the ones that
produce `RecursionPattern`:

| Function | Migration |
|---|---|
| `classify_recursion_pattern` (line 2564) | Produce CallPattern → `lower_call_pattern` |
| `classify_parser_scc_recursion_pattern` (line 1367) | Produce ParserAdvanceCall → lower |
| `classify_scc_recursion_pattern` (line 3237) | Produce CallPattern → lower |

The other 5 are evidence-gathering or formatting — they feed *into*
classification, not *out of* it.

### Phase 3: Dissolve hardcoded heuristics (medium risk, careful)

Replace string-matching heuristics with structural facts. Each
hardcoded string is a point where `complexity.dag` "knows" something
the model should tell it.

- `"children"`, `"expr_data"` → reference model's structural children spec
- `"fold"`, `"map"`, `"filter"`, `"flat_map"` → reference MethodSemantics
  (partially done: `is_algebra_iteration_method` reads AlgebraMethodSemantics)
- `"state"`, `"skip"`, `"first"`, `"last"` → reference parser field spec
- `lambda_param_names |> last` convention → read element-parameter position
  from AlgebraMethodSemantics or fold signature (currently assumes element
  is always the last lambda parameter)

### Phase 4: Flatten CostExpr/SizeExpr (high value, deferred)

CostExpr becomes flat: cost = product of SizeBounds from the lowering table.

```
type Cost = { bounds: List<SizeBound>, per_step: Int, certainty: Certainty }
```

Requires Phase 1-3 to stabilize first. This eliminates:
- 18 functions that do recursive descent over CostExpr
- 230+ generated Rust references to CostExpr variants

---

## 5 migration blockers

1. **Parser always-advancing inference** (`infer_parser_always_advancing_members`,
   line 1190) — worklist fixed-point algorithm. Must map to ParserAdvanceCall
   evidence.

2. **Witness mechanism** (`descending_witness_names`,
   `self_calls_have_descending_witness`) — tracks which variables carry
   descent evidence through let-bindings. Must wire to DescentEvidence
   propagation in termination.dag.

3. **Hardcoded IR field mapping** (`is_children_of_param`,
   `is_accessor_of_param`) — needs to read from model's structural
   children spec rather than string-match `"children"`.

4. **Incremental var threading** (`collect_descent_vars`,
   `collect_evidence_incremental`) — accumulates descent variable sets
   through let-bindings. Must integrate with CallPattern classification.

5. **Multidimensional proof combination** (`classify_arg_multidim`,
   `collect_scc_multidim_edges`) — produces multi-dimension ProofEdges
   for lexicographic proofs. Must compose with termination.dag's
   TerminationProof structure.

---

## PR scope

**This PR**: Concept DAG model (computation.dag, iteration.dag updates)
+ this design doc. No computation-model-to-code migration yet — the
analyzer changes in this PR (CX-A/C/D) are separate work items that
do not yet use CallPattern/LoweringTarget.

**Next PR (Phase 1 + partial Phase 2)**:
1. Delete duplicate types from primitives.dag
2. Import CallPattern/LoweringTarget/SizeBound into complexity.dag
3. Rewire RecursionPattern → LoweringTarget
4. Rewire `classify_recursion_pattern` to produce CallPattern then lower
5. Kill `UnresolvableRecursion` — no rejected patterns

**Follow-up PRs**:
- Phase 2 completion: parser SCC classification → CallPattern
- Phase 3: dissolve hardcoded strings
- Phase 4: flatten CostExpr (separate design review)

---

## Relationship to CX work items

| CX item | Phases it maps to |
|---------|-------------------|
| CX-A (DescentEvidence lattice) | Done — termination.dag landed |
| CX-B (CostExpr/SizeExpr dissolution) | Phase 1 + Phase 4 |
| CX-C (Signature-driven fold evidence) | Phase 2 (FoldBodyCall) |
| CX-D (MatchPattern + remaining) | Phase 3 |
| CX-E (Re-enable gate) | After Phase 4 |
