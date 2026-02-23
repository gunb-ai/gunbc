# R4 Completion: Store Mapping via Structural DAG Equivalence

Date: 2026-02-18
Task: `R4`

## Implemented

- Replaced `validate_store_behavior_mapping()` logic from simple operation-name
  presence checks to structural behavior-contract equivalence:
  - validates required operation ids exist (`get_object`, `put_object`,
    `list_objects`, `delete_object`)
  - registers behavior DAGs for `gcp.gcs` and `aws.s3`
  - derives comparable behavior shapes from DAG marker nodes
  - compares per-operation structure (properties, input/output contracts,
    optional wrappers)

## New Helpers

- `behavior_contract_shape`
- `parse_input_marker`
- `parse_output_marker`

## Tests Added

- `validate_store_behavior_mapping_accepts_structurally_equivalent_models`
- `validate_store_behavior_mapping_rejects_structural_mismatch`

## Validation

- `cargo test -p gunbc-ir system_model -- --nocapture` passes.
