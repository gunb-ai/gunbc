> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 1 Stage 1b, all of Lane 2, Lane 2 Stage 2f, all of Lane 4

# Design DB-5 — Substrate keyed-lookup API

**Design blocker:** DB-5
**Consumers:** Lane 1 Stage 1b (implementation); Lane 2 (all lenses consume); Lane 4 Stages 4b/4c (new dimension lenses)
**Status:** Design ready for implementer review.

---

## Problem

The meta-review flagged: `Dag.ports` and `Dag.nodes` are linear lists, so every lens reinvents `find_port(ports, id)` and `find_behavior(nodes, id)`. That reconstruction is debt-shifting — the substrate stays weak; consumers pay forever.

Rust-side `Dag` has `dag.port(id) -> &DagPort` and `dag.node(id) -> &Behavior` as total accessors backed by HashMaps. But the `.dag` layer doesn't see these; `src/v3/std/substrate.dag` only declares the linear lists.

Lane 1 Stage 1b's design doc ([lane1-stage-b-substrate-keyed-lookup.md](./lane1-stage-b-substrate-keyed-lookup.md)) names the fix in concept. This doc finalizes the concrete API so implementers have no ambiguity.

---

## Design

### Canonical substrate.dag additions

```dag
// src/v3/std/substrate.dag additions

type Dag {
  declarations: List<Declaration>
  nodes: List<Behavior>
  ports: List<DagPort>
  // NEW — keyed accessors, materialized eagerly at Dag construction
  port_by_id: Map<PortId, DagPort>
  node_by_id: Map<NodeId, Behavior>
}

// 🟢 TERMINAL. Keyed accessors are durable substrate facts.
// Lenses MUST consume these, not reconstruct locally (INVARIANTS.md L-7).

// Function-style accessor (preferred API for .dag lenses)
fn port(d: Dag, id: PortId) -> DagPort? {
  lookup(d.port_by_id, id)
}

fn node(d: Dag, id: NodeId) -> Behavior? {
  lookup(d.node_by_id, id)
}

// Bind-pass-through resolver. Walks through Bind naming hops until a
// real producer (Value/Transform/Branch/Loop) is reached, or until
// the walk hits a port with no producer (parameter → None).
fn resolve_producer(d: Dag, port_id: PortId) -> Behavior? {
  match port(d, port_id) {
    None => None
    Some(p) => match p.produced_by {
      None => None
      Some(node_id) => resolve_bind_chain(d, node_id)
    }
  }
}

fn resolve_bind_chain(d: Dag, node_id: NodeId) -> Behavior? {
  match node(d, node_id) {
    None => None
    Some(behavior) => match behavior {
      Bind(b) => resolve_producer(d, b.value)
      other => Some(other)
    }
  }
}
```

### Rust-side mapping

`src/v3/compiler/src/dag.rs` already has:
- `dag.port(id: PortId) -> &DagPort` (total; panics on unknown id — treats as invariant)
- `dag.node(id: NodeId) -> &Behavior` (total, panics same)
- Backing storage is `HashMap<PortId, DagPort>`, `HashMap<NodeId, Behavior>`

Change: expose these maps as public fields `port_by_id`, `node_by_id` (or continue as methods; the .dag-side `port(d, id)` lowers to the Rust method call). The substrate reflection bootstrap that builds Rust's `Dag` from the `.dag` `type Dag` declaration must include the new fields.

**Note on Option vs panic**: Rust-side uses panic-on-unknown (fail-closed treat-as-invariant). `.dag`-side returns `Option<T>` because `.dag` can't panic structurally. Consumers unwrap with `match`; panic path happens only at the Rust boundary for malformed substrate.

### Lookup primitive on `Map<K, V>`

`Map<K, V>` is a new substrate type. `std.map.dag` (new):

```dag
// src/v3/std/map.dag
module std.map

import std.list { List }

// 🟢 TERMINAL. Structural map over a keyed collection. Realized as
// Rust HashMap / Go map / Python dict / etc. via per-target spec.
type Map<K, V>

fn lookup<K, V>(m: Map<K, V>, key: K) -> V?

fn insert<K, V>(m: Map<K, V>, key: K, value: V) -> Map<K, V>

fn contains<K, V>(m: Map<K, V>, key: K) -> Bool

fn keys<K, V>(m: Map<K, V>) -> List<K>

fn values<K, V>(m: Map<K, V>) -> List<V>
```

