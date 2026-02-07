# gunbc Handbook

This handbook is the practical guide to the gunbc codebase. It explains the core concepts, how the system is structured, and the recurring patterns you should recognize when reading or extending the code.

Companion documents:
- `docs/design/overview.md` for design rationale and formal framing
- `SPEC.md` for the formal IR specification
- `docs/design/testgen.md` for the test generation model
- `docs/design/unified-emission.md` for the rendering unification plan
- `docs/design/unified-registration.md` for the registration unification plan
- `AGENT.md` for onboarding guardrails and repo pointers

This file is self-contained: [Appendix A](#appendix-a-pattern-reference) has every pattern reference, [Appendix B](#appendix-b-end-to-end-examples) has full pipeline walkthroughs.

## Doc Map

| Doc | Focus | Use When |
| --- | --- | --- |
| `docs/handbook.md` | Practical overview, pattern catalog, e2e examples | You need the conceptual map, pattern details, or concrete examples |
| `docs/design/overview.md` | Philosophy + formal model | You want design rationale and invariants |
| `SPEC.md` | Formal IR spec | You need canonical definitions |
| `docs/design/testgen.md` | Test generation theory | You are touching testgen or proof obligations |
| `docs/design/unified-emission.md` | Rendering unification | You are touching rendering or codegen |
| `docs/design/unified-registration.md` | Registration unification | You are adding tools or auto-discovery |
| `AGENT.md` | Onboarding + guardrails | You are new to the repo or doing refactors |

## Mental Model

gunbc is a Rust-based workflow IR where **everything is a DAG**.

Core claims:
- Structural soundness means: acyclic, type-compatible, cardinality-compatible, and sub-DAG interfaces match.
- Nodes are pure transformations. In the runtime DAG, world I/O only happens at explicit transport execution nodes.
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

Boundaries are about interface, not I/O. In the runtime DAG, actual I/O happens only at transport execution nodes.

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
- Environment nodes that emit `ToolHandle`, `FilesystemHandle`, `Timestamp`, `Credential`, or `Platform`.
- Tool consumer nodes that take `ToolHandle` as an input.

Intercepted nodes require explicit mocks for all outputs. There are no silent defaults.

## Transport System

Transport requests/responses are defined in `core/ir/src/transport/mod.rs`. Runtime DAG I/O is performed only by `TransportOps::Execute` in `lib/transport`. Direct I/O outside the DAG is limited to explicit bootstrap/generator/tooling boundaries (see `TODO/TODONE/clippy-pragma-audit.md`).

Key invariants:
- Pure ops **prepare** `TransportRequest` values.
- Only `TransportOps::Execute` performs runtime DAG I/O.
- Direct I/O is limited to `lib/transport` plus explicit exceptions (codegen/testgen binaries, CLI tool layer, deps manifest/installer; see `TODO/TODONE/clippy-pragma-audit.md`).

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

Direct execution lives in the transport layer (see `lib/transport/src/cli.rs`) so tool I/O stays out of pure crates. `ToolHandle` is the capability form when you want the executor to manage acquisition and pass a resolved tool path into ops.

### Planning and satisfiability (ToolDef)

Platform-aware install planning uses `ToolDef` in `core/ir/src/transport/tool.rs`.

Example: `lib/tools/deps` uses the tool registry to generate `deps.toml`.

## Pattern Catalog

Quick reference of all patterns. Full details in [Appendix A](#appendix-a-pattern-reference).

### DAG Patterns (structural)

| Pattern | Intent | Key File |
|---------|--------|----------|
| Multi-Phase: Upsert | Check → Create → Resolve (idempotent) | `core/ir/src/patterns/upsert.rs` |
| Multi-Phase: Transaction | Begin → Body → Commit/Rollback | `core/ir/src/patterns/transaction.rs` |
| Multi-Phase: Atomic | Precondition → Op → Postcondition | `core/ir/src/patterns/atomic.rs` |
| Multi-Phase: Content Upsert | Render → Read → Compare → Write (skippable) | `gunbc-dag/src/makegen/graph.rs` |
| Control Flow: Branch | Conditional execution with merge | `core/ir/src/patterns/branch.rs` |
| Control Flow: Loop | Iteration over collections | `core/ir/src/patterns/loop_pattern.rs` |
| Control Flow: Repeat | Retry, While, Poll | `core/ir/src/patterns/repeat.rs` |
| Fractal Sub-DAG | Reusable subgraphs as nodes | `core/ir/src/node.rs` |

### System Patterns (cross-cutting)

| Pattern | Intent | Key File |
|---------|--------|----------|
| Transport Boundary | All I/O through request/response | `lib/transport/src/ops.rs` |
| Registration | Auto-discovery of registrable units | `core/testgen-registry/` |
| Emission | IR → Renderer → Output | `core/codegen/src/testgen/` |
| Resource Acquisition | Typed resources with conflict detection | `core/ir/src/resource/` |
| Credential Lifecycle | Provider → acquire → Credential | `lib/transport/src/credential.rs` |
| Mock Specification | Declarative test fixtures | `core/test/src/mock_spec.rs` |
| Content Hashing | Deterministic content-addressed hashing | `core/infra/src/hash.rs` |
| Freshness Check | Mtime fast-path before full hash | `core/infra/src/freshness.rs` |

## Repo Map

| Path | Purpose |
| --- | --- |
| `core/ir/` | Core IR types, patterns, transport model, resource system |
| `core/exec/` | Execution engine, DryRun interception, simulation |
| `core/codegen/` | CLI and test generation | 
| `core/test/` | MockSpec and test utilities |
| `lib/transport/` | Canonical runtime I/O boundary; a few bootstrap/generator/tooling crates do direct I/O by exception (see `TODO/TODONE/clippy-pragma-audit.md`) |
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

---

# Appendix A: Pattern Reference

Complete reference for every pattern in the codebase.

---

## A.1 Fractal Sub-DAG

**Intent:** Build reusable subgraphs as single nodes. A node's body can be an entire DAG, and its interface is inferred from the inner DAG's unconnected ports.

```
Outer DAG:
  [A] ──▶ [SubDag(inner)] ──▶ [B]

Inner DAG (inside SubDag node):
  [X] ──▶ [Y] ──▶ [Z]
  ^entrypoint      ^boundary
```

The SubDag node's inputs = inner entrypoints, outputs = inner boundaries.

```rust
// Create a SubDag node — interface inferred from inner DAG
let node = Node::subdag("my_subdag", inner_dag);
// node.inputs = inner_dag's entrypoint ports
// node.outputs = inner_dag's boundary ports
```

| File | Role |
|------|------|
| `core/ir/src/node.rs` | `Node::subdag()` with auto-inference |
| `core/ir/src/boundary.rs` | Boundary detection (unconnected outputs) |
| `core/ir/src/entrypoint.rs` | Entrypoint detection (unconnected inputs) |
| `core/exec/src/lower.rs` | Lowering flattens SubDags for execution |

**Design decisions:**
- **Interface inference**: `Node::subdag()` scans the inner DAG for unconnected ports. No manual interface declaration needed.
- **Resource access preservation**: SubDag auto-inference preserves `resource_access` from inner ports. Resource accounting composes fractally.
- **Lowering flattens**: The executor lowers SubDags into a flat node list for execution. SubDags are a structural abstraction, not a runtime concept.

All pattern builders (Upsert, Transaction, Branch, Loop, etc.) produce SubDag nodes. The pattern library is built on fractal composition.

---

## A.2 Multi-Phase Patterns (Upsert, Transaction, Atomic, Content Upsert)

These patterns are variants of **guarded multi-phase operations**. Upsert, Transaction, and Atomic are packaged as SubDags with explicit guards; Content Upsert is typically a 6-node chain wired directly in the outer DAG (and can be wrapped in a SubDag if you want a single reusable node).

### A.2.1 Upsert

**Intent:** Idempotent acquisition: if it exists, use it; if not, create it; then verify.

```
check ──[exists?]──▶ create ──▶ resolve
          │ guard: !exists        │
          └───────────────────────┘
```

Three nodes in a SubDag:
- **check**: Pure read to test existence (outputs a boolean/status)
- **create**: Guarded by `!check` — only runs if check fails
- **resolve**: Runs unconditionally — verifies the result is usable

```rust
UpsertBuilder::new("tool_install")
    .with_check(CheckOp)       // Pure: does it exist?
    .with_create(CreateOp)     // Guarded: create if not
    .with_resolve(ResolveOp)   // Pure: verify + return handle
    .build()                   // → Node<T> with SubDag body
```

| File | Role |
|------|------|
| `core/ir/src/patterns/upsert.rs` | Pattern builder |
| `core/ir/src/transport/cli.rs` | `build_cli_upsert()` — tool installation |
| `lib/tools/clippy/src/graph.rs` | Clippy tool acquisition |

**Design decisions:**
- Guard on create, not resolve — resolve always runs because the tool might exist but be in a bad state.
- SubDag, not three loose nodes — consumers see one node with tool handle outputs.
- Idempotency by construction — running the pattern twice is safe.

### A.2.2 Transaction

**Intent:** Multi-step operation with rollback on failure: Begin → Body → Commit/Rollback.

```
[begin] ──▶ [body] ──▶ [commit]
                │
                └──▶ [rollback]  (on failure)
```

| File | Role |
|------|------|
| `core/ir/src/patterns/transaction.rs` | `TransactionBuilder` |

Used when an operation must be atomic — either all steps succeed or the system rolls back. The codegen commit/rollback flow is the canonical example.

### A.2.3 Atomic

**Intent:** Operation with precondition and postcondition checks: validate before, execute, verify after.

```
[precondition] ──▶ [operation] ──▶ [postcondition]
    (pure)          (may do I/O)      (pure)
```

| File | Role |
|------|------|
| `core/ir/src/patterns/atomic.rs` | `AtomicBuilder` |

**Design decisions:**
- Preconditions are pure and fast — they validate state before committing to an expensive operation.
- Postconditions verify the result, not just success. They check that the world state matches expectations after the operation.

### A.2.4 Content Upsert

**Intent:** Idempotent file generation — render expected content, read current file, compare, skip write if fresh.

```
generate ─┬─→ prepare_read → execute_read ─→ compare ─┬─→ execute_write
          └─→ prepare_write ──────────────────────────┘
```

Six nodes in the chain:
- **generate**: Pure renderer that produces expected content (String).
- **prepare_read**: Builds a read request from `path` (pure).
- **execute_read**: Transport boundary read (`TransportOps::Execute`).
- **compare**: `BlobOps::CompareContent` — compares expected vs actual content (pure).
- **prepare_write**: Builds a write request from `path` + content (pure).
- **execute_write**: Transport boundary write, **skippable** via compare outputs.

**Port contract:**
- Generator content wires to `compare.expected_content` and `prepare_write.content`.
- `compare` takes `response` from execute_read and optional `check_mode`.
- `compare` outputs `fresh`, `skip`, `skip_reason`.
- `execute_write` must accept `skip` and optional `skip_reason` so it can no-op when fresh.

| File | Role |
|------|------|
| `lib/blob/src/lib.rs` | `BlobOps::CompareContent` semantics (fresh/skip/skip_reason) |
| `gunbc-dag/src/makegen/graph.rs` | Single-chain reference implementation |
| `gunbc-dag/src/testgen_dag/graph.rs` | Dynamic N chains (shared helper) |

**Design decisions:**
- Comparison is pure; all I/O stays in the transport read/write nodes.
- `check_mode` forces `skip = true` even if content differs (used for check-only runs).
- `fresh` is typically left as a boundary output for reporting.

**Relationship to A.11 (Freshness):** A.11 is the infra-level mtime fast path. Content upsert is the graph-level pattern that compares expected vs actual content (hashing when possible) once you decide a file might be stale.

**Relationship to A.2.1 (Upsert):** Upsert acquires resources (check → create → resolve). Content upsert generates files (render → read → compare → write). Different intent, different shape.

**Helper API:** Use `add_content_upsert_chain` in `core/ir/src/patterns/content_upsert.rs` to stamp out the 6-node pattern with standard wiring.

**Examples:**
- `gunbc-dag/src/testgen_dag/graph.rs` — Dynamic N chains
- `gunbc-dag/src/pragma/graph.rs` — 3 static parallel chains
- `gunbc-dag/src/bootstrap/graph.rs` — 2 parallel chains after scan
- `gunbc-dag/src/makegen/graph.rs` — single chain

### Relationship between the patterns

| Variant | Guard strategy | Failure handling |
|---------|---------------|------------------|
| Upsert | Guard on create (skip if exists) | No rollback — resolve verifies |
| Transaction | Guard on commit (rollback on failure) | Explicit rollback branch |
| Atomic | Guard on operation (precondition must pass) | No rollback — postcondition verifies |
| Content Upsert | Guard on write (skip if fresh or check_mode) | No rollback — compare outputs freshness + reason |

**Known issue:** The emission pattern (Prepare → Format → Write) is structurally an upsert but doesn't use UpsertBuilder.

---

## A.3 Control Flow Patterns (Branch, Loop, Repeat)

These patterns model execution flow as SubDags with guards — not as control-flow nodes. This keeps the DAG acyclic while supporting conditionals, iteration, and repetition.

### A.3.1 Branch

**Intent:** Conditional execution: evaluate a predicate, then run one of two sub-DAGs, then merge results.

```
[predicate] ──true──▶  [then_branch]  ──▶ [merge]
            └─false──▶ [else_branch]  ──▶
```

| File | Role |
|------|------|
| `core/ir/src/patterns/branch.rs` | `BranchBuilder` |

**Design decisions:**
- Guards are data dependencies, not control jumps — keeps the DAG acyclic.
- The merge node receives outputs from whichever branch ran. Ports from the non-taken branch have `Skipped` values.

### A.3.2 Loop

**Intent:** Iterate over a collection, applying a sub-DAG to each element.

```
[input: List<T>] ──▶ [loop_body: SubDag(T → U)] ──▶ [output: List<U>]
```

| File | Role |
|------|------|
| `core/ir/src/patterns/loop_pattern.rs` | `LoopBuilder` |

**Design decisions:**
- Loop is a SubDag whose body processes one element. The executor handles iteration and result collection.
- Cardinality is preserved: `List<T>` in → `List<U>` out.

### A.3.3 Repeat

**Intent:** Execute a sub-DAG repeatedly until a condition is met. Three variants.

```
Retry:  [body] ──fail──▶ [retry] ──▶ [body]  (up to N times)
While:  [check] ──true──▶ [body] ──▶ [check]  (while condition holds)
Poll:   [check] ──false──▶ [wait] ──▶ [check]  (until condition holds)
```

| File | Role |
|------|------|
| `core/ir/src/patterns/repeat.rs` | `RetryBuilder`, `WhileBuilder`, `PollBuilder` |

**Design decisions:**
- Bounded by default: Retry has max attempts, While/Poll have max iterations. Unbounded loops are not expressible (prevents infinite execution).
- Modeled as SubDag with back-edges that the executor handles specially.

---

## A.4 Transport Boundary

**Intent:** Runtime DAG world I/O happens at explicit transport execution nodes. Pure nodes prepare requests and parse responses. Direct I/O outside the DAG is limited to explicit bootstrap/generator/tooling exceptions (see `TODO/TODONE/clippy-pragma-audit.md`).

```
[prepare]  ──▶  [execute]  ──▶  [parse]
  (pure)       (boundary)       (pure)
```

- **prepare**: Builds a `TransportRequest` from domain data (pure transformation)
- **execute**: Performs I/O via `TransportOps::Execute` (the runtime I/O node)
- **parse**: Extracts domain data from `TransportResponse` (pure transformation)

```rust
pub enum TransportRequest {
    Rest(RestRequest), Http(HttpRequest), File(FileRequest),
    Tcp(TcpRequest), Shell(ShellRequest),
}

pub enum TransportResponse {
    Rest(RestResponse), Http(HttpResponse), File(FileResponse),
    Tcp(TcpResponse), Shell(ShellResponse),
}
```

| File | Role |
|------|------|
| `core/ir/src/transport/mod.rs` | Request/Response enums |
| `lib/transport/src/ops.rs` | `TransportOps` — canonical runtime I/O boundary |
| `lib/transport/src/executor.rs` | Actual HTTP, file, shell execution |
| `core/exec/src/intercept.rs` | DryRun interception of transport nodes |

**Enforcement:**
- `clippy.toml` disallows `std::fs` and `std::process` in all crates except explicit allowlist entries; crate-level exemptions are `lib/transport` and `core/codegen` (see `TODO/TODONE/clippy-pragma-audit.md`).
- DryRun mode intercepts all nodes whose inputs include `TransportRequest`.

**Design decisions:**
- Enum union, not trait objects — transport types are a closed set with exhaustive dispatch.
- Interception at execute, not prepare — DryRun replaces execute outputs with mocks. Prepare still runs (validates request construction). Parse still runs (validates response handling).
- No fallbacks — missing mocks are errors, not silent defaults.

Every tool follows this pattern:
```
gist:    prepare_gist_request → execute → parse_gist_response
review:  prepare_llm_request  → execute → parse_llm_response
deps:    prepare_file_write   → execute → (terminal node)
```

**Known issue:** `core/codegen/src/main.rs` (the bootstrapper) bypasses transport for file I/O — by design (circular dependency).

---

## A.5 Registration

**Intent:** Auto-discover registrable units at link time so that adding a new unit requires annotating one function — not updating a manual list.

```
#[target_macro(metadata...)]    →    inventory::submit!(Registration { ... })
pub fn my_spec() -> T { ... }        ↓
                                 inventory::collect!(Registration)
                                      ↓
                                 iter_targets() → all registrations
```

Three layers:
1. **Proc macro** — parses attributes, validates required fields at compile time
2. **inventory crate** — collects submissions across crate boundaries at link time
3. **Iterator API** — downstream code iterates over all registered units

```rust
pub struct TestgenTarget {
    pub origin_crate: &'static str,
    pub name: &'static str,
    pub output_path: &'static str,
    pub generate: fn(&TestgenTargetDef) -> String,
}

inventory::collect!(TestgenTarget);

pub fn iter_targets() -> impl Iterator<Item = &'static TestgenTarget> {
    inventory::iter::<TestgenTarget>.into_iter()
}
```

| File | Role |
|------|------|
| `core/testgen-registry-macros/src/lib.rs` | `#[testgen_target]` proc macro |
| `core/testgen-registry/src/lib.rs` | `TestgenTarget` struct, `iter_targets()`, shared codegen helper |
| `gunbc-dag/src/bin/testgen.rs` | Binary that collects and runs all targets |

Usage:
```rust
#[testgen_target(
    name = "llm-openai",
    output = "lib/llm-ops/src/generated_tests.rs",
    module = "llm_openai_generated_tests",
    builder = "crate::graph::build_chat_completion_graph()",
    no_boundary_tests
)]
pub fn openai_mock_spec() -> MockSpec { ... }
```

**Design decisions:**
- `&'static str` fields — registration happens at link time with no heap allocation.
- `origin_crate` + path rewriting — `module_path!()` returns crate-qualified paths; the registry rewrites to `crate::` for generated code that lives inside the same crate.
- Function pointer, not trait object — no vtable, no allocation, works in static context.
- Validation test — scans source files for `pub fn *mock_spec()` and checks they have `#[testgen_target]`.

**Current status:**

| Registration Kind | Mechanism | Auto? |
|---|---|---|
| Testgen targets | `inventory` + proc macro | Yes |
| Tool definitions | `all_tools()` hardcoded vec | **No** |
| Graph builders | `GraphBuilderId` enum | **No** |
| Boundary mocks | Dual definition (registry + MockSpec) | **No** |
| Resource defs | Hardcoded glob patterns | **No** |

See `docs/design/unified-registration.md` for the plan to unify all registration to the `inventory` pattern.

---

## A.6 Emission

**Intent:** Transform structured data into a target-specific format and write it somewhere. All rendering follows IR → Renderer → Output. Backends are swappable.

```
┌────────────┐     ┌──────────┐     ┌────────────┐
│  Prepare   │ ──▶ │  Format  │ ──▶ │   Write    │
│  (pure)    │     │  (pure)  │     │ (boundary) │
└────────────┘     └──────────┘     └────────────┘
   Build IR      Apply renderer    TransportOps::Execute
```

The gold standard implementation (testgen):
```
analyze_dag() → collect_obligations() → build TestFile IR → TestRenderer::render_file() → String
```

```rust
// Language-neutral test IR
pub struct TestFile { sections: Vec<TestSection> }
pub enum Stmt { Let { .. }, Assert(Assert), Expr(Expr), Comment(String), ... }
pub enum Assert { Eq { left, right, message }, Contains { .. }, NonEmpty { .. }, ... }

// Language-specific rendering
pub trait TestRenderer {
    fn render_file(&self, file: &TestFile) -> String;
    fn render_expr(&self, expr: &Expr) -> String;
    fn render_stmt(&self, stmt: &Stmt) -> String;
    fn render_assert(&self, assert: &Assert) -> String;
}
```

| File | Role |
|------|------|
| `core/codegen/src/testgen/test_ir.rs` | Test IR types (TestFile, Stmt, Expr, Assert) |
| `core/codegen/src/testgen/render.rs` | `TestRenderer` trait |
| `core/codegen/src/testgen/render_rust.rs` | Rust backend (630 lines) |
| `core/codegen/src/testgen/render_python.rs` | Python stub (validates trait surface) |
| `core/codegen/src/testgen/codegen.rs` | IR construction (never constructs strings) |
| `gunbc-dag/src/makegen/render.rs` | Makefile rendering |
| `core/ir/src/transport/ci/render.rs` | CI YAML rendering |

**Current implementations:**

| System | Has IR? | Has Renderer Trait? |
|--------|---------|---------------------|
| Testgen | Yes (TestFile) | Yes (TestRenderer) |
| Makegen | No | No |
| CI YAML | Yes (SharedStep) | Yes (CiRenderer) |
| CLI gen | No | No |
| Terminal | No | No |

**Design decisions:**
- IR before rendering — codegen.rs builds IR only, never constructs strings. Enables multi-backend rendering.
- Exhaustive value matching — adding a new `ValueExpr` variant forces all renderers to handle it.
- Stubs validate design — Python and TypeScript renderers are `todo!()` stubs that prove the trait surface.

**Known issues:** Five rendering systems, four different traits, two with no IR at all. See `docs/design/unified-emission.md` for the unification plan.

---

## A.7 Resource Acquisition

**Intent:** Model typed resources (files, credentials, handles) with explicit access modes so the system can detect conflicts, enforce ordering, and simulate lifecycles.

```
Port::resource("name", "TypeId", AccessMode::Write)
    ↓
Node outputs: "res:name" port
    ↓
detect_conflicts() checks all resource accesses in the DAG
    ↓
AccessMode determines compatibility:
  Read + Read     = OK
  Read + Write    = CONFLICT
  Write + Write   = CONFLICT
  Exclusive + *   = CONFLICT
```

```rust
pub enum AccessMode { Read, Write, Exclusive }

pub struct ResourceInput {
    pub name: String,      // "res:credential" (always prefixed)
    pub type_id: TypeId,
    pub mode: AccessMode,
}
```

| File | Role |
|------|------|
| `core/ir/src/resource/mod.rs` | Resource types, conflict detection |
| `core/ir/src/resource/registry.rs` | ResourceRegistry, dependency resolution |
| `core/ir/src/resource/defs.rs` | Built-in resource definitions |
| `core/test/src/mock_spec.rs` | ResourceSimulation for test lifecycle |

**Design decisions:**
- `res:` prefix convention — resource ports are prefixed to distinguish from data ports.
- Conflict detection is structural — `detect_conflicts()` walks the DAG once. No runtime overhead.
- SubDag auto-inference — `Node::subdag()` preserves `resource_access` from inner ports. Resource accounting composes fractally.
- Freshness = mtime then hash — `check_freshness_mtime()` is a fast path. Only if mtime suggests staleness does the system compute the full content hash.

---

## A.8 Credential Lifecycle

**Intent:** Model credential acquisition, expiry, refresh, and revocation as typed resource flows through DAG boundaries.

```
[resolve_auth]  ──▶  [credential_env]  ──▶  [execute]
    (pure)           (env boundary)         (transport)
  maps provider     acquires credential    uses credential
  to env_var/scheme  from environment       in request
```

```rust
pub trait CredentialProvider: Debug {
    fn service_id(&self) -> &str;
    fn acquire(&self) -> Result<Credential, CredentialError>;
}

pub struct Credential {
    secret: Secret,
    scheme: AuthScheme,
}

pub enum AuthScheme { Bearer, Header { name: String } }
```

| File | Role |
|------|------|
| `core/ir/src/transport/credential.rs` | Credential, AuthScheme, CredentialProvider trait |
| `lib/transport/src/credential.rs` | Providers (GitHub, LLM, Mock) + CredentialOp |
| `core/test/src/mock_spec.rs` | ResourceType::Credential, refresh/revoke simulation |
| `lib/llm-ops/src/graph_mock.rs` | Credential lifecycle MockSpec |

**Providers:**

| Provider | Service | Env Var | Scheme |
|----------|---------|---------|--------|
| `GitHubEnvVarProvider` | github | `GITHUB_TOKEN` | Bearer |
| `LlmEnvVarProvider::openai()` | openai | `OPENAI_API_KEY` | Bearer |
| `LlmEnvVarProvider::anthropic()` | anthropic | `ANTHROPIC_API_KEY` | Header(x-api-key) |
| `MockCredentialProvider` | configurable | N/A | configurable |

**Resource simulation:**
```rust
ResourceType::Credential { expiry_ms: Some(3_600_000), refreshable: true }

// Behaviors:
ResourceBehavior::RefreshSucceeds { new_ttl_ms: 3_600_000 }
ResourceBehavior::RefreshFails { error: "token revoked".into() }
ResourceBehavior::RevokeSucceeds
```

**Design decisions:**
- CredentialOp has two modes — Static (pre-configured providers) and FromInputs (reads service/env_var/scheme from DAG inputs at runtime).
- Secret type wraps `Secret` which tracks provenance (`SecretSource`) and redacts in Display. No accidental logging.
- Capability marker prevents forgery via plain Value construction.

---

## A.9 Mock Specification

**Intent:** Declarative test fixtures that describe what mock values boundary nodes provide and what input constraints upstream must satisfy.

```rust
MockSpec::new("tool-name")
    .boundary("node", "port", mock_value)        // what boundary outputs
    .transport_mock("node", "port", mock_value)   // transport interception
    .expects_input("port", constraint)             // upstream constraints
    .node_example(NodeExample::new("node")         // I/O examples for pure nodes
        .input("port", value)
        .output("port", OutputMatcher::exact(value)))
    .resource_credential("id", expiry)             // resource simulation
```

```rust
pub struct MockSpec {
    boundary_mocks: Vec<BoundaryMock>,
    transport_mocks: Vec<TransportMock>,
    input_expectations: Vec<InputExpectation>,
    node_examples: Vec<NodeExample>,
    resource_mocks: ResourceMocks,
}

pub enum OutputMatcher {
    Exact(Box<Value>), Contains(String), NonEmpty,
    IsBool, IsInt, IsString,
    IntGe(i64), IntLe(i64),
    Satisfies { description, predicate },
    Any,
}
```

| File | Role |
|------|------|
| `core/test/src/mock_spec.rs` | MockSpec, OutputMatcher, ResourceSimulation |
| `core/test/src/mock_requirements.rs` | `extract_mock_requirements()` — derive from DAG |
| `core/test/src/boundary.rs` | `assert_boundary_mockable()` |
| `core/exec/src/intercept.rs` | `BoundaryMocks` — runtime interception |

**File convention:** Every tool with a DAG has a `graph_mock.rs` adjacent to its `graph.rs`:
```
lib/tools/gist/src/
├── graph.rs         (DAG builder)
└── graph_mock.rs    (MockSpec + #[testgen_target])
```

**Two consumers:**
1. **Testgen** — generates test code from MockSpec + DAG analysis
2. **DryRun** — `to_boundary_mocks()` converts MockSpec to runtime BoundaryMocks

**Design decisions:**
- Declarative, not imperative — MockSpecs describe *what* to mock, not *how*.
- Chain validation — `validate_chain()` checks that A's mock output satisfies B's expected input. Catches incompatibilities at spec-definition time.
- NodeExample for pure nodes — pure nodes have deterministic behavior. NodeExamples provide input/output pairs for I/O verification tests.
- OutputMatcher typed hierarchy — prefer `IsBool`, `IntGe(n)` over `Satisfies` because codegen can emit real assertions for typed matchers.

**Known issue:** Boundaries are also defined in `registry.rs` for CLI dry-run (dual-source problem). See `docs/design/unified-registration.md`.

---

## A.10 Content Hashing

**Intent:** Deterministic content-addressed hashing for freshness checks, cache keys, and resource identification. All hashing is centralized in one leaf crate.

```
gunbc-infra::hash
├── ContentHash         — opaque hash of file/blob content
├── HashBuilder         — streaming file hasher (SHA-256)
├── hash_parts(parts)   — deterministic multi-part ID hashing
└── codegen_hash        — compute_codegen_input_hash() with glob patterns
```

```rust
pub struct ContentHash(String);  // hex-encoded SHA-256

pub struct HashBuilder { hasher: Sha256 }

impl HashBuilder {
    pub fn new() -> Self;
    pub fn update(&mut self, data: &[u8]);
    pub fn finish(self) -> ContentHash;
    pub fn hash_file(path: &Path) -> io::Result<ContentHash>;
}

pub fn hash_parts(parts: &[&str]) -> String;
```

| File | Role |
|------|------|
| `core/infra/src/hash.rs` | ContentHash, HashBuilder, hash_parts |
| `core/infra/src/codegen_hash.rs` | compute_codegen_input_hash() with glob patterns |
| `core/infra/src/manifest.rs` | ManifestEntry stores ContentHash |

**Design decisions:**
- Leaf crate — `gunbc-infra` has no internal dependencies. Every crate can use hashing.
- Centralized — all crates delegate to `infra::hash`. No duplicate SHA-256 implementations.
- Deterministic multi-part — `hash_parts(&["a", "b"])` always produces the same output regardless of platform.

---

## A.11 Freshness Check

**Intent:** Fast-path mtime check before expensive content hashing. If mtimes haven't changed, skip the full hash computation.

```
check_freshness_mtime(manifest_entry, current_files)
    ↓
MtimeResult::Fresh          → skip codegen/testgen (fast path)
MtimeResult::MaybeStale(r)  → compute full ContentHash → compare with manifest
```

```rust
pub enum MtimeResult {
    Fresh,
    MaybeStale(String),  // reason: "file modified", "file count changed", etc.
}

pub struct ManifestEntry {
    pub input_hash: ContentHash,
    pub input_file_count: usize,
    pub outputs: Vec<PathBuf>,
    pub timestamp: SystemTime,
}
```

| File | Role |
|------|------|
| `core/infra/src/freshness.rs` | `check_freshness_mtime()` |
| `core/infra/src/manifest.rs` | ManifestEntry with file count |
| `core/infra/src/codegen_hash.rs` | `compute_codegen_input_hash()` returns (hash, count) |
| `gunbc-dag/src/ci/ops.rs` | Wires freshness check into CI pipeline |

**Design decisions:**
- File count as fast-fail — if the number of input files changed, mtime check immediately returns `MaybeStale`. Catches added/deleted files.
- Glob patterns as constants — `CODEGEN_GLOB_PATTERNS` and `CODEGEN_EXTRA_FILES` are compile-time constants.

---

# Appendix B: End-to-End Examples

These examples trace real code through the full pipeline: DAG definition → MockSpec → testgen registration → generated tests.

---

## B.1 Clippy Tool (Minimal Upsert)

The simplest real tool. Shows the upsert pattern with no custom nodes.

### Define the tool (`lib/tools/clippy/src/graph.rs`)

```rust
use gunbc_ir::node::Node;
use gunbc_ir::transport::cli::{self, build_cli_upsert, CliToolOp};

pub fn build_clippy_upsert(args: &[&str]) -> Node<CliToolOp> {
    build_cli_upsert(&cli::CLIPPY, args)
}

pub fn build_clippy_lint_all() -> Node<CliToolOp> {
    build_clippy_upsert(&["--all-targets", "--", "-D", "warnings"])
}
```

`build_cli_upsert` produces a SubDag node:

```text
[check_clippy] ──exists?──▶ [install_clippy] ──▶ [run_clippy]
       │                                              ▲
       └──────────already installed──────────────────┘
```

### Compose into CI graph (`gunbc-dag/src/ci/graph.rs`)

```rust
let clippy = build_clippy_lint_all();
dag.add_node(clippy);
```

Fractal composition: each tool is a self-contained SubDag node that the CI graph treats as a single step.

---

## B.2 Gist Tool (Full Pipeline)

Medium-complexity tool with multiple modes, transport boundaries, and resource access.

### Step 1: Define the DAG (`lib/tools/gist/src/graph.rs`)

```rust
pub fn build_gist_graph(
    mode: GistMode, extensions: Vec<String>, public: bool,
) -> Result<Dag<GistGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Environment nodes (intercepted in DryRun)
    let fs_env = builder.add_root_node(Node::opaque(
        "fs_env", vec![],
        vec![port("fs:write", "FilesystemHandle")],
        GistGraphOp::Gist(GistOps::FsEnv { scope: Scope::Write }),
    ))?;

    // Mode-specific content acquisition
    let render_markdown = match mode {
        GistMode::Snapshot => build_snapshot_acquire(&mut builder, extensions)?,
        GistMode::Diff { base_ref } => build_diff_acquire(&mut builder, &base_ref, extensions)?,
        GistMode::Recent => { /* ... */ },
    };

    // Transport boundary: prepare (pure) → execute (I/O) → parse (pure)
    let prepare = builder.add_node_after(Node::opaque(
        "prepare_gist_request",
        vec![scalar("markdown", "String")],
        vec![scalar("request", "TransportRequest")],
        GistGraphOp::Gist(GistOps::PrepareRequest { public }),
    ), &render_markdown)?;

    let execute = builder.add_node_after(Node::opaque(
        "execute_gist",
        vec![scalar("request", "TransportRequest")],
        vec![scalar("response", "TransportResponse")],
        GistGraphOp::Transport(TransportOps::Execute),
    ), &prepare)?;

    let parse = builder.add_node_after(Node::opaque(
        "parse_gist_response",
        vec![scalar("response", "TransportResponse")],
        vec![scalar("url", "String")],
        GistGraphOp::Gist(GistOps::ParseGistResponse),
    ), &execute)?;

    builder.build()
}
```

### Step 2: Write the MockSpec (`lib/tools/gist/src/graph_mock.rs`)

MockSpec is extracted from the DAG's actual structure — not hand-written:

```rust
fn gist_mock_spec(mode: &GistMode) -> MockSpec {
    let dag = build_gist_graph(mode.clone(), vec![], false).expect("gist graph should build");

    let mut reqs = extract_mock_requirements(&dag, "gist")
        .boundary("fs_env", "fs:write", mock_fs_handle())
        .expect("fs:write mock should match type")
        .boundary("clock_env", "clock", mock_clock())
        .expect("clock mock should match type");

    match mode {
        GistMode::Snapshot => {
            reqs = reqs
                .transport_response("execute_list_files", "response",
                    TransportResponse::Shell(ShellResponse::ok("src/main.rs\nREADME.md\n")))
                .expect("type check")
                .transport_response("execute_read_files", "response",
                    TransportResponse::Shell(ShellResponse::ok("fn main() {}\n")))
                .expect("type check");
        }
        // ... other modes
    }

    reqs.boundary_str("parse_gist_response", "url",
        "https://gist.github.com/mock/abc123def456")
        .expect("type check")
        .build_unchecked()
}
```

### Step 3: Register with testgen

```rust
#[gunbc_testgen_registry_macros::testgen_target(
    name = "gist-snapshot",
    output = "lib/tools/gist/src/generated_tests_snapshot.rs",
    module = "gist_snapshot_generated_tests",
    builder = "crate::build_gist_graph(crate::GistMode::Snapshot, vec![], false).unwrap()",
    signature = "crate::gist_signature(&crate::GistMode::Snapshot)"
)]
pub fn gist_snapshot_mock_spec() -> MockSpec {
    gist_mock_spec(&GistMode::Snapshot)
}
```

No manual registration list — adding `#[testgen_target]` is the only step.

### Step 4: What testgen generates

For the credential lifecycle graph (5 nodes), testgen produces tests organized by obligation bucket:

**Header:**
```rust
// Generated tests for llm_credential_lifecycle_generated_tests DAG.
// Generated by gunbc-testgen
// DO NOT EDIT - regenerate with: make testgen
// Obligations: 23 obligations (9 discharged, 14 testable: A=6, B=5, C=3, D=0)
// Content-Hash: 04affa725267b9dd...
```

**Bucket A — Execution Semantics:**
```rust
#[test]
fn test_dryrun_completion() {
    let dag = crate::graph::build_chat_completion_graph();
    let log = execute_with_mode(&dag,
        ExecutionMode::DryRun(mock_spec().to_boundary_mocks()))
        .expect("DryRun execution should complete without crash");
    assert!(!log.entries.is_empty());
}

#[test]
fn test_transport_interception() {
    let dag = crate::graph::build_chat_completion_graph();
    let result = assert_boundary_mockable(&dag, mock_spec().to_boundary_mocks());
    assert!(result.is_ok());
    assert!(result.boundary_nodes.iter().any(|n| n == "execute"));
}
```

**Bucket C — Scenario Coverage:**
```rust
#[test]
fn test_scenario_all_succeed() {
    let dag = crate::graph::build_chat_completion_graph();
    let log = execute_with_mode(&dag,
        ExecutionMode::DryRun(mock_spec().to_boundary_mocks()))
        .expect("all-succeed scenario should complete");
    let entry = log.get("execute").expect("'execute' should be in log");
    assert!(entry.was_intercepted);
}

#[test]
fn test_skip_propagation_execute() {
    let dag = crate::graph::build_chat_completion_graph();
    let mut mocks = mock_spec().to_boundary_mocks();
    mocks.set_value("execute", "response", Value::Skipped);
    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks))
        .expect("skip propagation should not crash");
    assert!(log.get("parse").is_some());
}
```

**Bucket D — Resource Simulation:**
```rust
#[test]
fn test_resource_credential_llm_acquire() {
    let spec = mock_spec();
    let resource = spec.get_resource("credential:llm").expect("resource should exist");
    let result = resource.acquire();
    assert!(matches!(result, ResourceAcquireResult::Acquired));
}

#[test]
fn test_resource_credential_llm_timeout() {
    let spec = mock_spec();
    let resource = spec.get_resource("credential:llm").expect("resource should exist");
    assert!(!resource.should_timeout(1800000));
    assert!(resource.should_timeout(3600001));
}
```

**Windowed Segment Tests:**
```rust
#[test]
fn test_window_credential_env_through_execute() {
    let dag = crate::graph::build_chat_completion_graph();
    let flat = lower(&dag).expect("lower should succeed");
    let baseline = execute_with_mode(&dag,
        ExecutionMode::DryRun(mock_spec().to_boundary_mocks()))
        .expect("baseline should succeed");
    let window = Window::from_nodes(&flat, vec!("credential_env", "execute"));
    let mut mocks = mock_spec().to_boundary_mocks();
    apply_window_inputs(&flat, &window, &baseline, &mut mocks)
        .expect("window inputs derivable from baseline");
    let window_dag = window_subdag(&flat, &window);
    let log = execute_with_mode(&window_dag, ExecutionMode::DryRun(mocks))
        .expect("window execution should succeed");
    assert_window_outputs(&flat, &window, &baseline, &log)
        .expect("window outputs should match baseline");
}
```

---

## B.3 CI Graph (Large Composed DAG)

The CI graph (`gunbc-dag/src/ci/graph.rs`) composes multiple tool SubDag nodes:

```text
[deps_exists] -> [codegen_exists] -> [codegen] -> [testgen] -> [build] -> [test]
                                                                             |
                                                     [clippy_lint] <---------+
                                                            |
                                                     [guardrail_check]
                                                            |
                                                        [report]
```

**Generated obligation stats:**
```
Obligations: 133 obligations (58 discharged, 75 testable: A=27, B=30, C=16, D=2)
Proven by construction: acyclicity, type compatibility, cardinality satisfaction.
```

Meaning: 58 obligations proven statically (no test needed), 75 tests generated across 4 buckets.

---

## B.4 Pipeline Summary

```text
+-------------------------------+
| 1. Define DAG                 |   graph.rs
|    Node::opaque / DagBuilder  |   prepare -> execute -> parse
+-------------------------------+
              |
              v
+-------------------------------+
| 2. Write MockSpec             |   graph_mock.rs
|    extract_mock_requirements  |   type-checked against DAG structure
|    .boundary() / .transport() |
+-------------------------------+
              |
              v
+-------------------------------+
| 3. Register with testgen      |   #[testgen_target(name, output, builder)]
|    proc macro + inventory      |   auto-discovered at build time
+-------------------------------+
              |
              v
+-------------------------------+
| 4. Analyze DAG                |   analyze.rs
|    boundaries, transport,     |   structural facts + proof obligations
|    cardinalities, resources   |
+-------------------------------+
              |
              v
+-------------------------------+
| 5. Generate tests             |   codegen.rs
|    TestGenerator + buckets    |   A: execution, B: contracts,
|    (only for Unknown proofs)  |   C: scenarios, D: resources
+-------------------------------+
              |
              v
+-------------------------------+
| 6. Output: generated_tests.rs |   content-hash header
|    make testgen regenerates   |   50-150+ tests per DAG
+-------------------------------+
```

**Key invariants:**

1. **MockSpec is derived from DAG structure**, not invented independently. `extract_mock_requirements()` reads the actual DAG and type-checks each mock.
2. **Registration is automatic.** Adding `#[testgen_target(...)]` is sufficient. No manual list.
3. **Tests are obligation-driven.** Only obligations that cannot be proven statically generate test code.
4. **Content hashing detects drift.** Each generated file includes a `Content-Hash`. If the DAG or MockSpec changes, `make testgen` overwrites the file.
5. **Windowed tests compose fractally.** Sliding windows of size 2..max are extracted as sub-DAGs, fed baseline inputs, and verified against baseline outputs.
