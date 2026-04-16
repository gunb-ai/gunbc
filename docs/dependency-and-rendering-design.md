# Dependency Graph, Ownership, and Parallelism — Unified Design

> **Parent docs:** `THESIS.md`, `INVARIANTS.md` §"Facts Flow
> Forward", `src/v3/SELF_HOSTING.md` §14.7.
>
> **Purpose:** the DAG's dependency graph is the shared
> foundation for ownership, parallelism, complexity, provenance,
> and dead-code detection. This doc defines the compositional
> `.dag` model — three typed layers with explicit boundaries —
> and each projection that reads them.
>
> **Status:** design for review.

---

## §1. Motivation

v3's emitter inserts `.clone()` on every port reference. The
first generated artifact (287 lines) contains 90 clones. All
90 are reads. The first-pass fix (classify immediate consumers
as read vs construct) dropped this to 6 — but 5 of the 6
expose a deeper issue: whether a callee consumes its parameter
is not decidable from the caller's behavior type.

`cons(head, tail)` and `is_empty(list)` are both
`Transform(callable, [args])` in the substrate. But `cons`
embeds its arguments in the return value while `is_empty`
only inspects them. The immediate behavior type is the same.
The distinction lives in the callee.

The fix is not more special cases. The fix is three typed
layers with explicit boundaries: spec-independent structure,
per-callable consumption, and target-specific rendering.
Projections compose on top with layer opacity.

---

## §2. Three layers

### Layer 1: `DependencyFacts` — spec-independent DAG structure

Computed from the DAG alone. No realization spec. No target
knowledge. Pure structural facts.

```dag
module std.dependency

import std.list { List }
import std.substrate { Dag, PortId, NodeId }

type ConsumerEdge {
  port: PortId           // the value being consumed
  consumer: NodeId       // the behavior that uses it
}

type PortConsumers {
  port: PortId
  edges: List<ConsumerEdge>
  count: Int
}

type DependencyFacts {
  consumers: List<PortConsumers>
}

fn compute_dependencies(dag: Dag) -> DependencyFacts
```

This is a reverse index of `produced_by` — for each port,
who reads it? Built by scanning all behaviors' input lists.
Spec-independent. Target-independent. Pure DAG structure.

**Why Layer 1 has no `consumed` flag.** Whether a callee
consumes or borrows its parameter depends on the callee's
body (for .dag) or the realization spec (for external). That
requires reading something beyond the DAG. Putting `consumed`
here would leak target knowledge into a target-agnostic fact.
Parallelism reads Layer 1 only — it must stay spec-free.

### Layer 2: `ConsumptionFacts` — per-callable parameter disposition

Computed from the DAG AND the realization spec. This is where
target knowledge enters.

```dag
module std.consumption

import std.list { List }
import std.substrate { Dag, DeclarationId }

type ParameterDisposition = Consumed | Borrowed

type CallableConsumption {
  callable: DeclarationId
  params: List<ParameterDisposition>
}

type ConsumptionFacts {
  callables: List<CallableConsumption>
}

fn compute_consumption(dag: Dag, spec: RealizationSpec) -> ConsumptionFacts
```

Two authorities by callable kind:

**.dag function (UserDefined body):** derive from the body
DAG. Walk from the function's return port backward through
`produced_by` edges. If a parameter port is reachable AND the
path passes through a construct site (record field, list cons,
return value), the parameter is `Consumed`. If the parameter
is only reachable through reads (function call arguments,
match scrutinees, field access), it is `Borrowed`. If the
parameter is not reachable from the return at all, it is
`Borrowed` (unused — the unused-parameters lens catches this
separately).

**No annotation on Arrow.** The body IS the authority for .dag
callables. Adding a consumption field to Arrow would be a
parallel representation of a derivable fact.

**ExternalRealization:** declared in the realization spec.
Each `CallableRealization` in `rust.dag` declares parameter
dispositions:

```dag
data rust_cons: CallableRealization = {
  strategy: ListCons
  param_disposition: [Consumed, Consumed]
}

data rust_is_empty: CallableRealization = {
  strategy: ListIsEmpty
  param_disposition: [Borrowed]
}

data rust_fold: CallableRealization = {
  strategy: ListFold
  param_disposition: [Borrowed, Consumed, Borrowed]
  // list: borrowed, init: consumed (becomes acc), fn: borrowed
}
```

**Why this is one concept, not two open edges.** The earlier
draft separated "transitive ownership transfer" (§3.5) from
"fold accumulator linearity" (§3.7). They are the same fact:
*does this callee consume this parameter?* `cons` consumes
both. `fold` consumes init. `is_empty` consumes nothing. One
type (`ParameterDisposition`), two authorities by callable
kind.

### Layer 3: Projections — target-specific rendering

Each projection reads Layer 1, Layer 2, or both through
typed interfaces. Layer opacity applies.

```dag
// Spec-free — reads Layer 1 only
fn detect_parallelism(dag: Dag, deps: DependencyFacts) -> List<IndependenceFact>

// Reads Layer 1 + Layer 2 + target spec
fn compute_ownership(dag: Dag, deps: DependencyFacts, cons: ConsumptionFacts, rendering: RenderingModel) -> OwnershipFacts

// Reads Layer 1 + Layer 2 (clone cost from ownership)
fn compute_complexity(dag: Dag, deps: DependencyFacts, cons: ConsumptionFacts) -> CostReport

// Reads Layer 1 only (one-hop)
fn compute_provenance(dag: Dag, deps: DependencyFacts) -> List<ProvenanceFact>
```

Spec-coupling is visible in the signature. Parallelism takes
no spec — two nodes are independent regardless of target.
Ownership takes the spec — Rust borrows, Go passes by value.
The layer boundary is the function signature.

---

