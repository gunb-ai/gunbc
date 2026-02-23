# R5 Completion: PortType-Driven Rust Type Derivation

Date: 2026-02-18
Task: `R5`

## Implemented

- Removed `rust_type_for_type_id()` indirection in
  `core/ir/src/system_model.rs`.
- Updated contract harness generation to derive Rust types directly from:
  - `PortType::from(type_id)`
  - `rust_type_for_port_type(&port_type, type_id)`

## Validation

- `cargo test -p gunbc-ir system_model -- --nocapture` passes.
