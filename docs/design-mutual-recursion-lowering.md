> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 3 Stage 3a

# Design DB-9 — Mutual recursion → Loop lowering

**Design blocker:** DB-9
**Consumers:** Lane 3 Stage 3a (M2 feature parity)
**Status:** Design ready for implementer review.

---

## Problem

Today v3 supports single recursion: `fn f(n) = ...f(n-1)...`. The substrate represents this as a `Loop` node with structural descent evidence (n → n-1).

`compiler.dag` (and many real programs) needs **mutual recursion**: `fn a(n) = ...b(n-1)...; fn b(n) = ...a(n-1)...`. Functions call each other with the call graph forming a cycle. SELF_HOSTING.md §2.4 identifies this as a blocker.

The substrate has no current way to represent a mutually-recursive cluster. Each function's body is a separate Bind; the call edge from `a → b → a` crosses Bind boundaries. The termination checker needs to treat the whole cluster (the **SCC** — strongly connected component) as one structural-descent target.

This design specifies:
- How to detect mutually-recursive clusters (SCC computation)
- What substrate representation the cluster lowers to
- How structural descent evidence is carried
- How emission handles mutually-recursive functions in Rust/Go/Python

---

## Design

### SCC detection pass

New pass in the lowering pipeline, after individual function bodies are lowered:

```rust
// src/v3/compiler/src/mutual_recursion.rs (new)
pub struct RecursionCluster {
    pub bind_ids: Vec<NodeId>,           // the mutually-recursive Binds
    pub descent_witness: DescentEvidence,  // structural argument shared across the cluster
}

pub fn detect_clusters(dag: &Dag) -> Vec<RecursionCluster> {
    // 1. Build call graph: edges from Bind A to Bind B iff A's body contains a
    //    Transform whose target resolves to B.
    // 2. Compute SCCs via Tarjan's algorithm (standard, O(V+E)).
    // 3. Each non-trivial SCC (size ≥ 2, OR size 1 with self-edge) is a RecursionCluster.
    // 4. For each cluster, find the SHARED descent witness: a parameter position that
    //    decreases structurally across all calls within the cluster.
    ...
}
```

The `descent_witness` is the crux. For a cluster `{a, b}` where `a` calls `b(n-1)` and `b` calls `a(n-1)`, both recursive calls decrease `n` by the same structural step. The termination checker must verify this.

### Substrate representation

Add a new `Behavior` variant:

```dag
// src/v3/std/substrate.dag
type Behavior
  = Value(ValueNode)
  | Transform(TransformNode)
  | Branch(BranchNode)
  | Loop(LoopNode)
  | Bind(BindNode)
  | MutualLoop(MutualLoopNode)  // NEW
```

```dag
type MutualLoopNode {
  id: NodeId
  bodies: List<NodeId>          // the Binds in the cluster
  descent_witness: DescentEvidence
  entry: NodeId                  // the Bind that's called from outside the cluster
  result_port: PortId
  span: SourceSpan
}
```

`entry` identifies the Bind that's the "public face" of the cluster — the one external callers reach. All other Binds in the cluster are only reachable via intra-cluster calls.

### Lowering from surface

Parser/lowering pipeline:

1. Parse user code as before — `fn a` and `fn b` each produce a `Bind` node
2. Type-check and infer as before
3. After inference: run `detect_clusters(dag)` — identifies `{a, b}` is mutually recursive
4. Wrap the cluster: replace the standalone Binds with a single `MutualLoop` node carrying both as `bodies`
5. External callers now reference the `MutualLoop`'s entry port, not the individual Binds
6. Termination check: verify `descent_witness` holds across all intra-cluster call edges; fail-closed if not

### Structural descent for clusters

Current single-recursion termination: `a(n)` calling `a(n-1)` is allowed iff `n-1 < n` structurally (for integers: arithmetic, for lists: tail).

For a cluster `{a, b}`: every call edge `a → b` or `b → a` must pass a structurally-smaller argument for the SAME position. Cluster terminates iff `∀edge. arg_at_edge < arg_caller`.

```dag
// src/v3/lenses/termination.dag (extended)
fn verify_cluster_descent(cluster: MutualLoopNode) -> TerminationWitness {
  let descent_param_idx = cluster.descent_witness.parameter_position
  let edges = collect_intra_cluster_call_edges(cluster)
  for edge in edges {
    let caller_arg = edge.caller_params[descent_param_idx]
    let callee_arg = edge.call_args[descent_param_idx]
    if !is_structurally_smaller(callee_arg, caller_arg) {
      return TerminationWitness::FailedAt { edge, reason: "non-decreasing on shared witness" }
    }
  }
  TerminationWitness::Proved { cluster_id: cluster.id }
}
```

