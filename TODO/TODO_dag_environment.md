# Design: DAG Environment Model

**Status**: Phases 1-2 COMPLETE, Phase 3 open questions
**Date**: 2026-02-04 (updated 2026-02-07)

## Ownership
- [x] Taken by Codex (2026-02-05)

How should DAG nodes access system resources (filesystem, platform,
clock, env vars, tools)? Today this is ad-hoc. This doc maps the
current state, proposes a direction, and poses design questions.

---

## The Problem

Every DAG node is a pure function: `HashMap<String, Value> → HashMap<String, Value>`.
System resources should flow through edges like any other value. But
today, some nodes cheat — they reach outside the DAG to grab what they
need. Several of these have been fixed in specific tools, but the
general environment-node pattern is still ad-hoc:

```
GOOD (tools):
  EnvOp ──tool:clippy──→ LintNode     (handle flows through edge)

GOOD (filesystem, clock — tool-specific):
  GistOps::FsEnv/ClockEnv → PrepareRequest (handles flow through edges)

GOOD (platform — deps):
  PlatformEnv → GenerateScripts         (platform passed in as input)

BAD (env vars, edge case):
  CI provider detection                (reads env at boundary)
```

The `EnvOp` pattern works: acquire at the boundary, flow through edges.
But it only covers tools. Filesystem, platform, clock, and env vars
have no equivalent.

---

## Current State: How Each Resource Flows

### Tools — SOLVED ✓

```
EnvOp ──tool:clippy→ LintNode
        │
        └─ upsert_tool() at I/O boundary
           ToolHandle flows as Value through edges
           DryRun intercepts → ToolHandle::mock()
```

**Key file**: `gunbc-dag/src/ci/env.rs`

EnvOp is a root node with no inputs. It performs I/O (check/install),
emits `ToolHandle` values on typed ports, and downstream nodes receive
handles through edges. DryRun mode intercepts this node and substitutes
`ToolHandle::mock()`. This is the gold standard.

### Transport (shell, http, file) — SOLVED ✓

```
PrepareOp ──request→ TransportOps::Execute ──response→ ParseOp
                      │
                      └─ actual I/O boundary
                         DryRun intercepts → mock response
```

**Key file**: `lib/transport/src/ops.rs`

Pure nodes build `TransportRequest` values. A single `TransportOps::Execute`
node performs the I/O. Pure nodes parse the response. DryRun intercepts
the Execute node. This is also the gold standard.

### Filesystem handles — MODELED (tool-specific)

```
GistOps::FsEnv ──fs:write→ GistOps::PrepareRequest
```

**Key files**: `lib/gist-ops/src/lib.rs` (FsEnv + require_filesystem_handle),
`lib/tools/gist/src/graph.rs` (fs:write port wiring)

Gist now acquires a `FilesystemHandle` at the DAG boundary via `FsEnv`
and passes it to pure ops. This solves mockability and makes the handle
visible in the graph, but the pattern is still tool-specific rather than
a shared environment node.

### Platform — MODELED (deps)

```
PlatformEnv ──platform→ DepsOps::GenerateScripts
```

**Key files**: `lib/tools/deps/src/env.rs`, `lib/tools/deps/src/graph.rs`,
`lib/tools/deps/src/ops.rs`

Deps now acquires platform at the DAG boundary (`PlatformEnv`) and passes
it to `GenerateScripts` via `res:platform`. Platform detection still exists
in convenience constructors/tests, but the DAG path is now explicit.

### Clock — MODELED (tool-specific)

```
GistOps::ClockEnv ──clock→ GistOps::PrepareRequest
```

**Key files**: `lib/gist-ops/src/lib.rs`, `lib/tools/gist/src/graph.rs`

Gist now captures time at the DAG boundary and passes a `Timestamp`
through edges. This is still tool-specific; no shared clock env node.

### Environment variables — MODELED AT DAG BOUNDARY

Auth env var resolution now happens via `CredentialOp` at the DAG boundary.
`TransportOps` expects a resolved `Credential` (`res:credential`) and applies
it before execution; the executor no longer reads env vars directly.

Step-mode CLIs now capture `env::vars()` once at the boundary and pass an
env dict into `load_step_inputs_from_env()` / `emit_step_outputs()` to keep
CI behavior injectable in generated runners.

**Key files**: `lib/transport/src/credential.rs`, `lib/transport/src/ops.rs`

