## Engineering Standards

These serve sustainability indirectly by reducing the blast radius of
changes:

- **Clear interfaces.** Every public module should have a small,
  well-defined API surface. Prefer returning values over mutating
  shared state.

- **Pure core logic.** Deterministic functions from inputs to outputs.
  Side effects (filesystem, network, process spawning) belong at the
  edges, not in the middle of computation.

- **Documented I/O boundaries.** Any function that performs I/O must
  document that fact in its signature or doc comment. Callers should
  never be surprised by hidden I/O.

- **No flags in codegen.** Boolean flags that change compilation behavior
  globally (like `force_clone`) are forbidden. Every compilation decision
  must be derived from the actual type and context of the expression
  being compiled, not from a global check. Flags silently degrade and
  are impossible to remove incrementally.

