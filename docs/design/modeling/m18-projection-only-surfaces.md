# M18: Single Semantic Authority / Projection-Only Surfaces

**Status**: Design
**Lane**: E (Global Minimality Proof)
**Depends on**: M17

## Problem

Workflow dependencies, effects, and claims are authored in multiple places:
- DSL pipeline definitions (`dsl/workflows/*.dag`)
- Process unit specs (`workflow/spec_builders.rs`)
- Makefile targets (`makegen/`)
- CLI argument parsing (`cli_gen.rs`)

When one surface drifts, the others don't notice.

## Design

### 1. Canonical semantic source

The global flattened DAG (from M17) is the single semantic authority. All other
representations are projections.

### 2. Projection contract

```rust
pub trait Projection {
    type Output;
    fn project(global: &GlobalDag) -> Self::Output;
    fn validate(global: &GlobalDag, existing: &Self::Output) -> Vec<DriftViolation>;
}
```

Implementations:
- `MakefileProjection` — derives Makefile targets from global DAG
- `CliProjection` — derives CLI flags from global DAG
- `ReportProjection` — derives status reports from global DAG

### 3. Drift validator

```rust
#[test]
fn makefile_is_consistent_with_global_dag() {
    let global = flatten_all_workflows();
    let violations = MakefileProjection::validate(&global, &current_makefile());
    assert!(violations.is_empty(), "Makefile has drifted: {violations:?}");
}
```

### 4. Generated projections

For Makefile and CLI, replace manual authoring with generation from the
global DAG. The `makegen` tool already does this for tool targets; extend to
workflow targets.

## Files

- `gunbc-app/src/workflow/` — Projection trait, GlobalDag
- `gunbc-app/src/makegen/` — MakefileProjection
- `gunbc-app/tests/` — drift tests

## References

- `docs/design/modeling/m17-global-flattening.md` — prerequisite
- `gunbc-app/src/makegen/registry.rs` — existing makegen derivation
