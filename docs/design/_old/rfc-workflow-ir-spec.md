# RFC: Workflow IR Specification

> **Status**: Draft  
> **Goal**: Push all structural/type/wiring guarantees to compile time. Make invalid workflows unrepresentable.

---

## 1. Motivation

Traditional workflow systems validate at runtime:
```
define workflow → execute → runtime error: "port not found"
```

gunbc's goal is the workflow equivalent of a sound type system:
```
define workflow → compile (reject invalid) → execute (only dynamic failures possible)
```

### Invariant Ladder

| Level | What's Proven | Failures Possible |
|-------|---------------|-------------------|
| **L1: Static IR** | Graph structure, types, cardinality, SubDag interface | None (rejected at definition) |
| **L2: Lowering** | Semantics-preserving flattening | None (preserves L1 guarantees) |
| **L3: Codegen** | Valid execution artifact | None (derived from proved IR) |
| **L4: Execution** | Interpreter for proved program | Only inherently-dynamic (network, timeout, resource contention) |

**Key insight**: By L4, the only possible failures are *external* — the workflow structure itself is proven correct.

---

## 2. Workflow IR Formal Specification

### 2.1 Core Types

```
Port       := { name: PortName, type: TypeId, cardinality: Card }
Card       := Zero | One | ZeroOrOne | ZeroOrMore | OneOrMore
Edge       := { from: (NodeId, PortName), to: (NodeId, PortName) }
Node<E,R>  := { id: NodeId, inputs: [Port], outputs: [Port], body: Body<E,R> }
Body<E,R>  := Opaque<E,R> | SubDag(Dag<E,R>)
Dag<E,R>   := { nodes: [Node<E,R>], edges: [Edge] }
```

Where:
- `E` is the effect type (see §3)
- `R` is the resource requirement type (see §4)

### 2.2 Cardinality Algebra

Cardinalities form a lattice under the "satisfies" relation:

```
                ZeroOrMore (⊤)
                /         \
           ZeroOrOne    OneOrMore
                \         /
                   One
                    |
                  Zero (⊥)
```

**Satisfies relation** (`A ⊑ B`): Output cardinality `A` satisfies input requirement `B` iff all possible outputs of `A` are acceptable inputs to `B`.

```
satisfies : Card → Card → Bool
satisfies A B = ∀v. possible(A, v) → accepts(B, v)

-- Derived rules:
One       ⊑ One, ZeroOrOne, ZeroOrMore, OneOrMore
ZeroOrOne ⊑ ZeroOrOne, ZeroOrMore
OneOrMore ⊑ ZeroOrMore, OneOrMore
ZeroOrMore ⊑ ZeroOrMore
Zero      ⊑ Zero, ZeroOrOne, ZeroOrMore
```

### 2.3 Well-Formedness Judgment

A DAG is well-formed (`⊢ D : wf`) iff:

```
─────────────────────────────────────────────────────────────
                    WELL-FORMED DAG
─────────────────────────────────────────────────────────────

(WF-NODES)    ∀n ∈ D.nodes. unique(n.id) ∧ wf_node(n)
(WF-EDGES)    ∀e ∈ D.edges. wf_edge(D, e)
(WF-ACYCLIC)  acyclic(D.edges)
(WF-SUBDAG)   ∀n ∈ D.nodes. body(n) = SubDag(D') → 
                  ⊢ D' : wf ∧ interface_match(n, D')

───────────────────────────────────────────────────────────── 
                          ⊢ D : wf


─────────────────────────────────────────────────────────────
                    WELL-FORMED EDGE
─────────────────────────────────────────────────────────────

e = (n₁.p₁ → n₂.p₂)
n₁ ∈ D.nodes,  p₁ ∈ n₁.outputs
n₂ ∈ D.nodes,  p₂ ∈ n₂.inputs
type(p₁) = type(p₂)                    -- Type agreement
card(p₁) ⊑ card(p₂)                    -- Cardinality satisfaction
─────────────────────────────────────────────────────────────
                    wf_edge(D, e)


─────────────────────────────────────────────────────────────
                    SUBDAG INTERFACE
─────────────────────────────────────────────────────────────

∀p ∈ n.inputs.  ∃e ∈ D'.edges. entrypoint(D', p)
∀p ∈ n.outputs. ∃e ∈ D'.edges. boundary(D', p)
─────────────────────────────────────────────────────────────
                interface_match(n, D')
```

