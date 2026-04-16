# Dependency Graph, Ownership, and Parallelism — Unified Design

> **Parent docs:** `THESIS.md`, `INVARIANTS.md` §"Facts Flow
> Forward", `src/v3/SELF_HOSTING.md` §14.7.
>
> **Purpose:** the DAG's dependency graph is the shared
> foundation for ownership, parallelism, complexity, provenance,
> and dead-code detection. This doc defines the compositional
> `.dag` model and each projection.
>
> **Status:** design for review. Core insight validated
> (read-vs-construct dropped clones 72→6). Pressure-tested
> and reworked: §3.1 reframed as derived fact, §3.5/§3.7
> merged, compositional shape spelled out.

---

## §1. Motivation

v3's emitter currently inserts `.clone()` on every port
reference. The first generated artifact (287 lines) contains
90 clones. All 90 are reads — function parameters, match
scrutinees, field accesses. No consumer needs ownership.

The first-pass fix (classify immediate consumers as read vs
construct) dropped this to 6. But 5 of the 6 remaining expose
a deeper issue: whether a callee consumes its parameter is not
decidable from the caller's behavior type. `cons(head, tail)`
and `is_empty(list)` are both `Transform(callable, [args])` in
the substrate, but `cons` embeds its arguments in the return
value while `is_empty` only inspects them.

The fix is not more special cases. The fix is to model
dependency facts as a compositional `.dag` concept — computed
once from the DAG, consumed by every projection (ownership,
parallelism, complexity) through layer opacity.

---

## §2. The dependency graph is already in the DAG

The raw dependency information is created at lowering and
persists unchanged through inference to emission:

- **Forward edges:** `behavior.inputs` = which ports this
  node reads
- **Backward edges:** `port.produced_by` = which node
  produced this value

No analysis creates this graph. Lowering creates it. It
persists. The question is not "how do we compute dependencies"
but "how do we make the derived views composable."

---

## §3. The compositional shape

### §3.1 Types in `std/dependency.dag`

```dag
module std.dependency

import std.list { List }
import std.substrate { Dag, PortId, NodeId, DeclarationId }

// Whether a callee consumes (embeds in return value) or
// borrows (inspects and discards) a parameter.
type ParameterDisposition = Consumed | Borrowed

// Per-edge fact: who consumes this port, and how?
type ConsumerEdge {
  port: PortId               // the value being consumed
  consumer: NodeId           // the behavior that uses it
  disposition: ParameterDisposition  // consumed or borrowed
}

// Per-port derived view: all consumers + their dispositions.
type PortConsumers {
  port: PortId
  edges: List<ConsumerEdge>
  count: Int                 // length(edges)
}

// Per-callable derived view: does each parameter get consumed?
type CallableConsumption {
  callable: DeclarationId
  params: List<ParameterDisposition>  // one per parameter
}

// The complete dependency index — computed once from the DAG.
type DependencyFacts {
  consumers: List<PortConsumers>
  callable_consumption: List<CallableConsumption>
}
```

### §3.2 Computing dependency facts

```dag
fn compute_dependencies(dag: Dag) -> DependencyFacts
```

A pure function. Reads the DAG's `produced_by` edges and
behavior input lists. Returns the derived views. The
computation has two parts:

**Part 1: Consumer index.** For each behavior node, record
its input ports as consumer edges. This is a reverse-index
of `produced_by` — straightforward scan.

**Part 2: Parameter consumption.** For each callable (Arrow
declaration with a body), determine whether each parameter
is consumed or borrowed. Two authorities by callable kind:

- **.dag function (UserDefined body):** derive from the body
  DAG. Walk from the function's return port backward. If the
  parameter port is reachable through a construct site
  (record field assignment, list cons, return value), the
  parameter is `Consumed`. If the parameter is only reachable
  through reads (function arguments, match scrutinees, field
  access), it is `Borrowed`.

  **No annotation on Arrow.** The body IS the authority. The
  function's Arrow declaration carries no consumption field.
  This is structurally derivable; declaring it would create a
  parallel representation.

