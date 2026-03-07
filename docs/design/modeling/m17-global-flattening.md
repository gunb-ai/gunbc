# M17: Global Flattening + Context-Free Work Identity

**Status**: Design
**Lane**: E (Global Minimality Proof)
**Blocks**: M18, M19

## Problem

The `ci` and `test_all` workflows share several process units (compilation,
codegen, testgen) but each workflow defines them independently. The planner
cannot deduplicate equivalent work across workflow boundaries because work
identity is tied to the orchestration node name (`ci.compilation_ensure` vs
`test_all.build_compile`).

## Design

### 1. WorkIdentity model

```rust
pub struct WorkIdentity {
    /// Content-addressed identity: hash of (operation, input schema, resource claims).
    pub content_hash: ContentHash,
    /// Human-readable label for diagnostics.
    pub label: String,
}
```

Two process units are equivalent iff they have the same `WorkIdentity.content_hash`.

### 2. Flattening pass

```rust
pub fn flatten_workflows(
    workflows: &[WorkflowSpec],
) -> (GlobalDag, Vec<DeduplicationEvent>)
```

1. Expand all process-invocation references into concrete process units
2. Compute `WorkIdentity` for each unit
3. Merge units with identical `WorkIdentity` into single vertices
4. Rewire fan-out edges from merged vertex to all downstream consumers
5. Return flattened DAG + dedup events for diagnostics

### 3. Key payload independence

The `content_hash` is computed from:
- Operation semantics (op type + parameters)
- Input port schema (names + types)
- Resource claims (resource IDs + access modes)

It does NOT include:
- Workflow name
- Stage name
- Orchestration node ID

### 4. Cross-workflow dedup test

```rust
#[test]
fn ci_and_test_all_share_compilation() {
    let global = flatten_workflows(&[ci_spec(), test_all_spec()]);
    let compilation_vertices: Vec<_> = global.dag.nodes
        .iter()
        .filter(|n| n.operation == "compilation_ensure")
        .collect();
    assert_eq!(compilation_vertices.len(), 1, "shared work should merge");
}
```

## Files

- `gunbc-app/src/workflow/` — WorkIdentity, flatten_workflows()
- `core/infra/src/hash.rs` — content hashing for work identity

## References

- `gunbc-app/src/workflow/spec_builders.rs` — workflow spec construction
- `gunbc-app/src/workflow/process_registry.rs` — process unit specs
