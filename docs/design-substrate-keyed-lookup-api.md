> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 1 Stage 1b, all of Lane 2, Lane 2 Stage 2f, all of Lane 4

# Design DB-5 — Substrate lookup API (functions, not duplicated state)

**Design blocker:** DB-5
**Consumers:** Lane 1 Stage 1b (implementation); Lane 2 (all lenses consume); Lane 4 Stages 4b/4c (new dimension lenses)
**Status:** Design ready for implementer review.

---

## Problem

The meta-review flagged: `Dag.ports` and `Dag.nodes` are linear lists, so every lens reinvents `find_port(ports, id)` and `find_behavior(nodes, id)`. That reconstruction is debt-shifting — the substrate stays weak; consumers pay forever.

Rust-side `Dag` has `dag.port(id) -> &DagPort` and `dag.node(id) -> &Behavior` as total accessors backed by HashMaps. But the `.dag` layer doesn't see these; `src/v3/std/substrate.dag` only declares the linear lists.

Lane 1 Stage 1b's design doc ([lane1-stage-b-substrate-keyed-lookup.md](./lane1-stage-b-substrate-keyed-lookup.md)) names the fix in concept. This doc finalizes the concrete API.

**Correction from an earlier revision:** an earlier version of this doc proposed adding `port_by_id: Map<PortId, DagPort>` and `node_by_id: Map<NodeId, Behavior>` as new fields on `Dag`. Meta-review on PR #491 correctly flagged that as parallel-authority debt: two representations of the same fact (the list AND the map), so modifying one requires modifying the other. That's forked authority, not dissolution.

**The corrected fix:** the substrate SHAPE stays as-is. The existing lists remain the single authority. What's new is **substrate-level query functions** that lenses consume. Rust-side implementation can (and does) cache in a HashMap for O(1) lookup; that cache is an implementation detail, not a substrate field.

---

## Design

### Canonical substrate.dag additions

```dag
// src/v3/std/substrate.dag additions

type Dag {
  declarations: List<Declaration>  // existing — unchanged
  nodes: List<Behavior>            // existing — SINGLE authority for node identity
  ports: List<DagPort>             // existing — SINGLE authority for port identity
  // NO new fields. Substrate schema is unchanged.
}

// 🟢 TERMINAL. Query functions over the authoritative lists.
// Lenses MUST consume these, not reconstruct locally (INVARIANTS.md L-7).
// Rust-side implementation may cache in a HashMap for O(1) lookup;
// that cache is NOT reflected in the substrate schema.

fn port(d: Dag, id: PortId) -> DagPort?

fn node(d: Dag, id: NodeId) -> Behavior?

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
- `dag.port(id: PortId) -> &DagPort` (total; panics on unknown id)
- `dag.node(id: NodeId) -> &Behavior` (total; panics same)
- Backing storage is `HashMap<PortId, DagPort>`, `HashMap<NodeId, Behavior>` — implementation cache

**No substrate changes to the Rust `Dag` struct.** The HashMap fields stay private implementation detail. What changes is the `.dag`-side: new query functions in `substrate.dag` that lenses can invoke. These functions lower to calls on the existing Rust methods (returning `Option<&T>` to match the `.dag`-side `T?` signature, with Rust's panic path preserved for genuinely malformed-substrate calls at the Rust boundary).

**Note on Option vs panic**: Rust-side accessors panic on unknown ids (treats as invariant violation — malformed substrate). `.dag`-side returns `Option<T>`. Consumers match on None; panic occurs only when an upstream caller bypassed the `.dag` function and called Rust directly with an id that isn't in the substrate.

### Optional lens-local Map usage

`Map<K, V>` remains a useful **lens-local** abstraction — not for the `Dag` schema, but for accumulators that benefit from keyed state.

`std.map.dag` (new):

```dag
// src/v3/std/map.dag
module std.map

import std.list { List }

// 🟢 TERMINAL. Generic keyed collection. Realized as Rust HashMap /
// Go map / Python dict / etc. via per-target spec. NOT used in the
// core substrate schema (Dag keeps List<DagPort>, List<Behavior>
// as authorities) — but available as a lens accumulator type.
type Map<K, V>

fn lookup<K, V>(m: Map<K, V>, key: K) -> V?

fn insert<K, V>(m: Map<K, V>, key: K, value: V) -> Map<K, V>

fn contains<K, V>(m: Map<K, V>, key: K) -> Bool

fn keys<K, V>(m: Map<K, V>) -> List<K>

