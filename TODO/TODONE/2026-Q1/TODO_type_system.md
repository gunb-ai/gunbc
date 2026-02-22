# Type System Evolution (Remaining)

**Status**: Completed
**Date**: 2026-02-04
**Updated**: 2026-02-05

Remaining work for the type-system roadmap. Completed items were moved to
`TODO/TODONE/2026-Q1/TODO_type_system.md`.

## Completed Tasks (2026-02-07)

1) **Structural type DAG comparison in ports**
- `TypeRegistry::is_compatible` now checks structural refinement (cardinality,
  base type, predicate entailment) with `Any` handling.
- `DagBuilder` and `validate_subdag_interfaces` use registry-backed compatibility.
- Added `TypeRegistry::with_core_types()` for common refined types (Url, FilePath, etc.).

2) **Wire `contract::witnesses()` into testgen**
- Testgen mock values now prefer contract-derived witnesses when types are registered.
- Minimal input generation and cardinality coverage tests use witness values.

## Notes

- Design principles and completed milestones live in `TODO/TODONE/2026-Q1/TODO_type_system.md`.
