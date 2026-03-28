# v2 Compiler Design Principles

These are the governing design decisions for the self-hosted compiler.
For current status, phases, and workboards, see `ROADMAP.md`.
For invariants, see `INVARIANTS.md`.

## Core claim

The compiler is a pure transform: `.dag source → backend artifacts`.
Five stages, each a pure function:

```
.dag source → tokenize → parse → resolve → typecheck → emit → output files
```

No interpreter in the pipeline. Interpretation is a downstream concern.

## Types are values, not references

Types are structural values that flow through the pipeline. No registry,
no deferred lookup. The structure IS the type. A `List<Span>` is
structurally represented, not stored as a string `"List<Span>"`.

## The compiler is just another .dag program

The v2 compiler imports from `std.types` — the same shared type library
that all DSL programs use. The compiler's own invariants are enforced by
the compiler itself.

## fn vs func

- `fn` = pure function (expression body, no side effects)
- `func` = workflow function (may use resources, services, transport)

## Testing: the compiler owns its downstream

When the compiler emits Rust, it also emits Rust tests. The tests
compile and run in the target language. If the compiler produces broken
output, `cargo test` on the emitted tests catches it.

## Diagnostics are first-class

Every stage returns `{ value, diagnostics }`. Diagnostics thread through
the pipeline. No diagnostic loss across stage boundaries.
