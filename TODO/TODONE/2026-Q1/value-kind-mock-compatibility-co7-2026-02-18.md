# CO7 Completion: ValueKind-Based Mock Compatibility

Date: 2026-02-18
Task: `CO7`

## What Changed

### 1) Added `ValueKind` on `Value`

File: `core/ir/src/value.rs`

- Added new enum:
  - `ValueKind::{Unit, Bool, String, Int, List, Set, Map, Json, TransportRequest, TransportResponse, Secret, Skipped}`
- Added `Value::kind() -> ValueKind`.
- Added `ValueKind::type_name()` for canonical diagnostic labels.
- Added `Display` for `ValueKind`.
- Added tests:
  - `value_kind_matches_variants`
  - `value_kind_type_name_is_canonical`

### 2) Switched backing compatibility from string labels to `ValueKind`

File: `core/ir/src/types.rs`

- Replaced `ValueBacking::accepts_value_type(&str)` with:
  - `ValueBacking::accepts_value_kind(ValueKind)`
- Added test:
  - `test_value_backing_accepts_value_kind`

### 3) Removed string-manufacturing smell in testgen/mock validation

Files:
- `core/codegen/src/testgen/codegen.rs`
- `core/test/src/mock_requirements.rs`

- Reworked compatibility checks to use:
  - `let actual_kind = value.kind();`
  - `value_backing_for_type_id(expected).accepts_value_kind(actual_kind)`
- Kept mismatch error messages stable by rendering:
  - `actual_kind.type_name()`
- Eliminated `mock_value_type_name` helper in testgen.

### 4) Re-exported new type

File: `core/ir/src/lib.rs`

- Added `ValueKind` to public re-exports.

## Validation

- `cargo test -p gunbc-ir value_kind -- --nocapture`
- `cargo test -p gunbc-test mock_requirements::tests::test_ -- --nocapture`
- `cargo test -p gunbc-codegen testgen::codegen::tests::test_mock_type_compatibility -- --nocapture`
- `cargo test -p gunbc-codegen testgen::codegen::tests::test_input_mock_type_mismatch_detected -- --nocapture`
