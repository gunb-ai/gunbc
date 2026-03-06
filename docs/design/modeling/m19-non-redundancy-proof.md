# M19: Formal Non-Redundancy Proof Harness

**Status**: Design
**Lane**: E (Global Minimality Proof)
**Depends on**: M17, M18

## Problem

The planner must guarantee three invariants over the global execution DAG:
1. **At-most-once execution**: No work identity executes more than once
2. **Minimal dirty closure**: Only transitive dependents of changed inputs execute
3. **Single-writer ordering**: No two concurrent writers target the same resource

Violations are currently detectable only via manual inspection.

## Design

### 1. Invariant checkers

```rust
pub fn check_at_most_once(global: &GlobalDag) -> Vec<DuplicateExecution>;
pub fn check_minimal_closure(
    global: &GlobalDag,
    changed: &[InputId],
) -> Vec<UnnecessaryExecution>;
pub fn check_single_writer(global: &GlobalDag) -> Vec<ConcurrentWriterConflict>;
```

### 2. Property tests

Use `proptest` to generate:
- Random subsets of changed inputs
- Verify the execute set is minimal (no vertex could be removed)
- Verify single-writer constraint holds for all resource IDs

### 3. Diagnostics

Each checker returns structured diagnostics:
```rust
pub struct DuplicateExecution {
    pub work_identity: WorkIdentity,
    pub locations: Vec<NodeId>,
}

pub struct ConcurrentWriterConflict {
    pub resource_id: ResourceId,
    pub writers: Vec<(NodeId, AccessMode)>,
}
```

### 4. CI gate

```rust
#[test]
fn planner_invariants_hold_for_all_workflows() {
    let global = flatten_all_workflows();
    assert!(check_at_most_once(&global).is_empty());
    assert!(check_single_writer(&global).is_empty());
    // Minimal closure tested via proptest
}
```

## Files

- `gunbc-app/src/workflow/` — invariant checkers
- `gunbc-app/tests/` — property tests

## References

- `docs/design/modeling/m17-global-flattening.md` — GlobalDag
- `docs/design/modeling/m18-projection-only-surfaces.md` — projection consistency
- `core/ir/src/resource/mod.rs` — AccessMode conflict detection
