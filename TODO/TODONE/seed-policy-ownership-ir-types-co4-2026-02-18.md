# CO4 Completion: Seed Policy Ownership in IR Types

Date: 2026-02-18
Task: `CO4`

## Change Summary

Moved seed-policy context logic from testgen-local code into IR types, so policy ownership is centralized in `core/ir`.

### Added to `core/ir/src/types.rs`

- New public context enum:
  - `SeedContext::{RealSingleNode, Scenario, LiveFlow}`
- New context-aware APIs:
  - `seed_placeholder_policy_for_type_id_in_context(type_id, context)`
  - `requires_explicit_seed_for_type_id(type_id, context)`
- New `TypeId` methods:
  - `seed_placeholder_policy_for_context(context)`
  - `requires_explicit_seed(context)`
- Added test:
  - `test_seed_placeholder_policy_in_context`

### Re-exported from `core/ir/src/lib.rs`

- `SeedContext`
- `seed_placeholder_policy_for_type_id_in_context`
- `requires_explicit_seed_for_type_id`

### Updated `core/codegen/src/testgen/codegen.rs`

- Removed local seed-policy matrices and context policy logic.
- Delegated to IR-owned APIs:
  - `seed_placeholder_policy_for_type_id`
  - `seed_placeholder_policy_for_type_id_in_context`
  - `requires_explicit_seed_for_type_id`
- Kept local helper function names as thin wrappers for call-site stability.

## Validation

- `cargo test -p gunbc-ir test_seed_placeholder_policy_in_context -- --nocapture`
- `cargo test -p gunbc-codegen seed_matrix -- --nocapture`
- `cargo test -p gunbc-codegen optional_inputs_require_explicit_semantic_seed -- --nocapture`