### Emission

**Rust** — mutually-recursive functions are already first-class; emission generates N function declarations, one per cluster member. The walker (DB-2) descends into each Bind normally; the MutualLoop node is a grouping wrapper, not a separate code structure.

```rust
// Rust emission of a MutualLoop{bodies: [a, b]}:
fn a(n: i64) -> i64 { if n == 0 { 0 } else { b(n - 1) } }
fn b(n: i64) -> i64 { if n == 0 { 1 } else { a(n - 1) } }
```

**Go** — same shape; Go supports mutual recursion natively.

**Python** — same; Python's late binding makes mutual recursion trivial.

**SPICE / Verilog** — N/A. These targets don't express recursion at all; a MutualLoop in the source means the program isn't emittable to those targets. Emit error: "SPICE doesn't support recursion." This is correct fail-closed behavior.

### Differentiation from single-recursion Loop

| Feature | `LoopNode` (single-recursion) | `MutualLoopNode` (mutual) |
|---|---|---|
| Bodies | 1 | N ≥ 2 (or N = 1 with self-edge and other structural reason for separation) |
| Entry | implicit (the Loop itself) | explicit `entry: NodeId` |
| Descent witness | on the single body | shared across all bodies |
| Emission | single `fn` | N `fn`s |
| Termination check | single call edge | all intra-cluster edges |

Single self-recursion (`fn f(n) = ...f(n-1)...`) continues to lower to `LoopNode`, not `MutualLoopNode`. The SCC detection correctly identifies `{f}` with a self-edge but the single-recursion case is a strict subset — if the compiler can use the simpler `LoopNode` representation, prefer it. `MutualLoopNode` is only for N ≥ 2 OR N = 1 with structural complexity the single-body Loop can't express.

### Receipts

```dag
// 🟡 SCAFFOLD until the compiler's dependency resolver can natively
// represent recursive callable types. Today, `MutualLoopNode.bodies`
// is a flat list — resolution order within the cluster is
// (arbitrary but deterministic). Once type-level recursive callable
// support lands, this may graduate to 🟢.
```

---

## Rationale

**Why a new Behavior variant, not just "multiple Binds"?** Because the substrate must represent the cluster as a unit for termination checking. Without `MutualLoopNode`, the termination lens would have to recompute the SCC on every check — re-derivation of a structural fact.

**Why `descent_witness: DescentEvidence` shared across the cluster?** Because that's the invariant: all intra-cluster calls decrease the SAME parameter. If each function has its own independent witness, the termination check degenerates to per-edge instead of cluster-wide — and cycles can still occur (a decreases one param; b decreases a different one; the composition doesn't decrease anything consistently).

**Why `entry: NodeId` explicit?** Because external callers need a single referenceable entry point. If user code calls `a(5)` externally, the call edge from outside the cluster targets the cluster's `a` Bind, not `b`. Making the entry explicit means external callers don't need to know the cluster's internal structure.

**Why Tarjan's algorithm for SCC?** Standard, linear-time (O(V+E)), well-understood. Kosaraju's alternative is equally valid; Tarjan chosen for single-pass simplicity.

**Why fail fast on cluster descent violation?** Because a non-decreasing cluster is provably non-terminating. The termination lens's job is compile-time rejection; rejecting the cluster at lowering means downstream passes don't see an invalid node.

**Why emission unchanged for Rust/Go/Python?** Because those languages have mutual recursion as a primitive. The MutualLoop is an INTERMEDIATE representation that helps the compiler; the target language doesn't need anything special. SPICE/Verilog fail-close.

---

## Rejected alternatives

**Represent mutual recursion via annotated Bind flags (`Bind { mutually_recursive_with: List<NodeId> }`)** — spreads the cluster information across N nodes instead of centralizing. Every consumer reconstructs the cluster. Rejected.

**Track the call graph as a separate `Dag.call_graph: Graph<NodeId>` field** — duplicates information already computable from node inspection. Rejected; compute on demand via `detect_clusters`.

**Allow mutually-recursive clusters without termination proof** — punts non-termination to runtime. Violates the "properties are inescapable" thesis. Rejected.

**Auto-inline the cluster into a single Bind with a sum-type dispatch** — loses the function identity; makes emission awkward (every intra-cluster call becomes a dispatch match). Heavier than needed when the target languages already support mutual recursion. Rejected.

**Require users to annotate `mutual` explicitly** — users shouldn't have to declare something the compiler can detect. Rejected; SCC detection is fully automatic.

---

## Implementation notes

### Detection performance

For a compiler-scale program (~100 functions), Tarjan's O(V+E) is negligible. No performance concerns.

