# tests/unit/

**Layer:** unit (TESTING.md § test layers — <5ms targets).

Unit tests exercise a single lens, accessor, single-pass behavior, or
typed handle against a **hand-constructed minimal `Dag`**. They must
not compile source text end-to-end; compile-pipeline tests belong in
`tests/integration/` (or `tests/boundary/` for target-language
roundtrips).

## Current state (Phase 0)

Empty on initial landing. Two placement tracks feed this directory:

1. **In-crate `#[cfg(test)] mod tests`** — the primary home for unit
   tests today, per TESTING.md § "Mocks and dependency injection".
   `pub(crate)` builder helpers like `Dag::alloc_port` are reachable
   from inside `src/v3/compiler/src/`. Most lenses already host their
   tests this way.

2. **`tests/unit/*.rs`** — reserved for unit tests whose subject is
   the **public** `Dag` builder surface. Activation depends on the
   tracked follow-up in TESTING.md: landing `push_value`,
   `push_transform`, `push_bind`, … helpers as public (or
   `pub(crate)`-with-test-harness-access) APIs. Until that surface
   exists, unit tests stay in-crate.

## Adding a file here

Each test file carries a header:

```rust
//! **Layer:** unit (TESTING.md § test layers — <brief-subject>).
```

Include it from `tests/integration.rs` via a `#[path =
"unit/<name>.rs"] mod …` declaration. The consolidated binary shares
one bootstrap across all three directories; do not introduce a
separate `tests/unit.rs` binary without director sign-off (see
`tests/integration/sg0_census_test.rs`).

Tests whose wall-clock exceeds 2s per `#[test]` violate the ratchet
at `scripts/check-test-timeout.sh`. A unit test reaching that bar
almost certainly belongs in `tests/integration/` or is doing full-
pipeline work that should be mocked (TESTING.md § 5).
