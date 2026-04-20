### L-7: Lenses consume declared substrate query functions (2026-04-17)

A lens (`.dag` consumer of substrate facts) MUST NOT define its own
`find_port` / `find_behavior` / `resolve_producer` / `lookup_node` /
`lookup_port` / similar lookup helpers over `Dag.ports` or `Dag.nodes`.
These are declared once in `src/v3/std/substrate.dag` (`port(d, id)`,
`node(d, id)`, `resolve_producer(d, id)`) and consumed by every lens
via the DB-14 external-primitive pattern.

**Why:** local reconstruction forks the substrate's truth. When the
substrate gains a new edge type (alias nodes, macro expansions), every
lens must learn it independently. Centralized substrate functions make
substrate extensions propagate automatically.

**Why functions, not fields:** the substrate has a single authority
for port/node identity — the existing `List<DagPort>` / `List<Behavior>`
fields. Adding parallel `Map<PortId, DagPort>` fields would be duplicate
authority. The query functions abstract over implementation (Rust
caches in HashMap; the lens doesn't see it). See DB-5 for the API
design and DB-14 for the declare-without-eager-lowering mechanism.

**Mechanical gate:** CI grep check blocks new local accessor
declarations in any lens file:

```
grep -nE "fn (find_port|find_behavior|resolve_producer|lookup_node|lookup_port)" src/v3/lenses/*.dag
```

Zero matches required. If a new accessor is genuinely needed, add it
to `substrate.dag` using the DB-14 pattern (stub body +
`SubstrateAccessorBinding` per target), not as a private lens helper.

**Origin:** meta-review 2026-04-17 — "every lens reinvents local
lookup scaffolding." Lane 1 Stage 1b landed the three canonical
accessors and migrated provenance.dag + unused_parameters.dag to
consume them.