Each function has per-target realization in `spec/rust.dag`, `spec/python.dag`, etc. (uses existing extdeps realization pattern).

### Migration pattern for existing lenses

Before (pre-L1 1b):
```dag
// in complexity.dag
fn lookup_cost(acc: List<CostEntry>, port_id: PortId) -> CostLookup =
  match acc {
    Empty => MissingCost
    Cons(payload) =>
      if payload.head.port == port_id then payload.head.cost
      else lookup_cost(payload.tail, port_id)
  }
```

After:
```dag
// accumulator IS a Map now
fn compute_costs(d: Dag) -> Map<PortId, CostLookup> =
  fold(d.nodes, initial_cost_map(d), |acc, behavior|
    insert(acc, behavior_port(behavior), entry_for(acc, behavior))
  )

// lookup is the substrate primitive, not a lens-local helper
fn cost_of(d: Dag, port_id: PortId) -> CostLookup =
  match lookup(compute_costs(d), port_id) {
    None => MissingCost
    Some(c) => c
  }
```

Gain: no linear walk, no local `lookup_cost` function, standard `Map` API.

### Invariant L-7 (new)

Added to `INVARIANTS.md`:

```markdown
**L-7 — Lenses consume declared substrate accessors, not local reconstructions.**

A lens (`.dag` consumer of substrate facts) MUST NOT define its own
`find_port`, `find_behavior`, `resolve_producer`, or similar lookup
helpers. These are declared once in `src/v3/std/substrate.dag` and
consumed by every lens.

**Why:** local reconstruction forks the substrate's truth. When the
substrate gains a new edge type (e.g., alias nodes, macro expansions),
every lens must learn it independently. Centralized accessors make
substrate extensions propagate automatically.

**Mechanical gate:** CI grep check blocks new local accessor declarations.

```
grep -nE "fn (find_port|find_behavior|resolve_producer|lookup_node|lookup_port)" src/v3/lenses/*.dag
```

Zero matches required. New accessors go in substrate.dag.

**Origin:** meta-review 2026-04-17.
```

---

## Rationale

**Why function API `port(d, id)` not method `d.port(id)`?** The `.dag` language's method-call syntax is field-access-like; `d.port_by_id` is a field access that yields a Map. The function-style `port(d, id)` composes cleanly as a substrate primitive that's obviously pure. Method syntax can be added later as sugar if desired.

**Why `port_by_id` as a field, AND `port(d, id)` as a function?** The field is the canonical storage; the function is the consumer-facing API. Lenses call `port(d, id)`, which dispatches to `lookup(d.port_by_id, id)`. Separates "how it's stored" from "how it's used."

**Why `resolve_producer` walks Bind hops?** Every lens today manually chains through Bind. Centralizing this one walk means new substrate Bind-like nodes (aliases, macro expansions) automatically propagate to every lens.

**Why not just add `dag.producer_of(port) -> Behavior?` directly?** Because `resolve_producer` does TWO things: lookup the port, then chain through Bind. Splitting them lets lenses stop at the port level if they don't want Bind pass-through (rare, but real — e.g., a lens checking "was this port directly produced by a Bind?" wants `node(d, port.produced_by)`, not the recursive resolve).

**Why `Map<K, V>` as a new substrate type?** Because it's needed structurally, not just for this feature. Many upcoming lenses want keyed state (idempotency lens accumulates `Map<OperationId, EffectShape>`, etc.). Declaring `Map` once in std is the single-authority move.

**Why `lookup` returns `Option<V>` on `.dag` side?** No panic primitive in `.dag`. Fail-closed consumers match on None; the substrate boundary translates Rust's `HashMap::get -> Option<&V>` directly.

---

## Rejected alternatives

**Keep linear-list-only substrate, rely on local memoization in each lens** — debt compound. Rejected.

**Expose the `HashMap<K, V>` type directly from Rust as substrate type** — leaks Rust implementation. Targets that aren't Rust (Go's `map`, Python's `dict`, Verilog's … no-equivalent) need a language-agnostic substrate type. `Map<K, V>` as substrate-layer abstraction is the clean move. Rejected.

**`resolve_producer` returns `NodeId?` instead of `Behavior?`** — forces every caller to then call `node(d, id)` anyway. Combine them. Rejected.

**Add `port_producer(d, port) -> Behavior?` as the only new primitive, skip `resolve_producer`** — doesn't handle Bind pass-through. Every lens still needs the recursive chain. Rejected (covered by both).

---

## Implementation notes

### Rust side

1. `substrate_reflection` (the pass that builds Rust's `Dag` type from `substrate.dag` declarations) learns to recognize `Map<K, V>` and generate `HashMap<K, V>` storage.
2. `Dag::new` (or equivalent constructor) populates `port_by_id` and `node_by_id` after the linear-list population. Single pass, no ordering dependency issues.
3. Existing `dag.port(id)` and `dag.node(id)` methods stay (they ARE the implementation of the new `.dag` function `port(d, id)`). Signature unchanged.

### .dag side

1. Existing three lenses (complexity, provenance, unused_parameters) migrate to use `lookup(d.port_by_id, id)` OR (preferred) `port(d, id)`.
2. Delete local `find_port`, `find_behavior`, `lookup_cost`-on-list, `contains`-on-referenced-list, etc. Substitute with the `Map` API.
3. Line count should drop meaningfully (≥20% across the three lens files, per Lane 1 Stage 1b acceptance).

### Testing

- Unit test per new accessor: construct a `Dag`, verify `port(d, id)` for known id returns `Some`, for unknown id returns `None`.
- Integration: migrated lenses pass their existing oracle tests.
- CI gate: grep check for local accessor declarations (L-7).

---

## Associations

- **Lane 1 Stage 1b** ([lane1-stage-b-substrate-keyed-lookup.md](./lane1-stage-b-substrate-keyed-lookup.md)) — this is the concrete API for that stage's implementation
- **Lane 2 all stages** ([lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md)) — new lenses consume these accessors exclusively
- **Lane 4 Stages 4b/4c** ([lane4-completion.md](./lane4-completion.md)) — side effects / space bounds lenses same pattern
- **Update `src/v3/std/substrate.dag`** — add `port_by_id`, `node_by_id` fields + function accessors
- **Create `src/v3/std/map.dag`** — new; Map<K, V> with lookup/insert/contains/keys/values
- **Update `src/v3/lenses/complexity.dag`, `provenance.dag`, `unused_parameters.dag`** — migrate
- **Update `INVARIANTS.md`** — add L-7

---

## Acceptance (Lane 1 Stage 1b owns)

- [ ] `Map<K, V>` declared in `src/v3/std/map.dag` with lookup/insert/contains/keys/values primitives
- [ ] `Dag.port_by_id`, `Dag.node_by_id` fields added to `substrate.dag`
- [ ] `port(d, id)`, `node(d, id)`, `resolve_producer(d, id)` functions declared in `substrate.dag`
- [ ] Rust side: `substrate_reflection` handles `Map<K, V>` via `HashMap<K, V>`; existing `dag.port(id)`, `dag.node(id)` continue to work
- [ ] Three existing lenses migrated; oracle tests pass; line count reduced ≥20%
- [ ] `INVARIANTS.md` L-7 entry added with grep-gate command
- [ ] CI job runs the grep gate and fails on matches
- [ ] Zero local `find_port`/`find_behavior`/`resolve_producer`/`lookup_node`/`lookup_port` declarations in `src/v3/lenses/`

---

## Open questions

1. **`Map<K, V>` insertion order semantics?** `HashMap` iteration order is nondeterministic. If a lens emits something based on `Map` iteration, output is nondeterministic. Probably fine for lenses (they aggregate to a terminal value), but flag for Lane 3 Stage 3c (fixed-point ratchet) — if emission ever iterates a Map, need `BTreeMap` backing or sorted iteration. Defer decision until concrete case arises.

2. **Should `Map` support `remove`?** Not in initial API. Add when a consumer needs it.

3. **Should `resolve_producer` short-circuit on the first producer, or always recurse through nested Binds?** Current design: recurse always (walks through arbitrary Bind chains). If a lens needs "stop at the first Bind," add `producer_or_bind(d, port) -> Behavior?` as a separate primitive. Defer.
