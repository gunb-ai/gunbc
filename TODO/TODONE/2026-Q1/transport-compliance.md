# Transport Compliance Restructure

**Status**: Completed
**Date**: 2026-01-30

## Goal

Make all I/O go through the transport layer (`lib/transport`), enabling:
- Proper dry-run interception for all I/O operations
- Complete mocking at the transport boundary
- Observable I/O in DAG execution

## What Was Done

### Phase 1: Deleted Noncompliant Primitives

Removed from `lib/primitives/src/io.rs`:
- `ReadFileOp` - direct `fs::read_to_string()`
- `WriteFileOp` - direct `fs::write()`
- `ExecuteOp` - direct `Command::new().output()`
- `ListFilesOp` - direct `Command::new("git")` + `fs::read_dir()`
- `ReadFilesOp` - direct `fs::read_to_string()` in loop

Also removed helper functions: `list_files_impl()`, `list_files_recursive()`

Removed `#![allow(clippy::disallowed_methods)]` from primitives crate - no longer needed.

### Phase 2-7: Updated All Tools

| Tool | Changes |
|------|---------|
| **gist** | Replaced `ListFilesOp`/`ReadFilesOp` with transport-based implementations using `ShellRequest("git ls-files")` and `FileRequest::read` |
| **deps** | Replaced `ReadFileOp`/`ExecuteOp` with `execute_transport()` calls |
| **ci** | Replaced `run_config_command()` helper to use `execute_transport()` instead of `ExecuteOp.execute()` |
| **buck2** | Replaced `ReadFileOp.execute()` and `.exists()` calls with `execute_transport()` |
| **makegen** | Replaced `.exists()` calls in `needs_codegen()` with `FileRequest::exists` via transport |
| **bootstrap** | Replaced `std::fs::read_dir` with `ShellRequest("find")` via transport |

## Design Pattern Used

All tools now use `execute_transport()` from `gunbc_lib_transport`:

```rust
// Before (noncompliant):
let read_result = ReadFileOp.execute(inputs)?;

// After (compliant):
let request = TransportRequest::File(FileRequest::read(&path));
let response = execute_transport(&request)?;
```

## Infrastructure Already Present

The codebase already had the correct infrastructure:

1. **DryRun interception** - `is_transport_execution_node()` in `core/exec/src/execute.rs` checks for `TransportRequest` inputs
2. **All Prepare* ops** - `PrepareFileReadOp`, `PrepareFileWriteOp`, `PrepareFileExistsOp`, `PrepareShellOp`, `PrepareDirectoryListOp`
3. **TransportOps::Execute outputs** - Shell: stdout/stderr/exit_code/success; File: content/written_path
4. **Clippy enforcement** - `clippy.toml` already bans `std::fs::*` and `Command::new`
5. **Pattern builders** - `UpsertBuilder`, `TransactionBuilder`, `LoopBuilder` in `core/ir/src/patterns/`

## What's Not Graph-Exposed Yet

The current implementation uses `execute_transport()` **inside** opaque nodes. This routes I/O through the transport layer but doesn't expose it as graph-level nodes.

For full DryRun interception at the graph level, a future refactor could:
1. Make domain ops pure (return `TransportRequest`)
2. Add explicit `TransportOps::Execute` nodes in graphs
3. Use `UpsertBuilder`/`LoopBuilder` patterns where appropriate

See: `TODO/TODONE/graph-level-transport.md` for remaining work.

## Files Changed

- `lib/primitives/src/io.rs` - Deleted noncompliant ops, kept only Prepare* ops
- `lib/primitives/src/lib.rs` - Removed deleted exports and enum variants
- `lib/tools/gist/src/graph.rs` - Transport-based ListFiles/ReadFiles
- `lib/tools/deps/src/ops.rs` - Transport-based file read and shell execution
- `lib/tools/ci/src/ops.rs` - Transport-based command execution
- `lib/tools/buck2/src/ops.rs` - Transport-based file read and exists checks
- `lib/tools/makegen/src/registry.rs` - Transport-based exists checks
- `lib/tools/bootstrap/src/ops.rs` - Transport-based directory listing
- `lib/tools/deps/Cargo.toml` - Added gunbc-lib-transport dependency
- `lib/tools/ci/Cargo.toml` - Added gunbc-lib-transport dependency

## Notes

### Why Not Full Graph Exposure Now?

Full graph exposure would require significant restructuring of each tool's graph to:
1. Add `PrepareShellOp`/`PrepareFileReadOp` nodes
2. Add `TransportOps::Execute` nodes
3. Wire the `TransportRequest` through edges

The current approach achieves the primary goal (all I/O through transport) with minimal disruption. The graph-level exposure can be done incrementally.

### Remaining Allowances

After this refactor, only two crates need `#![allow(clippy::disallowed_methods)]`:
- `lib/transport/` - Legitimate I/O boundary (executes the requests)
- `core/codegen/` - Bootstrap code, can't use transport (chicken/egg)

The following allowances were REMOVED:
- `lib/primitives/` - No longer has direct I/O ops
