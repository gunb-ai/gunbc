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

## Testing Invariants

- **Behavioral only.** Tests assert observable behavior — outputs given inputs,
  error messages, public API contracts. Never assert internal implementation
  details like which private functions were called, what order internal steps
  execute in, or how many times an internal helper runs.

- **Hermetic unit tests only.** Tests must not touch the filesystem, network,
  or environment. All external dependencies are injected or mocked. A test
  that passes on one machine must pass on every machine.

- **No tautological tests.** A test that mirrors the implementation — restating
  the production code in test form — proves nothing. Tests must encode an
  independent specification of *what* the code should do, not *how* it does it.
  If deleting the test body and replacing it with a copy of the production code
  would still pass, the test is tautological.
