# R3 Bug-Fix Worker Brief — CallGraph forward-only authority

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068) lane scope; worker dispatch via Substrate Mgr standing authority.
**Authority parent**: gpt-5-5-pro reflective analysis on `main@cf1d523` Finding 3; PM dispatch at gunbc#846 #issuecomment-4413207527.
**Priority**: MEDIUM — illegal-state-representable; clean refactor.

---

## §0. Problem statement

`CallGraph` at `dsl/std/graph.dag:15-26` stores both forward AND reverse adjacency as parallel authorities:

```
type CallGraph {
  forward: Map<String, List<String>>
  reverse: Map<String, List<String>>
}
```

`build_call_graph_from_proof_edges` (`dsl/std/graph.dag:57-75`) derives both from the same `ProofEdge` list in one pass. `graph_has_multi_node_scc` (`dsl/std/graph.dag:121-136`) reads `graph.forward` in pass 1 and `graph.reverse` in pass 2.

The builder keeps them aligned; **the type itself admits incoherent values**. A user or later `.dag` stage can construct `CallGraph { forward: A, reverse: B }` where `B` is not the reverse of `A`. INVARIANTS P2 single-authority violation inside the data model.

## §1. Required outcome

Store edges as single authority; reverse derived as needed.

## §2. Fix options

PM recommends **Option A** (edges-as-authority) — cleanest dissolution; `forward` and `reverse` become derived views.

### Option A — store edges only (preferred)

```
type CallGraph {
  edges: List<ProofEdge>
}

fn forward_adjacency(graph: CallGraph) -> Map<String, List<String>> = ...
fn reverse_adjacency(graph: CallGraph) -> Map<String, List<String>> = ...
```

Consumers call `forward_adjacency(graph)` / `reverse_adjacency(graph)` instead of accessing field. Builder constructs `CallGraph { edges }`; reverse computation moves to derivation function.

**Performance note**: if `graph_has_multi_node_scc` (or other consumers) need both adjacencies in tight loops, the derivation can return both at once via `fn build_adjacency_views(graph: CallGraph) -> { forward, reverse }`, OR consumers cache locally. Not a substrate-shape concern.

### Option B — forward-only with derived reverse

```
type CallGraph {
  forward: Map<String, List<String>>
}

fn reverse_adjacency(graph: CallGraph) -> Map<String, List<String>> = ...
```

Smaller diff but less clean — `forward` is still potentially constructible inconsistently with the original edges (e.g., direct map construction omitting an edge). Option A pushes the authority all the way back to the edge list.

## §3. Files (expected scope)

**Option A**:
- `dsl/std/graph.dag` (CallGraph shape + build_call_graph_from_proof_edges + graph_has_multi_node_scc + new derivation functions)
- Any callers of `CallGraph.forward` / `CallGraph.reverse` field accesses (grep audit)
- Cementing test pinning that `forward_adjacency(build_call_graph_from_proof_edges(edges))` recovers expected adjacency
- If parallel v3 mirror exists (`src/v3/std/graph.dag`?), reconcile

**Option B**: similar but smaller — keep forward field; only delete reverse.

## §4. Cross-cutting constraints

- **No new hand-Rust tests** — `.dag` TestClaim form preferred per locked design `docs/design-tests-as-data-completeness.md` §C5 / §C1.
- **STOP-and-PING via PM inbox (#846)** if grep reveals consumers that depend on `CallGraph.reverse` for performance-critical paths and derivation introduces unacceptable cost. Likely not — but if so, the fix is to provide both views via single derivation function, not to keep the dual-authority field.
- **Substrate authority canonical**: `dsl/std/graph.dag` is canonical; if v3 mirror exists, reconcile.

## §5. Receipt

When work lands:
- `CallGraph` cannot represent inconsistent forward/reverse state (illegal state unrepresentable structurally)
- Cementing test pinning that derived adjacencies are coherent with edge list
- INVARIANTS P2 strengthened: dual-authority inside data model dissolved
- Substrate + Rust mirror synchronized (if applicable)
- SG-0 census: any new test entries marked with dissolution-trigger comment

## §6. Dispatch trigger

PM-authored brief; awaiting worker dispatch. Illegal-state representability remains live on main until fixed.

## §7. Risk note

Smaller scope than the other 3 bug-fix tasks (Tasks 1-3). Substrate-shape change but contained — `CallGraph` is internal substrate, not user-facing. Should be a single small PR.

---

**End of brief.**
