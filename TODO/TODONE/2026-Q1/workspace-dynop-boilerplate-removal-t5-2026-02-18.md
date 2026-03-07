# T5 Completion: Workspace/FileOps Boilerplate Removal via DynOp

Date: 2026-02-18
Task: `T5`

## What Changed

### 1) Removed `WorkspaceOp` enum layer

Files:
- Deleted: `gunbc-app/src/workspace/ops.rs`
- Updated: `gunbc-app/src/workspace/mod.rs`

Changes:
- Replaced enum-based dispatch (`WorkspaceOp` + large `Executable` match + `From` impls)
  with a direct type alias:
  - `pub type WorkspaceOp = gunbc_exec::DynOp`
- Removed the `WorkspaceOp::Dynamic` bridge pattern and enum variant mapping surface.

### 2) Removed `FileOpsGraph<T>` generic wrapper

Files:
- Deleted: `gunbc-app/src/file_ops_graph.rs`
- Updated: `gunbc-app/src/lib.rs`
- Updated: `gunbc-app/src/testgen_dag/graph.rs`
- Updated: `gunbc-app/src/fs_env.rs`

Changes:
- `TestgenGraphOp` now aliases `DynOp`.
- Testgen graph construction now wraps concrete ops with `DynOp::new(...)`.
- Fs-env helper tests no longer depend on `FileOpsGraph`.
- Removed `file_ops_graph` module export/re-export from crate root.

### 3) Rewired workspace subdag builders to DynOp

Files:
- `gunbc-app/src/workspace/subdags/build.rs`
- `gunbc-app/src/workspace/subdags/ci.rs`
- `gunbc-app/src/workspace/subdags/codegen.rs`
- `gunbc-app/src/workspace/subdags/docgen.rs`
- `gunbc-app/src/workspace/subdags/pragma.rs`
- `gunbc-app/src/workspace/subdags/bootstrap.rs`
- `gunbc-app/src/workspace/subdags/makegen.rs`
- `gunbc-app/src/workspace/subdags/deps.rs`
- `gunbc-app/src/workspace/subdags/gist.rs`
- `gunbc-app/src/workspace/subdags/dag_viz.rs`
- `gunbc-app/src/workspace/subdags/clippy.rs`
- `gunbc-app/src/workspace/subdags/languages.rs`
- `gunbc-app/src/workspace/subdags/testgen.rs`

Changes:
- DSL-backed subdags now embed `Dag<DynOp>` directly (no convert-to-WorkspaceOp layer).
- Manual workspace subdags now construct nodes with `DynOp::new(...)` instead of enum variants.
- Clippy subdag moved to `build_clippy_graph(...)` (executable op wrapper path), avoiding raw `CliToolOp` dispatch.
- Languages subdag uses a tiny `LanguageExecOp` adapter for non-executable `LanguageOp` metadata nodes.

## Validation

- `cargo check -p gunbc-app`
- `cargo test -p gunbc-app --lib workspace::subdags:: -- --nocapture`
- `cargo test -p gunbc-app --lib fs_env::tests::add_fs_env_root_node_uses_standard_shape -- --nocapture`
- `cargo test -p gunbc-app --lib testgen_dag::graph::tests:: -- --nocapture`