Shell requests still model env vars explicitly via `ShellRequest.env`.
Codegen reads env vars inline (`core/codegen/src/cli_gen.rs:724,748`).

### CI context — EXTERNAL TO DAG

```
main()
  └─ CiContext::detect()
       └─ passed to execute_with_ci() as optional parameter
```

**Key file**: `core/exec/src/ci_context.rs`

CI context is acquired at the application entry point and passed into
the executor as a side-channel — not through edges. Nodes never see it.
This works for output formatting, but if a node ever needed to branch
on "am I in CI?", it would have to smuggle that in.

---

## What Exists That We Can Build On

### 1. EnvOp pattern (env.rs)
Root node, no inputs, I/O at boundary, emits typed handles on ports.
Mockable in DryRun. **This is the pattern to generalize.**

### 2. ResourceId / ResourceAccess (resource.rs)
Already has `ResourceId::file()`, `ResourceId::tool()`, `AccessMode::Read/Write/Exclusive`.
Conflict detection via `detect_conflicts()` with transitive ordering.
**This could be extended to declare what resources a node touches.**

### 3. Entrypoint / Boundary detection (entrypoint.rs, boundary.rs)
Unconnected input ports = entrypoints (world reads).
Unconnected output ports = boundaries (world writes).
**An environment node's outputs are "world reads" made explicit.**

### 4. FilesystemHandle (filename.rs)
Capability-based handle with Scope (Read/Write), follows ToolHandle
pattern. Already has `for_filesystem()`, `for_targets()`, `cross_platform()`.
**Ready to be emitted by an environment node.**

### 5. DryRun interception (execute.rs:631-654)
Type-based detection: intercepts nodes with `TransportRequest` inputs
or `ToolHandle` outputs. **Can be extended to intercept environment
nodes that emit other handle types.**

---

## Design Direction: Scoped Environment Nodes

Generalize `EnvOp` from "tool acquisition" to "environment acquisition."
An environment node is a root node that acquires system resources and
emits them as typed values on output ports.

### Sketch

```
                   ┌─────────────────────────────────┐
                   │        RuntimeEnv (root)         │
                   │                                  │
                   │  Acquires:                       │
                   │   - FilesystemHandle (cross-plat)│
                   │   - Platform (detected/target)   │
                   │   - Clock snapshot (SystemTime)   │
                   │   - Env vars (HashMap<S,S>)      │
                   │   - ToolHandles                  │
                   │                                  │
                   │  Outputs:                        │
                   │   ├─ fs:write → FilesystemHandle │
                   │   ├─ platform → Platform         │
                   │   ├─ clock → Timestamp           │
                   │   ├─ env → EnvVars               │
                   │   ├─ tool:clippy → ToolHandle    │
                   │   └─ tool:cargo → ToolHandle     │
                   └──────┬──┬──┬──┬──┬──┬────────────┘
                          │  │  │  │  │  │
              ┌───────────┘  │  │  │  │  └─────────────┐
              ▼              ▼  │  ▼  ▼                 ▼
          LintNode    PrepGist │ GenScripts         AuthResolve
                               ▼
                          PrepFilename
```

### What changes

1. **Graph builders** add edges from environment node to every node
   that needs a resource
2. **Pure nodes** declare what they need as input ports (e.g.,
   `port("fs:write", "FilesystemHandle")`)
3. **Executable implementations** receive resources through `inputs`
   instead of constructing them
4. **DryRun** intercepts environment nodes and substitutes mocks
5. **ResourceAccess** annotations can be derived from the port types

---

## Open Design Questions

### Q1: One big environment node or many small ones?

**Option A — Single RuntimeEnv node** (like a constructor):
```
RuntimeEnv → emits all resources on typed ports
```
Pro: One place to mock, one place to configure.
Con: Every graph needs the same node. Some resources are expensive
to acquire (tools). Graphs that don't need tools still pay the cost.

**Option B — Per-resource environment nodes** (current EnvOp style):
```
ToolEnv → tool:clippy, tool:cargo
FsEnv → fs:write
PlatformEnv → platform
ClockEnv → clock
```
Pro: Graphs only acquire what they need. Lighter mocking.
Con: Multiple root nodes to wire. More boilerplate.

**Option C — Layered** (compose small into big):
```
ToolEnv + FsEnv + ClockEnv → compose into RuntimeEnv if desired
```
Pro: Flexibility. Con: Complexity.

