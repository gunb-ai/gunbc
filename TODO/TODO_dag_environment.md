# Design: DAG Environment Model

**Status**: Design / Open Questions
**Date**: 2026-02-04

How should DAG nodes access system resources (filesystem, platform,
clock, env vars, tools)? Today this is ad-hoc. This doc maps the
current state, proposes a direction, and poses design questions.

---

## The Problem

Every DAG node is a pure function: `HashMap<String, Value> → HashMap<String, Value>`.
System resources should flow through edges like any other value. But
today, some nodes cheat — they reach outside the DAG to grab what they
need:

```
GOOD (tools):
  EnvOp ──tool:clippy──→ LintNode     (handle flows through edge)

BAD (filesystem):
  GistOps::PrepareRequest              (constructs FilesystemHandle inline)

BAD (platform):
  DepsOps::GenerateScripts             (calls Installer::new() → Platform::detect())

BAD (clock):
  gist_ops::generate_gist_filename     (calls SystemTime::now() inline)

BAD (env vars):
  executor::execute_rest               (calls std::env::var() inline)
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

### Filesystem handles — NOT MODELED

```
GistOps::PrepareRequest
  └─ calls sanitize_branch_for_filename()
       └─ constructs FilesystemHandle::cross_platform(Scope::Write) INLINE
```

**Key file**: `lib/gist-ops/src/lib.rs:113`

No DAG node acquires a filesystem handle. The function that needs it
just creates one. This means:
- Can't swap the filesystem target (e.g., test with FAT32 constraints)
- Can't mock it in DryRun
- The handle isn't visible in the DAG graph

### Platform — NOT MODELED

```
DepsOps::GenerateScripts
  └─ calls Installer::new()
       └─ calls Platform::detect() INLINE (compile-time cfg!)
```

**Key files**: `lib/tools/deps/src/installer.rs:39`, `ops.rs:211`

Platform is detected at compile time and baked into the Installer.
No DAG node emits platform info. Downstream nodes can't declare
"I need platform info" as an input port.

### Clock — NOT MODELED

```
gist_ops::generate_gist_filename()
  └─ calls SystemTime::now() INLINE
```

**Key file**: `lib/gist-ops/src/lib.rs:145`

No DAG node captures "now". The timestamp is grabbed inline by the
function that formats it. Tests can't control the time.

### Environment variables — PARTIALLY MODELED

Transport layer reads env vars inline for auth:

```
executor::execute_rest()
  └─ match AuthMethod::EnvVar(var) => std::env::var(var) INLINE
```

**Key file**: `lib/transport/src/executor.rs:81,97`

Shell requests DO model env vars correctly — `ShellRequest.env` is an
explicit field. But auth token resolution bypasses this: the executor
reads OS env vars directly instead of receiving resolved tokens.

Codegen also reads env vars inline (`core/codegen/src/cli_gen.rs:724,748`).

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

Today `AuthMethod::EnvVar("GITHUB_TOKEN")` is resolved by the transport
executor (`executor.rs:81`). This is a layer violation — the executor
should receive concrete tokens, not env var names.

Options:
- **Resolve at graph build time**: Replace `EnvVar(name)` with `Bearer(value)` before execution
- **Resolve in an environment node**: `AuthEnv` reads env vars, emits resolved auth
- **Resolve in PrepareRequest**: The pure Prepare node receives env vars as input and resolves

### Q6: What about resources that aren't handles?

ToolHandle and FilesystemHandle are capabilities. But `SystemTime` is
a snapshot value, and `Platform` is a static enum. Do these need the
handle pattern, or are they just values?

Proposal: Distinguish between:
- **Capabilities** (handle pattern): FilesystemHandle, ToolHandle — acquired, scoped, operations through handle
- **Observations** (value pattern): Platform, Timestamp, EnvVars — captured at boundary, passed as plain values

---

## Where Violations Live Today

(Cross-reference with `TODONE/TODO_consolidate_di.md` for details)

| Resource | Violation site | What happens | What should happen |
|----------|---------------|--------------|-------------------|
| Filesystem | `gist-ops/lib.rs:113` | Constructs handle inline | Receive through input port |
| Platform | `deps/installer.rs:39` | `Platform::detect()` in constructor | Receive through input port |
| Platform | `deps/ops.rs:211` | `Installer::new()` in Executable | Receive Platform as DAG input |
| Clock | `gist-ops/lib.rs:145` | `SystemTime::now()` inline | Receive timestamp as input |
| Env vars | `transport/executor.rs:81,97` | `std::env::var()` in executor | Resolve before executor |
| Env vars | `codegen/cli_gen.rs:724,748` | `env::vars()` in generated code | Accept env dict parameter |

---

## Incremental Plan

This doesn't need to be solved all at once. Incremental steps:

### Phase 1: Make violations explicit (mechanical)
- [x] `sanitize_branch_for_filename(&FilesystemHandle, &str) → String`
- [x] `generate_gist_filename(&FilesystemHandle, &str, SystemTime) → String`
- [x] `Installer::for_platform(Platform)` everywhere (drop `::new()`)
- [x] Resolve `AuthMethod::EnvVar` before executor

These are pure refactors — change function signatures, thread the
values from callers. No new DAG infrastructure needed.

### Phase 2: Add environment acquisition to graphs
- [ ] Add `FsEnv` node to gist graph (emits `FilesystemHandle`)
- [ ] Add `PlatformEnv` node to deps graph (emits `Platform`)
- [ ] Add `ClockEnv` node where timestamps are needed
- [ ] Wire edges from env nodes to consumers
- [ ] DryRun interception for new env node types

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
