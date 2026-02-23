# M10: Mandatory Resource Declarations

**Status**: Design
**Lane**: B (Resource System)
**Blocks**: M11 (Resource Inference)

## Problem

Effectful DAG operations (filesystem writes, network calls, shell commands) can
currently be constructed without declaring resource ports. This makes the
resource conflict detector (`derive_resource_accesses()`) blind to nodes that
should declare their side-effects. Undeclared effects are invisible to the
scheduler, the dry-run interceptor, and contract validation.

## Goal

Every effectful operation must declare its resource ports at build time. A DAG
that performs filesystem I/O without a `res:file:*` port, or executes shell
commands without a `res:tool:*` port, should fail validation before execution.

## Existing Primitives

### Port::resource() — `core/ir/src/dag.rs`

```rust
pub fn resource(name, type_id, mode: AccessMode) -> Port
```

Creates a `res:`-prefixed port with an `AccessMode` tag. Already handles
auto-prefixing and wildcard normalization.

### AccessMode — `core/ir/src/resource/mod.rs`

```rust
pub enum AccessMode { Read, Write, Exclusive }
```

Three-value enum. `Read + Read` is non-conflicting; all other pairs conflict.

### derive_resource_accesses() — `core/ir/src/resource/mod.rs`

```rust
pub fn derive_resource_accesses<T>(dag: &Dag<T>) -> Result<Vec<ResourceAccess>, Vec<ResourceAccessError>>
```

Walks top-level nodes, extracts `ResourceAccess` from `res:*` input ports.
Returns errors if any `res:*` port is missing its `resource_access` tag.
SubDag ports are auto-inferred from inner DAGs.

### validate_resource_wiring_recursive() — `core/ir/src/validate.rs`

```rust
pub fn validate_resource_wiring_recursive<T>(dag: &Dag<T>) -> Vec<UnwiredResource>
```

Recursive descent through SubDag nodes, detecting unconnected `res:*`
entrypoints at any nesting level.

### should_intercept_for_mode() — `core/exec/src/execute.rs`

```rust
fn should_intercept_for_mode<T>(node: &Node<T>, mode: &ExecutionMode) -> bool
```

Already classifies nodes as effectful: transport executors, tool env nodes,
resource env nodes, tool consumers. Returns true when the node should be
mocked in DryRun/Simulate.

## Design

### 1. Build-Time Validator

Add `validate_resource_completeness()`:

```rust
pub fn validate_resource_completeness<T>(dag: &Dag<T>) -> Vec<MissingResourceDeclaration>
```

For each node in the DAG:
1. Determine if the node is effectful using the same heuristics as
   `should_intercept_for_mode()` (transport executor, tool env, resource env,
   tool consumer).
2. Check whether the node declares at least one `res:*` input port.
3. If effectful but no resource port, emit `MissingResourceDeclaration`.

```rust
pub struct MissingResourceDeclaration {
    pub node_id: NodeId,
    pub effect_kind: EffectKind,
}

pub enum EffectKind {
    TransportExecution,
    ToolEnvironment,
    ResourceEnvironment,
    ToolConsumption,
}
```

### 2. Auto-Wiring Rules

For common patterns, provide auto-wiring helpers that inject resource ports
during DAG construction:

| Effect Pattern | Resource Port | AccessMode |
|---|---|---|
| `content_upsert_chain` (execute_read) | `res:file:{path}` | Read |
| `content_upsert_chain` (execute_write) | `res:file:{path}` | Write |
| Shell command execution | `res:tool:{command}` | Read |
| Service transport (HTTP) | `res:network:{endpoint}` | Write |
| Manifest read/write | `res:manifest` | Write |

The `add_content_upsert_chain()` helper in `core/ir/src/patterns/content_upsert.rs`
already constructs 5-node chains. Extend it to auto-attach `res:file:*` ports
on the read/write transport nodes.

### 3. Validation Pipeline Position

```
build DAG
  -> validate_dag() (existing structural checks)
  -> validate_resource_wiring_recursive() (existing: unwired ports)
  -> validate_resource_completeness()  [NEW: missing declarations]
  -> derive_resource_accesses() (existing: conflict detection)
  -> execute / schedule
```

The new validator fires after structural validation but before execution.
It runs in the same phase as `validate_resource_wiring_recursive()`.

### 4. Migration Path

Phase 1 (warn): `validate_resource_completeness()` returns violations as
warnings, logged but not blocking. Existing DAGs continue to work.

Phase 2 (enforce): After all existing DAG construction sites are updated,
switch to hard errors. Gate this on a `ResourceValidationMode` enum:

```rust
pub enum ResourceValidationMode {
    Warn,
    Enforce,
}
```

Default to `Warn` initially. CI can opt into `Enforce` per-crate as each
crate's DAG builders are updated.

Phase 3 (delete): Remove `Warn` mode once all crates pass `Enforce`.

### Open Questions

1. **Granularity of filesystem resources**: Should `res:file:*` use the exact
   path (`res:file:Makefile`) or a coarse category (`res:file:workspace`)?
   Currently the DSL uses categories (`@file(WRITE, "workspace")`). Coarse
   categories are simpler but may miss fine-grained conflicts.

2. **Manifest resources**: The manifest system already has freshness checking
   via `core/infra/src/freshness.rs`. Should manifest access be modeled as a
   resource, or is the existing freshness system sufficient?

3. **Subdag boundary**: `validate_resource_wiring_recursive()` already handles
   SubDag auto-inference. Should `validate_resource_completeness()` also recurse
   into SubDags, or trust that inner validation catches missing declarations?

## References

- `core/ir/src/dag.rs` — Port::resource()
- `core/ir/src/resource/mod.rs` — AccessMode, derive_resource_accesses()
- `core/ir/src/validate.rs` — validate_resource_wiring_recursive()
- `core/exec/src/execute.rs:1190` — should_intercept_for_mode()
- `core/ir/src/patterns/content_upsert.rs` — add_content_upsert_chain()
