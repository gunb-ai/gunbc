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

**More importantly**: because `.dag` programs are pure (`feedback_no_textual_enforcement_bridges` + thesis Tier 1 / Tier 2), **upstream change usually doesn't propagate beyond interface/edge cases**. The affected-set is **strictly smaller** than transitive-downstream — but **only relative to the structural dimensions whose values actually changed**.

**Dimension-aware affected-set discipline** (per `THESIS.md:87-89` + `modeling-discipline.md:69-75`): gunbc treats complexity/cost/effect as structural correctness dimensions, not just runtime annotations. A function whose **I/O return-value behavior** is unchanged but whose **cost/complexity/effect shape changed** still affects consumers carrying structural claims on those dimensions:

- Cost/complexity claim consumers: callers with `apply_lens(complexity, fn, Enforce{...})` or memoization decisions tied to the changed function's cost
- Effect-shape consumers: callers whose effect_enum projection composes through the changed function
- Bottleneck-lens consumers: callers whose T-CostLens-Composition rollups read the changed function's cost contribution

Affectedness is therefore not "did the function's return value behavior change?" but rather "did **any structural dimension** the consumer reads change?" — value-equivalence is one projection of the affected-set predicate; cost/complexity/effect/etc. are others, each yielding a (possibly different) affected-set. The full affected-set is the **union** across all dimensions the consumer set reads.

This matters because gunbc's thesis (`THESIS.md:374-376`) names suboptimal-complexity contract violations as compile-time obligations — a build system that silently skips downstream tests when only cost changed would dilute that structural-correctness promise.

---

## §2. Definition

Given two compiled `.dag` graph states `Dag_before` and `Dag_after`, the **dimension-parameterized affected-set** is:

```
affected_set(Dag_before, Dag_after, dim) →
  Set<NodeRef> where each N satisfies:
    N exists in Dag_after AND
    (
      // Case A: N is the changed Node itself, in this dimension
      N has a PROVEN delta in dimension `dim` between before/after
      OR
      // Case B: N consumes a changed dimension through a typed edge
      ∃ edge (M → N) such that:
        M has a PROVEN delta in dimension `dim_M` between before/after
        AND
        N.behavior reads `dim_M` via that edge (i.e., the dimension
        flows through the edge into N's projection of `dim`)
    )
```

The aggregate affected-set across dimensions:

```
affected_set(Dag_before, Dag_after) =
  ⋃ over dim in {value, cost, complexity, effect, refinement, ...}
    affected_set(Dag_before, Dag_after, dim)
```

**Fail-closed discipline** (per INVARIANTS P1/P3): identity-change alone is NOT sufficient for propagation. The propagation trigger is a **proven dimension delta**:

- If `delta(M, dim_M)` can be **proven empty** by the lens → consumer N reading `dim_M` is excluded from the affected-set for that dimension
- If `delta(M, dim_M)` **cannot be proven empty** (e.g., the lens lacks the substrate to compute the dimension delta, or the delta is unbounded) → consumer N is **included** by default (fail-closed)

This makes the lens **strict-narrower-than-downstream** when deltas are provable AND **fail-closed-safe** when they aren't.

**Strictly excluded from PROPAGATION** (when delta proofs hold) — these nodes may themselves be in the affected-set but the affected-set does NOT transitively expand through them:

- **Transitive non-readers**: Nodes that transitively reference a changed Node but don't read any changed dimension (e.g., function calls the changed function with value-equivalence proven AND no cost/effect read → not propagated through)
- **Test nodes**: a TestClaim that asserts properties of a changed Node IS in the affected-set itself (so the test runs in CI), but production code consumers of the test do not exist (tests have no downstream production graph), so the affected-set does not expand through them
- **Documentation / comments / non-structural metadata**: no dimension flow through any edge; not in the affected-set at all

