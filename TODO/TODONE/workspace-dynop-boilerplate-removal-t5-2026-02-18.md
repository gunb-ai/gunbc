# T5 Completion: Workspace/FileOps Boilerplate Removal via DynOp

Date: 2026-02-18
Task: `T5`

## What Changed

### 1) Removed `WorkspaceOp` enum layer

Files:
- Deleted: `gunbc-dag/src/workspace/ops.rs`
- Updated: `gunbc-dag/src/workspace/mod.rs`

Changes:
- Replaced enum-based dispatch (`WorkspaceOp` + large `Executable` match + `From` impls)
  with a direct type alias:
  - `pub type WorkspaceOp = gunbc_exec::DynOp`
- Removed the `WorkspaceOp::Dynamic` bridge pattern and enum variant mapping surface.

### 2) Removed `FileOpsGraph<T>` generic wrapper

Files:
- Deleted: `gunbc-dag/src/file_ops_graph.rs`
- Updated: `gunbc-dag/src/lib.rs`
- Updated: `gunbc-dag/src/testgen_dag/graph.rs`
- Updated: `gunbc-dag/src/fs_env.rs`

Changes:
- `TestgenGraphOp` now aliases `DynOp`.
- Testgen graph construction now wraps concrete ops with `DynOp::new(...)`.
- Fs-env helper tests no longer depend on `FileOpsGraph`.
- Removed `file_ops_graph` module export/re-export from crate root.

### 3) Rewired workspace subdag builders to DynOp

Files:
- `gunbc-dag/src/workspace/subdags/build.rs`
- `gunbc-dag/src/workspace/subdags/ci.rs`
- `gunbc-dag/src/workspace/subdags/codegen.rs`
- `gunbc-dag/src/workspace/subdags/docgen.rs`
- `gunbc-dag/src/workspace/subdags/pragma.rs`
- `gunbc-dag/src/workspace/subdags/bootstrap.rs`
- `gunbc-dag/src/workspace/subdags/makegen.rs`
- `gunbc-dag/src/workspace/subdags/deps.rs`
- `gunbc-dag/src/workspace/subdags/gist.rs`
- `gunbc-dag/src/workspace/subdags/dag_viz.rs`
- `gunbc-dag/src/workspace/subdags/clippy.rs`
- `gunbc-dag/src/workspace/subdags/languages.rs`
- `gunbc-dag/src/workspace/subdags/testgen.rs`

Changes:
- DSL-backed subdags now embed `Dag<DynOp>` directly (no convert-to-WorkspaceOp layer).
- Manual workspace subdags now construct nodes with `DynOp::new(...)` instead of enum variants.
- Clippy subdag moved to `build_clippy_graph(...)` (executable op wrapper path), avoiding raw `CliToolOp` dispatch.
- Languages subdag uses a tiny `LanguageExecOp` adapter for non-executable `LanguageOp` metadata nodes.

## Validation

- `cargo check -p gunbc-dag`
- `cargo test -p gunbc-dag --lib workspace::subdags:: -- --nocapture`
- `cargo test -p gunbc-dag --lib fs_env::tests::add_fs_env_root_node_uses_standard_shape -- --nocapture`
- `cargo test -p gunbc-dag --lib testgen_dag::graph::tests:: -- --nocapture`
