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

## Crate Map

| Crate | Purpose |
|-------|---------|
| `daglang/` | Compiler pipeline: parse → typecheck → lower → emit |
| `ir/` | Core IR types, DAG structure, patterns, transport model |
| `exec/` | DAG execution engine |
| `resolve/` | LoweredOp → DynOp resolution, builder, dry-run |
| `codegen/` | CLI and test code generation |
| `infra/` | Leaf crate: hashing, manifests, freshness, workspace model |
| `cli/` | CLI argument parsing and dispatch |
| `test/` | Test utilities |
| `testgen-registry/` | Testgen target metadata registry |
| `workflow/` | Workflow planner |
| `primitives/` | Leaf DAG operations: parse, extract, format, map, filter, fold |
| `blob/` | Blob content acquisition: inline, file, git, S3, HTTP |
| `transport/` | I/O boundary — the only crate that performs direct I/O |