## §3. Projection detail: Ownership

### §3.1 The rendering decision

For each consumer edge, the emitter looks up:
1. `ParameterDisposition` from Layer 2 (consumed or borrowed)
2. `is_copy` from the type's realization (Copy or not)
3. Last-use from Layer 1's consumer list (is this the final
   consumed edge for this port?)

**Rendering table for Rust:**

| Disposition | is_copy | Last consumed? | Rendering |
|-------------|---------|----------------|-----------|
| Borrowed | any | n/a | `&value` |
| Consumed | true | any | `*value` (deref) |
| Consumed | false | yes | `value` (move) |
| Consumed | false | no | `value.clone()` |

For Go: always `value`. No distinctions needed.

### §3.2 Copy type derivation

- **Leaf types:** `is_copy` declared in realization spec
  (Int → true, Bool → true, PortId → true, String → false)
- **Compound types (Conj):** `is_copy` = ALL fields are Copy.
  Derived from field types, not declared. Prevents drift —
  you can't accidentally declare a String-containing record
  as Copy.

### §3.3 Target rendering model

```dag
type RenderingModel {
  borrow_syntax: String      // Rust: "&{V}"
  move_syntax: String        // Rust: "{V}"
  clone_syntax: String       // Rust: "{V}.clone()"
  deref_syntax: String       // Rust: "*{V}"
}
```

### §3.4 Expected result

After full implementation (Layer 1 + Layer 2 + Copy
derivation), the generated unused_parameters lens should
have **1 clone**: `SourceSpan` (contains String, not Copy)
at a record construction site where the source is borrowed.

The 1-clone residue is genuinely necessary — a non-Copy
value at a construct site where the source is a borrow from
the input BindNode.

---

## §4. Projection detail: Parallelism

Reads Layer 1 only. Spec-free.

**Independence:** two ports are independent if their
transitive dependency sets don't overlap. For pure .dag code,
independent operations are always safe to parallelize.

**Fold decomposition:** in a fold body, which nodes
transitively depend on `acc`? Acc-independent nodes are the
"map" part (parallelizable). If all per-element work is
acc-independent, the fold IS a map.

**Effects boundary (future):** ExternalRealization operations
may have side effects. Independent + pure → parallel.
Independent + effectful → needs sync. L2 M3 work.

---

## §5. Projection detail: Complexity

Reads Layer 1 + Layer 2. The longest dependency chain is the
critical path. Clone cost comes from ownership (Layer 2):
clone = O(size), move = O(1), borrow = O(1).

---

## §6. The pipeline composition

```dag
fn compile(source: String, file: String, spec: LanguageSpec) -> String {
  let dag         = parse(source, file) |> lower |> infer
  let deps        = compute_dependencies(dag)          // Layer 1
  let consumption = compute_consumption(dag, spec)     // Layer 2
  let ownership   = compute_ownership(dag, deps, consumption, spec.rendering)
  let complexity  = compute_complexity(dag, deps, consumption)
  let parallelism = detect_parallelism(dag, deps)
  emit(dag, ownership, complexity, parallelism, spec)
}
```

Each projection reads typed facts. None rebuilds from the DAG.
The dependency computation runs once after inference.

---

## §7. Verification approach

The model claims: if `ParameterDisposition` is correctly
computed for every callable and `is_copy` is correctly derived
for every type, the emitter produces correct code with minimal
clones. No class of "missing clone" bugs exists — a wrong
result means the fact computation is wrong, not that the
emitter has a bug.

**Tests verify the facts:**

1. **ParameterDisposition per callable.** `cons` →
   [Consumed, Consumed]. `is_empty` → [Borrowed]. `fold` →
   [Borrowed, Consumed, Borrowed]. Assert against the
   computed `ConsumptionFacts`.

2. **is_copy derivation.** Int → true. PortId → true.
   SourceSpan → false. Record { a: Int, b: Int } → true.
   Record { a: Int, b: String } → false.

3. **Rendering parity.** Generated lens matches handwritten
   oracle on all fixtures.

4. **Roundtrip compilation.** Every generated artifact
   compiles with rustc.

5. **Clone-count pinning.** Exact count on generated lens:
   ~6 at Phase 1, 1 at Phase 2.

---

## §8. What v2 reconstructed vs what composes

| Fact | v2 (719 lines) | v3 (compositional) |
|---|---|---|
| Who consumes this port? | Walk tree, count names | Layer 1: `DependencyFacts.consumers` |
| Does callee consume param? | Not modeled | Layer 2: `ConsumptionFacts` |
| Copy type? | Hardcoded heuristics | Realization spec + field derivation |
| Last use? | Not modeled | Layer 1 consumer order |
| Independent operations? | Not modeled | Layer 1 transitive reach |
| Fold = map? | Not modeled | Layer 1 acc-reachability |
| Critical path? | Not modeled | Layer 1 longest chain |

---

## §9. Phasing

| Phase | When | What |
|---|---|---|
| **Phase 1** | L1.5 | Layer 1: `std/dependency.dag` types + `compute_dependencies`. Emitter renders `&T` for all params (conservative Borrowed default). `is_copy` on leaf types. Clone count ~6. |
| **Phase 2** | L2 | Layer 2: `std/consumption.dag` types + `compute_consumption`. `param_disposition` on ExternalRealization. Body-walk derivation for .dag callables. `is_copy` composition. Clone count → 1. |
| **Phase 3** | L2+ | Parallelism (Layer 1 only). Complexity reads both layers. Dead-code skipping. |
| **Phase 4** | L3 | Self-analysis. Clone count zero. Parallel emission. |

---

## §10. When this doc updates

- Phase 1 lands → clone count pinned at ~6
- Phase 2 lands → consumption verified, clone count → 1
- All phases → doc archives
