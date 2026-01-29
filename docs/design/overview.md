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

### No Meta-Annotations — Types Are Fractal DAGs

A core design principle: **all behavior must be expressed through the type system, not through meta-annotations**.

Meta-annotations are external modifiers that can contradict or circumvent the type system. They create semantic holes where the type says one thing but runtime behavior differs.

**Banned pattern** (meta-annotation):
```rust
// DON'T: guard as external modifier that can contradict cardinality
Port { type_id: "String", cardinality: One, guard: Some(condition) }
// Type says "definitely one value" but guard can cause "no value"
```

**Correct pattern** (type expresses behavior):
```rust
// DO: optionality is in the type itself
Port { type_id: "Optional<String>", cardinality: ZeroOrOne }
// Type honestly describes that value may be absent
```

**Why types are fractal DAGs**: Types themselves can be DAGs that express complex behavior. An `Optional<T>` type is conceptually a small DAG:
```
[input: T] → [presence check] → [output: T | None]
```

This keeps the type system **closed and self-consistent**. Any behavior (optionality, validation, transformation) is expressed as a type, which is itself a DAG structure. No external annotations can break invariants.

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

**Bypass policy**:

- For **generated operations** using the transport system, bypass is structurally prevented (code generation controls I/O points).
- For **custom Opaque nodes**, bypass is allowed but opts out of guarantees (DryRun interception, resource inference) for that node. Such nodes should be treated conservatively.

The benefit: issues are corralled to specific nodes (custom Opaque) rather than spread throughout the graph. Generated workflows get full guarantees.

**Future possibility**: A `Node` trait with type-level input/output specifications could bring node interfaces into the proof system. For now, nodes are a trust boundary.

### Proof Obligations

| Property | Developer Proves | System Proves |
|----------|------------------|---------------|
| Node honors its declared interface | ✓ (trusted) | |
| Edges are type-compatible | | ✓ (validated) |
| Cardinality flows correctly | | ✓ (validated) |
| DAG is acyclic | | ✓ (validated) |
| Boundary/entrypoint ports are inferred | | ✓ (structural) |
| Business logic is correct | ✓ (tests) | |

---

## Guarantee Hierarchy

We push guarantees as early as possible:

| Level | Method | When | Example |
|-------|--------|------|---------|
| **1. Impossible by Structure** | Type system prevents invalid states | Compile | Boundary ports inferred from connectivity |
| **2. Impossible by Generation** | Code generation only produces valid code | Build | Generated CLI always has correct args |
| **3. Validated at Build** | Explicit checks during build/validation | Build | Builder rejects cardinality mismatch |
| **4. Validated at Runtime** | Checks during execution | Run | Mock spec constraint checking |
| **5. Tested** | Unit/integration tests | Test | Boundary node interception works |

**Preference**: 1 > 2 > 3 > 4 > 5.

**Note**: Level 1 examples refer to the typed builder API (target design). Current raw IR construction relies on structural constraints in the builder patterns.

### Honesty About Current State

Not everything is at Level 1 yet. Here's where key guarantees actually sit:

| Guarantee | Current Level | Target Level | Gap |
|-----------|---------------|--------------|-----|
| **Acyclicity** | 2 (builder) | 1 (structural) | Generational builder needed |
| **Type compatibility** | 2 (builder) | 1 (structural) | Type-level edges needed |
| **Cardinality satisfaction** | 2 (builder) | 1 (structural) | Type-level cardinality needed |
| **Port uniqueness** | 2 (builder) | 1 (structural) | Newtype ports needed |
| **Boundary/entrypoint detection** | 1 (structural) | 1 | ✓ Already there |
| **Effect ordering** | 5 (untested) | 1 (structural) | Resource system needed |
| **Resource conflicts** | 4 (runtime) | 1 (structural) | Resource typing needed |

---

## The Core Model

> **Status**: Fully implemented in `core/ir/`.

### DAG<T> — The Universal Structure

```rust
Dag<T>   := { nodes: [Node<T>], edges: [Edge] }
Node<T>  := { id, inputs: [Port], outputs: [Port], body: Opaque(T) | SubDag(Dag<T>) }
Port     := { name, type_id, cardinality }
Edge     := { from: (node, port), to: (node, port) }
```