- **ExternalRealization:** declared in the realization spec.
  Each `CallableRealization` in `rust.dag` declares parameter
  dispositions:

  ```dag
  data rust_cons: CallableRealization = {
    strategy: ListCons
    param_disposition: [Consumed, Consumed]  // head + tail
  }

  data rust_is_empty: CallableRealization = {
    strategy: ListIsEmpty
    param_disposition: [Borrowed]  // list
  }

  data rust_fold: CallableRealization = {
    strategy: ListFold
    param_disposition: [Borrowed, Consumed, Borrowed]
    // list: borrowed, init: consumed (becomes acc), fn: borrowed
  }
  ```

### §3.3 Why this is one fact, not two open edges

The earlier draft (§3.5 and §3.7) separated "transitive
ownership transfer" from "fold accumulator linearity." They
are the same question: *does this callee consume this
parameter?*

- `cons(head, tail)` — both parameters consumed (embedded in
  return Cons node)
- `fold(list, init, fn)` — init consumed (becomes first acc),
  list and fn borrowed
- `is_empty(list)` — list borrowed (inspected and discarded)
- `transform(x)` — depends on what `transform` does with `x`

One fact (`ParameterDisposition`), two authorities by callable
kind (.dag body derivation vs realization spec declaration).

---

## §4. Projections

Each projection reads `DependencyFacts` through the typed
interface. Layer opacity applies — projections don't see how
the facts were computed.

### §4.1 Ownership

```dag
fn compute_ownership(
  dag: Dag,
  deps: DependencyFacts,
  rendering: RenderingModel
) -> OwnershipFacts
```

For each consumer edge in `deps.consumers`:
- If `disposition == Borrowed` → render as `read`
  (Rust: `&T`, Go: pass by value)
- If `disposition == Consumed` → render as `construct`
  (Rust: move if last use, clone if non-Copy + non-last,
  copy if Copy type)

**Copy type classification.** A type is Copy if:
- Leaf type declared `is_copy: true` in realization spec
  (Int, Bool, PortId, NodeId — primitives)
- Compound type (Conj) where ALL fields are Copy types
  (derived, not declared — prevents drift)

**Last-use determination.** A consumed edge is the "last use"
if no subsequent consumer of the same port also has
`disposition == Consumed`. Derived from evaluation order +
consumer list in the dependency index.

**Target rendering model:**

```dag
type RenderingModel {
  borrow_syntax: String      // Rust: "&{V}"
  move_syntax: String        // Rust: "{V}"
  clone_syntax: String       // Rust: "{V}.clone()"
  deref_syntax: String       // Rust: "*{V}" (Copy deref)
}
```

The rendering decision table for Rust:

| Disposition | is_copy | Last consumed use? | Rendering |
|-------------|---------|-------------------|-----------|
| Borrowed | any | n/a | `&value` |
| Consumed | true | any | `*value` |
| Consumed | false | yes | `value` (move) |
| Consumed | false | no | `value.clone()` |

### §4.2 Parallelism

```dag
fn detect_parallelism(
  dag: Dag,
  deps: DependencyFacts
) -> List<IndependenceFact>
```

Two ports are independent if their transitive dependency sets
don't overlap. For pure .dag code, independent operations are
always safe to parallelize (immutable values, no side effects).

**Fold decomposition:** in a fold's body lambda, which nodes
transitively depend on the `acc` parameter? Acc-independent
nodes are the "map" part (parallelizable). Acc-dependent nodes
are the "reduce" part (sequential). If all per-element work is
acc-independent, the fold IS a map.

The `ParameterDisposition` on the fold's `acc` parameter
reinforces this: if acc is `Consumed` (embedded in return),
the fold is genuinely sequential. If some body sub-expression
doesn't reach acc, that sub-expression is a parallel map.

**Effects boundary.** For `ExternalRealization` operations
(side effects), the effects lens (L2 M3) classifies
operations. Independent + pure → parallel. Independent +
effectful → needs synchronization from target spec.

### §4.3 Complexity

```dag
fn compute_complexity(
  dag: Dag,
  deps: DependencyFacts
) -> CostReport
```

The dependency graph's longest chain = critical path =
inherent sequential cost. `lens_cost` already walks
`produced_by`. With `DependencyFacts`, it can also report:
- Critical path length
- Parallelizable fraction (total work - critical path)
- Clone cost from ownership (clone = O(n), move = O(1))

### §4.4 Provenance + dead code

**Provenance:** one-hop backward — `port.produced_by`.
Already `lens_provenance`. Trivial read of existing substrate.

