# CL3 Completion: IR Modeling Gaps (Documented + Targeted Fix)

Date: 2026-02-18
Task: `CL3`

## Gaps Discovered During CL1/CL2

1. Layer-1 exec-runtime emission had narrow runtime classification coverage.
   - `tools.makegen` and `tools.pragma` emitted successfully.
   - modules like `tools.build` failed at emit time with:
     - `no runtime op classification for Callable { module: "tools.build", ... }`
2. Go lint tooling availability in this environment is partial.
   - `go vet` available and used.
   - `golint` / `staticcheck` not installed.

## Fix Landed

### Deferred runtime handler for known-unmapped modules

File: `core/daglang/daglang-emit/src/rust_exec_runtime.rs`

- Added `HandlerKind::DeferredCallable`.
- Updated classification to treat known module families as deferred instead of
  hard failing unresolved:
  - `tools.build`, `tools.codegen`, `tools.bootstrap`, `tools.docgen`,
    `tools.testgen`, `tools.clippy`, `tools.deps`, `pipelines.ci`,
    `shared.dag_util`, `std.patterns`, `std.resources`,
    `services.shell`, `services.cargo`, `services.gcp.secret_manager`,
    `services.gcp.sts`.
- Added deferred handler body that returns explicit runtime error:
  - `"deferred callable is not runtime-mapped yet for exec-runtime generation"`
- Preserved strict failure for truly unknown modules.

### Test coverage

- Added:
  - `emit_exec_runtime_defers_supported_unmapped_modules`
- Existing strictness test still passes:
  - `emit_exec_runtime_rejects_unknown_module`

## Validation

- `cargo test -p daglang-emit rust_exec_runtime -- --nocapture` passes.
