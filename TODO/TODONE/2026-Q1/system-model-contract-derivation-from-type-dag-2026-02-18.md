# R3 Completion: Contract Derivation From Type DAG Markers

Date: 2026-02-18
Task: `R3`

## Implemented

- Updated `derive_contract_test_specs(models)` to derive behavior properties
  from registered behavior-type DAG nodes:
  - scans `TypeOp::Validate(Predicate::Custom("property:<Property>"))`
  - maps markers back to `Property` variants
  - drives phase selection (`Check`, `Create`, `Resolve`) from derived markers
- Keeps a compatibility fallback to in-struct `behavior.properties` if behavior
  DAG registration fails.

## File Changes

- `core/ir/src/system_model.rs`
  - `derive_contract_test_specs` now uses type DAG predicate markers
  - added helpers:
    - `behavior_properties_from_type_dag`
    - `parse_property_marker`
  - added test:
    - `derive_contract_specs_uses_property_markers_from_behavior_type_dag`

## Validation

- `cargo test -p gunbc-ir system_model -- --nocapture` passes.
