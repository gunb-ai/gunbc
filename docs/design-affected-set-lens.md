# Affected-Set Lens — Design + Worked Examples

**Status**: R4 wishlist (R4.B queries-as-data lane); pre-R3-close working prototype dispatched at [gunbc#2699](https://github.com/gunb-ai/gunbc/issues/2699).

**Authority**: PM-authored design doc per operator directive at gunbc#846 (2026-05-11) — "buck2/bazel-style fine-grained build system for `.dag`."

**Scope**: design framing + 5 worked examples. **Not** a substrate-shape ratification; not a §1.8 gate addition. Prototype lens lives under [gunbc#2699](https://github.com/gunb-ai/gunbc/issues/2699) worker scope.

---

## §1. Problem framing

Build systems trade off two costs:

- **Analysis time**: figuring out what must rebuild
- **Execution time**: actually doing the rebuild

Bazel parallelizes execution but keeps analysis serial — the dependency graph is computed walk-by-walk for each target. **Buck2** fixed this with parallel analysis: the graph is built incrementally and shared across queries.

Both systems still answer **"what is downstream of X?"** with a transitive-edge closure. That's a coarse over-approximation: every transitive consumer of X is in the set, regardless of whether the change to X actually affects them.

**gunbc's structural leverage**: the substrate itself IS the dependency graph. Every `Node` declares its outgoing edges typed-and-structured (per [`docs/architecture.md`](architecture.md) and `feedback_compiler_is_dag_processor`). There's no separate analysis phase to parallelize because the analysis IS the substrate.

**More importantly**: because `.dag` programs are pure (`feedback_no_textual_enforcement_bridges` + thesis Tier 1 / Tier 2), **upstream change usually doesn't propagate beyond interface/edge cases**. A function whose internal cost changes but whose I/O behavior is unchanged has **no consumers in the affected set** — only the function itself.

The affected-set is **strictly smaller** than transitive-downstream.

---

## §2. Definition

Given two compiled `.dag` graph states `Dag_before` and `Dag_after`, the **affected-set** is:

```
affected_set(Dag_before, Dag_after) →
  Set<NodeRef> where each N satisfies:
    N exists in Dag_after AND
    (
      // Case A: N is the changed Node itself
      N has different structural identity in before vs after
      OR
      // Case B: N consumes the change through a typed edge
      ∃ edge (M → N) such that M.identity changed AND
        N.behavior structurally depends on M via that edge
    )
```

**Strictly excluded** from affected-set:

- Nodes that transitively reference a changed Node but don't structurally depend on the changed behavior (e.g., function calls the changed function but doesn't observe the changed code path)
- Test-only Nodes that test the changed Node directly (they're in the affected-set themselves but their existence doesn't propagate)
- Documentation / comments / non-structural metadata

The "depends on via that edge" predicate is structurally checkable per substrate:

- For data edges: `M` produces a value that `N` consumes → `N` is affected if `M.value` changed
- For control edges (`Loop`, `Repeat`): `M` is the iterator bound or termination predicate → `N` is affected only if the bound/predicate semantics changed
- For coercion edges (target-realization): `M` is a primitive realization row → `N` (consumer of that realization) is affected only if the realization shape changed
- For algebra edges (`Conj` references): `M` is the algebra carrier definition → `N` is affected if it walks `M.surface`

---

## §3. Substrate composition

The affected-set lens composes existing R3 substrate (no new substrate required):

| Substrate piece | File | Role in lens |
|---|---|---|
| `Dag` forward graph | `src/v3/std/` + `dag.rs` | Reverse-traverse edges to find consumers |
| `DescentEvidence` (gate #72 CONSUMER_LANDED) | `src/v3/std/descent_evidence.dag` | For each callsite, which port the recursion descends on — narrows propagation |
| `SubValueRelation` (gate #78 in-flight) | `src/v3/std/sub_value_relation.dag` | Atomic-level subvalue tracking: which sub-piece flows where |
| `Cardinality` lens | `src/v3/lenses/` | Distinguishes "data shape changed" from "data value changed" |
| `cross_target_coverage` | `src/v3/std/cross_target_coverage.dag` | Per-substrate-variant × target emission paths — narrows cross-target propagation |
| TestClaim DB-15 | `src/v3/std/verification.dag` | Every test is data; can intersect affected-set with TestClaim references |
| `apply_lens` framework | `src/v3/lenses/` | Lens-as-data declaration; this design fits the existing surface |

The structural shape:

```
affected_set: Lens<Dag × Dag → Set<NodeRef>> where
  body(dag_before, dag_after) =
    let changed = nodes_with_different_identity(dag_before, dag_after)
    in transitive_closure(
         changed,
         next_step: λ M. {N : edge(M → N) ∧ structural_dependency(M, N)}
       )
```

`structural_dependency(M, N)` is the load-bearing predicate. Per substrate-shape thesis, it's a **fold over the edge types** — Conj / Disj / Cardinality / Bit (the only types the compiler knows per `feedback_compiler_is_dag_processor`).

---

## §4. Worked examples

The 5 cases below show the affected-set output for representative `.dag` mutations. Each example: source diff → lens output → commentary on why-that-output.

### §4.1 Case A — Function body change, identical signature

**Setup**:

```dag
// before:
function multiply_then_add(x: Int, y: Int, z: Int) -> Int {
  let product = x * y
  product + z  // straight sum
}

// after:
function multiply_then_add(x: Int, y: Int, z: Int) -> Int {
  let product = x * y
  product + z + 0  // identity-preserving rewrite
}
```

**Lens output**:

```
affected_set = { multiply_then_add }
```

**Commentary**: the function's body changed (internal Node tree differs), but its **I/O behavior is identical** for every input. Consumers that call `multiply_then_add(x, y, z)` observe the same return value before and after. Per substrate purity, downstream is **not affected**.

Compare to transitive-downstream which would include every call site — typically 10-100× larger.

**Test selection**: the function's own TestClaim runs; downstream tests skip.

### §4.2 Case B — Signature change (port added/removed)

**Setup**:

```dag
// before:
function add(x: Int, y: Int) -> Int {
  x + y
}

// after:
function add(x: Int, y: Int, z: Int) -> Int {
  x + y + z
}
```

**Lens output**:

```
affected_set = {
  add,
  // every binder of `add`:
  caller_alpha,
  caller_beta,
  caller_gamma,
  // ...
}
```

**Commentary**: signature change means every binder must reconcile against the new shape. **Direct reverse-edge query**: `{N : N.callable_ref → add}` is the affected-set. No DescentEvidence needed (this is a port-arity change, not a recursion shape).

**Test selection**: any TestClaim that references `add` directly OR references any binder runs.

### §4.3 Case C — Algebra carrier definition change

**Setup**:

```dag
// before:
type Int = AbelianGroup<Nat>  // (note: structurally wrong per P1 axiom-violation; this is illustrative)

// after:
type Int = AbelianGroup<GroupCompletion<Nat>>  // Slice 3 PR #1466 Q6 single-authority form
```

**Lens output**:

```
affected_set = {
  Int,
  // every Node walking Int's algebra surface:
  add (it consumes AbelianGroup.op),
  subtract (it consumes AbelianGroup.inverse),
  multiply (it consumes AbelianGroup<Int>.op composed with multiplication ring),
  Real (it composes through ApproximateField<FieldOfFractions<Int>>),
  Rational,
  // every primitive realization row that walks Int:
  rust_int (TypeRealization.carrier composes Int),
  python_int,
  go_int,
  // ...
}
```

**Commentary**: algebra carrier change is a substrate-level shape change. Per cardinality + descent-evidence substrate, every Node that **walks the algebra surface** (not just references the name) is affected. The Cardinality lens distinguishes "uses Int as opaque carrier" from "walks Int's algebraic structure" — only the latter is affected.

This is the case where **affected-set is materially larger** than typical cases — but still smaller than transitive-downstream because Nodes that carry `Int` opaquely (e.g., a list of `Int` where the algebra isn't unpacked) are excluded.

**Test selection**: tests that walk algebra rules through `Int` run; tests using `Int` as opaque value carrier (e.g., `List<Int>` literal equality) skip.

### §4.4 Case D — Test-only change (TestClaim added)

**Setup**:

```dag
// before: (test does not exist)

// after:
// src/v3/compiler/tests/dag/t_new_property_test.dag
test new_property_holds: TestClaim {
  claim: forall x: Int. x + 0 == x
}
```

**Lens output**:

```
affected_set = { new_property_holds }
```

**Commentary**: a TestClaim addition has **no consumers in production code** (production doesn't import from test). The TestClaim itself runs once when CI fires it; no other Node is affected.

**Test selection**: only `new_property_holds` runs in addition to whatever the baseline test set is.

This is the case where **most-coarse transitive-downstream** would absurdly mark the whole codebase as affected (because TestClaim references many Nodes); the structural lens correctly identifies that the test doesn't propagate.

### §4.5 Case E — Refinement type tightening

**Setup**:

```dag
// before:
function array_index(array: List<Byte>, index: Int) -> Byte {
  // unbounded Int index
  ...
}

// after:
function array_index<N: MachineWidth<32>>(array: List<Byte>, index: Int<N>) -> Byte {
  // refined to Int<32>; out-of-bounds at compile time
  ...
}
```

**Lens output**:

```
affected_set = {
  array_index,
  // every caller passing an unrefined Int as index:
  callsite_alpha,  // had `array_index(buf, 42)` — Int literal 42 must lift to Int<32>
  callsite_beta,   // had `array_index(buf, my_unrefined_int)` — my_unrefined_int must refine or fail
  // ...
  // BUT NOT:
  // callsite_gamma that already passed Int<32> via prior refinement — unchanged
}
```

**Commentary**: refinement change affects only consumers that flow through the refined port. Callers that already supplied a compatible refinement are **not affected**. The SubValueRelation lens tracks which sub-piece of each caller's input flows into the changed port; only callers where that sub-piece's refinement is incompatible are in the affected-set.

**Test selection**: tests exercising callers in the affected-set run; tests of callers with pre-existing matching refinements skip.

---

## §5. CI integration sketch (deferred to R4 full delivery)

A production build-system integration would compose:

1. **PR pre-step**: compute `Dag_before` (main HEAD) and `Dag_after` (PR HEAD); run `affected_set` lens
2. **Test selection**: intersect affected-set with TestClaim references; produce the minimum test set to run
3. **CI runtime**: execute only the minimum set; on green, allow merge

The current CI runs ~all tests; with this lens, typical PR runtimes would shrink by 10-100× depending on PR shape. Substrate-shape PRs (Case C) would run more; function-body PRs (Case A) would run a single TestClaim.

**Out of scope here**: implementation of the CI integration. The prototype demonstrates the lens output; the CI integration is R4 full-delivery work.

---

## §6. Coupling to R4.B queries-as-data

This lens is **one consumer** of the queries-as-data infrastructure per [`WISHLIST.md`](../WISHLIST.md) §R4.B. Other consumers using the same lens substrate:

| R4.B use case | Lens shape | Reuses |
|---|---|---|
| Refactoring impact (R4.B #2) | `refactor_impact(Dag, Refactor) → Set<NodeRef>` | Same substrate; different traversal predicate |
| Coverage gap (R4.B #4) | `coverage_gap(Dag, TestSet) → Set<NodeRef>` | Same substrate; intersects with TestClaim subgraph |
| Effect-shape (R4.B #3) | `effect_shape(Dag, NodeRef) → EffectSet` | Same substrate; reads effect_enum projection |
| Performance bottleneck (R4.B #1) | `bottleneck(Dag, Workflow) → Vec<NodeRef>` | Already partially exists via T-CostLens-Composition |

The affected-set lens is the **simplest** of the R4.B family (no new substrate; pure-fold over existing edges) and demonstrates the architectural claim: **lenses applied to the substrate yield queries the LLM/IDE can consume**.

---

## §7. Pre-R3-close prototype scope (worker dispatch)

Worker at gunbc#2699 will deliver:

1. Prototype `affected_set_lens.dag` (composes existing substrate; no new substrate)
2. 5 concrete `.dag` examples matching §4 cases above
3. Real-PR test: run against 2-3 recently-merged PRs (e.g., #2693 v2 delete, #2679 gate #4, #2647 quantifier substrate); document lens output vs naive transitive-downstream
4. Closeout: structural surprises documented; cross-link to this design doc

**Worker is not delivering**:

- IDE integration
- CI integration
- Cross-language affected-set (waits on R4.A omni-ingest)
- Production saturation (R4 full work)

The prototype validates the structural claim; R4 full delivery operationalizes it.

---

## §8. WISHLIST entry

This design doc supports a WISHLIST.md addition under R4.B:

> **Affected-set lens for fine-grained build system**: query "I changed X; what Y is affected?" with structural strict-narrower-than-transitive-downstream semantics. Pure-substrate gives buck2/bazel-style fine-grained dep management for free. Pre-R3-close prototype at [gunbc#2699](https://github.com/gunb-ai/gunbc/issues/2699); full delivery part of R4.B queries-as-data substrate.

---

**End of design doc.**