fn values<K, V>(m: Map<K, V>) -> List<V>
```

### Migration pattern for existing lenses

**Substrate accessor usage** (the main migration):

Before — lens reconstructed lookup:
```dag
fn entry_for(acc: List<CostEntry>, behavior: Behavior) -> CostEntry =
  match behavior { ... }

// AND a local helper walking the linear list
fn lookup_cost(acc: List<CostEntry>, port_id: PortId) -> CostLookup =
  match acc {
    Empty => MissingCost
    Cons(payload) =>
      if payload.head.port == port_id then payload.head.cost
      else lookup_cost(payload.tail, port_id)
  }
```

After — local `lookup_cost` stays (it's operating on the lens's own accumulator, not on `d.ports`); what disappears is any local `find_port(d.ports, id)`. Those get replaced with `port(d, id)`:

```dag
// When the lens needs to look up by ID on the Dag's own ports:
// Before:
//   fn find_port(ports: List<DagPort>, id: PortId) -> DagPort? = ...
//   let p = find_port(d.ports, port_id)
// After:
//   let p = port(d, port_id)
```

**Optional lens-local Map usage** (secondary optimization):

```dag
// If the lens's accumulator would benefit from keyed lookup, use Map:
fn compute_costs(d: Dag) -> Map<PortId, CostLookup> =
  fold(d.nodes, initial_cost_map(d), |acc, behavior|
    insert(acc, behavior_port(behavior), entry_for(acc, behavior))
  )

fn cost_of(d: Dag, port_id: PortId) -> CostLookup =
  match lookup(compute_costs(d), port_id) {
    None => MissingCost
    Some(c) => c
  }
```

This is an optional optimization on a per-lens basis. The substrate-side lookup functions (`port`, `node`, `resolve_producer`) are the non-optional part.

### Invariant L-7 (new)

Added to `INVARIANTS.md`:

```markdown
**L-7 — Lenses consume declared substrate query functions, not local reconstructions.**

A lens (`.dag` consumer of substrate facts) MUST NOT define its own
`find_port`, `find_behavior`, `resolve_producer`, or similar lookup
helpers over `d.ports` or `d.nodes`. These are declared once in
`src/v3/std/substrate.dag` (`port(d, id)`, `node(d, id)`,
`resolve_producer(d, id)`) and consumed by every lens.

**Why:** local reconstruction forks the substrate's truth. When the
substrate gains a new edge type (e.g., alias nodes, macro expansions),
every lens must learn it independently. Centralized functions make
substrate extensions propagate automatically.

