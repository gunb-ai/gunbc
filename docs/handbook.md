# gunbc Handbook

This handbook is the practical guide to the gunbc codebase. It explains the core concepts, how the system is structured, and the recurring patterns you should recognize when reading or extending the code.

Companion documents:
- `docs/design/overview.md` for design rationale and formal framing
- `SPEC.md` for the formal IR specification
- `docs/design/testgen.md` for the test generation model
- `AGENT.md` for onboarding guardrails and repo pointers

## Doc Map

| Doc | Focus | Use When |
| --- | --- | --- |
| `docs/handbook.md` | Practical overview + pattern catalog | You need the conceptual map and concrete examples |
| `docs/design/overview.md` | Philosophy + formal model | You want design rationale and invariants |
| `SPEC.md` | Formal IR spec | You need canonical definitions |
| `docs/design/testgen.md` | Test generation theory | You are touching testgen or proof obligations |
| `AGENT.md` | Onboarding + guardrails | You are new to the repo or doing refactors |

## Mental Model

gunbc is a Rust-based workflow IR where **everything is a DAG**.

Core claims:
- Structural soundness means: acyclic, type-compatible, cardinality-compatible, and sub-DAG interfaces match.
- Nodes are pure transformations. World I/O only happens at explicit transport execution nodes.
- If it validates, it is structurally sound.

## Core IR: Dag, Node, Port, Edge

Canonical types live in `core/ir/src/dag.rs`, `core/ir/src/node.rs`, and `core/ir/src/types.rs`.

```rust
use gunbc_ir::{Dag, Node, Port, Edge, NodeBody};

struct MyOp;

let mut dag: Dag<MyOp> = Dag::new();

dag.add_node(Node::opaque(
    "fetch",
    vec![Port::new("url", "String")],
    vec![Port::new("body", "String")],
    MyOp,
));

dag.add_node(Node::opaque(
    "parse",
    vec![Port::new("body", "String")],
    vec![Port::new("result", "Json")],
    MyOp,
));

dag.add_edge(Edge::new("fetch", "body", "parse", "body"));
```

Key points:
- `NodeBody::Opaque(T)` means the op is trusted and not inspected.
- `NodeBody::SubDag(Dag<T>)` enables fractal composition.
- `Port` includes `type_id` and `cardinality` as first-class structure.

## Cardinality

Cardinality is the canonical expression of optionality and shape. It lives in `core/ir/src/types.rs`.

| Cardinality | Meaning |
| --- | --- |
| `Zero` | Signal only, no values |
| `One` | Exactly one value |
| `ZeroOrOne` | Optional |
| `ZeroOrMore` | List, possibly empty |
| `OneOrMore` | Non-empty list |

Partial order (not a lattice):

```text
            ZeroOrMore (top)
            /        \
       ZeroOrOne    OneOrMore
          |    \       /
        Zero    One
```

## Fractal DAGs and Patterns

A node can contain a sub-DAG. `Node::subdag(...)` infers its interface from the inner DAG’s entrypoints and boundaries. This is the basis of the “fractal DAG” pattern.

Primary pattern library: `core/ir/src/patterns/`.

## Boundaries and Entrypoints

Boundaries and entrypoints are inferred structurally:
- Boundary outputs are **unconnected output ports**.
- Entrypoint inputs are **unconnected input ports**.

See `core/ir/src/boundary.rs` and `core/ir/src/entrypoint.rs`.

ASCII view:

```text
world -> [A] -> [B] -> world
   ^        ^      ^
   |        |      |
entrypoint  edge   boundary
```

Boundaries are about interface, not I/O. Actual I/O happens only at transport execution nodes.

## Workflow Signatures

Workflow signatures prevent silent interface drift. See `core/ir/src/signature.rs`.

```rust
use gunbc_ir::{WorkflowSignature, SignaturePort};
use gunbc_ir::types::Cardinality;

let signature = WorkflowSignature::new()
    .with_input("url", "String", Cardinality::ONE)
    .with_output("result", "Json", Cardinality::ONE);
```

Signature inference excludes tool ports (`tool:*`) since those are framework-provided.

## Execution Model

Execution lives in `core/exec/`. Key file: `core/exec/src/execute.rs`.

Modes:
- `Real` executes normally.
- `DryRun` intercepts I/O and resource acquisition.
- `Simulate` runs with timing/resource modeling.

DryRun intercepts these node types:
- Transport executors with an input port of type `TransportRequest`.
- Environment nodes that emit `ToolHandle`, `FilesystemHandle`, `Timestamp`, `AuthToken`, or `Platform`.
- Tool consumer nodes that take `ToolHandle` as an input.

Intercepted nodes require explicit mocks for all outputs. There are no silent defaults.

## Transport System

Transport requests/responses are defined in `core/ir/src/transport/mod.rs`. Actual I/O is performed only by `TransportOps::Execute` in `lib/transport`.

Key invariants:
- Pure ops **prepare** `TransportRequest` values.
- Only `TransportOps::Execute` performs I/O.
- `lib/transport` is the only crate (besides codegen) that uses direct I/O.

See `lib/transport/src/lib.rs` and `lib/transport/src/ops.rs`.

## Resource Model

Resources are typed values flowing through edges. Core types live in `core/ir/src/resource/`.

Highlights:
- `AccessMode` controls conflict detection (Read, Write, Exclusive).
- Capabilities are marked with a secret capability marker to prevent forgery.
- `detect_conflicts()` identifies unordered access to the same resource.

