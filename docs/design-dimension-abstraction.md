> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 2 Stage 2f, Lane 4 Stages 4b/4c

# Design DB-3 — `Dimension` abstraction for compile-time proofs

**Design blocker:** DB-3
**Consumers:** Lane 2 Stage 2f (user-declared dimensions); Lane 4 Stage 4b (side effects); Lane 4 Stage 4c (space bounds); informs Lane 2 Stages 2b/2d/2e (idempotency, symbolic cost, parallelism — each becomes a Dimension instance)
**Status:** Design ready for implementer review.

---

## Problem

Lane 2 adds compile-time lenses for idempotency (2b), symbolic cost (2d), parallelism-as-diagnostic (2e). Lane 4 adds side effects (4b) and space bounds (4c). Each is a structural property that walks a workflow, composes per-operation evidence via an algebraic operation, and emits a diagnostic when composition breaks.

Without a shared abstraction:
- Each lens duplicates the walk-compose-diagnose pattern
- Adding user-declared dimensions (a thesis goal, Stage 2f) means wrapping each ad-hoc lens in yet another shape
- Side effects / space bounds become bespoke code

**The `Dimension` abstraction names the common shape so every compile-time property — built-in and user-declared — fits one interface.**

---

## Design

### Core type

```dag
// src/v3/std/dimensions.dag (new file)
module std.dimensions

import std.list { List }
import std.map { Map }
import std.substrate { Dag, NodeId, PortId, Behavior }
import std.diagnostics { Diagnostic, DiagnosticKind }

// 🟢 TERMINAL. A Dimension is a compile-time-enforced structural property
// of `.dag` programs. Idempotency, symbolic cost, parallelism, side
// effects, and space bounds are all Dimension instances. Users declare
// new Dimensions by declaring the four fields below.
type Dimension<Carrier> {
  name: String                                            // human-facing label
  witness_of: fn(Dag, Behavior) -> Witness<Carrier>       // extract per-op evidence
  compose: fn(Carrier, Carrier) -> Carrier                // combine across a workflow
  identity: Carrier                                        // unit for fold
  break_diagnostic: fn(Behavior, Carrier) -> Diagnostic?  // emit when composition breaks
}

// Evidence that a behavior inhabits the dimension's algebra.
// `Inhabits` = behavior is an ok member of the dimension's algebra
// `Violates` = behavior breaks the dimension (e.g., non-idempotent op in an idempotent workflow)
type Witness<Carrier>
  = Inhabits(Carrier)
  | Violates { reason: String, at: Behavior }
```

### Dimension evaluation

One evaluation function works for every Dimension instance:

```dag
// std/dimensions.dag
fn analyze<Carrier>(
  d: Dag,
  workflow: NodeId,
  dim: Dimension<Carrier>
) -> DimensionReport<Carrier>

type DimensionReport<Carrier> {
  dimension_name: String
  composed: Carrier            // aggregate value for the whole workflow
  violations: List<Diagnostic> // from break_diagnostic on each Violates witness
  witnesses: List<Witness<Carrier>>  // per-operation evidence, in workflow order
}
```

Implementation (sketched):

```dag
fn analyze<Carrier>(
  d: Dag,
  workflow: NodeId,
  dim: Dimension<Carrier>
) -> DimensionReport<Carrier> {
  let ops = flatten_workflow(d, workflow)  // List<Behavior> in execution order
  let witnesses = map(ops, |op| dim.witness_of(d, op))
  let composed = fold(witnesses, dim.identity, |acc, w|
    match w {
      Inhabits(c) => dim.compose(acc, c)
      Violates(_) => acc  // composition stops being meaningful; diagnostics carry the violation
    }
  )
  let violations = filter_map(ops zip witnesses, |(op, w)|
    dim.break_diagnostic(op, composed_up_to(op))  // gives the diagnostic access to partial compose
  )
  DimensionReport { dimension_name: dim.name, composed, violations, witnesses }
}
```