**Why functions, not fields:** the substrate has a single authority
for port/node identity — the existing `List<DagPort>` and
`List<Behavior>` fields. Adding parallel `Map<PortId, DagPort>` fields
would be duplicate authority. The query functions abstract over
implementation (Rust caches in HashMap; the lens doesn't see it).

**Mechanical gate:** CI grep check blocks new local accessor declarations.

```
grep -nE "fn (find_port|find_behavior|resolve_producer|lookup_node|lookup_port)" src/v3/lenses/*.dag
```

Zero matches required. New accessors go in substrate.dag.

**Origin:** meta-review 2026-04-17.
```

---

## Rationale

**Why function API `port(d, id)` not method `d.port(id)`?** The `.dag` language's method-call syntax is field-access-like. Function-style composes cleanly as a substrate primitive that's obviously pure. Method syntax can be added later as sugar if desired.

**Why NOT add `port_by_id` as a substrate field?** Because the List IS the authority; adding a Map of the same facts is parallel authority debt (meta-review on PR #491). Rust-side HashMap is an implementation cache, not a substrate shape concern. Lenses consume the query FUNCTIONS, which abstract over whatever backing the implementation uses. This preserves single authority.

**Why `resolve_producer` walks Bind hops?** Every lens today manually chains through Bind. Centralizing this one walk means new substrate Bind-like nodes (aliases, macro expansions) automatically propagate to every lens.

**Why not just add `dag.producer_of(port) -> Behavior?` directly?** Because `resolve_producer` does TWO things: lookup the port, then chain through Bind. Splitting them lets lenses stop at the port level if they don't want Bind pass-through (rare, but real).

**Why `Map<K, V>` as a std type (even though not in substrate schema)?** Because it's useful for lens-local keyed state. Declaring `Map` once in std is the single-authority move for that type; each lens doesn't invent its own.

**Why `lookup` returns `Option<V>` on `.dag` side?** No panic primitive in `.dag`. Fail-closed consumers match on None.

---

## Rejected alternatives

**Add `port_by_id: Map<PortId, DagPort>` as a substrate field (earlier revision of this doc)** — parallel authority debt flagged by meta-review. Rejected.

**Keep linear-list-only substrate, rely on local memoization in each lens** — debt compound (the original problem). Rejected.

**Expose the `HashMap<K, V>` type directly from Rust as substrate type** — leaks Rust implementation. Rejected.

**`resolve_producer` returns `NodeId?` instead of `Behavior?`** — forces every caller to then call `node(d, id)` anyway. Combine them. Rejected.

**Add `port_producer(d, port) -> Behavior?` as the only new primitive, skip `resolve_producer`** — doesn't handle Bind pass-through. Rejected.

---

## Implementation notes

### Rust side

1. No changes to `Dag` struct. HashMap fields stay private.
2. Expose `dag.port(id)` / `dag.node(id)` Rust methods to the `.dag` level via the substrate-reflection mechanism — when `.dag` calls `port(d, id)`, it lowers to `match dag.port(id) { ... }` at the Rust boundary, translating panic-on-unknown to `None` at the call site (or keeping panic if malformed substrate was the caller's contract).
3. Alternatively — and simpler — the Rust side exposes `dag.port_opt(id) -> Option<&DagPort>` and `.dag`'s `port(d, id)` lowers to that directly. This avoids panic-to-Option conversion complexity.

### .dag side

1. Existing three lenses (complexity, provenance, unused_parameters) migrate: replace any `find_port(d.ports, id)` / `find_behavior(d.nodes, id)` with `port(d, id)` / `node(d, id)`.
2. Delete corresponding local helpers.
3. Bind-chain walks replace with `resolve_producer(d, port_id)`.
4. Line count should drop meaningfully (≥15% across the three lens files, per Lane 1 Stage 1b acceptance — lower threshold than previous design since less code changes).

### Testing

- Unit test per function: construct a `Dag`, verify `port(d, id)` for known id returns `Some`, for unknown id returns `None`.
- Integration: migrated lenses pass their existing oracle tests.
- CI gate: grep check for local accessor declarations (L-7).

---

## Associations

- **Lane 1 Stage 1b** ([lane1-stage-b-substrate-keyed-lookup.md](./lane1-stage-b-substrate-keyed-lookup.md)) — this is the concrete API for that stage's implementation
- **Lane 2 all stages** ([lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md)) — new lenses consume these functions exclusively
- **Lane 4 Stages 4b/4c** ([lane4-completion.md](./lane4-completion.md)) — side effects / space bounds lenses same pattern
- **Update `src/v3/std/substrate.dag`** — add the three query functions; NO new fields
- **Create `src/v3/std/map.dag`** — Map<K, V> for lens-local use
- **Update `src/v3/lenses/complexity.dag`, `provenance.dag`, `unused_parameters.dag`** — migrate
- **Update `INVARIANTS.md`** — add L-7
- **Meta-review correction** — this doc's earlier revision added parallel Map fields; corrected per meta-review on PR #491 (2026-04-17)

---

## Acceptance (Lane 1 Stage 1b owns)

- [ ] `Map<K, V>` declared in `src/v3/std/map.dag` with lookup/insert/contains/keys/values primitives
- [ ] `Dag` type in `substrate.dag` is UNCHANGED (still `declarations: List<...>`, `nodes: List<Behavior>`, `ports: List<DagPort>`)
- [ ] `port(d, id)`, `node(d, id)`, `resolve_producer(d, id)` functions declared in `substrate.dag` as pure query functions over the authoritative lists
- [ ] Rust side: `dag.port_opt(id) -> Option<&DagPort>` (or equivalent) is what the `.dag` function lowers to; no new substrate fields added to Rust's `Dag` struct
- [ ] Three existing lenses migrated; oracle tests pass; line count reduced ≥15%
- [ ] `INVARIANTS.md` L-7 entry added with grep-gate command
- [ ] CI job runs the grep gate and fails on matches
- [ ] Zero local `find_port`/`find_behavior`/`resolve_producer`/`lookup_node`/`lookup_port` declarations in `src/v3/lenses/`

---

## Open questions

1. **`Map<K, V>` insertion order semantics?** `HashMap` iteration order is nondeterministic. If a lens emits something based on `Map` iteration, output is nondeterministic. Probably fine for lenses (they aggregate to a terminal value), but flag for Lane 3 Stage 3c (fixed-point ratchet) — if emission ever iterates a Map, need `BTreeMap` backing or sorted iteration. Defer decision until concrete case arises.

2. **Should `Map` support `remove`?** Not in initial API. Add when a consumer needs it.

3. **Should `resolve_producer` short-circuit on the first producer, or always recurse through nested Binds?** Current design: recurse always (walks through arbitrary Bind chains). If a lens needs "stop at the first Bind," add `producer_or_bind(d, port) -> Behavior?` as a separate primitive. Defer.
