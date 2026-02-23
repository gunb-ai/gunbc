# T6 Completion: daglang-exec-bridge Simplification

Date: 2026-02-18
Task: `T6`

## What Changed

### 1) Removed bridge-only runtime classifier API from `daglang-lower`

File: `core/daglang/daglang-lower/src/lib.rs`

- Deleted `RuntimeOpId` enum.
- Deleted `classify_runtime_op(&LoweredOp) -> Option<RuntimeOpId>`.
- Kept lower-layer APIs focused on lowering metadata and obligation/service classification.

### 2) Inlined runtime handler classification in `daglang-emit`

File: `core/daglang/daglang-emit/src/rust_exec_runtime.rs`

- Removed dependency on `daglang_lower::{RuntimeOpId, classify_runtime_op}`.
- Added direct `LoweredOp` pattern matching inside `classify_handler` for:
  - makegen runtime callables (`load_registry`, `fs_env`, `render_makefile`, `makegen`, and content-upsert chain nodes)
  - collection ops
- Removed `HandlerKind::from_runtime_id` indirection.

### 3) Bridge crate removal status

Files removed in this task stream:

- `core/daglang/daglang-exec-bridge/Cargo.toml`
- `core/daglang/daglang-exec-bridge/src/lib.rs`

This leaves runtime execution codegen routed directly through `daglang-emit` handler classification.

## Validation

- `cargo test -p daglang-emit rust_exec_runtime::tests:: -- --nocapture`
- `cargo check -p daglang-lower`
