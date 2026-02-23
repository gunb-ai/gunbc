# M3 Completion: Codegen DSL vs Hand-Built Behavior Parity

Date: 2026-02-18
Task: `M3`

## What Was Added

### Parity test

File: `gunbc-dag/src/dsl_builder.rs`

- Added test:
  - `codegen_dsl_matches_hand_built_dry_run_behavior`
- Test builds:
  - hand-built graph: `build_codegen_graph()`
  - DSL graph: `build_codegen_graph_dsl()`
- Runs both in DryRun with deterministic mocks.
- Asserts semantic parity:
  - `success` behavior matches
  - `ran` behavior matches

## Runtime Mapping/Compat Needed for DSL Path

### Resolver mappings

File: `gunbc-dag/src/resolve.rs`

- Added executable adapters for:
  - `service_transport::prepare::shell.Codegen::Check`
  - `service_transport::parse::shell.Codegen::Check`
  - `service_transport::prepare::shell.Codegen::Run`
  - `service_transport::parse::shell.Codegen::Run`
- Added `IdentityCallableOp` and mapped `tools.codegen::codegen` to it for
  entrypoint wrapper compatibility.
- Added resolver test:
  - `resolve_services_shell_codegen_transport_ops`

### API compatibility wrappers

Files:
- `gunbc-dag/src/codegen/graph.rs`
- `gunbc-dag/src/codegen/mod.rs`
- `gunbc-dag/src/ci/graph.rs`
- `gunbc-dag/src/ci/mod.rs`

- Restored compatibility exports/wrappers for:
  - `build_codegen_graph_with_mode(...)`
  - `build_ci_graph_with_mode(...)`

## Validation

- `cargo test -p gunbc-dag --lib resolve_services_shell_codegen_transport_ops -- --nocapture`
- `cargo test -p gunbc-dag --lib codegen_dsl_matches_hand_built_dry_run_behavior -- --nocapture`
