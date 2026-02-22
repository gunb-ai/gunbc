# CO3 Completion: Probe-Observer Single-Source Bundle

Date: 2026-02-18
Task: `CO3`

## Change Summary

Consolidated probe-observer bundle ownership into one module.

### Centralized in `core/codegen/src/testgen/probe_observer.rs`

- Added `ProbeObserverBundle`:
  - `analysis: ProbeObserverAnalysis`
  - `report: String`
  - `lowering_error: Option<String>`
- Added `ProbeObserverBundle::has_coverage()`.
- Added `build_probe_observer_bundle(dag, spec, analysis)`:
  - prefers lowered DAG analysis (`gunbc_exec::lower`)
  - falls back to original DAG analysis on lowering error
  - always returns analysis + report + lowering diagnostics

### `codegen.rs` now consumes shared bundle

File: `core/codegen/src/testgen/codegen.rs`

- Removed local duplicate `ProbeObserverBundle` struct/impl.
- Removed local duplicate bundle-build logic.
- Switched generation path to call shared:
  - `build_probe_observer_bundle(self.dag, spec, &analysis)`

### Re-exported via testgen module

File: `core/codegen/src/testgen/mod.rs`

- Added exports:
  - `build_probe_observer_bundle`
  - `ProbeObserverBundle`

## Tests Added

In `core/codegen/src/testgen/probe_observer.rs`:

- `test_build_probe_observer_bundle_single_source`
- `test_probe_observer_bundle_has_coverage_false_when_empty`

## Validation

- `cargo test -p gunbc-codegen probe_observer -- --nocapture`
