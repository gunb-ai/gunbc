# gunbc Design Overview

> **Goal**: Structural proof of workflow correctness. If it validates, it is structurally sound.

**Structurally sound** means: the graph is well-formed, acyclic, all edges satisfy type/cardinality compatibility, and all subDAG interfaces match. Structural soundness excludes dynamic failures (I/O, environment, op semantics).

---

## Philosophy: Causality is a DAG

We are modeling cause-and-effect systems. Causality is inherently a DAG.

Every computation, workflow, validation, or transformation can be understood as:
- **Causes** (inputs) that flow into
- **Effects** (outputs) that result from
- **Transformations** (nodes) that connect them

Effects cannot precede their causes. Dependencies must be acyclic. Information flows forward through time.

**Therefore: Everything is a DAG.**

We model a **single execution** as a DAG. Repetition (retry/while/poll) is represented explicitly as a higher-order construct whose body is a subDAG, not as cycles. See [Loops, Retries, and Repetition](#loops-retries-and-repetition).

| What We Model | Causal Interpretation |
|---------------|----------------------|
| **Workflows** | Cause: inputs → Effect: outputs |
| **Types** | Cause: raw value → Effect: validated value |
| **Validation** | Cause: predicate → Effect: pass/fail |
| **Resources** | Cause: acquire → Effect: release |
| **Tests** | Cause: mock inputs → Effect: expected outputs |

---

## Scope and Threat Model

### What We Protect Against

| Threat | Protected? | How | Current → Target |
|--------|-----------|-----|------------------|
| Cycles in workflow | ✓ | Validation (target: structural) | 3 → 1 |
| Type mismatch at edges | ✓ | Validation (target: structural) | 3 → 1 |
| Cardinality mismatch | ✓ | Validation (target: structural) | 3 → 1 |
| Missing/dangling ports | ✓ | Validation | 3 |
| SubDag interface mismatch | ✓ | Validation | 3 |
| Race conditions on resources | Partial | Data dependencies only (target: resource system) | 5 → 1 |
| Node panics on valid input | ✗ | **Out of scope** — nodes are trusted | — |
| Wrong business logic | ✗ | **Out of scope** — higher-order concern | — |

### What's Explicitly Out of Scope

**Node correctness is trusted.** We prove the wiring is correct. We trust that nodes honor their declared interfaces (types, cardinalities, effects).

The `Opaque(T)` in `NodeBody::Opaque(T)` means: "this operation's internals are not our concern." If a node declares it takes `One` input and produces `One` output, we trust it does so. If it panics, that's a bug in the node implementation, not a structural error.

**Bypassing the system**: Developers who want to bypass guarantees (e.g., call `std::fs` directly in an `Opaque` node) are allowed to — we can't stop them. The system provides guarantees for those who use it correctly. The benefit: issues are corralled to specific nodes rather than spread throughout the graph.

**Future possibility**: A `Node` trait with type-level input/output specifications could bring node interfaces into the proof system. For now, nodes are a trust boundary.

### Proof Obligations

| Property | Developer Proves | System Proves |
|----------|------------------|---------------|
| Node honors its declared interface | ✓ (trusted) | |
| Edges are type-compatible | | ✓ (validated) |
| Cardinality flows correctly | | ✓ (validated) |
| DAG is acyclic | | ✓ (validated) |
| Border ports are inferred | | ✓ (structural) |
| Business logic is correct | ✓ (tests) | |

---

## Guarantee Hierarchy

We push guarantees as early as possible:

| Level | Method | When | Example |
|-------|--------|------|---------|
| **1. Impossible by Structure** | Type system prevents invalid states | Compile | Border ports inferred from connectivity |
| **2. Impossible by Generation** | Code generation only produces valid code | Build | Generated CLI always has correct args |
| **3. Validated at Build** | Explicit checks during build/validation | Build | `validate_dag()` catches cardinality mismatch |
| **4. Validated at Runtime** | Checks during execution | Run | Mock spec constraint checking |
| **5. Tested** | Unit/integration tests | Test | Border node interception works |

**Preference**: 1 > 2 > 3 > 4 > 5.

**Note**: Level 1 examples refer to the typed builder API. Raw IR construction is Level 3 (validated).

### Honesty About Current State

Not everything is at Level 1 yet. Here's where key guarantees actually sit:

| Guarantee | Current Level | Target Level | Gap |
|-----------|---------------|--------------|-----|
| **Acyclicity** | 3 (validated) | 1 (structural) | Builder pattern needed |
| **Type compatibility** | 3 (validated) | 1 (structural) | Type-level edges needed |
| **Cardinality satisfaction** | 3 (validated) | 1 (structural) | Type-level cardinality needed |
| **Port uniqueness** | 3 (validated) | 1 (structural) | Newtype ports needed |
| **Border port detection** | 1 (structural) | 1 | ✓ Already there |
| **Effect ordering** | 5 (untested) | 1 (structural) | Resource system needed |
| **Resource conflicts** | 4 (runtime) | 1 (structural) | Resource typing needed |

---

## The Core Model

### DAG<T> — The Universal Structure

```rust
Dag<T>   := { nodes: [Node<T>], edges: [Edge] }
Node<T>  := { id, inputs: [Port], outputs: [Port], body: Opaque(T) | SubDag(Dag<T>) }
Port     := { name, type_id, cardinality }
Edge     := { from: (node, port), to: (node, port) }
```

The `T` parameter is the operation type: `GistOp`, `DepsOp`, `TypeOp`, etc.

### Cardinality — Set-Theoretic Bounds

Cardinality describes how many items can flow through a port per execution:

| Cardinality | Set of Possible Counts | Meaning |
|-------------|------------------------|---------|
| `Zero` | {0} | Signal only — indicates completion but carries no data (like `()` in Rust) |
| `One` | {1} | Exactly one value (required) |
| `ZeroOrOne` | {0, 1} | Optional |
| `ZeroOrMore` | {0, 1, 2, ...} | List (may be empty) |
| `OneOrMore` | {1, 2, 3, ...} | Non-empty list |

**Zero cardinality** is useful for:
- **Ordering**: A must complete before B, but B doesn't need A's output
- **Completion signals**: "this step finished"
- **Resource release**: "lock released"

For `Zero` ports, `type_id` is always `Unit` — type compatibility is trivial for signals.

**Satisfaction**: Output `A` satisfies input `B` iff every count A might produce is acceptable to B. Formally: `A ⊆ B`.

```
                ZeroOrMore (⊤)
                /        \
           ZeroOrOne    OneOrMore
              |    \       /
            Zero    One ──┘
            
      (Zero and One are siblings, not ordered)
```

**Note**: `Zero` and `One` are **incomparable** — neither is a subset of the other. A signal (0 items) cannot satisfy a need for data (1 item), and vice versa. They are at the same level in the partial order.

#### Design Tradeoff: Partial Order vs Lattice

This forms a **partial order**, not a true lattice. A true lattice would require a bottom element (`Impossible = {}`) below both Zero and One, enabling meet/join operations on any pair.

We chose partial order because:
- `Impossible` doesn't map to real usage (when would a port have "no valid cardinality"?)
- The satisfaction check only needs subset relationships, not meet/join
- Zero and One being incomparable is semantically correct

The tradeoff: some lattice-theoretic operations (computing greatest lower bound of Zero and One) are undefined. In practice, this hasn't been needed.

### Borders — Where DAG Meets World

We distinguish two orthogonal concepts (like border routers in networking — outside our known network, but known destinations):

**Border Ports** (structural — where data crosses the DAG boundary):
- **Border input**: Input port with no upstream edge (world → DAG)
- **Border output**: Output port with no downstream edge (DAG → world)

Border ports are inferred structurally — if a port has no edge, it's automatically a border. You can't forget to mark it.

**Border Nodes** (effectful — where side effects happen):
- Nodes with `Effect != Pure` (WorldRead or WorldWrite)
- These are where I/O actually occurs

**These are orthogonal concepts:**
- A `Pure` node can have border ports (takes input from world, does no I/O)
- A `WorldWrite` node can have no border ports (all edges internal to DAG)

**What DryRun intercepts**: Border *nodes* (effectful), not border ports (structural). DryRun replaces the effect behavior, not the data interface.

**Note on interception**: Border node interception is guaranteed when effects go through the framework's capability system. Developers who bypass the framework (e.g., direct `std::fs` calls) opt out of this guarantee. The system helps those who use it correctly.

### Effects — What Operations Do

```
Pure <: WorldRead <: WorldWrite
```

| Effect | Meaning |
|--------|---------|
| `Pure` | No external I/O, deterministic, safe to parallelize, can be made durable |
| `WorldRead` | Reads external state |
| `WorldWrite` | Modifies external state, must be ordered |

---

## Execution Model

**Constraint-based execution**: Nodes run when their dependencies are satisfied — not in batches, not as streams. A node executes once its inputs are available.

```rust
ExecutionMode := Real | DryRun(BorderMocks) | Simulate(SimConfig)
```

- **Real**: Execute world I/O through border nodes
- **DryRun**: Intercept border nodes (effectful), return mock values
- **Simulate**: Full simulation with timing/resources

DryRun is not a "different mode" — it's the same interpreter with a different effect handler. This means dry-run is **semantics-preserving**: it changes observations (what border nodes return), not structure (which nodes run, in what order).

**Key invariant**: Each node executes once per DAG execution. Cardinality describes the shape of the value produced/consumed per run (like `Option`, `Vec`, `NonEmptyVec`), not streaming.

---

## Loops, Retries, and Repetition

A single DAG execution is acyclic — no cycles exist in the execution graph. But real workflows need retries, polling, and iteration.

### The Principle

Repetition is modeled as **higher-order constructs** that re-execute subDAGs, not as cycles in the graph. The DAG remains a DAG; the runtime "unrolls" virtual cycles according to specified behavior.

### Constructs (Target Design)

| Construct | Behavior | Interface |
|-----------|----------|-----------|
| `Retry(n, backoff)` | Re-execute subDAG up to n times on failure | Same as inner subDAG |
| `While(condition)` | Re-execute subDAG while condition holds | Condition + inner interface |
| `Poll(interval, timeout)` | Re-execute subDAG periodically until success | Same as inner subDAG |
| `Map(collection)` | Execute subDAG once per item | Item type → result type |

### Implementation Considerations

This will test our JIT/runtime design. Options:
1. **Unroll at build time** (if bounds are known)
2. **Dynamic unrolling** (runtime determines iterations)
3. **Specify conditions at node creation** (enables static analysis)

The key invariant: **No cyclic edges exist in the execution graph; repetition is explicit as a construct.**

Common interfaces (like `gunb.ai` patterns) should be standardized so retry/poll behavior is predictable.

---

## Effect and Resource Ordering

This section documents a known gap that needs detailed design.

### The Problem

Two `WorldWrite` nodes without a data dependency can race:

```rust
dag.add_node(write_config);   // WorldWrite to filesystem
dag.add_node(write_manifest); // WorldWrite to filesystem
// No edge between them — executor might run in parallel
```

### Solution: Signal Edges

Use `Zero`-cardinality edges to create explicit ordering without passing data:

```rust
// Signal edge: write_config must complete before write_manifest
dag.add_edge(
    write_config.signal_out(),      // Zero cardinality output
    write_manifest.ordering_in(),   // Zero cardinality input
);
```

This keeps the DAG "honest" — ordering is visible in the graph, not hidden in implicit resource edges.

### Resource System (Target)

A **resource system** that tracks what each node reads/writes:

```rust
struct Resource {
    kind: ResourceKind,      // File, Network, Lock, etc.
    id: String,              // Path, URL, lock name, etc.
}

// Nodes declare their resources
Node<T> {
    reads: Vec<Resource>,
    writes: Vec<Resource>,
    ...
}
```

**Open questions:**
- Are resources declared explicitly or inferred from the operation type?
- Is `Resource` a type (structural) or a value (runtime)?
- Do resource conflicts create implicit edges, or require explicit ordering?

### Target Invariant

```
RESOURCE ORDERING INVARIANT:
∀ nodes A, B where A.writes ∩ B.writes ≠ ∅:
    edge(A, B) ∨ edge(B, A) ∨ resources_independent(A, B)
```

If two nodes might conflict on the same resource, the DAG must contain an explicit ordering edge, or the resource system must prove independence. This makes the invariant checkable.

---

## Fan-In/Fan-Out

This section documents a known gap that needs detailed design.

### The Principle

Fan-in (multiple outputs → one input) and fan-out (one output → multiple inputs) should be **type-derived**, not explicit policy. The compiler should do as much work as possible.

When developers use framework types (lists, maps, etc.), the compiler can infer correct behavior because we define the types from the ground up.

### Current Semantics (Sketch)

| Scenario | Semantics |
|----------|-----------|
| Fan-out: `One` output → multiple inputs | Broadcast — each input gets the same value |
| Fan-in: multiple `One` outputs → `One` input | **Error** — ambiguous which value |
| Fan-in: multiple `One` outputs → `ZeroOrMore` input | Collect into list (order: topological + edge order) |
| Fan-in: `ZeroOrMore` + `One` → `ZeroOrMore` | Concatenate |

### Target: Compiler-Inferred Merging

If the input type implements a merge/collect law (derivable from framework types), fan-in is permitted. Otherwise, it's rejected at validation.

```rust
// Pseudo-trait that framework types implement
trait Collectable<T> {
    fn collect(items: impl Iterator<Item = T>) -> Self;
}

// Fan-in permitted: ZeroOrMore<T> implements Collectable<T>
// Fan-in rejected: One<T> doesn't implement Collectable<T> (ambiguous)
```

**Invariant**: Any implicit aggregation must be justified by a derivable combining law; otherwise fan-in is illegal.

---

## Validation and Typestate

`validate_dag()` proves at build time:

| Check | Error |
|-------|-------|
| Type compatibility | `TypeMismatch` |
| Cardinality satisfaction | `CardinalityMismatch` |
| Acyclicity | `CycleDetected` |
| Port existence | `PortNotFound` |
| SubDag interfaces | `SubDagInterfaceMismatch` |

**If validation passes, these failures cannot occur at runtime.**

### Error Quality

Validation errors should point to the specific contradiction:
- Which edge violates which constraint
- What the actual vs expected type/cardinality is
- Where in the DAG the problem occurs

A vague "CardinalityMismatch somewhere" doesn't help. Errors are part of the value proposition.

### Making Validation Structural (Target)

Currently, validation is a runtime check. To make "structural errors can't happen" literally true:

```rust
struct Dag<T, State> { ... }
struct Unvalidated;
struct Validated;

impl<T> Dag<T, Unvalidated> {
    fn validate(self) -> Result<Dag<T, Validated>, DagError> { ... }
}

// Only validated DAGs can be executed
fn execute<T>(dag: &Dag<T, Validated>, mode: ExecutionMode) -> Result<...> { ... }
```

**Key property**: `Dag<T, Validated>` is immutable. To modify a validated DAG:
1. You get a `Dag<T, Unvalidated>` back and must re-validate, OR
2. You use `DagBuilder` which maintains validity by construction

This makes "proof once" literally true — you can't call `execute()` without proving validation succeeded, and you can't silently invalidate a validated DAG.

**Invariant**: No executable artifact exists without a validation witness.

---

## System Invariants

The system proves these properties once, so developers never have to re-prove them in their own code or tests.

### True Structural Invariants (Level 1 — Impossible to Violate)

| Invariant | How It's Enforced |
|-----------|-------------------|
| **Border ports are inferred** | No edge = border, automatically detected |
| **Border nodes are identifiable** | Effect type marks world interaction |

### Validated Invariants (Level 3 — Checked, Not Structural)

| Invariant | How It's Enforced | Should Be Level 1? |
|-----------|-------------------|-------------------|
| **DAGs are acyclic** | `validate_dag()` rejects cycles | Yes |
| **Edges connect compatible types** | `TypeId` equality checked | Yes |
| **Cardinality flows correctly** | `satisfies()` checked | Yes |
| **All ports exist** | Edge validation checks endpoints | Yes |
| **SubDag interfaces match** | Input/output ports verified | Yes |

### Not Yet Enforced

| Invariant | Current State | Risk |
|-----------|---------------|------|
| **Effect ordering** | Implicit in data flow | WorldWrite nodes can race |
| **Resource conflicts** | Runtime MockSpec | Parallel execution can conflict |
| **Node transformation totality** | **Out of scope** — trusted | Node could panic on valid inputs |
| **Determinism under parallelism** | Assumed for `Pure` | Shared mutable state could race |

---

## Path to Level 1: Structural Enforcement

### Making Cycles Structurally Impossible

**Current (Level 3)** — validation catches cycles after construction:

```rust
dag.add_node(node_a);
dag.add_node(node_b);
dag.add_edge(a -> b);
dag.add_edge(b -> a);  // Cycle — compiles fine
validate_dag(&dag)?;   // Rejected here
```

**Target (Level 1)** — builder prevents cycles by construction:

```rust
let mut builder = DagBuilder::new();
let a = builder.add_node(node_a);           // Generation 0
let b = builder.add_node_after(node_b, &a); // Generation 1
builder.add_edge(a.out("x"), b.in("y"));    // OK: gen 0 → gen 1
builder.add_edge(b.out("z"), a.in("w"));    // COMPILE ERROR: gen 1 → gen 0
```

The builder tracks "generations" — edges can flow from any earlier generation to any later generation (i < j), enabling diamond patterns. Cycles become unrepresentable.

**Note**: The generational builder is one authoring mechanism. Typestate validation (`Dag<T, Validated>`) is the hard gate that ensures execution never sees a cycle.

### Making Cardinality Structurally Impossible

**Current (Level 3)** — cardinality checked after edge creation:

```rust
let edge = Edge::new("filter", "out", "process", "in");
validate_dag(&dag)?;  // CardinalityMismatch caught here
```

**Target (Level 1)** — cardinality encoded in types:

```rust
fn add_edge<A, B>(from: Output<A>, to: Input<B>) -> Edge
where
    A: Satisfies<B>,  // Compile-time proof required
{ ... }

dag.add_edge(
    filter.output::<ZeroOrMore>("out"),
    process.input::<OneOrMore>("in"),
);
// COMPILE ERROR: ZeroOrMore does not satisfy OneOrMore
```

---

## The "Proof Once" Principle

Traditional approach:
```
Developer A: writes workflow → writes tests for type safety → writes tests for cardinality
Developer B: writes workflow → writes tests for type safety → writes tests for cardinality
Developer C: ...same...
```

gunbc approach:
```
System: proves type safety, cardinality, acyclicity, borders ONCE
Developer A: writes workflow → system rejects invalid, no tests needed for these
Developer B: writes workflow → system rejects invalid, no tests needed for these
Developer C: ...same...
```

**Developers write tests for business logic, not structural correctness.**

---

## What We Have (Current State)

| Component | Location | Status |
|-----------|----------|--------|
| DAG structure | `core/ir/src/dag.rs` | ✓ |
| Node/Port/Edge | `core/ir/src/node.rs` | ✓ |
| Cardinality | `core/ir/src/types.rs` | ✓ |
| Validation | `core/ir/src/validate.rs` | ✓ |
| Border detection | `core/ir/src/boundary.rs` | ✓ (note: file uses "boundary", doc uses "border") |
| Lowering | `core/exec/src/lower.rs` | ✓ |
| Execution | `core/exec/src/execute.rs` | ✓ |
| Mock specs | `core/test/src/mock_spec.rs` | ✓ |

### Lowering

Lowering flattens SubDags into the parent DAG, preserving all structural invariants:
- Inner border ports become edges to the parent
- Outer border ports remain borders
- Type and cardinality relationships are preserved
- Effect ordering is preserved

A validated SubDag, when lowered, produces a validated flat DAG.

---

## What's Next

| Gap | Current | Target |
|-----|---------|--------|
| Structural acyclicity | `validate_dag()` | `DagBuilder` with generations |
| Structural cardinality | `satisfies()` check | Type-level `Satisfies<B>` trait |
| Typestate validation | Runtime `validate_dag()` | `Dag<T, Validated>` typestate |
| Effect typing | Implicit | Explicit `Node<T, E>` |
| Resource system | None | Declared resources, signal edges, conflict detection |
| Fan-in/fan-out | Undefined | Compiler-inferred from framework types |
| Loop constructs | None | Retry, While, Poll, Map as higher-order nodes |
| Type DAGs | `TypeId` strings | `Dag<TypeOp>` with predicates |

---

## File Structure

```
gunbc/
├── core/
│   ├── ir/           # DAG types, validation, border detection
│   ├── exec/         # Execution, lowering, interception
│   ├── test/         # MockSpec, resource simulation
│   ├── testgen/      # Test code generation
│   └── codegen/      # CLI/entrypoint generation
├── lib/
│   ├── primitives/   # ReadFiles, WriteFiles, etc.
│   ├── transport/    # HTTP, File transports
│   └── tools/        # gist, deps, makegen, viz, ci, buck2, bootstrap
└── docs/
    └── design/       # This document
```
