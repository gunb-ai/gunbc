# gunbc Design Overview

> **Goal**: Structural proof of workflow correctness. If it validates, it is structurally sound.

Companion docs: `docs/handbook.md` (practical guide) and `SPEC.md` (formal IR spec).

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

#### Definition: Semantic Meta-Annotation

> **Semantic meta-annotation:** any attribute that can change observable behavior (values, branching, ordering, I/O, retries, failure behavior), **without being representable as nodes/edges/types** in the core DAG model and therefore without being validated by structural rules.

Guards on ports were the canonical example: a port could declare `cardinality: One` but a guard could cause it to produce nothing, violating the type contract.

#### The Erasure Lemma

This invariant makes the ban operational:

> **Metadata erasure is semantics-preserving:** removing all non-semantic metadata does not change the workflow's observable behavior (given the same transport/mock results).

If you can defend this statement, you've successfully eliminated semantic holes.

#### Metadata Classification

| Class | Allowed? | Rule | Examples |
|-------|----------|------|----------|
| **Descriptive** | ✅ Yes | Must be erasable without behavior change | Display names, docs, tags, ownership, version, source spans, logging labels, visualization hints |
| **Optimization hints** | ✅ Yes (with rule) | Must not change functional results | Cost estimates (cpu/mem/time), parallelism hints, cache hints, placement hints |
| **Semantic modifiers** | ❌ Banned | Must be modeled structurally | Guards that skip required values, implicit resource edges, "world write" tags not tied to transport |

**Optimization hint rule:** If a hint can change results, it is not a hint — it must be modeled structurally (nodes/edges/types).

#### Correct Pattern: Cardinality Expresses Optionality

```rust
// DO: optionality expressed through cardinality
Port::optional("result", "Response")  // cardinality: ZeroOrOne
// Type system knows value may be absent and validates accordingly
```

Any "nice syntax" that *looks* like an annotation must **desugar** into explicit DAG structure (Branch/Repeat/Collect/etc.) before validation/execution.

#### Why Types Are Fractal DAGs

Types themselves can be DAGs (`Dag<TypeOp>`) that express validation and transformation:

```
String (raw) → [NonEmpty check] → [URL pattern check] → Url (validated)
```

This is not an analogy — type validation IS a causal chain. Using `Dag<TypeOp>` makes this explicit and reuses all DAG infrastructure. See `type_op.rs` and `type_lib.rs`.

**Note:** Types may *eventually* be fully represented as DAGs of type-operations. For now, `TypeId` is an opaque identifier with cardinality inference via `TypeRegistry`, but the system is designed to lift type semantics into graph structure.

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

#### Normalization: Cardinality Is The Canonical Shape Layer

**Cardinality is the single source of truth for value shape.** Type IDs describe the *kind* of data; cardinality describes *how many*.

| Port Declaration | Canonical Form | Type | Cardinality |
|-----------------|----------------|------|-------------|
| `Port::optional("x", "String")` | ✅ Canonical | `String` | `ZeroOrOne` |
| `Port::scalar("x", "String")` | ✅ Canonical | `String` | `One` |
| `Port::list("x", "String")` | ✅ Canonical | `String` | `ZeroOrMore` |

**Anti-pattern** (redundant/contradictory):
```rust
// DON'T: type_id encodes optionality AND cardinality also encodes it
Port { type_id: "Optional<String>", cardinality: ZeroOrOne }  // redundant
Port { type_id: "Optional<String>", cardinality: One }         // contradictory!
```

**Rule:** If the type registry contains wrapper types (e.g., types built with `type_lib::optional()`), their cardinality is inferred via `TypeRegistry::infer_cardinality()`. The canonical pattern is to use unwrapped types with explicit cardinality:

```rust
Port::optional("result", "String")  // type: String, cardinality: ZeroOrOne
```

This prevents contradictions like `Optional<T>` + `OneOrMore`.

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

> **Status**: Implemented. No separate Effect enum needed for scheduling. Resource conflict detection implemented in `resource.rs`.

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

**What's implemented (`resource.rs`):**
- **Resource identity**: `ResourceId` identifies resources (files, locks, connections)
- **Conflict detection**: `detect_conflicts()` finds unordered accesses to the same resource
- **Access modes**: `AccessMode` (Read/Write/Exclusive) determines what conflicts