### Q2: How does a node declare what environment it needs?

Today ports are stringly typed (`port("tool:clippy", "ToolHandle")`).
Should environment dependencies be:

- **Just ports** — same as today, port name convention like `env:fs`
- **Declared on the op** — `impl NeedsEnvironment for GistOps { fn needs() -> Vec<EnvReq> }`
- **Inferred from the type** — if an op's execute() takes a `FilesystemHandle`,
  the framework ensures one flows in

### Q3: Should environment be scoped per-subgraph?

Today a DAG is flat. But if we have sub-DAGs (LoopBuilder bodies,
nested workflows), should they inherit the parent's environment or
acquire their own?

```
OuterDAG
  ├─ RuntimeEnv (acquires fs, platform, etc.)
  └─ LoopBuilder
       └─ BodyDAG
            └─ does this node see the outer RuntimeEnv?
```

If yes: environment flows through LoopBuilder boundaries.
If no: BodyDAG must have its own environment node.

### Q4: Where does runtime filesystem detection live?

`FilesystemHandle::cross_platform()` is a build-time decision.
But detecting the actual filesystem at a path (e.g., ext4 vs NTFS
mount) requires `statfs()`  — that's I/O.

Options:
- **Transport op**: `PrepareDetectFs → Execute(statfs) → ParseDetectFs → FilesystemHandle`
- **Environment node**: `FsEnv` does the detection internally (like EnvOp does upsert)
- **Deferred**: Start with `cross_platform()` and add detection later

### Q5: How do we handle env var resolution for auth?

Previously env-var auth (e.g., `GITHUB_TOKEN`) was resolved by the transport
executor, which was a layer violation. This is now resolved at the DAG
boundary via `CredentialOp`, and the executor only sees concrete credentials.

Options:
- **Resolve at graph build time**: Replace env-var references with concrete bearer tokens before execution
- **Resolve in an environment node**: `CredentialOp` reads env vars, emits resolved credentials
- **Resolve in PrepareRequest**: The pure Prepare node receives env vars as input and resolves

### Q6: What about resources that aren't handles?

ToolHandle and FilesystemHandle are capabilities. But `SystemTime` is
a snapshot value, and `Platform` is a static enum. Do these need the
handle pattern, or are they just values?

Proposal: Distinguish between:
- **Capabilities** (handle pattern): FilesystemHandle, ToolHandle — acquired, scoped, operations through handle
- **Observations** (value pattern): Platform, Timestamp, EnvVars — captured at boundary, passed as plain values

---

## Completed Phases

**Phase 1 (Make violations explicit)** and **Phase 2 (Environment acquisition
in graphs)** are both complete. All six original violation sites are resolved:
filesystem handles, platform detection, clock snapshots, and env var resolution
now flow through DAG edges via per-resource env nodes (FsEnv, PlatformEnv,
ClockEnv, CredentialOp). DryRun requires explicit mocks for all intercepted nodes.

See `TODO/TODONE/design-resource-acquisition.md` for the full design.

---

## Incremental Plan

### Phase 2.5: Resolve auth env var at DAG boundary (COMPLETE)

Resolved by `CredentialOp` boundary nodes: auth env vars are read at the
boundary and emitted as `Credential` values, and `TransportOps` requires
`res:credential` when a request uses env-var auth. This makes auth resolution
visible in the graph and interceptable by DryRun.

- [x] Resolve env-var auth → concrete credential at the DAG boundary
      via `CredentialOp` (see `TODO_credential_lifecycle.md` for the unified design).
      **Files**: `lib/transport/src/credential.rs`, `lib/transport/src/ops.rs`

### Phase 3: Generalize to RuntimeEnv (if needed)
- [ ] Decide Q1 (single vs many env nodes)
- [ ] Decide Q2 (declaration mechanism)
- [ ] Decide Q3 (sub-DAG scoping)
- [ ] Build the general mechanism based on answers

---

## Notes

- The DAG system is well-suited for this. The entrypoint/boundary
  detection, resource conflict checking, and DryRun interception
  already assume resources flow through edges. We just need to make
  that true for all resources, not just tools and transport.

- Phase 1 is the highest-value, lowest-risk work. Changing function
  signatures to accept resources instead of constructing them is
  a mechanical refactor that makes every callsite testable, with
  zero new infrastructure.

- The deeper design (Q1-Q6) can wait until Phase 1 is done and we
  see what patterns emerge from threading resources manually.
