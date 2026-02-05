# Consolidation: Completed Tasks

**Status**: Done
**Date**: 2026-02-05

Tasks completed from `TODO/consolidation.md`. Original doc retained for
reference material (patterns, architecture notes).

---

## High priority (widespread, low effort)

- [x] Add `require_str`, `require_json`, `optional_str` input helpers (80 call sites) — `core/exec/src/helpers.rs`
- [x] Add `OutputMap` builder (164 call sites) — `core/exec/src/helpers.rs`
- [x] Add `TransportResponseExt::require_shell/rest/file` methods (51 call sites) — `core/exec/src/helpers.rs`
- [x] Add `ShellRequest::into_transport_request()` — `core/ir/src/transport/mod.rs`

## High priority (new — widespread, low effort)

- [x] Add `propagate_skipped` helper (8 call sites across 3 files) — §12
- [x] Add `ExecError::context()` + `ResultExt` trait (27 call sites across 10 files) — §13
- [x] Add `ShellResponse::ok()` / `ShellResponse::failed()` constructors (39 sites, 16 files) — §14
- [x] Add `From<ShellRequest> for TransportRequest` (+ FileRequest, RestRequest, etc.) — all 5 request types + 5 response types

## High priority (migrations)

- [x] Add `InputsExt` trait to replace free-function `require_*/optional_*` calls — shrinks import lists
- [x] Migrate call sites to use `propagate_skipped` (8 sites, 3 files)
- [x] Migrate call sites to use `ShellResponse::ok()/failed()` + `From` impls (graph_mock, bin, ops, tests)
- [x] Migrate remaining call sites to use `ExecError::context()` / `ResultExt` / `IntoExecResult` (~42 sites, 12 files)

## Medium priority

- [x] Migrate remaining raw `HashMap::new()` output construction to `OutputMap` — §15
  - Migrated: `lib/tools/cargo/src/ops.rs` (3 sites), `lib/tools/deps/src/graph.rs` (2 sites), `lib/primitives/src/collection.rs` (1 site)
  - Remaining (acceptable): `core/ir/src/transport/cli.ts` (can't use OutputMap — core/ir doesn't depend on core/exec), test helpers
- [x] Add `assert_boundaries` test helper to `core/test` (8 files) — §11
- [x] Collapse `default_mock_for_type` into `cardinality_case_mock_value` in testgen codegen — single type→mock mapping
- [x] Extract `hash_finding_id` to `lib/primitives` as `StableHashOp` — §1
- [x] Unify blob hash with review hash (both should use SHA256) — §1
- [x] Extract `FormatDiffArtifact` to `lib/primitives` as `FormatMapOp` — §1

## Lower priority (codebase fragility)

- [x] Eliminate `type_id == "List"` dual encoding — already resolved: loop_pattern uses element type + cardinality, debug_assert guards CliEntrypoint
- [x] Replace string-based builder references with enum keys — `GraphBuilderId` enum in `cli_gen.rs`
- [x] Extract `buck-out/gen` to a single constant — was `target/codegen`, constants already exist, migrated CI ops
- [x] Fix CODEGEN_SOURCES hardcoded path list — already resolved per `TODO/TODONE/TODO_hacks_general.md`

---

## Summary of Infrastructure Added

### `core/exec/src/helpers.rs`
- `InputsExt` trait: `inputs.require_str("key")`, `inputs.optional_bool("key")`
- `OutputMap` builder: `OutputMap::new().str("k", v).bool("b", true).ok()`
- `TransportResponseExt`: `response.require_shell()?`
- `propagate_skipped()`: propagate Value::Skipped to all outputs

### `core/exec/src/error.rs`
- `IntoExecResult` trait: `.exec_context("msg")`, `.with_exec_context(|| format!(...))`

### `core/ir/src/transport/mod.rs`
- `From<ShellRequest> for TransportRequest` (and 4 other request types)
- `From<ShellResponse> for TransportResponse` (and 4 other response types)
- `ShellResponse::ok()`, `ShellResponse::failed()`

### `core/test/src/mock_spec.rs`
- `assert_boundaries(spec, &[("node", "port")])`
- `assert_transport_mocks(spec, &[("node", "port")])`

### `lib/primitives/src/data.rs`
- `StableHashOp`: SHA256-based stable ID generation
- `FormatMapOp`: structured map → formatted text rendering

### `core/codegen/src/cli_gen.rs`
- `GraphBuilderId` enum: compile-time safe graph builder references
