# gunbc Handbook

This handbook is the practical guide to the gunbc codebase. It explains the core concepts, how the system is structured, and the recurring patterns you should recognize when reading or extending the code.

Companion documents:
- `docs/design/v4/dsl-design.md` for the full DSL language specification
- `docs/design/service-codegen.md` for DSL-driven service codegen architecture
- `docs/design/overview.md` for design rationale and formal framing
- `SPEC.md` for the formal IR specification
- `docs/design/testgen.md` for the test generation model
- `docs/ab-writing-workflows.md` for A/B workflow comparisons
- `AGENT.md` for onboarding guardrails and repo pointers

This file is self-contained: [Appendix A](#appendix-a-pattern-reference) has every pattern reference, [Appendix B](#appendix-b-end-to-end-examples) has full pipeline walkthroughs.

## Doc Map

| Doc | Focus | Use When |
| --- | --- | --- |
| `docs/handbook.md` | Practical overview, pattern catalog, e2e examples | You need the conceptual map, pattern details, or concrete examples |
| `docs/design/v4/dsl-design.md` | DSL language specification | You are writing `.dag` files or extending the language |
| `docs/design/service-codegen.md` | Service codegen from DSL | You are adding a new service or modifying the emit pipeline |
| `docs/design/overview.md` | Philosophy + formal model | You want design rationale and invariants |
| `SPEC.md` | Formal IR spec | You need canonical IR type definitions |
| `docs/design/testgen.md` | Test generation theory | You are touching testgen or proof obligations |
| `docs/ab-writing-workflows.md` | A/B workflow comparisons | You want side-by-side imperative vs DAG examples |
| `AGENT.md` | Onboarding + guardrails | You are new to the repo or doing refactors |

## Mental Model

gunbc is a **DSL-first workflow compiler** where **everything is a DAG**.

The primary authoring surface is the `.dag` language — declarative definitions that compile to a typed Graph IR. The compiler pipeline is:

```text
.dag source  →  [parse]  →  [typecheck]  →  [lower]  →  Dag<LoweredOp>
                                                              ↓
                                               [emit] → Rust / Go / C / MIPS
```

Core claims:
- Structural soundness means: acyclic, type-compatible, cardinality-compatible, and sub-DAG interfaces match.
- Nodes are pure transformations. In the runtime DAG, world I/O only happens at explicit transport execution nodes.
- Service operations (REST, Shell, File) are defined once in `.dag` and compiled to all target languages.
- If it validates, it is structurally sound.

## Compositional Modeling Philosophy

**Principle:** Every external system is modeled as a **composition of layered concerns**, where each layer imposes its own invariants on the final generated code. Workflows never interact with lower layers directly — the abstractions handle it — but the model is complete and the generated code reflects every layer.

This principle is inspired by the `the-gunbai` Understanding pattern: external systems are described as structured data (behaviors, constraints, assumptions, dependencies), and the system derives blocks, tests, and prerequisites from those declarations. In gunbc, the DSL's type system, interface contracts, and annotation composition serve the same role.

### Example 1: Network stack layers (TCP → TLS → HTTP → REST → GitHub → Gist)

Consider what happens when the gist tool calls `github.Gist.Create()`:

```text
Layer 0: TCP        — reliable byte stream, port 443
Layer 1: TLS        — encrypted channel, certificate validation
Layer 2: HTTP       — request/response framing, status code semantics (RFC 9110)
Layer 3: REST       — policy overrides on HTTP (e.g., 304 = success), content-type negotiation
Layer 4: GitHub API — base URL, API versioning header, Bearer token auth, rate limiting
Layer 5: Gist       — POST /gists, file map payload, permission scopes
```

Each layer adds constraints:
- **TCP/TLS** are implicit in the transport executor (handled by the HTTP client library)
- **HTTP** imposes status code classification — 4xx is client error, 5xx is server error
- **REST** overrides HTTP where needed — 304 Not Modified is success, not error
- **GitHub API** adds `@endpoint`, `@auth(BearerToken)`, API version headers
- **Gist** adds `@rest(POST, "/gists")`, `@permissions(["gist"])`, typed input/output shapes

The workflow author writes `github.Gist.Create(files: files)` and sees none of this. But the compiler composes all six layers into the generated transport code: the correct URL, the correct headers, the correct auth scheme, the correct error classification, and the correct mock responses for testing.

**Key insight:** Layers 0-2 are handled by infrastructure (the transport executor). Layers 3-5 are captured in DSL annotations. The workflow only names Layer 5 (the operation). But the model is **complete** — every layer's invariants are enforced.

### Example 2: External dependency composition

External dependencies follow the same layered pattern. A tool like `curl` doesn't just "exist" — it has a dependency chain:

```text
curl/download
  → requires: network connectivity (infra/network)
    → requires: DNS resolution (resolve_dns)
    → requires: TCP connectivity (check_port)
  → requires: TLS certificates (system trust store)
  → requires: tool:curl binary (package manager install)
```

Each requirement imposes invariants:
- **Network** — if offline, fail before attempting download
- **DNS** — if resolution fails, fail with specific diagnostic
- **Tool binary** — if not installed, trigger upsert (check → install → resolve)

In the DSL, this is captured via `uses` declarations and interface contracts:
```
func download(url: Url) -> { content: Bytes }
  uses net: Network
  uses fs: Filesystem(mode: Write)
```

The compiler resolves `uses net: Network` transitively — the bound implementation's requirements (DNS, TCP) become prerequisites of the pipeline stage. The workflow author writes `download(url: url)` and the system ensures all prerequisites are satisfied.

### Example 3: Package manager modeling (typed install planning)

gunbc's deps tool demonstrates this pattern concretely. A package install is not just "run a shell command" — it's a composition:

```text
Layer 0: Platform     — Linux/macOS/Windows (determines available package managers)
Layer 1: PackageManager — apt/brew/cargo/script (typed enum, not strings)
Layer 2: SelectionPolicy — deterministic priority ranking (not declaration-order)
Layer 3: InstallPlan  — validated per-PM field requirements (apt needs packages, script needs body)
Layer 4: Upsert pattern — check installed → install if missing → verify
```

Each layer imposes invariants:
- **Platform** constrains which package managers are available
- **PackageManagerId** fails closed on unknown IDs (no string fallbacks)
- **SelectionPolicy** ensures reproducible selection across runs
- **InstallPlan::validate()** enforces per-PM field requirements
- **Upsert** ensures idempotency — running twice is safe

### What this means for extending the system

When adding a new service, tool, or external dependency:

1. **Identify the layer stack** — what protocols, auth schemes, and platform constraints does this involve?
2. **Declare each layer's invariants in the DSL** — `@rest`, `@auth`, `@endpoint`, `@permissions`, `@idempotent`, `@hermetic`, typed inputs/outputs
3. **Let the compiler compose them** — the generated transport code, mock specs, and test obligations reflect every layer's constraints
4. **The workflow author names only the top layer** — `github.Gist.Create()`, not "make an authenticated REST POST to the GitHub API over HTTPS on port 443"

This is the target state. Where the Rust substrate currently hand-wires what the DSL can derive (e.g., credential chains repeated across graph builders), those are consolidation targets — see `TODO/tasks.md` for the active lanes.

### File generation as a layer

Every file the system generates flows through `content_upsert` (literal paths) or is declared via `@outputs` (dynamic paths). The compiler extracts these paths during lowering and propagates them to `CompileOutput.output_paths`, which feeds the tool registry. The bootstrap tool reads the registry and generates `.gitignore` rules. CI verifies no generated files are committed. The base repo contains only source-of-truth.

```text
Layer 2: content_upsert     — "I write this file" (path is structural data, auto-extracted)
Layer 3: @outputs("glob")   — "I write dynamic files matching this" (annotation declaration)
Layer 4: Tool                — composes layers 2+3, carries outputs in ToolRegistration
Layer 5: Repository          — enforces: all outputs are gitignored + not committed
```

## Core IR: Dag, Node, Port, Edge

The Graph IR is the **compilation target** — you rarely construct it by hand. Instead, write `.dag` files and let the compiler produce `Dag<LoweredOp>`.

Canonical IR types live in `core/ir/src/dag.rs`, `core/ir/src/node.rs`, and `core/ir/src/types.rs`.

### DSL (primary authoring)

```
module example.fetch

fn parse_body(body: String) -> Json { /* ... */ }

func fetch_and_parse(url: String) -> { result: Json } {
  body = http.Get(url: url)
  result = parse_body(body: body.content)
  return { result: result }
}
```

The compiler lowers this to two nodes (`fetch`, `parse`) connected by an edge — the same Graph IR that the Rust API produces, but declared in tens of lines instead of hundreds.

### Graph IR (compilation target)

```rust
// You rarely write this directly anymore — the compiler generates it.
use gunbc_ir::{Dag, Node, Port, Edge, NodeBody};

let mut dag: Dag<MyOp> = Dag::new();
dag.add_node(Node::opaque("fetch",
    vec![Port::new("url", "String")], vec![Port::new("body", "String")], MyOp));
dag.add_node(Node::opaque("parse",
    vec![Port::new("body", "String")], vec![Port::new("result", "Json")], MyOp));
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

Transport requests/responses are defined in `core/ir/src/transport/mod.rs`. Runtime DAG I/O is performed only by `TransportOps::Execute` in `lib/transport`. Direct I/O outside the DAG is limited to `lib/transport` plus a small set of explicitly audited exceptions: build-time generators (`core/codegen`), bootstrap/config loaders (`gunbc-dag/src/bootstrap`), and the manifest/freshness layer (`core/infra`). Tests are exempt by pragma policy. The full exception list is maintained in `TODO/TODONE/clippy-pragma-audit.md`.

Key invariants:
- Pure ops **prepare** `TransportRequest` values.
- Only `TransportOps::Execute` performs runtime DAG I/O.
- Direct I/O is limited to `lib/transport` plus explicitly audited build-time/bootstrap exceptions (see `TODO/TODONE/clippy-pragma-audit.md`). Tests are exempt by pragma policy.

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
| `dsl/` | **Primary authoring surface** — all `.dag` source files |
| `dsl/services/` | Service definitions (REST, Shell): gcp, github, cargo, git, llm |
| `dsl/tools/` | Tool workflows: clippy, gist, codegen, makegen, etc. |
| `dsl/pipelines/` | Pipeline compositions: ci |
| `core/daglang/` | DSL compiler: discover → parse → resolve → typecheck → lower → derive → emit |
| `core/ir/` | Core IR types, patterns, transport model, resource system |
| `core/exec/` | Execution engine, DryRun interception, simulation |
| `core/codegen/` | CLI and test generation |
| `core/test/` | MockSpec and test utilities |
| `lib/transport/` | Canonical runtime I/O boundary; direct I/O elsewhere is banned (tests exempt by pragma policy) |
| `lib/tools/` | General-purpose tool wrappers (clippy, deps, gist) |
| `gunbc-dag/` | Repo-specific runtime resolver and CLI entrypoints |
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

### DSL-first workflow (primary path)

Most new work is done by writing or modifying `.dag` files. The compiler handles lowering, type checking, and multi-language emission.

**Add a new REST/Shell service:**
1. Identify the layer stack: what protocol (HTTP/REST/gRPC), what auth scheme (Bearer, Header, Basic), what provider (GitHub, GCP, Stripe), what operations?
2. Create `dsl/services/<provider>/<name>.dag` with `service` block and `operation` definitions.
3. Express each layer's invariants via annotations: `@endpoint` (provider base URL), `@auth` (auth scheme), `@rest`/`@shell` (transport method + path), `@permissions` (required scopes), `@idempotent`/`@readonly` (behavioral properties), `@mock_response` (test data).
4. Each annotation composes additively — the compiler generates transport code, mock specs, and test obligations reflecting all layers. The workflow author names only the top-level operation.

Example (adding a new REST service):
```
module services.stripe.payments

service stripe.Payments {
  @endpoint("https://api.stripe.com")
  @auth(BearerToken)

  operation CreateCharge {
    input { amount: Int, currency: String }
    output { id: String @json("id"), status: String @json("status") }
    @rest(POST, "/v1/charges")
  }
}
```

**Add a new tool workflow:**
1. Create `dsl/tools/<name>.dag` — import services, define `fn` (pure) and `func` (effectful) blocks.
2. The compiler generates the `prepare → execute → parse` triplet automatically from service calls.

**Add a new pipeline:**
1. Create `dsl/pipelines/<name>.dag` — import tools, define `pipeline` block with `stage` dependencies.

### Legacy Rust IR path (for framework internals)

If you need to modify the compiler, IR types, or execution engine:
- Add a new pattern: `core/ir/src/patterns/`.
- Add a new transport: `core/ir/src/transport/` plus executor support in `lib/transport/`.
- Extend the emit pipeline: `core/daglang/daglang-emit/src/` (add `service_emit` functions per backend).

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

**Intent:** Runtime DAG world I/O happens at explicit transport execution nodes. Pure nodes prepare requests and parse responses. Direct I/O outside the DAG is limited to the transport layer (`lib/transport`); tests are exempt by pragma policy.

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
- `clippy.toml` disallows `std::fs` and `std::process::Command::new` in all crates; the only crate-level exemption is `lib/transport` (tests are exempt by pragma policy).
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

**Known issue:** None — all I/O routes through transport boundaries.

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
| Tool definitions | `derive_tool_defs()` + `#[tool_target]` inventory | **Yes** |
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
[resolve_auth]      [cloud_env]
    (pure)          (env boundary)
       \             /
        \           /
         ──▶ [cloud_credential] ──▶ [execute]
             (secret manager)       (transport)
  maps provider     acquires secret     uses credential
  to scheme/header  + builds Credential in request
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
| `lib/cloud-ops/src/env.rs` | CloudEnv (provider-neutral config + OIDC inputs) |
| `lib/cloud-ops/src/graph.rs` | Cloud secret manager graphs (GCP/AWS/Azure) |
| `core/test/src/mock_spec.rs` | ResourceType::Credential, refresh/revoke simulation |
| `lib/llm-ops/src/graph_mock.rs` | Credential lifecycle MockSpec |

**Providers:**

| Provider | Service | Env Var | Scheme |
|----------|---------|---------|--------|
| `GitHubEnvVarProvider` | github | `GITHUB_TOKEN` | Bearer |
| `LlmEnvVarProvider::openai()` | openai | `OPENAI_API_KEY` | Bearer |
| `LlmEnvVarProvider::anthropic()` | anthropic | `ANTHROPIC_API_KEY` | Header(x-api-key) |
| `MockCredentialProvider` | configurable | N/A | configurable |

**Primary path:** CI/prod credentials are sourced via cloud secret manager
(GCP WIF today). Env-var providers remain for local dev and tests.

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

These examples trace real code through the full pipeline. The primary authoring surface is `.dag` files; the compiler handles lowering to Graph IR and emitting target-language code.

---

## B.1 Clippy Tool (Minimal Upsert)

The simplest real tool. Shows the upsert pattern for tool acquisition.

### DSL definition (`dsl/tools/clippy.dag`)

```
module tools.clippy

import std.patterns { upsert }
import std.resources { Filesystem }
import services.cargo

resource Clippy {
  kind: Capability
  mode: Read
  lifecycle: Persistent

  capability check {
    input {}
    output { exists: Bool }
    @shell(["cargo", "clippy", "--version"])
    @hermetic @readonly
  }

  capability install {
    input {}
    output { installed: Bool }
    @shell(["rustup", "component", "add", "clippy"])
    @hermetic
  }

  capability resolve {
    input {}
    output { handle: String }
    @shell(["cargo", "clippy", "--version"])
    @hermetic @readonly
  }
}

func clippy_lint(paths: List<String>?) -> { clean: Bool, findings: String }
  uses clippy: Clippy
{
  tool = upsert(
    check: clippy.check(),
    create: clippy.install(),
    resolve: clippy.resolve()
  )
  result = cargo.Build.Clippy() [after tool]
  return { clean: result.success, findings: result.stderr }
}
```

The compiler lowers this to a SubDag node with the upsert pattern:

```text
[check_clippy] ──exists?──▶ [install_clippy] ──▶ [run_clippy]
       │                                              ▲
       └──────────already installed──────────────────┘
```

### Compose into CI pipeline (`dsl/pipelines/ci.dag`)

```
stage lint_stage [after build_stage, when build_result.success] {
  lint_result = clippy_lint()
}
```

Fractal composition: each tool is a self-contained SubDag node that the CI pipeline treats as a single step.

---

## B.2 Gist Tool (Full Pipeline)

Medium-complexity tool with multiple modes, transport boundaries, and resource access.

### Step 1: Define the service (`dsl/services/github/gist.dag`)

```
service github.Gist {
  @endpoint("https://api.github.com")
  @auth(BearerToken)

  operation Create {
    input {
      description: String
      files: Map<String, String>
      public: Bool = false
    }
    output {
      url: Url @json("html_url")
      id: GistId
    }
    @rest(POST, "/gists")
    @permissions(["gist"])
  }
}
```

### Step 2: Define the tool workflow (`dsl/tools/gist.dag`)

```
module tools.gist

import services.git
import std.patterns { read_text_files }
import services.github.gist

fn render_snapshot(files: List<{ path: TextFilePath, content: String }>) -> String {
  let header = "# Code Snapshot\n\n"
  let sections = files
    |> map(f => "## `{f.path}`\n\n```\n{f.content}\n```")
    |> join("\n\n")
  "{header}{sections}"
}

func gist_snapshot(base_ref: CommitSha?) -> { url: Url }
  uses fs: Filesystem(mode: Read)
{
  ctx = branch_context()
  files = git.Core.LsFiles()
  read_result = read_text_files(paths: files.files)
  markdown = render_snapshot(files: read_result.files)
  result = share_content(markdown: markdown, branch: ctx.branch, base_ref: base_ref)
  return { url: result.url }
}
```

The compiler generates the transport triplet (`prepare → execute → parse`) automatically from the service call `github.Gist.Create()` — no hand-written `PrepareRequest`/`ParseGistResponse` structs needed.

### Step 3: MockSpec and testgen registration

MockSpec is extracted from the DAG's actual structure — the `@mock_response` annotation on the service operation provides the mock data:

```
operation Create {
    // ...
    @mock_response(
      status: 201,
      body: { "html_url": "https://gist.github.com/mock/{id}", "id": "{id}" }
    )
}
```

For the testgen registration, the `#[testgen_target]` proc macro auto-discovers the test target at link time:

```rust
#[testgen_target(
    name = "gist-snapshot",
    output = "lib/tools/gist/src/generated_tests_snapshot.rs",
    module = "gist_snapshot_generated_tests",
    builder = "crate::build_gist_graph(crate::GistMode::Snapshot, vec![], false).unwrap()",
)]
pub fn gist_snapshot_mock_spec() -> MockSpec { /* ... */ }
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
fn test_window_cloud_credential_through_execute() {
    let dag = crate::graph::build_chat_completion_graph();
    let flat = lower(&dag).expect("lower should succeed");
    let baseline = execute_with_mode(&dag,
        ExecutionMode::DryRun(mock_spec().to_boundary_mocks()))
        .expect("baseline should succeed");
    let window = Window::from_nodes(&flat, vec!("cloud_env", "cloud_credential", "execute"));
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

## B.3 CI Graph (Large Composed Pipeline)

The CI pipeline is defined in `dsl/pipelines/ci.dag` — a 12-stage composition that imports all gunbc tools and wires them together with stage dependencies, parallel groups, and aggregate reporting.

### DSL definition (`dsl/pipelines/ci.dag`)

```
module pipelines.ci

import tools.makegen
import tools.bootstrap
import tools.codegen
import tools.testgen
import tools.pragma
import tools.build
import tools.clippy { clippy_lint }
import tools.deps
import std.types { Summary, StageResult }
import shared.dag_util { aggregate_results, format_report, stage_result, stage_from_output }

pipeline ci {

  stage cloud_env {
    cloud_status = check_cloud_env()
  }

  stage codegen_stage [after cloud_env] {
    codegen_result = codegen()
  }

  stage bootstrap_stage [after codegen_stage, when codegen_result.success] {
    bootstrap_result = bootstrap()
  }

  stage generate [after codegen_stage, after bootstrap_stage, when codegen_result.success] {
    parallel {
      pragma_result = pragma(directives: load_pragma_directives())
      testgen_result = testgen(targets: discover_testgen_targets())
    }
  }

  stage build_stage [after generate] {
    build_result = build_all()
  }

  stage test_stage [after build_stage, when build_result.success] {
    test_result = cargo.Build.Test()
  }

  stage lint_stage [after build_stage, when build_result.success] {
    lint_result = clippy_lint()
  }

  stage guardrails [after generate] {
    guardrail_result = run_guardrail_checks()
  }

  stage verify [after generate] {
    parallel {
      verify_makegen = verify_makegen_output()
      verify_deps = verify_deps_config()
      verify_bootstrap = verify_bootstrap_output()
      verify_testgen = verify_testgen_output()
      verify_pragma = verify_pragma_output()
    }
    verify_aggregate = aggregate_verify(
      results: [verify_makegen, verify_deps, verify_bootstrap, verify_testgen, verify_pragma]
    )
  }

  stage report [after test_stage, after lint_stage, after guardrails, after verify] {
    stages = [
      stage_result(name: "codegen", success: codegen_result.success, stderr: ""),
      stage_result(name: "bootstrap", success: true, stderr: ""),
      // ... (all stages aggregated)
    ]
    summary = aggregate_results(stages: stages)
    report_text = format_report(summary: summary, stages: stages)
  }
}
```

### Dependency graph

```text
  cloud_env (root)
      |
  codegen
      |
      +---> deps_check
      |
      +---> bootstrap --------+
      |                       |
      +---> pragma -----------+---> lint (clippy)
      |                       |
      +---> testgen ----------+---> build ---> test ---+
      |                       |                        |
      |                       +---> guardrails --------+
      |                       |                        |
      +---> verify (5 checks) +---> aggregate ---------+---> report
```

Each tool (`clippy_lint`, `codegen`, `bootstrap`, etc.) is a self-contained SubDag node imported from `dsl/tools/`. The pipeline only declares **what depends on what** — the compiler resolves the stage ordering and parallel execution groups.

**Generated obligation stats:**
```
Obligations: 133 obligations (58 discharged, 75 testable: A=27, B=30, C=16, D=2)
Proven by construction: acyclicity, type compatibility, cardinality satisfaction.
```

Meaning: 58 obligations proven statically (no test needed), 75 tests generated across 4 buckets.

---

## B.4 Pipeline Summary

The DSL-first pipeline from authoring to generated tests:

```text
+-------------------------------+
| 1. Write .dag file            |   dsl/services/, dsl/tools/, dsl/pipelines/
|    service + operation blocks |   @rest/@shell annotations, @mock_response
|    fn/func blocks, pipeline   |   stage dependencies, parallel groups
+-------------------------------+
              |
              v
+-------------------------------+
| 2. Compile                    |   core/daglang/
|    discover → parse → resolve |   module graph + typed project
|    → typecheck → lower → derive | → Dag<LoweredOp> + derived artifacts
|    → emit (Rust/Go/C/MIPS)    |   ServiceOperationSpec → transport code
+-------------------------------+
              |
              v
+-------------------------------+
| 3. MockSpec (from annotations)|   @mock_response on service operations
|    extract_mock_requirements  |   type-checked against DAG structure
|    + graph_mock.rs bridge     |   boundary mocks + transport mocks
+-------------------------------+
              |
              v
+-------------------------------+
| 4. Register with testgen      |   #[testgen_target(name, output, builder)]
|    proc macro + inventory      |   auto-discovered at build time
+-------------------------------+
              |
              v
+-------------------------------+
| 5. Analyze DAG                |   analyze.rs
|    boundaries, transport,     |   structural facts + proof obligations
|    cardinalities, resources   |
+-------------------------------+
              |
              v
+-------------------------------+
| 6. Generate tests             |   codegen.rs
|    TestGenerator + buckets    |   A: execution, B: contracts,
|    (only for Unknown proofs)  |   C: scenarios, D: resources
+-------------------------------+
              |
              v
+-------------------------------+
| 7. Output: generated_tests.rs |   content-hash header
|    make testgen regenerates   |   50-150+ tests per DAG
+-------------------------------+
```

**Key invariants:**

1. **`.dag` files are the single source of truth.** Service definitions, tool workflows, and pipelines are all authored in the DSL. The compiler handles lowering, type-checking, and multi-language emission.
2. **MockSpec is derived from DAG structure**, not invented independently. `extract_mock_requirements()` reads the actual DAG and type-checks each mock. `@mock_response` annotations on service operations provide the mock data.
3. **Registration is automatic.** Adding `#[testgen_target(...)]` is sufficient. No manual list.
4. **Tests are obligation-driven.** Only obligations that cannot be proven statically generate test code.
5. **Content hashing detects drift.** Each generated file includes a `Content-Hash`. If the DAG or MockSpec changes, `make testgen` overwrites the file.
6. **Windowed tests compose fractally.** Sliding windows of size 2..max are extracted as sub-DAGs, fed baseline inputs, and verified against baseline outputs.
7. **Multi-language emission from single definition.** Service operations compile to transport code for all backends (Rust, Go, C, MIPS) from the same `.dag` file.