(Actual implementation may differ; the API shape is what's locked here.)

### Idempotency as a Dimension instance

Rewriting Lane 2 Stage 2b's lens through the abstraction:

```dag
// src/v3/lenses/idempotency.dag (Stage 2b)
import std.dimensions { Dimension, Witness, Inhabits, Violates }
import std.effects { EffectShape, is_idempotent_effect, compose_effects, ComposedEffect }

data idempotency_dimension: Dimension<ComposedEffect> = {
  name: "idempotency"
  witness_of: |d, behavior| witness_idempotency(d, behavior)
  compose: |a, b| compose_effects_pair(a, b)
  identity: empty_composed_effect()
  break_diagnostic: |behavior, composed| idempotency_diagnostic(behavior, composed)
}

fn witness_idempotency(d: Dag, behavior: Behavior) -> Witness<ComposedEffect> {
  let shape = effect_shape_of(d, behavior)
  if is_idempotent_effect(shape) {
    Inhabits(effect_as_composed(shape))
  } else {
    Violates {
      reason: "operation is non-idempotent: " + describe_shape(shape),
      at: behavior
    }
  }
}

// Lane 2b's analyze_workflow becomes:
fn analyze_workflow(d: Dag, workflow: NodeId) -> WorkflowIdempotencyReport {
  let report = analyze(d, workflow, idempotency_dimension)
  WorkflowIdempotencyReport {
    idempotent: is_empty(report.violations),
    breaking_op: first_breaking_op(report.witnesses),
    evidence_chain: report.witnesses,
    diagnostic: first(report.violations)
  }
}
```

### Side effects as Dimension instance (Lane 4 Stage 4b)

```dag
// src/v3/lenses/side_effects.dag
data side_effects_dimension: Dimension<EffectSet> = {
  name: "side_effects"
  witness_of: |d, behavior| witness_side_effects(d, behavior)
  compose: |a, b| union(a, b)
  identity: empty_effect_set()
  break_diagnostic: |behavior, composed|
    if declared_hermetic(behavior) && !is_empty(composed) {
      Some(Diagnostic {
        kind: HermeticViolation { op: behavior_name(behavior) }
        ...
      })
    } else {
      None
    }
}
```

### Space bounds as Dimension instance (Lane 4 Stage 4c)

```dag
// src/v3/lenses/space_bounds.dag
data space_bounds_dimension: Dimension<SpaceCost> = {
  name: "space_bounds"
  witness_of: |d, behavior| witness_space_cost(d, behavior)
  compose: |a, b| add_space(a, b)
  identity: SpaceCost::Zero
  break_diagnostic: |behavior, composed|
    if exceeds_declared_bound(behavior, composed) {
      Some(Diagnostic { kind: SpaceBoundExceeded { op: ..., bound: ..., actual: composed } ... })
    } else {
      None
    }
}
```

### User-declared dimension (Lane 2 Stage 2f)

Users declare a new Dimension by writing the four fields. Example: a `memory_bounded` dimension that enforces workflow memory stays under a declared limit.

```dag
// user code, e.g. my_project/dimensions.dag
import std.dimensions { Dimension }

data memory_bounded_dimension: Dimension<MemoryUsage> = {
  name: "memory_bounded"
  witness_of: |d, behavior| estimate_memory_usage(d, behavior)
  compose: |a, b| max_memory(a, b)
  identity: MemoryUsage::Zero
  break_diagnostic: |behavior, composed|
    if composed > memory_bound_of(behavior) {
      Some(Diagnostic { kind: MemoryBoundExceeded { ... } })
    } else {
      None
    }
}
```

The compiler picks up any declared `Dimension<_>` at bootstrap, runs `analyze(d, workflow, dim)` for each on relevant workflows, emits diagnostics.

### Algebraic constraints on Dimension parameters

The `compose` + `identity` pair must form a **monoid** on `Carrier`:
- `compose(identity, x) == x` (left identity)
- `compose(x, identity) == x` (right identity)
- `compose(compose(x, y), z) == compose(x, compose(y, z))` (associativity)

This isn't checked at compile time today (requires algebra inhabitance verification), but it's a documented requirement on Dimension authors. Violating it means `analyze` gives inconsistent results depending on fold direction.

Additional constraints for specific dimensions:

- **Idempotency**: compose is commutative AND idempotent (lattice meet). `compose(x, x) == x`.
- **Symbolic cost (additive)**: compose is associative; not commutative in general (sequencing matters for O(n + m) vs O(n·m)).
- **Space bounds**: compose can be max (peak usage) or sum (total allocation) — the specific dimension declares which.
- **Side effects**: compose is set union — commutative and idempotent.
- **Parallelism**: compose has complex semantics (identifying independent subgraphs) — may not fit the Dimension abstraction cleanly; see "Open questions" below.

---

## Rationale

**Why a generic abstraction over ad-hoc lenses?** Because the user-declared-dimensions thesis goal (Lane 2 Stage 2f) requires a uniform interface. Without it, declaring a new dimension means writing a full lens; with it, declaring = filling in four fields.

**Why expose `compose` + `identity` as fields, not hardcode monoid operations?** Because Carrier types differ across dimensions. Idempotency's Carrier is `ComposedEffect` with lattice meet; space's is `MemoryUsage` with addition; parallelism's might be `DependencyGraph` with more complex composition. One-size-fits-all doesn't.

**Why `Witness<Carrier>` instead of `Option<Carrier>`?** Because "no evidence" and "evidence says violation" are different. `None` could mean "this behavior is invisible to the dimension" (ok — compose skips it) OR "this behavior violates the dimension's algebra" (not ok — diagnostic fires). Splitting into `Inhabits` / `Violates` makes the distinction structural.

**Why does `break_diagnostic` take the partial composed value?** Because some violations are context-sensitive. A single `POST /logs` op is non-idempotent, but the diagnostic message is more useful if it says "this op breaks the otherwise-idempotent chain at step 3" rather than just "this op is non-idempotent in isolation."

**Why is user-dimension declaration an ordinary `data` item?** Because Dimension is just a record. No special syntax; no compiler-internal registration. The compiler walks `d.declarations` looking for `Dimension<_>` values at bootstrap.

**Why not make Dimension a type class / trait?** Because `.dag` doesn't have type classes; data+record is the idiomatic way to express this interface. If/when `.dag` grows type classes, Dimension can migrate — but record form doesn't lose expressiveness.

---

## Rejected alternatives

**Make each lens its own interface (no shared Dimension)** — blocks user-declared dimensions; every user dimension becomes a separate lens with bespoke integration. Rejected.

**Pass composition as a lattice / monoid carrier type** — forces users to declare algebra inhabitance for their Carrier before they can declare the Dimension. Too heavyweight; record + documented requirement is lighter. Revisit when algebra inhabitance is more widely used.

**Make Dimension parameterless (single global dimension type, dispatch by name)** — loses type safety on Carrier. Rejected.

**Include `parallelism_of` as a fifth field for parallel-fold optimization** — specific to parallelism dimension. Keep the abstraction clean; parallelism can add its own extra methods. Rejected.

**Compose function takes `List<Carrier>` (sum-over-list)** — fold suffices; requires left-or-right-fold choice that compose-pair doesn't. Rejected (compose-pair + fold is standard).

---

## Implementation notes

### Bootstrap discovery

Compiler walks `d.declarations` for `value_body: Some(ValueBody::Structural { fields })` where the declaration's type resolves to `Dimension<_>`. Each such declaration is a Dimension instance; runs `analyze` against every workflow in the program.

### Workflow detection

"Workflow" is a structural concept — a `Bind` whose body is a pipeline (sequence of `Transform` nodes calling service operations). Specifically, a Bind qualifies as a workflow if its body chains multiple `Transform` nodes targeting `Callable` or `ExternalRealization` callables that have declared effects.

This detection logic (`flatten_workflow`) lives in `std/workflows.dag` (new) and is itself target-agnostic substrate code.

### Test framework integration

Each Dimension instance also gets test-obligation generation (analogous to `generate_idempotency_obligations` in `dsl/std/effects.dag`):

```dag
fn generate_test_obligations<Carrier>(
  d: Dag,
  dim: Dimension<Carrier>
) -> List<TestObligation>
```

Each dimension declares what test shape to emit. For idempotency: `f(f(x)) == f(x)`. For space bounds: "runs within declared memory limit." These materialize via Lane 2 Stage 2c.

### Sequencing: Lane 2 ordering

Current Lane 2 master has stages 2b (idempotency) → 2d (symbolic cost) → 2e (parallelism) → 2f (user dimensions). This design suggests:

1. 2b implements idempotency DIRECTLY (bespoke lens)
2. 2d, 2e same
3. 2f extracts the `Dimension` abstraction and refactors 2b/2d/2e to use it
4. Lane 4 4b/4c ADD NEW dimensions using the abstraction

This ordering avoids over-engineering early (2b works without the abstraction) and harvests the pattern when it's visible from 3 concrete examples.

Alternate ordering: implement Dimension first in 2a prep, then 2b/2d/2e use it from the start. Cleaner but risks designing the abstraction against fewer examples. Choose at Lane 2 kickoff.

---

## Associations

- **Lane 2 Stage 2f** ([lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md)) — this design IS Stage 2f's output
- **Lane 2 Stages 2b/2d/2e** — will be refactored to use Dimension (or built against it from the start)
- **Lane 4 Stage 4b** ([lane4-completion.md](./lane4-completion.md)) — side effects as Dimension instance
- **Lane 4 Stage 4c** — space bounds as Dimension instance
- **DB-1 `Correction` shape** ([design-correction-shape.md](./design-correction-shape.md)) — diagnostics emitted by `break_diagnostic` use Correction
- **Create `src/v3/std/dimensions.dag`** — new file with `Dimension<C>`, `Witness<C>`, `DimensionReport<C>`, `analyze` function
- **Create `src/v3/std/workflows.dag`** — `flatten_workflow` helper + workflow detection
- **Update `src/v3/lenses/`** — each property lens becomes a Dimension instance
- **Thesis anchor** — THESIS.md §"Correctness is not one property — it is many orthogonal dimensions"

---

## Acceptance (Lane 2 Stage 2f owns)

- [ ] `std/dimensions.dag` declares `Dimension<C>`, `Witness<C>`, `DimensionReport<C>`, `analyze` with receipts 🟢
- [ ] At least 3 built-in dimension instances (idempotency, symbolic cost, parallelism) implemented via Dimension
- [ ] Lane 4 Stages 4b/4c add side effects and space bounds as Dimension instances with ~20 lines of lens code each (proving the abstraction)
- [ ] A user-declared dimension example (e.g., `memory_bounded`) in a test fixture compiles and enforces correctly
- [ ] Monoid laws documented as requirement on Dimension authors

---

## Open questions

1. **Does parallelism fit the Dimension abstraction?** Parallelism composition is about identifying independent subgraphs, not per-operation evidence. May need a different shape OR a Dimension whose Carrier is a whole dependency graph structure. Evaluate at Lane 2 Stage 2e kickoff; if it doesn't fit cleanly, accept that parallelism lives outside the abstraction (it's still a valid lens, just not a Dimension).

2. **Algebra-law enforcement on Dimension authors?** Monoid laws are a requirement but not mechanically checked. Follow-up work: add `monoid_witness: MonoidInstance<Carrier>` field to `Dimension` that ties to the algebra.dag inhabitance system. Deferred.

3. **Cross-dimension interaction?** E.g., a Dimension that references another ("space bound must be respected alongside idempotency"). Deferred; first concrete case arises in Lane 4 if at all.

4. **How does `analyze` handle branches in workflows?** Current sketch assumes linear sequencing of operations. Real workflows have conditionals. Each Dimension's `compose` may need to handle branch vs sequential composition differently. Decision: `compose_branch` as an optional extra field on Dimension, defaulting to `compose`. Add when first branched workflow fixture demands it.