## Tooling: Two Complementary Systems

### Runtime tool acquisition (CliToolDef)

Runtime acquisition uses `CliToolDef`, `CliToolOp`, and `ToolHandle` in `core/ir/src/transport/cli.rs`.

Pattern:
1. Define the tool as a `CliToolDef`.
2. Use `build_cli_upsert()` or `build_cli_ensure()`.
3. When using capability-based access, `ToolHandle` values flow through `tool:<id>` ports and are excluded from the user-facing signature.

Example: `lib/tools/clippy/src/graph.rs` uses `build_cli_upsert()`.

`CliToolOp` can execute directly as well. `ToolHandle` is the capability form when you want the executor to manage acquisition and pass a resolved tool path into ops.

### Planning and satisfiability (ToolDef)

Platform-aware install planning uses `ToolDef` in `core/ir/src/transport/tool.rs`.

Example: `lib/tools/deps` uses the tool registry to generate `deps.toml`.

## Pattern Catalog

### Upsert Pattern

Intent: Idempotent acquisition or generation: Check -> Create -> Resolve.

Structure:

```text
check -> create -> resolve
```

Key files:
- `core/ir/src/patterns/upsert.rs`
- `core/ir/src/transport/cli.rs`
- `lib/tools/clippy/src/graph.rs`

### Fractal Sub-DAG Pattern

Intent: Build reusable subgraphs as single nodes.

Key files:
- `core/ir/src/node.rs` (`Node::subdag`)
- `lib/tools/clippy/src/graph.rs`
- `gunbc-dag/src/workspace/`

### Boundary Inference

Intent: Interfaces are inferred structurally from unconnected ports.

Key files:
- `core/ir/src/boundary.rs`
- `core/ir/src/entrypoint.rs`
- `core/ir/src/signature.rs`

### Transport Boundary

Intent: All I/O happens at explicit `TransportOps::Execute` nodes.

Key files:
- `lib/transport/src/lib.rs`
- `lib/transport/src/ops.rs`
- `core/exec/src/execute.rs`

### Tool Planning and deps.toml Generation

Intent: Generate install plans and `deps.toml` from a tool registry.

Key files:
- `core/ir/src/transport/tool.rs`
- `lib/tools/deps/src/tool_upsert.rs`
- `lib/tools/deps/src/graph.rs`

### Resource Conflict Detection

Intent: Detect unordered access to the same resource.

Key files:
- `core/ir/src/resource/mod.rs`

## Concrete Examples

### Example 1: Minimal DAG + Signature

```rust
use gunbc_ir::{Dag, Node, Port, Edge, WorkflowSignature};
use gunbc_ir::types::Cardinality;

let mut dag: Dag<()> = Dag::new();

dag.add_node(Node::opaque(
    "fetch",
    vec![Port::new("url", "String")],
    vec![Port::new("body", "String")],
    (),
));

dag.add_node(Node::opaque(
    "parse",
    vec![Port::new("body", "String")],
    vec![Port::new("result", "Json")],
    (),
));

dag.add_edge(Edge::new("fetch", "body", "parse", "body"));

let sig = WorkflowSignature::new()
    .with_input("url", "String", Cardinality::ONE)
    .with_output("result", "Json", Cardinality::ONE);
```

### Example 2: Clippy Upsert Sub-DAG

```rust
use gunbc_clippy::build_clippy_lint_all;

let clippy_node = build_clippy_lint_all();
```

See `lib/tools/clippy/src/graph.rs`.

### Example 3: deps.toml Generation Graph

High-level flow in `lib/tools/deps/src/graph.rs`:

```text
LoadToolRegistry -> RenderDepsToml -> PrepareFileWrite -> TransportOps::Execute
```

## Repo Map

| Path | Purpose |
| --- | --- |
| `core/ir/` | Core IR types, patterns, transport model, resource system |
| `core/exec/` | Execution engine, DryRun interception, simulation |
| `core/codegen/` | CLI and test generation | 
| `core/test/` | MockSpec and test utilities |
| `lib/transport/` | The only crate that performs direct I/O |
| `lib/tools/` | General-purpose tool wrappers (clippy, deps, gist) |
| `gunbc-dag/` | Repo-specific DAGs and CLI entrypoints (ci, makegen, codegen, testgen, bootstrap) |
| `docs/design/` | Design documentation |

## Glossary

| Term | Meaning |
| --- | --- |
| Boundary | Unconnected output port (DAG -> world) |
| Entrypoint | Unconnected input port (world -> DAG) |
| Sub-DAG | A node whose body is a DAG |
| Transport executor | Node that executes a `TransportRequest` |
| ToolHandle | Capability for a CLI tool, passed via `tool:<id>` port |
| Resource | Typed value with an access mode (read/write/exclusive) |

## How to Extend

Common tasks and the primary starting points:
- Add a new pattern: `core/ir/src/patterns/` and `core/ir/src/patterns/mod.rs`.
- Add a new CLI tool: `core/ir/src/transport/cli.rs`, plus a wrapper crate under `lib/tools/` if needed.
- Add a new ToolDef for planning: `core/ir/src/transport/tool.rs` and `lib/tools/deps/` for deps.toml generation.
- Add a new repo-specific tool: `gunbc-dag/src/` plus a bin in `gunbc-dag/src/bin/`.
- Add a new transport: `core/ir/src/transport/` plus executor support in `lib/transport/`.