### Interaction with inference

Inference on a mutually-recursive cluster needs to handle "f's type depends on g's type depends on f's type." Standard fix: bottom-up inference on SCCs, using fixed-point iteration within each SCC. Already solved pattern in Haskell/OCaml/Rust compilers; v3's inference pass needs to recognize clusters.

This is its own sub-task. Deferred detail; the important structural fact is that **inference pass must run SCC detection BEFORE type-checking function bodies**, so each SCC can be type-checked as a unit.

### Error reporting on cluster termination failure

```
ERROR at <cluster span>: mutually-recursive cluster does not terminate

  fn a(n: Int) -> Int = if n == 0 then 0 else b(n)
                                                  ^ note: `n` passed unchanged to `b`

  fn b(n: Int) -> Int = if n == 0 then 1 else a(n - 1)

Cluster: {a, b}
Descent failure: call edge `a → b` passes `n` without decreasing it.

FIX: decrease `n` on every intra-cluster call, e.g., `b(n - 1)`.
```

Uses DB-1 Correction shape.

### Substrate reflection

`substrate.dag` already reflects the `Behavior` enum into Rust. Adding `MutualLoop(MutualLoopNode)` variant follows the existing pattern — one-for-one mapping between `.dag` enum variants and Rust enum variants.

---

## Associations

- **Lane 3 Stage 3a** ([lane3-self-hosting-cycle.md](./lane3-self-hosting-cycle.md)) — this is that stage's mutual recursion deliverable
- **DB-1 `Correction` shape** ([design-correction-shape.md](./design-correction-shape.md)) — cluster termination failure emits diagnostics with fixes
- **DB-7 Symbolic cost** ([design-symbolic-cost-algebra.md](./design-symbolic-cost-algebra.md)) — symbolic cost on a MutualLoop is `iterate(cluster_depth_bound, body_cost)`, using the shared descent witness to derive the depth bound
- **Update `src/v3/std/substrate.dag`** — add `MutualLoop(MutualLoopNode)` variant + `MutualLoopNode` type
- **Create `src/v3/compiler/src/mutual_recursion.rs`** — SCC detection pass
- **Update `src/v3/lenses/termination.dag`** — verify_cluster_descent function
- **Update `src/v3/compiler/src/emit.rs`** (DB-2 walker) — MutualLoop descent into individual bodies, no special emission logic
- **Thesis anchor** — SELF_HOSTING.md §2.4

---

## Acceptance (Lane 3 Stage 3a owns)

- [ ] `src/v3/std/substrate.dag` declares `MutualLoopNode` + adds `MutualLoop(MutualLoopNode)` variant to `Behavior`
- [ ] SCC detection pass correctly identifies `{a, b}` as a cluster for the example above
- [ ] Single-recursion functions still lower to `LoopNode`, not `MutualLoopNode` (strict subset relation)
- [ ] Termination lens verifies cluster descent; fails fixtures with non-decreasing calls
- [ ] Emission in Rust / Go / Python produces N separate `fn` declarations per cluster
- [ ] SPICE / Verilog / English emission fails with "recursion not supported" when a MutualLoop appears
- [ ] Test: `fn a(n) = if n == 0 then 0 else b(n - 1); fn b(n) = if n == 0 then 1 else a(n - 1)` compiles and runs (mock or via `dag run`)
- [ ] Test: `fn a(n) = b(n); fn b(n) = a(n)` fails compile with cluster termination diagnostic
- [ ] Test: `compiler.dag` (at least its parser / lowerer / emitter sub-modules that are mutually recursive) compiles

---

## Open questions

1. **What if the shared descent witness doesn't exist at any single parameter position?** E.g., `a(n) → b(n - 1)` and `b(n) → a(n + 1)` individually decrease THEIR callee's arg but the cluster doesn't terminate overall. Detection: compute a global ranking function over all parameters. Deferred — initial implementation requires single-position shared witness; extend when a concrete fixture demands it.

2. **Can a cluster have > 2 members?** Yes — Tarjan's finds SCCs of arbitrary size. Descent witness still works if N bodies all decrease the same parameter position.

3. **What about nested clusters?** SCCs are maximal by definition; a strongly-connected region is a single SCC. Nesting isn't possible in the usual sense. If a cluster's body internally calls another cluster, that's captured as a call edge between SCCs (no intra-cluster coupling).

4. **How does mutual recursion interact with generic functions?** E.g., `fn a<T>(x: T) → b<T>(x)` where `b<T>` calls `a<T>`. Each instantiation creates its own cluster? Or one cluster over the generic signatures? Likely: instantiate-then-detect. Defer until surface generics stabilize (Lane 3 Stage 3a sub-task).
