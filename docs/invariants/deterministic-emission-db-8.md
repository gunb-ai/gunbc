## Deterministic emission (Invariant D-1, DB-8)

The v3 emit pipeline (`src/v3/compiler/src/emit.rs` and per-target
`*_target.rs` bodies) must be **deterministic**: for the same `Dag` and
[`EmitTarget`], repeated calls to emit must yield **byte-identical** output.
Non-determinism from `HashMap` / `HashSet` iteration order, embedded
timestamps, absolute host paths in generated source, or platform-dependent
float formatting is a bug. Enforcement is structural: integration tests in
`src/v3/compiler/tests/determinism_test.rs` re-run emit **five times** per
fixture row on the shared matrix (`determinism_fixtures.rs`); Lane 3 Stage 3c’s
`self_host_fixed_point` binary consumes the same contract. Design:
[`docs/design-fixed-point-ratchet.md`](docs/design-fixed-point-ratchet.md).