**Critical**: the lens MUST emit a per-dimension proof receipt for each excluded consumer (similar to TestClaim fail-closed receipts per `verification.dag`). Without that receipt, the consumer falls back to the default-include (fail-closed) branch. No silent exclusions.

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
| `DescentEvidence` (gate #72 CONSUMER_LANDED) | `src/v3/std/termination.dag:17` (type defn; see also `src/v3/std/computation.dag:11/:89` consumers) | For each callsite, well-founded descent evidence (`Strict / NonIncreasing / DescentUnknown` lattice) — narrows propagation |
| `SubValueRelation` (gate #78 in-flight) | `src/v3/std/induction.dag:207` (type defn; see also call-site descent + composition helpers at `:220-385`) | Atomic-level subvalue tracking: which sub-piece flows where; threading provenance |
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

`structural_dependency(M, N)` is the load-bearing predicate. Per `THESIS.md:198-201`, the substrate shape is **two parallel surfaces**:

- **Type substrate**: `Atom | Conj | Disj | Arrow | Cardinality | Instantiation` (6 type connectives)
- **Computation substrate**: `Value | Transform | Branch | Loop | Bind` (5 L1 behaviors; `Transform` refers to `Arrow.body`)

`structural_dependency` is a **fold over both surfaces** + the dimension lens(es) the consumer reads. Each edge type encodes a specific kind of dependency:

- **Arrow → Bind (call site)**: consumer is affected if the callable's signature OR any read dimension changed
- **Cardinality → Branch (refinement)**: consumer is affected if the refinement boundary changed AND the consumer flows through it
- **Conj/Disj (structural composition)**: consumer is affected if the composed sub-pieces changed in dimensions the consumer projects
- **Loop (iteration)**: consumer is affected if the iteration bound/termination predicate semantics changed in a dimension the consumer reads
- **Instantiation (algebra surface)**: consumer is affected if it walks the algebra structure that changed (not just references the name)

**Distinct from the PB-runtime bounded kernel** (per `docs/design-pure-bootstrap-zero.md:118-119`: `Node` + `Conj` + `Disj` + `Cardinality` + `Bit` — recursive form only). The PB kernel is what stage-0 runs through; the substrate-level query surface for affected-set is the full 6+5 thesis shape. Conflating the two would under-model call/signature/refinement/algebra-walk dependencies that require Arrow/Instantiation/Branch.

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

**Lens output** (per consumer dimension):

```
// Value-equivalence dimension: I/O return is identical
affected_set_value = { multiply_then_add }

// Cost/complexity dimension: depends on whether the rewrite
// preserves cost shape
affected_set_cost =
  if cost_shape(before) == cost_shape(after):
    { multiply_then_add }
  else:
    { multiply_then_add } ∪ { consumers with cost claims that read this fn }

// Effect-shape dimension: pure→pure rewrite, no effect change
affected_set_effect = { multiply_then_add }

// Aggregate affected-set is the UNION across consumer dimensions
affected_set = affected_set_value ∪ affected_set_cost ∪ affected_set_effect
```

**Commentary**: the function's body changed (internal Node tree differs), but its **I/O return-value behavior is identical** for every input. The value-projection narrows to `{multiply_then_add}` only. **However**, the cost shape may differ (one extra `+ 0` instruction; trivial here, but consider a non-trivial body rewrite): if a consumer carries an `apply_lens(complexity, ..., Enforce)` contract on this function, that consumer IS in the affected-set even though I/O is unchanged.

Compare to transitive-downstream which would include every call site regardless of dimension — typically 10-100× larger. The structural lens narrows per dimension but **does not collapse to value-only**.

**Test selection**:
- Tests checking I/O behavior: only the function's own TestClaim runs
- Tests checking cost/complexity contracts on consumers: run if the cost shape changed and a consumer reads it
- Tests checking effect-shape: run if effect_enum projection changed

Per `TESTING.md:37-48`, tests are behavior contracts spanning all dimensions; the CI selection composes per-dimension affected-sets, not just the value projection.

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

1. **PR pre-step**: compute `Dag_before` (main HEAD) and `Dag_after` (PR HEAD); run `affected_set` lens **per dimension** (value / cost / complexity / effect / refinement); union the per-dimension sets
2. **Test selection**: intersect aggregate affected-set with TestClaim references — each TestClaim declares which dimensions it asserts about; selection keeps TestClaims whose asserted-dimensions intersect with changed-dimensions
3. **CI runtime**: execute only the minimum set; on green, allow merge

The current CI runs ~all tests; with this lens, typical PR runtimes would shrink by 10-100× depending on PR shape. Substrate-shape PRs (Case C) would run more (algebra walks → many dimension consumers); function-body PRs (Case A) would run only the dimensions actually changed (value-only rewrite → few tests; cost-shape rewrite → cost-contract tests on consumers also run).

**Critical**: the CI selection must NOT default to "value-equivalence only" — that would skip cost/effect/complexity-contract tests on downstream consumers, diluting the structural-correctness promise per `THESIS.md:374-376` (suboptimal-complexity contract violations are compile-time obligations). Selection is **dimension-aware**: per-dimension affected-set × per-TestClaim asserted-dimensions.

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
