# src/ — Compiler and Runtime

All Rust crates live here. The DSL compiler pipeline, execution engine,
IR, and runtime operations.

## Coding Standards

Adapted from Google C++ style for Rust:

- **Clear interfaces.** Every public module should have a small, well-defined
  API surface. Prefer returning values over mutating shared state.

- **Pure libraries and helper functions.** Core logic should be pure —
  deterministic functions from inputs to outputs. Side effects (filesystem,
  network, process spawning) belong at the edges, not in the middle of
  computation.

- **I/O documentation at interface boundaries.** Any function that performs I/O
  (reads files, spawns processes, makes network calls) must document that fact
  in its signature or doc comment. Callers should never be surprised by hidden
  I/O.

- **No backdoors.** The compiler should provide all metadata (tool definitions,
  output paths, type registries) through its own output types (`CompileOutput`,
  `InferredEntrypoint`, etc.), not through runtime `extern func` callbacks.
  The `extern func` feature has been eliminated.

- **No hacks or fallbacks.** Every code path either succeeds fully or fails
  with a clear error. No silent degradation: no lossy recovery that discards
  input, no `.ok()` that swallows errors on fallible operations, no `continue`
  that silently drops work, no fallback defaults that produce valid-looking but
  wrong output. If a function cannot complete its job, it must return `Err` —
  not an empty value, not `Value::Skipped`, not a quietly truncated result.
  Caching is the sole exception (cache miss on error is acceptable).
  See `POSTMORTEM.md` T13 for the full taxonomy of the ~70 violations that
  motivated this invariant.

## Crate Map

| Crate | Purpose |
|-------|---------|
| `0_foundation/` | Shared IR, contracts, and foundational macros |
| `1_surfaces/` | CLI, codegen, and workflow-facing entrypoints |
| `2_pipeline/` | Driver/orchestrator: prepare + ordered stage runner |
| `3_source/` | Syntax and module-graph discovery |
| `4_semantics/` | Typechecking and semantic validation |
| `5_graph/` | Lowering to GraphIR |
| `6_artifacts/` | Derived manifests and obligations |
| `7_emit/` | Code emission |
| `8_materialize/` | Runtime wiring, resolve, primitives, blob, transport |
| `9_execute/` | DAG execution engine |
| `10_test/` | Test utilities, generated tests, registry crates |