### 2.4 Soundness Statement

**Theorem (Static Soundness)**: If `⊢ D : wf`, then:
1. `lower(D)` succeeds and produces `D'` where `⊢ D' : wf`
2. `execute(D')` cannot fail due to:
   - Type mismatch
   - Cardinality violation
   - Missing port
   - Dangling edge
   - Cycle

**Proof sketch**: Induction on DAG structure; lowering preserves all WF judgments by construction.

---

## 3. Effect Model

### 3.1 Effect Types

```
Effect := Pure | WorldRead | WorldWrite

-- Lattice (subtyping):
Pure <: WorldRead <: WorldWrite
```

Intuition:
- `Pure`: No external I/O, deterministic, safe to parallelize/cache
- `WorldRead`: Reads external state, deterministic given state
- `WorldWrite`: Modifies external state, must be carefully ordered

### 3.2 Effect Inference

```
effect : Node<E,R> → Effect

effect(n) = match body(n) with
  | Opaque(e, _) → e
  | SubDag(D)    → ⊔ { effect(n') | n' ∈ D.nodes }
```

### 3.3 Effect Composition

For a DAG to be well-typed with effect `E`:

```
─────────────────────────────────────────────────────────────
                    EFFECT TYPING
─────────────────────────────────────────────────────────────

∀n ∈ D.nodes. effect(n) <: E
─────────────────────────────────────────────────────────────
                    D : Dag<E, R>
```

**Key property**: A `Dag<Pure, _>` contains only pure nodes — it can be memoized, parallelized without synchronization, and is referentially transparent.

### 3.4 Boundary Contract

**Definition**: A *boundary* is an output port with no downstream edge.

**Effect implication**:
- Boundary on `Pure` node: Signal only (e.g., completion)
- Boundary on `WorldRead` node: World-read result leaving DAG
- Boundary on `WorldWrite` node: World-write result leaving DAG

**Boundary Contract**:
```
BoundaryContract := {
    node: NodeId,
    port: PortName,
    effect: Effect,
    cardinality: Card,
    type: TypeId,
}
```

The boundary contract specifies what the outside world sees when the DAG executes.

---

## 4. Resource Model

### 4.1 Resource Types

```
Resource := Lock(ResourceId)
          | Lease(ResourceId, Duration)
          | SharedLock(ResourceId, MaxHolders)
          | PoolSlot(ResourceId, PoolSize)

ResourceReq := Set<Resource>
```

### 4.2 Resource Algebra

Resources form a commutative monoid under composition:

```
∅           : ResourceReq                    -- Identity
(⊕)         : ResourceReq → ResourceReq → ResourceReq
                                              -- Composition

-- Laws:
R ⊕ ∅ = R                                    -- Identity
R₁ ⊕ R₂ = R₂ ⊕ R₁                            -- Commutativity
(R₁ ⊕ R₂) ⊕ R₃ = R₁ ⊕ (R₂ ⊕ R₃)              -- Associativity
```

### 4.3 Resource Compatibility

Two nodes can execute in parallel iff their resources are compatible:

```
compatible : ResourceReq → ResourceReq → Bool

compatible(R₁, R₂) = ∀r₁ ∈ R₁, r₂ ∈ R₂. ¬conflict(r₁, r₂)

conflict(Lock(x), Lock(x))           = true   -- Exclusive
conflict(Lock(x), SharedLock(x, _))  = true   -- Write blocks read
conflict(SharedLock(x, n), _)        = count(x) > n
conflict(Lease(x, _), Lease(x, _))   = true   -- Exclusive
conflict(PoolSlot(x, n), _)          = count(x) >= n
conflict(_, _)                       = false
```

