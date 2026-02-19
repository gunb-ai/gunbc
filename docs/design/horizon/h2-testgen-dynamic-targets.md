# H2 Design: Testgen Dynamic Targets via Inventory

## Problem

Testgen target lists are manually enumerated. As new `DagSpecDef` entries are added, target wiring drifts and requires repetitive updates.

## Decision

Generate test targets by iterating an inventory of `DagSpecDef` entries at codegen time.

## Proposed Model

- Add a compile-time inventory source: `DagSpecInventory`.
- Add DSL/codegen meta construct: `for_each_spec(inventory_name)`.
- Emit one upsert/test target chain per discovered spec.

Example intent:

```text
for_each_spec("tool_dag_specs") as spec {
  emit test_target(spec.id)
  emit contract_target(spec.id)
}
```

## Invariants

- Inventory iteration order is deterministic (stable sort by `spec.id`).
- Duplicate IDs are rejected at generation time.
- Missing required fields fail generation, never silently skipped.

## Migration Plan

1. Introduce inventory registration helper for `DagSpecDef`.
2. Teach testgen codegen to iterate inventory.
3. Remove hardcoded target lists.
4. Add snapshot tests for generated target set.

## Follow-up Implementation Tasks

- `H2.1` Define inventory registration API for `DagSpecDef`.
- `H2.2` Implement deterministic inventory loader.
- `H2.3` Replace manual target loops in testgen emitter.
- `H2.4` Add duplicate-ID and missing-field failure tests.
- `H2.5` Add snapshot parity tests against current generated targets.
