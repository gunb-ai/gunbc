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

## Test cleanup

- [x] Delete 16 redundant `graph_mock.rs` Pattern A (boundary presence) and Pattern D
  (resource acquire) tests — testgen generates equivalent tests. 8 `graph_mock.rs` files
  reduced from 49 to ~33 hand-written tests.
- [x] Migration details for §13: `IntoExecResult::exec_context()` in `core/exec/src/error.rs`.
  Migrated `lib/llm-ops/src/lib.rs` (4 sites), `lib/tools/cargo/src/ops.rs` (1),
  `gunbc-app/src/ci/graph.rs` (1). Remaining sites lower-urgency.
- [x] Migration details for §14: `ShellResponse::ok()/failed()` migrated across 3 binaries
  (codegen, bootstrap, build), gist graph_mock (6), deps graph_mock (1), gist graph (1),
  build/ops tests (2), codegen registry templates (~10). Only `executor.rs:422` kept raw.
- [x] Migration details for §15: `gunbc-app/src/build/ops.rs` (7 sites) migrated to `OutputMap`.

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

## Integration Tests Added

All tests are hermetic (no network, use temp dirs, clean up after).

### `lib/transport/src/executor.rs`
- File transport: Read, Write, Append, Delete, Exists, CreateDir (15 tests)
- Shell transport: echo, stdin, stderr, exit codes, env vars, cwd (11 tests)
- Git transport: ls-files, current-branch, diff, diff-name-only, merge-base, pathspecs (6 tests)
- Dispatch: execute_transport routing tests (2 tests)

### `core/ir/src/transport/cli.rs`
- WhichResolver: finds git, fails for nonexistent (2 tests)
- MockResolver: configured paths, unconfigured failure (2 tests)
- resolve_tool_path_with: both resolvers (2 tests)
- ToolHandle: acquire, mock (2 tests)
- get_tool_by_id: lookup (1 test)
- CliToolError: display (1 test)
