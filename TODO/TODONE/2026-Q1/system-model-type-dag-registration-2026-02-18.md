# R2 Completion: Register System Behavior Type DAGs

Date: 2026-02-18
Task: `R2`

## Implemented

- Added behavior-type id helper:
  - `system_behavior_type_id(system_id, behavior_id) -> TypeId`
- Added registry integration entrypoint:
  - `register_system_behavior_type_dags(registry, models) -> Result<Vec<TypeId>, String>`
- Added behavior DAG materialization:
  - metadata/property markers as `TypeOp::Validate(Predicate::Custom(...))`
  - optional-input structural marker as `TypeOp::Wrap(WrapperKind::Optional)`
  - input/output type references validated against `TypeRegistry`

## File Changes

- `core/ir/src/system_model.rs`
  - new registration + DAG builder helpers
  - new tests:
    - `register_behavior_type_dags_adds_registry_entries`
    - `register_behavior_type_dags_rejects_unknown_input_type`

## Validation

- `cargo test -p gunbc-ir system_model -- --nocapture` passes.
