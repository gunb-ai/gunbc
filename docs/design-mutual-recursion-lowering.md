> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 3 Stage 3a.1

# Design DB-9 — Mutual recursion (lens-level, no substrate extension)

**Design blocker:** DB-9
**Consumers:** Lane 3 Stage 3a.1 (M2 feature parity — mutual recursion)
**Status:** Design ready for implementer review.

---

## Critical correction from prior revision

An earlier version of this doc proposed a new `Behavior` variant `MutualLoop` to represent mutually-recursive clusters. That proposal is **rejected** because it violates THESIS.md:604-629:

> *"Computation is five L1 behaviors... No sixth behavior. M0 validated these five under three reviewer rounds and never hit a stop signal... the five-behavior claim is the project's strongest empirical substrate commitment and is not revisited here."*

Reviewer (PR #491, 2026-04-17) correctly flagged this as a thesis violation. Adding a sixth Behavior is not a stage-level design choice; it would require reopening the substrate commitment in THESIS.md / ROADMAP.md through an explicit process.

**The corrected design below encodes mutual recursion entirely within the existing five behaviors.** The substrate does not change. Clustering becomes a lens-level fact, computed from the call graph at lowering time and consumed by the termination lens.

---

## Problem

`compiler.dag` (and real programs) needs mutual recursion: `fn a(n) = ...b(n-1)...; fn b(n) = ...a(n-1)...`. Functions call each other; the call graph forms a cycle.

Today's substrate handles single recursion (`fn f(n) = f(n-1)`) via `Loop` with structural descent on the argument. Mutual recursion needs:
1. Both functions to compile (no new substrate extension)
2. The termination checker to verify the cluster terminates (not just each function individually)
3. Emission in Rust / Go / Python to produce two separate `fn` declarations calling each other

Requirement (1) is satisfied by the existing substrate. Requirement (2) is the interesting part: per-Bind termination checks pass vacuously on mutual recursion because no single function recurses into itself. Requirement (3) is already handled by target languages with first-class mutual recursion.

---

## Design

### Substrate: unchanged

The two Binds remain two Binds. Their bodies reference each other via ordinary `Transform` + `FunctionRef` edges, which is how any two functions call each other in the substrate.

```
Bind(a) {
  body: ... Transform { target: Callable(b_decl_id), inputs: [...] } ...
}
Bind(b) {
  body: ... Transform { target: Callable(a_decl_id), inputs: [...] } ...
}
```

This is VALID substrate today. What fails today is that the termination lens doesn't verify cluster descent.

### Lens extension: cluster-aware termination

The termination lens gets extended:

1. **Detect clusters.** Build the call graph: edges from Bind A to Bind B iff A's body contains a `Transform` whose target resolves to B. Compute SCCs via Tarjan's algorithm (standard, O(V+E)).

2. **Per-cluster termination check.** For each non-trivial SCC:
   - Identify the shared descent witness: a parameter position that decreases structurally across all intra-cluster call edges
   - Verify every intra-cluster call passes a structurally-smaller value at that position
   - If no shared witness exists, emit diagnostic naming which edge fails

3. **Cluster membership lives IN the lens, not the substrate.** `ClusterRegistry` is a lens-computed fact keyed by cluster-ID, carrying member Bind NodeIds + shared descent witness. Other lenses (symbolic cost, parallelism) can consume it.

```dag
// src/v3/lenses/termination.dag — extended
fn verify_termination(d: Dag) -> TerminationReport {
  let clusters = detect_sccs(call_graph(d))
  let violations = fold(clusters, empty(), |acc, cluster|
    match verify_cluster_descent(d, cluster) {
      Terminates => acc
      FailsAt(edge, reason) => cons(diagnostic_for(edge, reason), acc)
    }
  )
  TerminationReport { clusters, violations }
}

// Non-thesis-violating: ClusterRegistry is a LENS fact, not substrate
type ClusterRegistry {
  clusters: List<RecursionCluster>
}

type RecursionCluster {
  bind_ids: List<NodeId>
  descent_witness: DescentPosition
  entry: NodeId?
}
```

### Emission: unchanged

Rust, Go, Python all support mutual recursion natively. The emitter generates N separate `fn` declarations, one per Bind in the cluster. Each `fn` has its own body that calls the other Binds via ordinary callable emission. No cluster-aware emission logic needed at the target language level.

```rust
// Rust emission of Bind(a) + Bind(b):
fn a(n: i64) -> i64 { if n == 0 { 0 } else { b(n - 1) } }
fn b(n: i64) -> i64 { if n == 0 { 1 } else { a(n - 1) } }
```

Clean. No dispatch. No sum-type wrapping. Target-native.

### SPICE / Verilog / English (Shape B — NOT compiler targets)

These are Shape B artifacts produced by `.dag` programs (see THESIS.md §"Two shapes"). A `.dag` program that produces a SPICE netlist is ordinary user code; if that user code wants to support mutual recursion in its domain model, that's a `.dag`-library concern, not a compiler concern.

---

## Why per-Bind termination fails on mutual recursion today (motivating failure)

Current per-function check:
- `a(n)` calls `b(n-1)` — no call to `a` itself visible → per-function check passes vacuously
- `b(n)` calls `a(n-1)` — no call to `b` itself visible → per-function check passes vacuously
- Cluster `{a, b}` has a non-terminating candidate like `fn a(n) = b(n); fn b(n) = a(n)` that today **compiles** despite being provably non-terminating

Cluster-aware check at the lens level is the fix. The substrate doesn't change.

---

## How does this satisfy ROADMAP.md §M2 "mutual recursion → Loop"?

The roadmap line reads: *"Remaining: ... mutual recursion → Loop (§2.4)"*. Interpreted literally as "add a new Behavior," this would violate the thesis.

Re-reading with the thesis-compliant lens: **the Loop behavior already handles bounded iteration**. The "mutual recursion → Loop" phrasing refers to the **lens's termination handling**, not a substrate change. Mutual recursion is proven terminating using the same structural-descent machinery `Loop` uses (cardinality-bounded iteration). The cluster-descent check reuses that machinery across a multi-function cluster.

If SELF_HOSTING.md §2.4 meant something different (a substrate extension), this design opens that discussion explicitly — but the thesis has stronger precedence, and the thesis says "no sixth behavior." Deferring to thesis. If SELF_HOSTING.md §2.4 conflicts, it's the SELF_HOSTING doc that needs reconciliation, not the thesis.

---

## Rejected alternatives

**Add `MutualLoop` as a 6th Behavior variant** (original DB-9 proposal) — **thesis violation per THESIS.md:604-629.** The five-behavior commitment is explicit and not a stage-level design choice. Rejected.

**Encode cluster as single Loop with sum-type dispatch body** — preserves substrate shape but destroys per-function identity at the surface Bind level. N Binds become 1 Bind at the substrate level, and emission must reconstruct the N `fn` names from cluster metadata. Possible but unnecessarily complicated — the cleaner design keeps N Binds as separate substrate nodes and pushes clustering up to the lens. Rejected for this design; kept as a fallback encoding if the lens-level approach proves insufficient.

**Annotate each Bind with cluster_id** — parallel authority. The call graph IS the authority for cluster membership; an annotation duplicates it and can drift. Rejected.

**Require users to declare `mutual` explicitly** — compiler can detect clusters automatically via SCC. User annotation would be ceremony. Rejected.

**Defer mutual recursion entirely** — `compiler.dag` needs it. Can't defer. Rejected.

---

## Implementation notes

### Lens walk

```dag
fn call_graph(d: Dag) -> List<CallEdge> =
  // For each Bind b in d.nodes, scan b.body for Transforms whose
  // target resolves to another Bind. Emit (caller=b, callee=target).
  ...

fn detect_sccs(edges: List<CallEdge>) -> List<SCC> =
  // Tarjan's algorithm. Standard.
  ...

fn verify_cluster_descent(d: Dag, cluster: SCC) -> ClusterTerminationWitness =
  if length(cluster.members) == 1 && !has_self_edge(cluster) then
    Terminates
  else
    match find_shared_descent(d, cluster) {
      Some(position) => verify_all_calls_decrease(d, cluster, position)
      None => FailsAt { edge: first_intra_cluster_edge(d, cluster), reason: "no shared descent witness" }
    }
```

### Interaction with inference

Mutual recursion complicates type inference (`f`'s type depends on `g`'s type depends on `f`'s type). Standard fix: bottom-up inference on SCCs with fixed-point iteration. v3's inference pass needs to recognize clusters at type-check time.

Sub-task of Lane 3 Stage 3a.1. NOT a substrate change — an inference-pass algorithm change.

### Error reporting

```
ERROR at <cluster span>: mutually-recursive cluster does not terminate

  fn a(n: Int) -> Int = if n == 0 then 0 else b(n)
                                                  ^ note: `n` passed unchanged to `b`

  fn b(n: Int) -> Int = if n == 0 then 1 else a(n - 1)

Cluster: {a, b}
Descent failure: call edge `a → b` passes `n` without decreasing it.

FIX: decrease `n` on every intra-cluster call, e.g., `b(n - 1)`.
```

Uses DB-1 Correction shape (source-level).

---

## Associations

- **Lane 3 Stage 3a.1** ([lane3-self-hosting-cycle.md](./lane3-self-hosting-cycle.md)) — this is that sub-stage's design
- **DB-1 Correction shape** ([design-correction-shape.md](./design-correction-shape.md)) — cluster termination diagnostics emit Corrections
- **DB-7 Symbolic cost** ([design-symbolic-cost-algebra.md](./design-symbolic-cost-algebra.md)) — symbolic cost for a cluster reads the lens's cluster registry to compute recursion depth bound
- **`src/v3/std/substrate.dag`** — **UNCHANGED.** No new Behavior variant. Substrate commitment per THESIS.md:604-629 preserved.
- **Update termination lens** (`src/v3/lenses/termination.dag` or equivalent) — add SCC detection + cluster descent verification
- **Update inference pass** — bottom-up SCC-aware type inference
- **Thesis anchor** — THESIS.md:602-629 (five behaviors, no sixth)

---

## Acceptance (Lane 3 Stage 3a.1 owns)

- [ ] Substrate unchanged: `type Behavior` in `src/v3/std/substrate.dag` still has exactly 5 variants (`Value | Transform | Branch | Loop | Bind`)
- [ ] Termination lens performs SCC detection + cluster descent verification
- [ ] Test: `fn a(n: Int) = if n == 0 then 0 else b(n - 1); fn b(n: Int) = if n == 0 then 1 else a(n - 1)` compiles and passes termination
- [ ] Test: `fn a(n: Int) = b(n); fn b(n: Int) = a(n)` fails compile with cluster termination diagnostic naming the non-decreasing edge
- [ ] Emission in Rust / Go / Python produces N separate `fn` declarations
- [ ] SELF_HOSTING.md §2.4 reconciled with this design (either: §2.4 already meant lens-level, or §2.4 is updated to match this)

---

## Open questions

1. **What if SELF_HOSTING.md §2.4 genuinely specifies a substrate extension?** Then SELF_HOSTING.md is older than THESIS.md:604-629's commitment and needs a forward-update. The thesis has higher authority; any conflict resolves in thesis's favor.

2. **Does cluster inference make type-checking slower?** SCC + fixed-point inference IS more expensive than per-Bind inference, but only on mutually-recursive clusters (rare). Implementation can short-circuit non-recursive clusters (common case stays fast).

3. **Cross-module clusters** — if `a` is in module X and `b` in module Y, does SCC detection cross module boundaries? Design says yes, but surfaces compilation ordering. Deferred — `compiler.dag`'s cluster likely lives within a single module; cross-module handled later.