### 4.4 Resource-Typed DAGs

```
resources : Node<E,R> → ResourceReq

resources(n) = match body(n) with
  | Opaque(_, r) → r
  | SubDag(D)    → ⊕ { resources(n') | n' ∈ D.nodes }
```

For a DAG to be well-typed with resources `R`:

```
─────────────────────────────────────────────────────────────
                    RESOURCE TYPING
─────────────────────────────────────────────────────────────

⊕ { resources(n) | n ∈ D.nodes } ⊆ R
─────────────────────────────────────────────────────────────
                    D : Dag<E, R>
```

---

## 5. Unified Execution Model

### 5.1 Interpreter Semantics

Execution is an interpreter over proved programs:

```
Interpreter := {
    mode: ExecutionMode,
    boundary_handler: BoundaryContract → Value,
    resource_scheduler: ResourceReq → Acquisition,
}

ExecutionMode := Real | DryRun(BoundaryMocks) | Simulate(SimConfig)
```

**Key insight**: `DryRun` is not a "different mode" — it's the same interpreter with a different `boundary_handler`:

```
-- Real execution
boundary_handler_real : BoundaryContract → IO Value
boundary_handler_real(bc) = perform_world_io(bc)

-- Dry-run execution  
boundary_handler_dryrun : BoundaryMocks → BoundaryContract → Value
boundary_handler_dryrun(mocks, bc) = lookup(mocks, bc.node, bc.port)
```

### 5.2 Semantic Preservation

**Theorem (DryRun Preservation)**: For a well-formed DAG `D` and mock set `M`:
```
If ∀bc ∈ boundaries(D). M(bc) : bc.type ∧ card(M(bc)) ⊑ bc.cardinality
Then execute(D, DryRun(M)) produces outputs structurally identical to execute(D, Real)
```

This means dry-run is *semantics-preserving* — it can't expose failures that wouldn't exist in real execution (modulo the inherently-dynamic boundary values).

### 5.3 Chain Validation as Contract Composition

When tools are composed:

```
Tool_A : Dag<E₁, R₁> with boundaries B₁
Tool_B : Dag<E₂, R₂> with entrypoints E₂

compose(A, B) valid iff:
  ∀(b ∈ B₁, e ∈ E₂) connected by edge.
    type(b) = type(e) ∧ 
    card(b) ⊑ card(e) ∧
    effect(b) <: expected_effect(e)
```

**This is contract composition, not tool-specific logic.**

---

## 6. Implementation Mapping

### 6.1 Current State → Formal Model

| Formal Concept | Current Implementation | Location |
|----------------|----------------------|----------|
| `Card` lattice | `Cardinality` enum | `types.rs` |
| `satisfies` | `Cardinality::satisfies()` | `types.rs` |
| `wf_edge` | `check_edges()` | `validate.rs` |
| `wf_acyclic` | `check_cycles()` | `validate.rs` |
| `interface_match` | `check_subdag_interface()` | `validate.rs` |
| `boundary` | `detect_boundaries()` | `boundary.rs` |
| `lower` | `lower()` | `lower.rs` |
| `BoundaryContract` | `BoundaryMock` (partial) | `mock_spec.rs` |
| `ResourceReq` | `ResourceMocks` | `mock_spec.rs` |

### 6.2 Gaps to Close

| Formal Concept | Current State | Target |
|----------------|--------------|--------|
| `Effect` type | Implicit (boundary = WorldWrite) | Explicit in `Node<E, R>` |
| Effect inference | Manual | Automatic from node type |
| Resource typing | Runtime `MockSpec` | Compile-time `Dag<E, R>` |
| Contract composition | `validate_chain()` function | Type-level constraint |

### 6.3 Migration Path

**Phase 1**: Add `Effect` to nodes (backward compatible)
```rust
pub struct Node<T, E = Effect> {
    // ...existing fields
    pub effect: E,
}
```