The `T` parameter is the operation type: `GistOp`, `DepsOp`, `TypeOp`, etc.

**Note**: Conditional execution is modeled through explicit Branch patterns and optional types (`ZeroOrOne` cardinality), not through guards. See [Conditional Execution](#conditional-execution).

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

### Boundaries — Where DAG Meets World

> **Status**: Implemented. See `core/ir/src/boundary.rs` and `core/ir/src/entrypoint.rs`.

We distinguish two concepts based on graph connectivity:

**Boundary Outputs** (where data exits the DAG):
- Output ports with no downstream edge (DAG → world)
- Detected by `detect_boundaries()` in `boundary.rs`
- Used for **signature/codegen** — defining the workflow's interface

**Entrypoint Inputs** (where data enters the DAG):
- Input ports with no upstream edge (world → DAG)
- Detected by `detect_entrypoints()` in `entrypoint.rs`
- These define the DAG's required inputs

Both are inferred structurally — if a port has no edge, it's automatically detected. You can't forget to mark it.

**Important distinction:**

> **Boundary outputs/entrypoints are interface only.** They describe where data enters/leaves the workflow, not where I/O happens.
>
> **World I/O is performed only by transport executor nodes** (nodes that consume `TransportRequest` and produce `TransportResponse`).

**What DryRun intercepts**: Transport execution nodes, not boundary outputs. This ensures internal I/O (e.g., a file read mid-pipeline that feeds downstream nodes) is intercepted regardless of whether outputs are boundary outputs.

**Transport boundary node**: A node that causes a `TransportRequest` to be executed. DryRun intercepts these nodes by swapping the transport executor with a mock transport executor.

### Resources Are Typed Values

> **Status**: Design principle - no separate Effect enum needed, but resource identity + conflict validation still required.

Instead of an `Effect` enum (Pure/WorldRead/WorldWrite), resource dependencies are handled through the **type system**:

- Files, locks, connections, etc. are **typed values** that flow through edges
- If Node B needs a file that Node A created, there's an edge carrying that file
- The executor simply runs nodes when their inputs are ready - no special "effect" logic
- Ordering is explicit in the graph structure, not implicit in annotations

This keeps the design true to "causality is a DAG" - **all dependencies are visible as edges**.

**Why no Effect enum for scheduling?**
- Resources flowing through edges already capture dependencies
- The executor doesn't need to know "what kind" of operation - just follow edges
- Parallelization is automatic: nodes run when inputs are ready

**What we still need:**
- **Resource identity**: When two nodes independently reference the same resource (e.g., same file path), we need to detect conflicts
- **Conflict validation**: Derived from transport requests where possible (see [Resource Conflict Invariant](#resource-ordering))

---

### Workflow Signature

> **Status**: Implemented. See `core/ir/src/signature.rs`.

To prevent silent interface drift (where forgotten edges become accidental public API), workflows have explicit signatures:

```rust
struct SignaturePort {
    name: PortName,
    type_id: TypeId,
    cardinality: Cardinality,
}

struct WorkflowSignature {
    inputs: Vec<SignaturePort>,
    outputs: Vec<SignaturePort>,
}
```

**Including cardinality** is essential — optionality matters to CLI/codegen and for catching interface changes like `ZeroOrOne` → `One`.

**API:**
```rust
// Declare a signature
let sig = WorkflowSignature::new()
    .with_input("url", "String", Cardinality::One)
    .with_output("response", "Response", Cardinality::One);

// Validate against a DAG
sig.validate(&dag)?;  // Returns SignatureError if mismatch

// Or infer from DAG structure
let inferred = infer_signature(&dag);
```

**Invariant:**

> `DeclaredSignature == InferredSignature`

The inferred signature (computed from unconnected ports) must match the declared signature (type + cardinality). This catches:
- **Silent interface drift**: Forgot to wire an edge? Now it's a new public input/output
- **Wiring bugs**: Intended `A -> B` but forgot the edge - validation fails instead of silently exposing ports
- **Cardinality drift**: Changed `ZeroOrOne` to `One`? Signature check catches it
- **Dead work**: Pure nodes not contributing to any output can be flagged

Signatures can be inferred initially and checked in CI, but they must exist as **the contract**.

---

## Transport System

> **Status**: Implemented in `core/ir/src/transport/`.

The transport system provides a unified interface for external I/O operations. Instead of nodes directly performing I/O, they construct `TransportRequest` values that are executed by a transport executor.

### Architecture

```rust
TransportRequest := Rest(RestRequest) | Http(HttpRequest) | File(FileRequest) 
                  | Tcp(TcpRequest) | Shell(ShellRequest)

TransportResponse := corresponding response types
```

This separation enables:
- **Boundary interception**: DryRun can mock any transport operation
- **Unified logging**: All I/O goes through one path
- **Request preparation**: Pure nodes can construct requests; boundary nodes execute them

### Supported Transports

**REST** (`rest.rs`):
- Full REST API support with automatic JSON handling
- Auth methods: Bearer token, Basic auth, API key (header/query), environment variable
- Query parameters, headers, request body

**HTTP** (`http.rs`):
- Raw HTTP requests: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS
- Full control over headers and body

**File** (`file.rs`):
- Operations: Read, Write, Append, Delete, Exists, CreateDir
- `create_parents` flag for automatic directory creation

**TCP** (`tcp.rs`):
- Raw TCP connections
- Configurable connect/read/write timeouts

**Shell** (in transport `mod.rs`):
- Command execution with arguments
- Working directory, environment variables, stdin support

**Gist** (`gist.rs`):
- GitHub Gist-specific builder
- Can convert to REST requests or shell commands (using `gh` CLI)

---

## Execution Model

> **Status**: Implemented in `core/exec/`.

**Constraint-based execution**: Nodes run when their dependencies are satisfied — not in batches, not as streams. A node executes once its inputs are available.

```rust
ExecutionMode := Real | DryRun(TransportMocks)
```

- **Real**: Execute all operations normally
- **DryRun**: Intercept transport execution nodes, return mock responses

**Target (not yet implemented)**:
- **Simulate**: Full simulation with timing/resources (`SimConfig`)

**Important**: DryRun intercepts **transport execution**, not boundary outputs. This ensures:
- Internal I/O (e.g., a file read mid-pipeline that feeds downstream nodes) is intercepted regardless of connectivity
- Pure boundary outputs (final computed results with no I/O) are not intercepted

DryRun is not a "different mode" — it's the same interpreter with a different transport executor. This means dry-run is **semantics-preserving**: it changes I/O results (what transport nodes return), not structure (which nodes run, in what order).

**Key invariant**: Each node **instance** executes at most once per execution trace. Higher-order constructs (retry, while, map) may create multiple instances of a template node (e.g., per retry attempt / per loop iteration / per map item). Cardinality describes the shape of the value produced/consumed per run (like `Option`, `Vec`, `NonEmptyVec`), not streaming.

### Input Availability

An input is **available** when every required upstream producer has completed and produced its (single-shot) output value for this run. Because we're not streaming, `ZeroOrMore` still arrives as a final collection (a `Vec`), not an ongoing stream.

### Conditional Execution

> **Status**: Implemented via Branch pattern. See `core/ir/src/patterns/branch.rs`.

Conditional execution is modeled through **explicit Branch patterns** and **optional types**, not through guards as meta-annotations.

**The Branch Pattern**:
```
┌─────────────────────────────────────────────────┐
│                    Branch                        │
│              ┌───────────┐                      │
│      ┌──────▶│ True DAG  │──────┐              │
│      │       └───────────┘      │              │
│  condition                      ▼              │
│     │                       ┌───────┐          │
│     │       ┌───────────┐   │ Merge │─▶ output │
│     └──────▶│ False DAG │──▶│       │          │
│             └───────────┘   └───────┘          │
└─────────────────────────────────────────────────┘
```

- Takes a boolean condition and routes input to one of two branches
- Each branch is a complete sub-DAG
- Merge combines results (exactly one branch produces output)

**Optional Types for Conditional Values**:

When a value may or may not be present, use optional types:
```rust
// Value that might not exist
Port::optional("result", "Response")  // cardinality: ZeroOrOne

// Downstream must handle optionality explicitly
Port::optional("maybe_result", "Response")  // accepts Optional
```

This keeps cardinality **honest** — if something might be absent, the type says so.

**Why not guards as meta-annotations?**

Guards as port annotations (`guard: Option<Guard>`) create a semantic hole: a port can declare `cardinality: One` (definitely produces a value) but the guard can cause it to produce nothing. This violates type safety.

By modeling conditional execution as explicit DAG structure (Branch) and optional types (`ZeroOrOne`), the type system remains closed and self-consistent. See [No Meta-Annotations](#no-meta-annotations--types-are-fractal-dags).

---

## Loops, Retries, and Repetition

A single DAG execution is acyclic — no cycles exist in the execution graph. But real workflows need retries, polling, and iteration.

### The Principle: Template DAG + Instance DAG

Repetition is modeled as **higher-order constructs** where:

- The loop body is a **template DAG** `G`
- At runtime, each iteration produces a new **instance** of `G` tagged with an `IterationId`
- The runtime creates **activation records** (think: `G@iter=0`, `G@iter=1`, …)
- Edges may connect `iter=k` outputs to `iter=k+1` inputs

> A loop construct is equivalent to expanding the workflow into a larger DAG where each iteration is a fresh copy of the body subDAG with a strictly increasing iteration index. This expansion is always acyclic.

The *semantic* loop is cyclic, but the *execution trace graph* remains a DAG because time/iteration indexes provide a natural order.

### Loop Expansion Invariant

> Every loop construct defines an expansion into an acyclic execution trace DAG (nodes are instances with monotonically increasing iteration indices).

This means:
- You don't need to precompute the full unroll
- You can unroll **incrementally**: create iteration `k+1` only when needed
- Caching/durability can be keyed by `(TemplateNodeId, IterationId, InputsHash)`

### Implemented Constructs

> **Status**: Partially implemented in `core/ir/src/patterns/`.

| Construct | Status | Location |
|-----------|--------|----------|
| `Map(collection)` / `Loop` | **Implemented** | `patterns/loop_pattern.rs` — `LoopBuilder` |
| `Branch` (if/else) | **Implemented** | `patterns/branch.rs` — `BranchBuilder`, `IfBuilder` |
| `Atomic` | **Implemented** | `patterns/atomic.rs` — precondition/operation/postcondition |
| `Transaction` | **Implemented** | `patterns/transaction.rs` — begin/commit/rollback |
| `Upsert` | **Implemented** | `patterns/upsert.rs` — check/create/resolve |

### Target Constructs (Not Yet Implemented)

| Construct | Behavior | Interface |
|-----------|----------|-----------|
| `Retry(n, backoff)` | Re-execute subDAG up to n times on failure | Body + failure classifier + policy |
| `While(condition)` | Re-execute subDAG while condition holds | Condition + body + optional loop-carried state |
| `Poll(interval, timeout)` | Re-execute subDAG periodically until success | Body + interval + timeout |

### Standard Repetition Interface

All repetition constructs share a common interface:

```rust
struct RepeatConstruct<T> {
    body: Dag<T, Validated>,           // Template DAG
    classifier: FailureClassifier,      // What counts as retryable (for Retry)
    policy: RepeatPolicy,               // Max attempts, backoff, jitter, timeout
    loop_state: Option<TypeId>,         // Optional loop-carried state type
}
```

`Retry` is a specialization of `Repeat` with a failure classifier.

---

## Resource Ordering

> **Status**: Handled by type system - resources flow through edges.

### The Principle

Since resources are typed values flowing through edges, ordering is **automatic**:

```rust
// Node A produces a file resource
let file = write_config();  // outputs File typed value

// Node B consumes that file resource - edge exists
let result = read_config(file);  // takes File as input
```

The edge carrying the `File` value ensures A runs before B. No separate "resource system" needed.

### When Nodes Don't Share Data But Share Resources

If two nodes write to the **same** file but don't pass data between them, use signal edges:

```rust
// Signal edge: write_config must complete before write_manifest
dag.add_edge(
    write_config.signal_out(),      // Zero cardinality output
    write_manifest.ordering_in(),   // Zero cardinality input
);
```

This keeps the DAG "honest" — ordering is visible in the graph, not hidden.

### Resource Conflict Invariant

For resources that aren't passed through edges, validation should check:

```
RESOURCE CONFLICT INVARIANT:
∀ nodes A, B where conflict(A, B):
    edge(A, B) ∨ edge(B, A)

where conflict(A, B) iff:
    (A.writes ∩ B.writes) ∪ (A.writes ∩ B.reads) ∪ (A.reads ∩ B.writes) ≠ ∅
```

This covers:
- **Write/write conflicts** - two nodes writing to the same resource
- **Write/read conflicts** - one node writes, another reads the same resource

If nodes conflict on a resource, the DAG must contain an explicit ordering edge.

---

## Fan-In/Fan-Out (Target Design)

> **Status**: Not yet implemented. This section documents target design.

### The Principle

Fan-in (multiple outputs → one input) and fan-out (one output → multiple inputs) should be **type-derived**, not explicit policy. The compiler should do as much work as possible.

When developers use framework types (lists, maps, etc.), the compiler can infer correct behavior because we define the types from the ground up.

### Current Semantics (Sketch)

| Scenario | Semantics |
|----------|-----------|
| Fan-out: `One` output → multiple inputs | Broadcast — each input gets the same value |
| Fan-in: multiple `One` outputs → `One` input | **Error** — ambiguous which value |
| Fan-in: multiple `One` outputs → `ZeroOrMore` input | Collect into list (canonical order) |
| Fan-in: `ZeroOrMore` + `One` → `ZeroOrMore` | Concatenate (canonical order) |

### Canonical Edge Ordering (Required for Determinism)

Collection order must be deterministic across builds. The canonical sort key:

```rust
// Edges are ordered by: (from_node_id, from_port_name, edge_index)
fn canonical_edge_order(edges: &[Edge]) -> Vec<&Edge> {
    edges.sorted_by_key(|e| (&e.from_node, &e.from_port, e.index))
}
```

**Tie-breaker**: `edge_index` handles cases where multiple edges have the same source node/port (rare but possible with future features).

**Invariant:**

> Collection order is deterministic and derived from a canonical ordering of incoming edges.

This ensures the same DAG always produces the same collection order, regardless of how it was constructed.

**Note on renaming**: Renaming ports can change canonical ordering. This is acceptable but should be documented as a potentially breaking change for serialized collections.

### Map/Dict Merge Laws

For map types, merge semantics must be explicit:

| Merge Strategy | Behavior |
|----------------|----------|
| `ErrorOnDuplicate` | Fail validation if duplicate keys possible |
| `LastWriteWins` | Later edge (by canonical order) overwrites |
| `CombineValues` | Values with same key are merged (requires nested Collectable) |

Default is `ErrorOnDuplicate` to prevent silent data loss.

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

## Structural Validity by Construction

> **Status**: Target design. The goal is to make invalid DAGs unrepresentable through the builder API.

### The Principle

Instead of "build anything, then validate," we aim for "the builder only produces valid DAGs."

When you use `DagBuilder`:
- Cycles are impossible (generational tracking)
- Type mismatches are compile errors (typed ports)
- Cardinality violations are compile errors (type-level cardinality)

### Current State

Currently, the system relies on structural constraints in the builder patterns. Validation is implicit in construction rather than an explicit pass.

### Target: DagBuilder with Compile-Time Guarantees

```rust
let mut builder = DagBuilder::new();
let a = builder.add_node(node_a);           // Generation 0
let b = builder.add_node_after(node_b, &a); // Generation 1
builder.add_edge(a.out("x"), b.in("y"));    // OK: gen 0 → gen 1
builder.add_edge(b.out("z"), a.in("w"));    // COMPILE ERROR: gen 1 → gen 0
```

The builder tracks "generations" — edges can flow from any earlier generation to any later generation (i < j). Cycles become unrepresentable.

### Why Not Runtime Validation?

Runtime validation ("build anything, then check") has downsides:
- Errors caught late, after invalid structure already exists
- Requires explicit validation calls that can be forgotten
- Doesn't fit "proof once" — every modification requires re-validation

The target is: **if it compiles, it's valid.**

---

## System Invariants

The system proves these properties once, so developers never have to re-prove them in their own code or tests.

### Guaranteed Invariants (Current Implementation)

These invariants are currently enforced and can be relied upon:

| # | Invariant | How Enforced |
|---|-----------|--------------|
| 1 | **Acyclic execution trace** | Even with loops, the instantiated trace is acyclic (iteration-indexed) |
| 2 | **Type + cardinality compatibility** | Every edge checked, `Zero` = `Unit` |
| 3 | **Resources flow through edges** | Dependencies are explicit; no hidden "effect" annotations |
| 4 | **Boundary/entrypoint detection** | Structural — output/input ports without edges automatically detected |

### Structural Invariants (Level 1 — Impossible to Violate)

| Invariant | How It's Enforced |
|-----------|-------------------|
| **Boundary outputs are inferred** | Output port with no downstream edge = boundary, automatically detected |
| **Entrypoint inputs are inferred** | Input port with no upstream edge = entrypoint, automatically detected |

### Design Target Invariants (Not Yet Enforced)

These are design goals that will be enforced in future implementations:

| Invariant | Current State | Target |
|-----------|---------------|--------|
| **Resource conflict ordering** | Not checked | Validation rejects unordered conflicts |
| **Fan-in canonical ordering** | Partially defined | Deterministic edge ordering with tie-breakers |
| **Workflow signature matching** | Not implemented | Declared == Inferred check |
| **Implicit aggregation is lawful** | Not checked | Fan-in only when type has merge/collect law |

### Out of Scope

| Invariant | Why |
|-----------|-----|
| **Node transformation totality** | Nodes are trusted — they honor declared interfaces |
| **Business logic correctness** | Higher-order concern — tested by developers |

---

## Path to Level 1: Structural Enforcement (Target Design)

> **Status**: None of the Level 1 structural enforcement described here is implemented yet. These are design targets.

### Making Cycles Structurally Impossible

**Current (Level 3)** — validation catches cycles after construction:

```rust
dag.add_node(node_a);
dag.add_node(node_b);
dag.add_edge(a -> b);
dag.add_edge(b -> a);  // Cycle — compiles fine
// Cycle detected at edge creation time
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

The generational builder makes cycles unrepresentable at compile time.

### Making Cardinality Structurally Impossible

**Current (Level 3)** — cardinality checked after edge creation:

```rust
let edge = Edge::new("filter", "out", "process", "in");
// CardinalityMismatch caught at edge creation time
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
System: proves type safety, cardinality, acyclicity, boundaries ONCE
Developer A: writes workflow → system rejects invalid, no tests needed for these
Developer B: writes workflow → system rejects invalid, no tests needed for these
Developer C: ...same...
```

**Developers write tests for business logic, not structural correctness.**

---

## What We Have (Current State)

### Core IR (`core/ir/`)

| Component | Location | Status |
|-----------|----------|--------|
| DAG structure | `dag.rs` | ✓ |
| Node/Port/Edge | `node.rs` | ✓ |
| Cardinality & Types | `types.rs` | ✓ |
| Boundary detection | `boundary.rs` | ✓ |
| Entrypoint detection | `entrypoint.rs` | ✓ |
| Runtime values | `value.rs` | ✓ |

### Patterns (`core/ir/src/patterns/`)

| Pattern | Location | Description |
|---------|----------|-------------|
| Loop/Map | `loop_pattern.rs` | Iterate over collections: Unpack → Body → Pack |
| Branch | `branch.rs` | Conditional if/else with merge |
| Atomic | `atomic.rs` | Precondition → Operation → Postcondition |
| Transaction | `transaction.rs` | Begin → Body → Commit/Rollback |
| Upsert | `upsert.rs` | Check → Create → Resolve |
| Retry | `repeat.rs` | Retry with policy (backoff, max attempts) |
| While | `repeat.rs` | Loop while condition holds, with loop-carried state |
| Poll | `repeat.rs` | Periodic execution until success/timeout |

### Transports (`core/ir/src/transport/`)

| Transport | Location | Operations |
|-----------|----------|------------|
| REST | `rest.rs` | Requests with auth (Bearer, Basic, ApiKey, EnvVar) |
| HTTP | `http.rs` | GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS |
| File | `file.rs` | Read, Write, Append, Delete, Exists, CreateDir |
| TCP | `tcp.rs` | Raw TCP connections with timeouts |
| Shell | (in `mod.rs`) | Command execution with args, cwd, env, stdin |
| Gist | `gist.rs` | GitHub Gist-specific builder |

### Execution (`core/exec/`)

| Component | Location | Status |
|-----------|----------|--------|
| Execution engine | `execute.rs` | ✓ |
| Lowering | `lower.rs` | ✓ |
| Topological sort | `topo.rs` | ✓ |
| Boundary interception | `intercept.rs` | ✓ |

### Testing (`core/test/`)

| Component | Location | Status |
|-----------|----------|--------|
| Mock specs | `mock_spec.rs` | ✓ — builder pattern, chain validation |
| Mock operations | `mock.rs` | ✓ — scripted and functional mocks |
| Mockable trait | `mockable.rs` | ✓ — test fixture generation |
| Boundary testing | `boundary.rs` | ✓ — dry-run verification |
| Composition testing | `composition.rs` | ✓ — type compatibility checking |

### Lowering

Lowering flattens SubDags into the parent DAG. This is **structure-preserving by construction**:

**Why lowering preserves validity:**

1. **Renaming**: Inner node IDs are namespaced by `(outer_subdag_node_id, inner_node_id)` to avoid collisions
2. **Rewiring**:
   - Edges `X → SubDag.in(p)` become `X → Inner.entry(p)`
   - Edges `SubDag.out(q) → Y` become `Inner.boundary(q) → Y`
   - Internal edges remain unchanged under renaming
3. **Acyclicity**: Outer DAG topo order + inner DAG topo order = valid combined order (no backward edges introduced)
4. **Type/cardinality**: Inner edges already compatible (validated); cross edges compatible by SubDag interface

**SubDag interface contract**: When constructing a `Node::SubDag`, the inner DAG's entrypoints and boundaries must match the node's declared input/output ports (name, type, cardinality). This interface matching is verified at construction time, making lowering a trusted operation.

**Result**: `lower(Dag<T>) → Dag<T>` preserves structural validity. No re-validation needed after lowering.

---

## What's Next

| Gap | Current | Target | Status |
|-----|---------|--------|--------|
| Structural acyclicity | `DagBuilder` with generations | Type-level prevention | ✓ Implemented |
| Structural cardinality | `satisfies()` check | Type-level `Satisfies<B>` trait | Target |
| Workflow signature | `infer_signature()` + `validate()` | Compile-time checked | ✓ Implemented |
| Retry/While/Poll | `RetryBuilder`, `WhileBuilder`, `PollBuilder` | Template + instance semantics | ✓ Implemented |
| Fan-in canonical ordering | `canonical_edge_order()` | Deterministic collection | ✓ Implemented |
| Fan-in/fan-out inference | Undefined | Compiler-inferred from framework types | Target |
| Simulate mode | None | `ExecutionMode::Simulate(SimConfig)` | Target |
| Type DAGs | `TypeId` strings | `Dag<TypeOp>` with predicates | Target |
| Resource conflicts | Runtime detection | Structural prevention | Target |

---

## File Structure

```
gunbc/
├── core/
│   ├── ir/               # DAG types, boundary detection
│   │   ├── src/
│   │   │   ├── dag.rs        # Dag<T>, Edge, Port
│   │   │   ├── node.rs       # Node<T>, NodeBody
│   │   │   ├── types.rs      # Cardinality, TypeId, NodeId
│   │   │   ├── builder.rs    # DagBuilder (generational, cycle-free)
│   │   │   ├── signature.rs  # WorkflowSignature, infer_signature()
│   │   │   ├── boundary.rs   # detect_boundaries()
│   │   │   ├── entrypoint.rs # detect_entrypoints()
│   │   │   ├── value.rs      # Runtime Value enum
│   │   │   ├── patterns/     # Loop, Branch, Atomic, Transaction, Upsert, Retry, While, Poll
│   │   │   └── transport/    # REST, HTTP, File, TCP, Shell, Gist
│   ├── exec/             # Execution, lowering, interception
│   │   ├── src/
│   │   │   ├── execute.rs    # execute(), ExecutionMode
│   │   │   ├── lower.rs      # SubDag flattening
│   │   │   ├── topo.rs       # Topological sort
│   │   │   └── intercept.rs  # TransportMocks
│   ├── test/             # MockSpec, test infrastructure
│   ├── testgen/          # Test code generation
│   └── codegen/          # CLI/entrypoint generation
├── lib/
│   ├── primitives/       # ReadFiles, WriteFiles, Parse, etc.
│   ├── transport/        # Transport executor
│   └── tools/            # gist, deps, makegen, viz, ci, buck2, bootstrap
└── docs/
    └── design/           # This document
```
