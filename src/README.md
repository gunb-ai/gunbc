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

## Tiered Test Execution (T11)

DAG execution tests use three tiers, each proving a different layer of
correctness. Every test explicitly chooses its tier via `ExecutionMode`.

### Tier 1 — DryRun (structure)

All transport, resource-environment, and tool nodes are intercepted with
explicit mocks (`ExecutionMode::DryRun(mocks)`). Pure nodes execute
normally. This tier proves DAG wiring, port cardinality, coercion, guard
evaluation, conditional branching, and topological ordering — without
performing any real I/O. The majority of existing tests operate at this
tier.

### Tier 2 — Selective Real (computation)

The DAG executes in `ExecutionMode::Real`, but the operations themselves
are limited to safe, hermetic effects: reading environment variables,
filesystem operations in temporary directories, timestamps, and
conditional logic. No external HTTP calls or cloud API interactions.
This tier proves that computation within the DAG produces correct
*values*, not just correct *shapes*.

Reference tests: `env_var_read_real_mode`,
`real_mode_executes_resource_environment_node` in
`src/09_execute/exec/src/execute/tests.rs`.

### Tier 3 — Full Real (integration)

All nodes execute for real against live services. Only viable in
controlled environments with sandboxed credentials (CI runners with
scoped tokens, disposable cloud resources). Proves end-to-end behavior
including HTTP transport and cloud API interactions. Not yet implemented;
requires credential injection infrastructure (see `POSTMORTEM.md` T11).