**Phase 2**: Make `Effect` required, infer for SubDags
```rust
impl<T> Node<T, Effect> {
    pub fn effect(&self) -> Effect {
        match &self.body {
            NodeBody::Opaque { effect, .. } => *effect,
            NodeBody::SubDag(dag) => dag.nodes.iter()
                .map(|n| n.effect())
                .fold(Effect::Pure, Effect::join),
        }
    }
}
```

**Phase 3**: Add resource requirements to type
```rust
pub struct Dag<T, E = Effect, R = ResourceReq> { ... }

// Composition requires proving resource compatibility
impl<T, E, R> Dag<T, E, R> {
    pub fn compose<E2, R2>(self, other: Dag<T, E2, R2>) -> Dag<T, E::Join<E2>, R::Compose<R2>>
    where
        E: EffectJoin<E2>,
        R: ResourceCompose<R2>,
    { ... }
}
```

**Phase 4**: Move validation to type system
```rust
// Instead of:
validate_dag(&dag)?;  // Runtime check

// We get:
let dag: Dag<Pure, NoResources> = ...;  // Type proves validity
// Invalid DAGs simply don't compile
```

---

## 7. Invariants Summary

### Compile-Time Invariants (L1)

| Invariant | Mechanism |
|-----------|-----------|
| No type mismatches | `wf_edge` type agreement |
| Cardinality satisfaction | `satisfies` relation |
| No cycles | `wf_acyclic` |
| SubDag interfaces match | `interface_match` |
| Unique node IDs | `wf_nodes` |

### Lowering Invariants (L2)

| Invariant | Mechanism |
|-----------|-----------|
| Preserves well-formedness | `lower` construction |
| Preserves effects | Effect join over SubDag |
| Preserves resources | Resource composition |

### Execution Invariants (L4)

| Invariant | Mechanism |
|-----------|-----------|
| DryRun = semantic interpreter | Uniform boundary handler |
| Real = semantic interpreter | Same structure, real I/O |
| Only dynamic failures | All static failures rejected at L1 |

---

## 8. Open Questions

1. **Cardinality refinement**: Can we push cardinality into the type system (`List<T, OneOrMore>`) rather than runtime annotation?

2. **Effect polymorphism**: Should nodes be polymorphic over effects (`Node<T, E: Effect>`) to allow generic composition?

3. **Resource inference**: Can resource requirements be inferred from operation types rather than declared?

4. **Subtyping complexity**: Is the effect/resource subtyping worth the type system complexity?

5. **Incremental adoption**: How do we migrate existing tools without breaking changes?

---

## 9. References

- Haskell effect systems (MTL, Polysemy, Effectful)
- Rust's ownership/borrowing as resource tracking
- Session types for protocol conformance
- Refinement types (Liquid Haskell, F*)
- Linear types for resource management

---

## Appendix A: Cardinality Satisfies Truth Table

| Output ↓ / Input → | Zero | One | ZeroOrOne | ZeroOrMore | OneOrMore |
|--------------------|------|-----|-----------|------------|-----------|
| **Zero** | ✓ | ✗ | ✓ | ✓ | ✗ |
| **One** | ✗ | ✓ | ✓ | ✓ | ✓ |
| **ZeroOrOne** | ✗ | ✗ | ✓ | ✓ | ✗ |
| **ZeroOrMore** | ✗ | ✗ | ✗ | ✓ | ✗ |
| **OneOrMore** | ✗ | ✗ | ✗ | ✓ | ✓ |

---

## Appendix B: Effect Lattice

```
        WorldWrite (⊤)
             |
        WorldRead
             |
          Pure (⊥)
```

Subtyping: `Pure <: WorldRead <: WorldWrite`

Join: `E₁ ⊔ E₂` = least upper bound

| ⊔ | Pure | WorldRead | WorldWrite |
|---|------|-----------|------------|
| **Pure** | Pure | WorldRead | WorldWrite |
| **WorldRead** | WorldRead | WorldRead | WorldWrite |
| **WorldWrite** | WorldWrite | WorldWrite | WorldWrite |
