# T4 Completion: Manual graph.rs Builder Cutover to DSL Wrappers

Date: 2026-02-18
Task: `T4`

## What Changed

Replaced hand-written DagBuilder implementations with thin DSL-backed wrappers for:

- `gunbc-app/src/pragma/graph.rs`
- `gunbc-app/src/codegen/graph.rs`
- `gunbc-app/src/makegen/graph.rs`
- `gunbc-app/src/bootstrap/graph.rs`
- `gunbc-app/src/build/graph.rs`
- `gunbc-app/src/docgen/graph.rs`
- `gunbc-app/src/ci/graph.rs`

### Structural updates

- All seven `*GraphOp` types now alias `gunbc_exec::DynOp`.
- `build_*_graph()` functions now delegate to corresponding `build_*_graph_dsl()` helpers.
- Compatibility wrappers kept where API surface expected them:
  - `build_codegen_graph_with_mode(...)`
  - `build_ci_graph_with_mode(...)`
- Existing workflow signature helpers were preserved (`*_signature`) to keep downstream interfaces stable.

### Supporting cleanup

- Removed old per-module runtime-op union enums and manual DagBuilder wiring from these files.
- Replaced per-file structural tests tied to manual node internals with DSL smoke-build tests.

## Result

- Manual graph builders were reduced from ~2,736 lines across these seven files to ~522 lines of wrapper/signature definitions.
- Binaries and public builder APIs continue to work through DSL-backed DAG compilation.

## Validation

- `cargo check -p gunbc-app`
- `cargo test -p gunbc-app --lib builds_ -- --nocapture`
