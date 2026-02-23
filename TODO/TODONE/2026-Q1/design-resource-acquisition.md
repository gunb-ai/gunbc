# Design: Resource Acquisition Model

> Resolves: TODO_dag_environment.md Q1–Q6, Phase 3 design decisions,
> and ../TODO_consolidate_di.md dependency injection items.
>
> Principle: **All environment needs are resource acquisition.**
> Dependencies are always injected and solved up front, not within a node.
> Sub-DAGs receive only explicitly delegated resources (zero-based budgeting).

## Ownership
- [x] Taken by Codex (2026-02-05)

## Status
**Phases 1-3 complete** (2026-02-05). Resource trait infrastructure implemented:
- `Resource` trait with `resource_id()`, `access_mode()`, `kind()`
- Per-resource environment nodes (FsEnv, PlatformEnv, ClockEnv, CredentialOp)
- `res:` port naming convention and validation
- DryRun mock extension for all resource types
- Migrated existing inline violations to resource acquisition pattern

**Remaining (tracked in TODO/TODONE/design-unified-resource-model.md):**
- Phase 4: Sub-DAG zero-based delegation model
- Phase 5: Resource accounting / auto-derive ResourceAccess

## 1. Problem Statement

Today, `EnvOp` acquires CLI tools and emits `ToolHandle` values through
edges — the gold standard. But filesystem handles, platform info, clock
snapshots, and env vars are all grabbed inline by the nodes that need
them, bypassing the DAG:

```
GOOD:   EnvOp ──tool:clippy──→ LintNode        (handle flows through edge)
BAD:    PrepareRequest          constructs FilesystemHandle inline
BAD:    GenerateScripts         calls Platform::detect() inline
BAD:    generate_gist_filename  calls SystemTime::now() inline
BAD:    execute_rest            calls std::env::var() inline
```

This breaks testability, DryRun interception, and the ability to reason
about what resources a DAG touches.

The existing `ResourceId` / `ResourceAccess` / `detect_conflicts()` model
(resource.rs) already reasons about resource contention between parallel
nodes. But it only covers analysis — there's no unified acquisition or
delegation model.

## 2. Design Principles

From project direction:

- **All environment needs are resource acquisition.** No special-casing
  for "tools vs filesystem vs clock." One model.
- **Dependencies are always injected and solved up front, not within a
  node.** Nodes receive capabilities through input ports. A node that
  constructs its own handle is a design violation.
- **Zero-based capability budgeting.** Sub-DAGs do not inherit the
  parent's full resource set. They receive only what is explicitly
  delegated. This is marshalling/delegation, not blanket inheritance.
- **Type-driven inference is acceptable but must scale.** The framework
  can infer resource requirements from port types, but the mechanism
  must not require scanning the entire type registry for every node.

## 3. Resource Taxonomy

### 3.1 Capabilities vs Observations

Two kinds of resources (resolves Q6):

| Kind | Nature | Examples | Acquisition | Serialization |
|------|--------|----------|-------------|---------------|
| **Capability** | Handle with operations | `ToolHandle`, `FilesystemHandle` | Active (I/O at boundary) | Opaque token |
| **Observation** | Snapshot value | `Platform`, `Timestamp`, `EnvVars` | Passive (read once) | Plain `Value` |

Both are resources. Both are acquired at environment nodes and flow
through edges. The difference is in semantics:

- **Capabilities** represent *permission to do something*. The handle
  is opaque — you can't forge one. `ToolHandle::acquire()` requires
  the tool to be installed. `FilesystemHandle::for_filesystem()`
  requires knowing the target filesystem.

- **Observations** represent *facts about the world*. `Platform::Linux`
  is a value, not a permission. `Timestamp(1706...) ` is a frozen
  moment. Observations are immutable once captured.

### 3.2 The Resource Trait

A base `Resource` trait unifies both kinds:

```rust
/// A resource acquired at a DAG boundary and flowed through edges.
///
/// Resources are the only way to access system state. Every environment
/// need — tools, filesystem, platform, clock, env vars — is modeled as
/// a resource.
pub trait Resource: Into<Value> + TryFrom<Value> {
    /// Unique identifier for this resource kind.
    fn resource_id(&self) -> ResourceId;

    /// Access mode for conflict detection.
    fn access_mode(&self) -> AccessMode;

    /// Is this a capability (active handle) or observation (snapshot)?
    fn kind(&self) -> ResourceKind;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    /// Active handle — permission to perform operations.
    Capability,
    /// Snapshot value — immutable fact about the world.
    Observation,
}
```

Existing types implement it naturally:

```rust
impl Resource for ToolHandle {
    fn resource_id(&self) -> ResourceId { ResourceId::tool(self.id()) }
    fn access_mode(&self) -> AccessMode { self.tool().access_mode }
    fn kind(&self) -> ResourceKind { ResourceKind::Capability }
}

impl Resource for Platform {
    fn resource_id(&self) -> ResourceId { ResourceId::new("platform") }
    fn access_mode(&self) -> AccessMode { AccessMode::Read }
    fn kind(&self) -> ResourceKind { ResourceKind::Observation }
}
```

### 3.3 Resource Catalog

| Resource | Kind | ResourceId | AccessMode | Acquisition |
|----------|------|-----------|------------|-------------|
| `ToolHandle` | Capability | `tool:{id}` | Per-tool (Read/Write) | `upsert_tool()` |
| `FilesystemHandle` | Capability | `fs:{scope}` | Read or Write | `for_filesystem()` |
| `Platform` | Observation | `platform` | Read | `Platform::detect()` |
| `Timestamp` | Observation | `clock` | Read | `SystemTime::now()` |
| `EnvVars` | Observation | `env` | Read | `std::env::vars()` |
| `Credential` | Capability | `credential:{service}` | Read | env var resolution |

## 4. Environment Nodes

### 4.1 Decision: Per-Resource Nodes (Q1 → Option B)

Each resource kind gets its own environment node. This follows the
existing `EnvOp` pattern and avoids paying for resources you don't need.

```
ToolEnv ──tool:clippy──→ LintNode
FsEnv ──fs:write──→ PrepareGist
PlatformEnv ──platform──→ GenerateScripts
ClockEnv ──clock──→ PrepareFilename
CredentialOp ──credential:github──→ PrepareRequest
```

Rationale:
- Graphs only acquire what they need. A graph that doesn't use tools
  pays nothing for tool acquisition.
- Each env node is independently mockable in DryRun.
- Composition is natural: a "full runtime env" is just multiple env
  nodes wired into the same graph.
- This matches the existing `EnvOp` pattern exactly — generalize, don't
  replace.

A convenience builder can compose multiple env nodes when desired:

```rust
RuntimeEnvBuilder::new()
    .with_tools(&["clippy", "cargo"])
    .with_filesystem(FsScope::Write)
    .with_platform()
    .build()  // Returns multiple nodes + internal edges
```

### 4.2 Environment Node Contract

Every environment node follows the same contract:

1. **Zero inputs.** Environment nodes are root nodes (entrypoints).
2. **Typed outputs.** Each output port emits a resource value.
3. **I/O at the boundary.** All system interaction happens inside
   the env node's `execute()`.
4. **Mockable.** DryRun intercepts the env node and substitutes mock
   resources. No downstream node is aware of the mock.

```rust
/// A typed environment node that acquires a specific resource.
pub trait EnvNode: Executable {
    /// The resource type this node acquires.
    type Resource: Resource;

    /// Create mock outputs for DryRun mode.
    fn mock_outputs(&self) -> HashMap<String, Value>;
}
```

### 4.3 Concrete Environment Nodes

```rust
/// Filesystem environment — acquires a FilesystemHandle.
pub struct FsEnv {
    pub scope: FsScope,
}

/// Platform environment — detects the current platform.
pub struct PlatformEnv;

/// Clock environment — captures the current timestamp.
pub struct ClockEnv;

/// Credential environment — resolves credentials from env vars.
pub struct CredentialOp {
    pub service: &'static str,
    pub env_var: &'static str,
}
```

Each follows the EnvOp pattern: root node, no inputs, I/O at boundary,
emits typed values, DryRun-interceptable.

## 5. Resource Declaration

### 5.1 Decision: Port-Based Declaration (Q2 → Option A with Convention)

Nodes declare resource needs as input ports with a naming convention.
No new trait or inference mechanism needed.

```rust
// A node that needs a filesystem handle and a tool:
Node::opaque(
    "prepare_gist",
    vec![
        port("content", "String"),       // data input
        port("res:fs", "FsHandle"),      // resource input
    ],
    vec![port("request", "TransportRequest")],
    PrepareGistOp,
)
```

Convention: resource ports use the `res:` prefix. This is a naming
convention, not a new type system feature. Benefits:

- **Scales.** No registry scanning. Port names are explicit.
- **Visible.** `detect_entrypoints()` shows which resource ports are
  unconnected — immediately reveals missing environment wiring.
- **Type-safe.** The port's `type_id` must match the resource's type.
  `port("res:fs", "FsHandle")` won't connect to a `ToolHandle` output.

### 5.2 Resource Wiring in Graph Builders

Graph builders wire environment nodes to consumers:

```rust
fn build_gist_graph() -> Dag<GistOp> {
    let mut builder = DagBuilder::new();

    // Environment: acquire resources up front
    let fs_env = builder.add_node(FsEnv { scope: FsScope::Write });
    let clock_env = builder.add_node(ClockEnv);
    let credential_env = builder.add_node(CredentialOp::new(vec![
        Arc::new(GitHubEnvVarProvider::new()),
    ]));

    // Business logic nodes
    let prepare = builder.add_node(PrepareGistOp);
    let filename = builder.add_node(FilenameOp);

    // Wire resources
    builder.add_edge(fs_env.output("fs:write"), prepare.input("res:fs"));
    builder.add_edge(clock_env.output("clock"), filename.input("res:clock"));
    builder.add_edge(
        credential_env.output("credential:github"),
        prepare.input("res:credential"),
    );

    // Wire data
    builder.add_edge(/* ... */);

    builder.build()
}
```

### 5.3 Validation: Unconnected Resource Ports

At build time, we can validate that every `res:*` input port has an
upstream edge. An unconnected resource port means a node needs something
that nobody provides — a build error.

```rust
fn validate_resource_wiring<T>(dag: &Dag<T>) -> Vec<UnwiredResource> {
    detect_entrypoints(dag)
        .iter()
        .filter(|ep| ep.port_name.0.starts_with("res:"))
        .map(|ep| UnwiredResource {
            node_id: ep.node_id.clone(),
            port_name: ep.port_name.clone(),
        })
        .collect()
}
```

## 6. Sub-DAG Resource Scoping

### 6.1 Decision: Zero-Based Budgeting (Q3 → Explicit Delegation)

Sub-DAGs **do not** inherit the parent's environment. Resources must
be explicitly delegated from parent to child. This is the zero-based
budgeting approach: every sub-DAG starts with nothing and receives only
what it's explicitly given.

```
OuterDAG
  ├─ FsEnv ──fs:write──→ OuterNode
  │                   └──→ LoopBuilder.delegate("res:fs")
  └─ LoopBuilder
       └─ BodyDAG
            └─ InnerNode ← receives fs:write via delegation
```

### 6.2 Why Not Inheritance?

Inheritance ("child sees everything parent has") creates these problems:

1. **Invisible dependencies.** A sub-DAG that works today because
   it inherits `fs:write` will break if the parent stops acquiring
   filesystem access. The dependency is invisible.

2. **Over-provisioning.** A sub-DAG that only needs `Platform`
   observation gets `ToolHandle` capabilities it doesn't need.
   This defeats the principle of least privilege.

3. **Resource accounting.** If the parent has `fs:write` (exclusive)
   and delegates it to a loop body that runs N times, is each
   iteration sharing the handle or getting its own? Inheritance
   doesn't answer this. Explicit delegation does.

### 6.3 Delegation Mechanism

The parent DAG explicitly passes resources into sub-DAGs through
the sub-DAG's input ports:

```rust
// Parent DAG: LoopBuilder receives resources as inputs
let loop_node = LoopBuilder::new("process_items")
    .with_resource_input("res:fs", "FsHandle")     // delegated
    .with_resource_input("res:platform", "Platform") // delegated
    .with_data_input("items", "List<Item>")
    .with_body(body_dag)
    .build();

// Wire resources from env nodes to loop's resource inputs
builder.add_edge(fs_env.output("fs:write"), loop_node.input("res:fs"));
builder.add_edge(platform_env.output("platform"), loop_node.input("res:platform"));
```

Inside the body DAG, resources appear as regular input ports. The body
DAG doesn't know or care whether the resource was acquired by a parent
or by its own env node.

### 6.4 Delegation Semantics by Resource Kind

| Resource Kind | Delegation | Per-Iteration |
|--------------|------------|---------------|
| Observation | Share (immutable) | Same value each iteration |
| Capability (Read) | Share | Same handle each iteration |
| Capability (Write/Exclusive) | Clone or serialize | Each iteration gets own scope |

For write capabilities in loops, the delegation must either:
- Clone the handle per iteration (independent writes)
- Serialize access (iterations run sequentially for that resource)

This is enforced by the existing `detect_conflicts()` analysis — if
two iterations write to the same resource without ordering, it's a
conflict.

## 7. DryRun Interception

### 7.1 Explicit Mocking (No Defaults)

DryRun interception remains type-based, but **does not auto-generate**
resource values. Every intercepted node (transport executors, env nodes,
tool consumers) must have **explicit mocks for every output port** via
`BoundaryMocks` / `MockSpec`. Missing mocks are errors.

### 7.2 Resource Mock Constructors

Use the resource constructors to build explicit mock values:

| Resource | Mock Value |
|----------|-----------|
| `ToolHandle` | `ToolHandle::mock(tool)` — `/mock/{id}` path |
| `FsHandle` | `FilesystemHandle::cross_platform(scope)` |
| `Platform` | `Platform::Linux` — deterministic default |
| `Timestamp` | `Timestamp(0)` — epoch for reproducibility |
| `EnvVars` | `HashMap::new()` — empty env |
| `Credential` | `Credential::new(Secret::from_env_var(env_var, "..."), AuthScheme::Bearer)` |

## 8. Relationship to Existing Code

| Existing | Role in New Design |
|----------|-------------------|
| `EnvOp` (env.rs) | Stays as-is for tools. Pattern generalized to other resources. |
| `ToolHandle` (cli.rs) | Implements `Resource` trait. Already follows the pattern. |
| `FilesystemHandle` (filename.rs) | Implements `Resource`. Moves from inline construction to env node emission. |
| `ResourceId` (resource.rs) | Unchanged. Identifies resources for conflict detection. |
| `ResourceAccess` (resource.rs) | Can be auto-derived from port types + `Resource::access_mode()`. |
| `detect_conflicts()` (resource.rs) | Unchanged. Gets richer input from explicit resource wiring. |
| `detect_entrypoints()` (entrypoint.rs) | Used to validate resource wiring (unconnected `res:*` ports = error). |
| `TypeContract` (contract.rs) | Resource types are regular types with contracts. No special handling needed. |
| `DryRun interception` (execute.rs) | Extended to intercept all env nodes, not just tool-emitting ones. |

## 9. Resolving Open Questions

| Question | Decision | Rationale |
|----------|----------|-----------|
| **Q1**: One big or many small env nodes? | **Many small** (Option B) | Pay only for what you need. Composable via builder. |
| **Q2**: How to declare resource needs? | **Port convention** (`res:` prefix) | Scales, visible, type-safe. No new traits needed. |
| **Q3**: Sub-DAG scoping? | **Zero-based budgeting** | Explicit delegation. No invisible inheritance. |
| **Q4**: Filesystem detection? | **Deferred** | Start with `cross_platform()`. Add detection env node later. |
| **Q5**: Env var resolution? | **CredentialOp node** | Resolves at boundary, emits concrete credential. Executor receives resolved auth. |
| **Q6**: Capabilities vs observations? | **Both are resources** | Distinguished by `ResourceKind`. Same acquisition pattern. |

## 10. Implementation Plan

### Phase 1: Resource Trait and Infrastructure

- [x] Add `Resource` trait to `core/ir/src/resource.rs`
- [x] Add `ResourceKind` enum (Capability, Observation)
- [x] Implement `Resource` for `ToolHandle`
- [x] Add `res:` port naming convention validation in `DagBuilder`
- [x] Add `validate_resource_wiring()` function

### Phase 2: Concrete Environment Nodes

- [x] Implement `PlatformEnv` (simplest — just detects platform)
- [x] Implement `ClockEnv` (snapshot `SystemTime::now()`)
- [x] Implement `FsEnv` (emits `FilesystemHandle`)
- [x] Implement `CredentialOp` (resolves env var → credential)
- [x] Add DryRun mock constructors for each

### Phase 3: Migrate Existing Violations

- [x] `sanitize_branch_for_filename()` — accept `&FilesystemHandle` input
- [x] `generate_gist_filename()` — accept `Timestamp` input
- [x] `Installer::for_platform()` — accept `Platform` input
- [x] `execute_rest()` — accept resolved `Credential`, stop reading env vars
- [x] Wire env nodes into gist graph, deps graph, transport graph

### Phase 4: Sub-DAG Delegation

> **Note:** Tracked in `TODO/TODONE/design-unified-resource-model.md` which consolidates
> this with build resource management.

- [ ] Add `with_resource_input()` to `LoopBuilder`
- [ ] Add resource delegation wiring in graph builders
- [ ] Validate: no `res:*` entrypoints in body DAGs (must be delegated)
- [ ] Handle write-capability delegation in loops (clone or serialize)

### Phase 5: Resource Accounting

> **Note:** Tracked in `TODO/TODONE/design-unified-resource-model.md` which consolidates
> this with build resource management.

- [ ] Auto-derive `ResourceAccess` from port types + edges
- [ ] Integrate with `detect_conflicts()` for compile-time conflict checking
- [ ] Surface unresolved resource requirements as build-time errors

## Checklist

- [x] `Resource` trait with `resource_id()`, `access_mode()`, `kind()`
- [x] `ResourceKind::Capability` / `ResourceKind::Observation`
- [x] Per-resource environment nodes (FsEnv, PlatformEnv, ClockEnv, CredentialOp)
- [x] `res:` port naming convention
- [x] Resource wiring validation (`validate_resource_wiring()`)
- [ ] Sub-DAG zero-based delegation model
- [x] DryRun mock extension for all resource types
- [x] Migrate existing inline violations to resource acquisition pattern
