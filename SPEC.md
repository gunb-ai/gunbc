# gunbc — Specification

**Status**: Draft — February 2026
**Companion docs**: `docs/handbook.md`, `docs/design/overview.md`
**Design rationale**: [`the-gunbai/docs/design/v2/v3-contracts-minimal.md`](https://github.com/gunb-ai/the-gunbai/blob/main/docs/design/v2/v3-contracts-minimal.md)
**Authoring spec**: [`the-gunbai/docs/design/v2/v2-contracts-design.md`](https://github.com/gunb-ai/the-gunbai/blob/main/docs/design/v2/v2-contracts-design.md)
**Formal inspiration**: [`ac.pdf`](./docs/ac.pdf) — Abstraction Calculus

---

## 0. Relationship to V2

V2 is the author-facing type system — the language tool authors write
against (patterns, typed semantics, verification strategies, lane
discipline). This spec (gunbc) is the canonical internal IR: all V2
constructs lower into Node/Dag. gunbc is not an alternative product
direction; it is the normalization target.

```
V2 (authoring)  →  gunbc IR (this spec)  →  flat executable DAG
   patterns,           Node/Dag/Edge,           all Opaque,
   typed semantics,    ports + cardinality,      no SubDags,
   lane discipline     type registry             topo-sorted
```

V2 concepts that don't appear in the IR are authoring conveniences —
they exist to make tool definitions readable and to enforce discipline
at write time, but they compile away during lowering.

---

## Refactor-Pressure Checklist (PR Gate)

See `AGENT.md` for the canonical checklist. Summary: single source of truth,
no stringly references, no hidden env/IO, no ambient globals, fast paths
declared, generated code linting fixes go to IR or clippy config.

---

## 1. Core Types

### 1.1 Node

```rust
struct Node<T> {
    id: NodeId,
    inputs: Vec<Port>,
    outputs: Vec<Port>,
    body: NodeBody<T>,
}

enum NodeBody<T> {
    Opaque(T),
    SubDag(Dag<T>),
}
```

A node is the sole unit of the system. It has typed input ports, typed
output ports, and a body that is either opaque (trusted, not inspected)
or a sub-DAG (recursive, same structure inside).

**On the type parameter `T`.** The structural claim is: one graph shape
(nodes, ports, edges, opaque vs sub-DAG) at every level. The payload
`T` is not literally the same type at every depth — a tool-level node
carries `ToolImpl`, an operation carries `OpImpl`, a leaf carries
`Executable`. In practice, `T` is either a sum type
(`enum Payload { Tool(..), Op(..), Exec(..) }`) or an erased trait
object. The structural uniformity is in `Node`/`Dag`/`Edge`/`Port` —
not in `T`. Payloads are opaque at level boundaries by definition:
the consumer sees ports, not `T`.

### 1.2 DAG

```rust
struct Dag<T> {
    nodes: Vec<Node<T>>,
    edges: Vec<Edge>,
}
```

A DAG is a collection of nodes connected by typed edges. Acyclicity is
a construction invariant — not checked at runtime, enforced by the
builder.

### 1.3 Port

```rust
struct Port {
    name: PortName,
    type_id: TypeId,
    cardinality: Cardinality,
}

enum Cardinality {
    Zero,        // Signal only (Unit type)
    One,         // Exactly one value (required)
    ZeroOrOne,   // Optional
    ZeroOrMore,  // List (may be empty)
    OneOrMore,   // Non-empty list
}
```

A port is a named, typed connection point on a node. Every port has a
cardinality that describes how many values can flow through it.
Cardinality is the canonical way to express optionality — use
`ZeroOrOne` for optional values, not external annotations.

### 1.4 Edge

```rust
struct Edge {
    from: (NodeId, PortName),
    to: (NodeId, PortName),
}
```

An edge connects an output port of one node to an input port of another.
The types of the connected ports must match. There is no other kind of
dependency — if it's not an edge, it doesn't exist.

### 1.5 Conditional Execution

Conditional execution is modeled through **explicit DAG structure**, not
through annotations on ports:

- **Branch pattern**: Routes input to one of two sub-DAGs based on a
  boolean condition. Only one branch executes.
- **Optional cardinality**: Use `ZeroOrOne` to express that a value may
  be absent. Downstream nodes must handle optionality explicitly.

There are no "guards" as user-facing port annotations. Any conditional
routing is expressed as explicit graph structure (Branch nodes) that the
type system can validate.

```
┌─────────────────────────────────────────────┐
│                   Branch                     │
│           ┌───────────┐                     │
│   ┌──────▶│ True DAG  │──────┐             │
│   │       └───────────┘      │             │
│ condition                    ▼             │
│   │                      ┌───────┐         │
│   │       ┌───────────┐  │ Merge │─▶ output│
│   └──────▶│ False DAG │──┘       │         │
│           └───────────┘          │         │
└─────────────────────────────────────────────┘
```

### 1.6 Identifiers

```rust
struct NodeId(String);    // hierarchical: "tool/zstd/check/which"
struct PortName(String);  // local to a node: "state", "binary"
struct TypeId(String);    // registered type: "ResourceState", "ResolvedHandle"
```

`NodeId` is hierarchical, reflecting the nesting structure. When a
sub-DAG node `check` lives inside node `tool/zstd`, its full ID is
`tool/zstd/check`. Port names are local to a node. Type IDs refer to
a global type registry.

---

## 2. Invariants

These must hold for any well-formed `Dag<T>`:

### 2.1 Acyclicity
The edge set forms a directed acyclic graph. No node can transitively
depend on itself.

### 2.2 Type agreement
For every edge `(a.port_x, b.port_y)`, the `type_id` of `a.port_x`
equals the `type_id` of `b.port_y`.

### 2.3 Port saturation
Every input port of every node is connected to exactly one edge.
No dangling inputs. (Output ports may be unconnected — unused outputs
are allowed.)

### 2.4 Sub-DAG interface agreement
If a node has body `SubDag(dag)`, then:
- The node's input ports correspond to source nodes (no incoming edges)
  in the sub-DAG.
- The node's output ports correspond to sink nodes (no outgoing edges)
  in the sub-DAG.
- Types match at the boundary.

### 2.5 Cardinality honesty
If a port declares `cardinality: One`, it must always produce exactly
one value. If a value may be absent, the port must declare
`cardinality: ZeroOrOne`. The type system enforces this — a `One` output
cannot connect to a `One` input through a conditional path without an
explicit merge that guarantees exactly one value.

### 2.6 Explicit opt-out
If a pattern exists in the registry, every tool must either instantiate
it (binding all required slots) or declare `NotApplicable` with a
reason. Silence — neither instantiating nor opting out — is a
validation error. This prevents the old failure mode where missing
structure was indistinguishable from "not modeled yet."

```rust
enum PatternDecision<T> {
    Instantiated(Dag<T>),       // tool implements this pattern
    NotApplicable { reason: &'static str },  // explicit opt-out
}
```

### 2.7 No side channels
An opaque node's observable effects on the graph are limited to its
declared output ports. Any communication between nodes that bypasses
edges is a contract violation. (Enforcement is external to the spec —
sandboxing, I/O manifests, or runtime detection.)

---

## 3. Patterns

A pattern is a `Dag<Slot>` — a DAG where node bodies are `Opaque(Slot)`
placeholders instead of concrete implementations.

```rust
struct Slot {
    role: SlotRole,       // e.g., Check, Create, Resolve
    required: bool,
}
```

A tool instantiates a pattern by binding each slot to a concrete
`NodeBody<T>`. Instantiation produces a `Dag<T>`. The pattern defines
the shape (nodes, edges, types, guards). The tool provides the
implementations.

### 3.1 Upsert (the first pattern)

```
Slots:  Check, Create, Resolve
Edges:  Check.state → Create.state (guarded: Missing)
        Check.state → Resolve.state
        Create.ref  → Resolve.ref
Types:  Check outputs ResourceState
        Create outputs GuardedOutput<ResourceRef>
        Resolve outputs ResolvedHandle
```

A pattern is valid when all required slots are bound and all invariants
from §2 hold on the resulting DAG.

### 3.2 Pattern promotion

A pattern begins as a single tool's sub-DAG shape. When 2+ tools share
the same shape, it is promoted to a named pattern. One tool never
justifies a named pattern — that's speculative abstraction.

---

## 4. Type Registry

The system maintains a registry of known types. Types are organized
into lanes:

| Lane | Meaning | Example |
|---|---|---|
| A | Core — used by 2+ patterns or tools | `ResourceState`, `ResolvedHandle` |
| B | Declared — used by one tool, typed | Tool-specific enums |
| C | Documentation — not consumed by any machine | Free-text descriptions |

Lane B types may be promoted to Lane A when a second consumer appears.
Lane C values are never consumed programmatically.

---

## 5. Compiler

The compiler is a recursive traversal:

### 5.1 Validate

Walk the DAG. For every node:
- If `SubDag`: recursively validate the inner DAG. Check all invariants
  from §2.
- If `Opaque`: check that declared ports have registered types.

Validation is complete when every node at every depth satisfies §2.

### 5.2 Lower

Recursively flatten `SubDag` bodies into `Opaque` leaves:

```
lower(dag: Dag<T>) -> Dag<T>:
    for each node in dag:
        if node.body is SubDag(inner):
            lowered_inner = lower(inner)
            inline lowered_inner's nodes and edges into dag
            rewire: parent's input edges → inner entrypoint nodes
            rewire: inner boundary nodes → parent's output edges
            remove the parent node (replaced by its contents)
    return dag (all nodes are now Opaque)
```

The result is `Dag<T>` where every node is `Opaque` — a flat execution
graph. The fractal structure exists only before lowering.

**Lowering is part of the trusted kernel**: it is only defined on
validated SubDag nodes whose interface contracts were verified at
construction, and it returns a validated DAG by construction. No
re-validation is needed after lowering.

### 5.3 Execute

The executor receives a flat `Dag<T>` (all `Opaque`). It:
1. Topologically sorts nodes.
2. Executes in dependency order (parallelizing independent nodes).
3. Runs each node when its inputs are available.

The executor has no knowledge of patterns, sub-DAGs, or levels. It sees
nodes and edges. Conditional execution is already encoded in the graph
structure (Branch patterns) — the executor simply follows data flow.

---

## 6. Node Contract

For the system's guarantees to hold, opaque nodes must satisfy:

1. **Port honesty**: All inputs arrive through declared input ports.
   All outputs leave through declared output ports.
2. **No side channels**: No communication with other nodes except
   through the graph's edges.
3. **Determinism** (recommended, not required): Given the same inputs,
   produce the same outputs. Non-deterministic nodes must declare it.

Enforcement is outside this spec. Options include sandboxed execution,
declared I/O manifests, and runtime detection of undeclared access.

---

## 7. Extension Points

- **New types**: Declare as Lane B. Promote to Lane A at 2+ consumers.
- **New patterns**: Extract from a single tool's sub-DAG. Promote to
  named pattern at 2+ tools.
- **New levels**: Open any opaque node into a sub-DAG. Same types,
  same invariants. No framework changes.
- **New conditional patterns**: Express as explicit DAG structure
  (like Branch) that the type system can validate.
- **Dynamic subgraphs**: A node may produce a `Dag<T>` as output,
  which the executor inlines and runs. This loses static analysis
  for that subgraph. Use sparingly.