See [Resource Conflict Invariant](#resource-ordering) for details.

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
    .with_input("url", "String", Cardinality::ONE)
    .with_output("response", "Response", Cardinality::ONE);

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

> **Status**: Implemented in `core/ir/src/patterns/`.

| Construct | Location | Description |
|-----------|----------|-------------|
| `Map(collection)` / `Loop` | `loop_pattern.rs` — `LoopBuilder` | Iterate over collections |
| `Branch` (if/else) | `branch.rs` — `BranchBuilder`, `IfBuilder` | Conditional execution |
| `Atomic` | `atomic.rs` | Precondition → operation → postcondition |
| `Transaction` | `transaction.rs` | Begin → body → commit/rollback |
| `Upsert` | `upsert.rs` | Check → create → resolve |
| `Retry(n, backoff)` | `repeat.rs` — `RetryBuilder` | Re-execute on failure with backoff |
| `While(condition)` | `repeat.rs` — `WhileBuilder` | Loop while condition holds |
| `Poll(interval, timeout)` | `repeat.rs` — `PollBuilder` | Periodic execution until success |

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

The edge carrying the `File` value ensures A runs before B.

**Clarification on resource systems:**

> **No separate effect system is needed for scheduling.** Resources flowing through edges already capture dependencies — the executor doesn't need to know "what kind" of operation, just follow edges.
>
> **A resource conflict checker IS still needed** for independently referenced resources (e.g., two nodes both write to the same file path without an edge between them). This is implemented in `resource.rs` with `detect_conflicts()` and `ResourceAccess`.

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
| **Resource conflict ordering** | Implemented (`resource.rs`) | Validation rejects unordered conflicts |
| **Fan-in canonical ordering** | Implemented (`dag.rs`) | Deterministic edge ordering with tie-breakers |
| **Implicit aggregation is lawful** | Not checked | Fan-in only when type has merge/collect law |

### Out of Scope

| Invariant | Why |
|-----------|-----|
| **Node transformation totality** | Nodes are trusted — they honor declared interfaces |
| **Business logic correctness** | Higher-order concern — tested by developers |

---

## Graph Invariants

These invariants define what makes a well-formed graph beyond structural validity. They ensure graphs are not just correct, but elegant, observable, and maintainable.

### I1. Node Purity

> **Every node is either pure (deterministic, no side effects) or a Transport Execute node (the designated I/O boundary).**

| Node Type | Properties | Example |
|-----------|------------|---------|
| **Pure** | Given same inputs, produces same outputs. No I/O. | `ParseOp`, `PrepareFileReadOp`, `ExtractOp` |
| **Transport Execute** | Takes `TransportRequest` input, produces `TransportResponse` output. The only I/O point. | `TransportOps::Execute` |

**Violation**: An opaque node that calls `execute_transport()` internally. The I/O is hidden, making the node impure without being a proper Transport Execute node.

**Why it matters**: Pure nodes can be memoized, parallelized, and reasoned about locally. Mixing I/O into pure nodes breaks these properties.

**Structural Enforcement**: The escape hatch (`execute_transport()` callable from ops) is being removed. Once `execute_transport()` is not exported from `lib/transport`, the only way to do I/O is through `TransportOps::Execute` nodes — which is enforced by the compiler, not linting.

**Key insight**: Custom pure ops are fine. The invariant is simply: **no `execute_transport()` calls outside `TransportOps::Execute`**. This is much simpler than "primitives only."

### I2. Transport Boundary

> **All world I/O flows through `TransportRequest` → `TransportOps::Execute` → `TransportResponse`.**

The transport layer (`lib/transport`) is the single point where I/O actually happens. All other code constructs requests or processes responses.

| Layer | Responsibility |
|-------|---------------|
| **Domain ops** | Construct `TransportRequest` values (pure) |
| **Transport Execute** | Execute requests, produce responses (I/O boundary) |
| **Result processing** | Parse/validate responses (pure) |

**Violation**: Direct `std::fs::*`, `std::process::Command::new`, or HTTP calls outside the transport layer.

**Enforcement**: `clippy.toml` bans these methods. Approved exceptions are limited to the I/O boundary (`lib/transport`); tests are exempt by pragma policy.

### I3. Observable I/O

> **All I/O operations are visible as explicit nodes in the graph structure.**

This is the graph-level consequence of I1 and I2. If I/O is hidden inside opaque nodes, it's not observable.

| Property | Observable | Hidden |
|----------|------------|--------|
| DryRun interception | ✓ Intercepts `TransportRequest` inputs | ✗ Can't see internal `execute_transport()` calls |
| Visualization | ✓ Shows I/O nodes explicitly | ✗ Opaque box hides I/O |
| Composition | ✓ Can wrap I/O in Retry/Circuit Breaker | ✗ I/O is internal implementation detail |

**Target structure**:
```
Pure (Prepare) → Transport Execute → Pure (Parse)
                      ↑
              DryRun intercepts here
```

**Violation structure**:
```
Opaque Node (I/O hidden inside)
      ↑
DryRun can't intercept
```

### I4. Minimal Graph

> **Workflows use the minimum nodes necessary, with maximum reuse of canonical patterns.**

Sub-properties:

| Property | Description |
|----------|-------------|
| **No redundancy** | Every node contributes to an output. No dead nodes. |
| **Pattern reuse** | Use `UpsertBuilder`, `LoopBuilder`, `TransactionBuilder` instead of ad-hoc equivalents |
| **No reinventing** | Don't hand-write check-create-verify when `Upsert` exists |
| **Elegance** | Graphs are clean, understandable, and match the problem structure |

**Canonical patterns** (in `core/ir/src/patterns/`):

| Pattern | Use When |
|---------|----------|
| `Upsert` | Check if exists → create if not → resolve |
| `Loop` | Apply body DAG to each item in collection |
| `Transaction` | Begin → body → commit/rollback |
| `Branch` | Conditional execution with merge |
| `Atomic` | Precondition → operation → postcondition |
| `Retry/While/Poll` | Repetition with policies |

**Violation**: Hand-writing a check-exists-then-create pattern instead of using `UpsertBuilder`.

### I5. Deterministic Ordering

> **Fan-in produces deterministic collection order via canonical edge ordering.**

When multiple edges feed into a single port, the order of values in the resulting collection is deterministic.

**Canonical sort key**: `(from_node_id, from_port_name, edge_index)`

**Status**: Implemented in `dag.rs` with `canonical_edge_order()`.

### I6. No Escape Hatches

> **The system cannot be bypassed. If a constraint exists, there is no way around it.**

| Principle | Meaning |
|-----------|---------|
| **No backdoors** | If I/O must go through transport, there's no function to call to skip it |
| **No special cases** | "Just this once" exceptions don't exist |
| **Compile-time enforcement** | If something is banned, it won't compile — not just flagged by a linter |

**Example**: `execute_transport()` is not exported from `lib/transport`. You literally cannot call it from outside the crate. The escape hatch doesn't exist.

**Violation**: Adding a `pub` function that provides a "back door" for convenience.

**Why it matters**: Escape hatches accumulate. One leads to ten. The system loses its guarantees.

### I7. No Fallbacks

> **Operations either succeed or fail. There is no silent degradation.**

| Principle | Meaning |
|-----------|---------|
| **No silent defaults** | Don't substitute default values when something is missing — fail |
| **No "best effort"** | Either it worked or it didn't. No partial success without explicit modeling |
| **Fail fast** | Detect and report problems at the earliest possible point |
| **Exceptions are explicit** | If compatibility fallback is required, it must be documented, observable, and covered by tests |

**Example**:
```rust
// BAD: Silent fallback
fn get_config(path: &str) -> Config {
    match read_file(path) {
        Ok(content) => parse(content),
        Err(_) => Config::default()  // Silent degradation!
    }
}

// GOOD: Explicit failure
fn get_config(path: &str) -> Result<Config, Error> {
    let content = read_file(path)?;  // Propagate error
    parse(content)
}
```

**Violation**: Using `.unwrap_or_default()` to hide errors, or `Option<T>` when the value is actually required.

**Why it matters**: Silent degradation hides bugs. You discover the problem far from where it occurred, or worse, never discover it.

### I8. No Warnings

> **Errors are clear signals, not optional advisories. There are no "warnings" that can be ignored.**

| Principle | Meaning |
|-----------|---------|
| **Errors are errors** | If something is wrong, the operation fails — it doesn't print a warning and continue |
| **No "informational" errors** | Don't log "FYI: this might be a problem" — either it's a problem or it isn't |
| **Clear failure modes** | When something fails, the error explains what, where, and ideally why |

**Example**:
```rust
// BAD: Warning that can be ignored
fn validate(input: &Input) -> ValidationResult {
    if input.name.is_empty() {
        eprintln!("Warning: name is empty");  // Just FYI!
    }
    // ... continues anyway
}

// GOOD: Clear error
fn validate(input: &Input) -> Result<ValidInput, ValidationError> {
    if input.name.is_empty() {
        return Err(ValidationError::EmptyName);  // Can't ignore this
    }
    // ... only continues if valid
}
```

**Violation**: Using `println!` or `eprintln!` to report problems instead of returning errors. Using `#[allow(warnings)]` to suppress instead of fix.

**Why it matters**: Warnings train people to ignore output. Eventually the real errors are hidden in noise.

---

## Structural I/O Enforcement

> **Status**: Partially complete. Escape hatch (`execute_transport()`) removed
> from public API. Clippy enforcement active. CI tool fully migrated. Remaining
> tools (gist, deps, buck2, bootstrap) still have hidden I/O in opaque ops.
> See Current State table below.

### The Problem (Solved)

Previously, two ways to do I/O existed:
1. **Correct**: `TransportOps::Execute` nodes (visible in graph, interceptable by DryRun)
2. **Escape hatch**: `execute_transport()` called inside ops (hidden, not interceptable)

### The Fix (Applied)

**Escape hatch removed structurally**: `execute_transport()` is no longer exported from `lib/transport`.

```rust
// lib/transport/src/lib.rs
// BEFORE:
pub use executor::execute_transport;  // Escape hatch!
pub use ops::TransportOps;

// AFTER:
pub use ops::TransportOps;  // Only the DAG node type
```

Now code that tries to call `execute_transport()` won't compile. The only way to do I/O is through `TransportOps::Execute` as a graph node.

### Key Insight: Custom Pure Ops Are Fine

The goal is NOT "decompose everything into primitives" — that creates massive cognitive load.

Custom `execute_*` functions are **fine** as long as they're **pure**:

```rust
// GOOD: Custom pure op (no I/O)
fn execute_parse_build_result(inputs) -> Result<...> {
    let response = inputs.get("response")?;  // Just parsing data
    let success = response.exit_code == 0;
    Ok(outputs)
}

// BAD: Custom op with hidden I/O  
fn execute_scan_workspace(inputs) -> Result<...> {
    let response = execute_transport(&request)?;  // HIDDEN I/O!
    Ok(outputs)
}
```

The invariant is simpler than "primitives only":

> **No `execute_transport()` calls outside `TransportOps::Execute`**

### Current State

| Tool | Status |
|------|--------|
| CI | ✅ Migrated (pure ops + explicit transport nodes) |
| clippy | ✅ Uses UpsertBuilder, SubDag exposed |
| gist | ❌ Hidden I/O in `ListFiles`, `ReadFiles` |
| deps | ❌ Hidden I/O in `LoadManifest`, `ExecuteInstalls` |
| buck2 | ❌ Hidden I/O in `ParseCargoToml` |
| bootstrap | ❌ Hidden I/O in `ScanWorkspace` |
| lib/fs | ❌ Direct `std::fs` side door |

### Migration Order

1. Build transport chain helper (make correct path cheap)
2. Migrate gist (proof of concept)
3. Make `execute_transport()` non-public (close escape hatch)
4. Migrate remaining tools (deps → buck2 → bootstrap)

See `TODO/TODONE/2026-Q1/graph-level-transport.md` for details.

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

### Tools (`lib/tools/`)

General-purpose tool crates live in `lib/tools/` and are reusable across repos.

| Tool | Purpose | Notes |
|------|---------|-------|
| gunbc-clippy | Clippy lint DAGs and config generation | Uses `build_cli_upsert()` |
| gunbc-deps | Tool registry planning and deps.toml generation | Owns deps.toml schema |
| gunbc-gist | GitHub Gist tooling | Multiple bins in one crate |

### Repo-Specific Tools (`gunbc-dag/`)

Repo-specific DAGs and CLI entrypoints live in `gunbc-dag/`.

| Tool | Location | Notes |
|------|----------|-------|
| gunbc-ci | `gunbc-dag/src/ci/` + `gunbc-dag/src/bin/ci.rs` | Repo CI pipeline |
| gunbc-makegen | `gunbc-dag/src/makegen/` + `gunbc-dag/src/bin/makegen.rs` | Makefile + gitignore generation |
| gunbc-codegen | `gunbc-dag/src/codegen/` + `gunbc-dag/src/bin/codegen.rs` | Codegen orchestration |
| gunbc-testgen | `gunbc-dag/src/bin/testgen.rs` | Test generation runner |
| gunbc-bootstrap | `gunbc-dag/src/bootstrap/` + `gunbc-dag/src/bin/bootstrap.rs` | Bootstrap graph |
| gunbc-build | `gunbc-dag/src/build/` + `gunbc-dag/src/bin/build.rs` | Build graph |

Most of these DAGs use `DagBuilder` and `WorkflowSignature` for structural validation.

**Example usage (deps tool):**

```rust
pub fn build_deps_graph() -> Result<Dag<DepsOp>, BuilderError> {
    let mut builder = DagBuilder::new();
    
    let load_manifest = builder.add_root_node(Node::opaque(...))?;
    let generate_scripts = builder.add_node_after(Node::opaque(...), &load_manifest)?;
    
    builder.add_edge(load_manifest.out("manifest_path"), generate_scripts.in_port("manifest_path"))?;
    
    Ok(builder.build())
}

pub fn deps_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        .with_input("manifest_path", "String", Cardinality::ONE)
        .with_output("executed", "Bool", Cardinality::ONE)
        // ... other outputs
}
```

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

**Trusted Kernel Statement:**

> **Lowering is part of the trusted kernel:** it is only defined on validated SubDag nodes whose interface contracts were verified at construction, and it returns a validated DAG by construction.

This means:
- Lowering can be typed as `lower(Dag<T, Validated>) → Dag<T, Validated>`
- "No re-validation needed" is a consequence of types, not a runtime promise
- Future contributors cannot slip in "best-effort lowering" — the type signature enforces it

**Result**: `lower(Dag<T>) → Dag<T>` preserves structural validity. No re-validation needed after lowering.

---

## What's Next

| Gap | Current | Target | Status |
|-----|---------|--------|--------|
| Structural acyclicity | `DagBuilder` with generations | Type-level prevention | ✓ Implemented |
| Structural cardinality | `satisfies()` + `infer_cardinality()` | Type-level `Satisfies<B>` trait | ✓ Implemented (runtime) |
| Workflow signature | `infer_signature()` + `validate()` | Compile-time checked | ✓ Implemented |
| Retry/While/Poll | `RetryBuilder`, `WhileBuilder`, `PollBuilder` | Template + instance semantics | ✓ Implemented |
| Fan-in canonical ordering | `canonical_edge_order()` | Deterministic collection | ✓ Implemented |
| Fan-in/fan-out inference | `edge_count_to_port()`, `fan_in_ports()` | Compiler-inferred | ✓ Implemented |
| Simulate mode | `ExecutionMode::Simulate(SimConfig)` | Full simulation | ✓ Implemented |
| Type DAGs | `Dag<TypeOp>`, `type_lib`, `TypeRegistry` | Types as fractal DAGs | ✓ Implemented |
| Resource conflicts | `detect_conflicts()`, `ResourceAccess` | Structural detection | ✓ Implemented |

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
│   │   │   ├── builder.rs    # DagBuilder
│   │   │   ├── signature.rs  # WorkflowSignature, infer_signature()
│   │   │   ├── boundary.rs   # detect_boundaries()
│   │   │   ├── entrypoint.rs # detect_entrypoints()
│   │   │   ├── value.rs      # Runtime Value enum
│   │   │   ├── type_op.rs    # TypeOp, Predicate, BaseType
│   │   │   ├── type_lib.rs   # Type library helpers
│   │   │   ├── type_registry.rs # TypeRegistry for named types
│   │   │   ├── contract.rs   # Contract tower
│   │   │   ├── resource/     # Resource system and conflict detection
│   │   │   ├── patterns/     # Loop, Branch, Atomic, Transaction, Upsert, Retry, While, Poll
│   │   │   └── transport/    # REST, HTTP, File, TCP, Shell, Tool defs
│   ├── exec/             # Execution, lowering, interception
│   │   ├── src/
│   │   │   ├── execute.rs    # execute(), ExecutionMode (Real, DryRun, Simulate)
│   │   │   ├── lower.rs      # SubDag flattening
│   │   │   ├── topo.rs       # Topological sort
│   │   │   └── intercept.rs  # BoundaryMocks
│   ├── test/             # MockSpec, test infrastructure
│   ├── codegen/          # CLI/entrypoint generation + testgen
│   └── infra/            # Shared infra (hashing, manifests, freshness)
├── lib/
│   ├── primitives/       # ReadFiles, WriteFiles, Parse, etc.
│   ├── transport/        # Transport executor (only direct I/O)
│   ├── tools/            # clippy, deps, gist
│   └── ...               # blob, git-ops, gist-ops, llm-ops, review, markdown
├── gunbc-dag/            # Repo-specific DAGs and tool entrypoints
└── docs/
    └── design/           # This document
```
