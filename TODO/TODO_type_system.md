# Type System Evolution (Remaining)

**Status**: In Progress
**Date**: 2026-02-04
**Updated**: 2026-02-05

Remaining work for the type-system roadmap. Completed items were moved to
`TODO/TODONE/TODO_type_system.md`.

## Remaining Tasks

1) **Structural type DAG comparison in ports**
- Ports still reference types by `TypeId` (string) + registry lookup.
- Goal: ports carry or reference the type DAG directly, and compatibility
  becomes structural subsumption (refinement) rather than name matching.
- Unblocks: Url satisfies String, predicate-aware compatibility.

2) **Wire `contract::witnesses()` into testgen**
- Replace ad-hoc mock generation with contract-derived witnesses so tests
  always reflect declared type predicates + wrappers.
- This removes the last hardcoded cardinality-case tables from testgen.

## Notes

- Design principles and completed milestones live in `TODO/TODONE/TODO_type_system.md`.
