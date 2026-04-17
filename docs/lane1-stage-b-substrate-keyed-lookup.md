> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) (Lane 1 Stage 1b)

# Lane 1 Stage 1b — Substrate keyed-lookup

**Lane:** 1 (Emission unification)
**Stage:** 1b (after L1.5 tail; before clean-emission invariant)
**Size:** M (unblocked by DB-14)
**Status:** Plan. No code changes yet.

> **Escalation update (2026-04-17).** The first overnight implementer attempt (session on `sharp-yak-750`) escalated — declaring `.dag` linear-walk bodies for the three accessors caused bootstrap pollution (every user DAG gained recursive Callable Transforms before the user's code). Attempt reverted; 1a shipped separately as PR #495. The mechanism for "declared fn with target-provided body" already exists in the substrate (`ArrowBody::ExternalRealization`) and is production-wired for the compiler's own pipeline stages (`src/v3/compiler/pipeline.dag`); the pattern just wasn't in the original 1b spec. [DB-14](./design-substrate-external-primitives.md) codifies the pattern. 1b now implements against DB-14.

---

## Motivation

From the meta-review (2026-04-17):

> The reflected substrate still exposes `Dag.ports` and `Dag.nodes` as linear lists, so migrated lenses keep inventing local lookup scaffolding… if `std.substrate` does not surface keyed lookup facts and the pass-through Bind edge structurally, future L2 migrations will keep copying the same reconstruction pattern. That is classic debt-shifting: the local instance gets treated, the producer stays weak.

Evidence: every lens in `src/v3/lenses/` reinvents its own `find_port` and `find_behavior`:

- `complexity.dag` — `lookup_cost(acc, port_id)` walks a linear list
- `provenance.dag` — inline port walks via `Dag.ports` iteration
- `unused_parameters.dag` — `expand_frontier_list` BFS with `contains(referenced, payload.head)`
- Pre-forward-fold `complexity.dag` had `find_port(ports, id) -> PortLookup` and `find_behavior(nodes, id) -> BehaviorLookup`

All three lenses compose `Behavior::Bind`'s pass-through edge manually: "if the producer is a Bind, walk to `bind.value`, retry." Every new lens writes this walk again.

**This is the substrate gap that Lane 2's compile-time-proofs work will hit repeatedly if not closed first.** Lane 2 adds idempotency, symbolic bounds, parallelism-as-lens — each is a new lens with the same substrate-consumer shape.

---

## Scope

Five deliverables:

### 1. Add keyed accessors to `substrate.dag`

New substrate fields (Rust-side HashMap materialized at Dag construction, exposed to .dag via typed accessors):

```
// in src/v3/std/substrate.dag — substrate shape UNCHANGED
type Dag {
  declarations: List<Declaration>
  nodes: List<Behavior>     // SINGLE authority for node identity
  ports: List<DagPort>      // SINGLE authority for port identity
  // Schema unchanged. See DB-5 for why no new fields.
}

// NEW: substrate-level query functions (not fields)
fn port(d: Dag, id: PortId) -> DagPort?
fn node(d: Dag, id: NodeId) -> Behavior?
```

The Rust side (`src/v3/compiler/src/dag.rs`) already has HashMap-backed O(1) lookup as implementation detail. That HashMap stays PRIVATE implementation — not exposed as a substrate field. The new `.dag`-side functions `port(d, id)` and `node(d, id)` lower to the existing Rust methods. Single authority preserved: the Lists ARE the substrate; the HashMap is a cache.

See [design-substrate-keyed-lookup-api.md](./design-substrate-keyed-lookup-api.md) for the full design rationale, including which earlier field-based proposals were rejected per meta-review.

### 2. Add Bind-pass-through primitive

```
// Returns the ultimate producer, walking through Bind-naming hops.
// Matches the handwritten oracle in every lens today.
fn resolve_producer(d: Dag, port_id: PortId) -> Behavior? =
  match port(d, port_id) {
    None => None
    Some(p) => match p.produced_by {
      None => None
      Some(node_id) => resolve_bind_chain(d, node_id)
    }
  }

fn resolve_bind_chain(d: Dag, node_id: NodeId) -> Behavior? =
  match node(d, node_id) {
    None => None
    Some(behavior) => match behavior {
      Bind(b) => resolve_producer(d, b.value)  // recurse through naming hop
      other => Some(other)                      // real producer reached
    }
  }
```

This is the canonical "follow the chain until you hit a non-Bind producer" walk. Every current lens writes this inline; consolidating once kills 3× reconstruction.

### 3. Migrate existing lenses to use the new accessors

- **complexity.dag**: replace any local `find_port(d.ports, id)` / `find_behavior(d.nodes, id)` with substrate-level `port(d, id)` / `node(d, id)`. The lens's own accumulator can keep using lists OR optionally switch to `Map<PortId, CostLookup>` for O(1) lookup (lens-local, not substrate)
- **provenance.dag**: replace inline `produced_by` chain walks with `resolve_producer(d, port_id)`
- **unused_parameters.dag**: same substitution — `find_port` / `find_behavior` → substrate-level functions

**Expected line reduction:** ~15% across the three lens files. Lower than the original 30% estimate because the primary change is a helper-deletion swap, not a full data-shape refactor.

### 4. Add invariant L-7 to INVARIANTS.md

```markdown
**L-7 — Lenses consume declared substrate accessors, not local reconstructions.**

A lens (`.dag` consumer of substrate facts) MUST NOT define its own
`find_port` / `find_behavior` / `resolve_producer` / similar lookup
helpers. These are declared once in `src/v3/std/substrate.dag` and
consumed by every lens.

**Why:** local reconstruction forks the substrate's truth. When the
substrate gains a new edge type (e.g., alias nodes, macro expansions),
every lens must learn it independently. Centralized accessors make
substrate extensions propagate automatically.

**Mechanical gate:** CI grep gate:
```
grep -nE "fn (find_port|find_behavior|resolve_producer|lookup_node|lookup_port)" src/v3/lenses/*.dag
```
must return zero matches. If a new accessor is needed, add it to
`substrate.dag`, not the lens.

**Origin:** meta-review 2026-04-17, identifying recurring
lens-reconstruction pattern as classic debt-shifting.
```

### 5. CI gate

Add the grep check above to `.github/workflows/ci.yml` as part of the v3 job. Zero matches = pass.

---

## Out-of-scope

- **Other substrate extensions** — declarations-by-id, span-by-node, etc. Real needs but not meta-review's finding. Separate work.
- **Rust-side `Dag::port(id)` change** — the Rust `Dag` already has keyed accessors. This stage exposes them to .dag, doesn't reshape Rust.
- **Performance optimization of existing walks** — the HashMap backing is already there; lens migration will naturally use it faster.
- **Full rewrite of any lens** — minimum changes to eliminate the reconstruction pattern only.

---

## Direction

**Substrate-first.** Declare the accessors in `substrate.dag` before touching any lens. That gives each lens migration a clear pattern to reference.

**One lens at a time.** Migrate complexity.dag first (smallest, just-touched). Then provenance.dag. Then unused_parameters.dag (most complex — full BFS over referenced set).

**Preserve test passes per lens.** After each lens migration, `cargo test -p v3-compiler --test m2_lens_<X>_migration_test` must pass (snapshot regen included).

**No re-derivation optimization.** If a lens's new shape is slightly less efficient than the linear walk (HashMap has constant factors), that's fine. Correctness over micro-optimization at this stage.

---

## Escalation criteria

Stop work and surface if:

1. **Substrate type emission missing a primitive** — e.g., the reflected map type isn't emitted to Rust HashMap cleanly. If that map-to-HashMap mapping has gaps, surface before hand-writing a workaround. The specific forbidden form (new map declaration in v3 std parallel to `dsl/std/types.dag`) is banked in `docs/post-l15-phase-plan.md § Banked dissolutions`; check DB-5 for the intended binding.

2. **A lens genuinely needs a lookup primitive not in substrate.dag** — beyond `port(d, id)`, `node(d, id)`, `resolve_producer(d, id)` (the three query functions locked in [DB-5](./design-substrate-keyed-lookup-api.md)). Don't invent locally; add to substrate with justification.

3. **Migration breaks oracle parity on any fixture** — the handwritten oracles in `m2_lens_*_migration_test.rs` must continue to match. If a migration changes behavior, that's a bug in the refactor, not the oracle.

4. **Line reduction is <10% across the three lenses** — the consolidation was supposed to delete reconstruction. If it doesn't shrink, the migration isn't actually using the new accessors, just calling them alongside the old walk.

---

## Acceptance gates

Stage complete when all five hold:

- [ ] `substrate.dag` declares the three query functions from [DB-5](./design-substrate-keyed-lookup-api.md) — `port(d, id)`, `node(d, id)`, `resolve_producer(d, id)` — as pure functions over the existing `List<DagPort>` / `List<Behavior>` authorities (no new parallel fields on `Dag`), with receipts (🟢 or 🟡)
- [ ] `complexity.dag`, `provenance.dag`, `unused_parameters.dag` all migrated; snapshot tests pass
- [ ] Line count in `src/v3/lenses/*.dag` reduced by ≥20% total (measure: commit with +X/-Y lines across all three files)
- [ ] INVARIANTS.md has L-7 entry referencing this design doc
- [ ] CI gate blocks new `find_port` / `find_behavior` / `resolve_producer` declarations in `src/v3/lenses/`

---

## Dependencies

- **Requires:** Lane 1 Stage 1a (L1.5 tail) complete — specifically, complexity.dag should be in its post-forward-fold state
- **Blocks:** Lane 1 Stage 1c (clean-emission invariant) — the clean-emission contract will reference substrate accessors as structural facts
- **Blocks:** Lane 2 entire — Lane 2 adds new lenses (idempotency, symbolic bounds, parallelism) that must consume keyed accessors, not reconstruct

---

## Estimate

- Substrate accessors + Bind-pass-through primitive: 1.5 days
- Lens migrations (complexity, provenance, unused_parameters): 2 days
- Snapshot regen + test verification: 0.5 day
- INVARIANTS.md + CI gate: 0.5 day
- Review + PR prep: 0.5 day

Total: ~5 implementer-days.

---

## Success signal

After this stage, a Lane 2 implementer writing the idempotency lens doesn't write a single `find_port` or `find_behavior`. They call `port(d, id)` and `node(d, id)` and get back `Option<T>`. The substrate is authoritative; the lens consumes.

When the substrate gains a new Behavior variant in the future (say, `Macro` or `Alias`), every lens handles it automatically via `resolve_producer`. No sweep across lenses needed. That's what "the substrate is the single authority" means in practice.
