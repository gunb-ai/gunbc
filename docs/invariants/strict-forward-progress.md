## Strict Forward Progress

Time flows strictly forward. Every execution step moves the
computation forward through a bounded structure — never revisiting,
never cycling.

This is the foundational invariant from which decidability,
complexity analysis, and termination all follow. A recursive
function is a logical description of a computation. Its execution
is a finite walk forward through time. The recursion is syntax;
the forward progress is physics.

**Cyclic relations are expressible; direct cyclic values are not.**
Cyclic domains (graphs with back-edges, circular dependencies,
mutual references) are representable via acyclic encodings — adjacency
maps keyed by stable IDs (`Map<NodeId, List<NodeId>>`), parent
pointers as ID references, etc. The cyclicity lives in the interpreted
relation, not in the stored value graph.

Direct cyclic object graphs (heap topology with back-edge pointers)
are not expressible as core values. Values are immutable, finitely
constructed, and acyclic in their physical structure.

**Computations over cyclic relations must still be bounded.**
An acyclic encoding does NOT automatically make every graph algorithm
acceptable. A traversal that follows adjacency links without a visited
set or other decreasing measure is still logically unbounded. The safe
rule: every computation over a cyclic relation must be justified by an
explicit finite measure — frontier size, unvisited-node count, or a
fold/repeat bound over |V| or |E|.

```
// CORRECT: fold over map_keys (finite container), not edge-following
fn reachable(g: Map<String, List<String>>) -> Map<String, Bool> {
  // bounded by |map_keys(g)| — the container, not the topology
  repeat(bound: map_keys(g) |> count, ...)
}

// REJECTED: recursive traversal without explicit finite measure
fn walk(g: Map<String, List<String>>, node: String) -> List<String> {
  let neighbors = map_get(g, node)
  neighbors |> flat_map(n => walk(g, n))  // unbounded — no visited set
}
```

**The three-part distinction:**
- Cyclic relations: **yes** (data can encode arbitrary graph topologies)
- Direct cyclic values: **no** (values are acyclic in physical structure)
- Traversals over cyclic relations: **yes, if bounded by explicit
  finite structure** (|V|, |E|, frontier size, visited count)

**Why this matters:** if execution can only move forward, then:
- Every program terminates (decidability)
- Every program has a computable cost (complexity analysis is total)
- Every program has a computable memory footprint (space analysis)
- Composition is closed (forward + forward = forward)
- The compiler itself provably terminates on all inputs