**Dead code:** `PortConsumers.count == 0` and not a function
return → dead. Emitter skips it.

---

## §5. The pipeline composition

```dag
fn compile(source: String, file: String, spec: LanguageSpec) -> String {
  let dag        = parse(source, file) |> lower |> infer
  let deps       = compute_dependencies(dag)
  let ownership  = compute_ownership(dag, deps, spec.rendering)
  let complexity = compute_complexity(dag, deps)
  emit(dag, ownership, complexity, spec)
}
```

Each projection reads `DependencyFacts`. None rebuilds the
index. The dependency computation runs once after inference.
All downstream projections compose on top of it.

---

## §6. Validated result and remaining gap

### §6.1 What the first-pass model achieved

The read-vs-construct classification at the immediate
consumer level dropped generated lens clones from 72 to 6.
This validates the DIRECTION: borrowing reads and cloning
only at construct sites is correct.

### §6.2 What the first pass got wrong

5 of the 6 remaining clones are from cases where the
immediate classification is insufficient:

- **3 fold accumulator clones:** Rust's `fold` closure takes
  `acc` by value (owned). The emitter classified `acc` as a
  function parameter (borrowed) but the fold calling convention
  gives ownership. Fix: `rust_fold`'s realization declares
  `param_disposition: [Borrowed, Consumed, Borrowed]`.

- **2 Copy-type clones:** `PortId` is Copy. `.clone()` works
  but `*value` (deref) is correct. Fix: derive `is_copy` from
  realization spec + field composition.

### §6.3 The 1-clone target

After implementing `ParameterDisposition` and `is_copy`
derivation, the expected clone count is **1**: the `SourceSpan`
field in the `UnusedParameter` record literal (SourceSpan
contains String, which is not Copy, and the span is borrowed
from the input BindNode while the record needs to own it).

This single clone is genuinely necessary — it's a non-Copy
value at a construct site where the source is borrowed. The
model correctly identifies it.

---

## §7. Verification approach

The model claims: if `ParameterDisposition` is correctly
computed for every callable, and `is_copy` is correctly
derived for every type, then the emitter produces correct
code with minimal clones. No class of "missing clone" bugs
exists — a wrong result means the fact computation is wrong.

**Tests verify the facts, not the symptoms:**

1. **ParameterDisposition tests.** For each callable in std/:
   assert the derived disposition matches the expected one.
   `cons` → [Consumed, Consumed]. `is_empty` → [Borrowed].
   `fold` → [Borrowed, Consumed, Borrowed].

2. **is_copy derivation tests.** Int → true. PortId → true.
   SourceSpan → false (contains String). A record of all-Copy
   fields → true. A record with one non-Copy field → false.

3. **Rendering parity.** Generated lens matches handwritten
   oracle (already tested). If the disposition or is_copy fact
   is wrong, parity fails.

4. **Roundtrip compilation.** Every generated artifact compiles
   with rustc. Rust's borrow checker rejects code where the
   model borrows but the callee needs ownership.

5. **Clone-count pinning.** Exact clone count on generated
   lens output, pinned at 1 after full implementation. Ratchet
   only goes down.

---

## §8. Phasing

| Phase | When | What |
|---|---|---|
| **Phase 1** | L1.5 | `std/dependency.dag` types. `compute_dependencies` with consumer index (Part 1) only — no callee consumption analysis yet. Emitter renders `&T` for all function params (safe conservative default). `is_copy` on leaf types in rust.dag. Expected: ~6 clones (current state). |
| **Phase 2** | L2 | Callee consumption analysis (Part 2). `param_disposition` on ExternalRealization callables. Derivation from body DAG for .dag callables. `is_copy` composition for compound types. Fold accumulator rendered with owned `mut acc`. Expected: 1 clone. |
| **Phase 3** | L2+ | Parallelism detection (independence, fold decomposition). Complexity reads DependencyFacts for critical path. Dead-code skipping. |
| **Phase 4** | L3 | Self-analysis. Clone count at zero on generated compiler code. Parallel emission for independent pipeline stages. |

---

## §9. When this doc updates

- Phase 1 lands → §8 graduates, clone count pinned at ~6
- Phase 2 lands → callee consumption verified, clone count
  pinned at 1
- All phases → doc archives
