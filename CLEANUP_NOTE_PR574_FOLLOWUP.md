# Cleanup note — public Dag test-surface debt (follow-up to PR #574)

Status: **standby**. Do not start until PR #574 (`session/sleek-deer-523`,
"Worker 7") merges to main. Nothing in this note is actionable against the
current main (`2a653c41c`) or this branch (`fa91cf7d`) — the surfaces named
below only exist on PR #574's branch.

Source of debt: commit `4e46270b2 "Inline Dag test support into lib"` on
PR #574 — moves the prior `src/v3/compiler/src/dag_test_support.rs` into
`src/v3/compiler/src/lib.rs` as `pub mod dag_test_support` (marked
`#[doc(hidden)]`, but still public API of the crate).

## Public surfaces to remove once #574 lands

In `src/v3/compiler/src/lib.rs`:

- `pub mod dag_test_support` and every wrapper inside it:
  - `alloc_port_with_shape`
  - `push_value`
  - `push_transform`
  - `push_bind`
  - `push_branch`
  - `push_loop`

These exist solely so external `tests/` crates can drive the otherwise
`pub(crate)` `Dag` builder. `#[doc(hidden)]` hides them from rustdoc but
they are still on the crate's public API surface — that is the debt.

In `src/v3/compiler/src/dag/builder.rs`:

- Audit each `pub(crate) fn` the wrappers call (`alloc_port_with_shape`,
  `push_value`, `push_transform`, `push_bind`, `push_branch`, `push_loop`).
  After the shim is gone, confirm each is still `pub(crate)` (not widened)
  and that no other external caller appeared.

## Tests to move in-crate (structural claims)

These currently live in `tests/integration/` only because they need direct
Dag construction. After the shim is deleted, they cannot compile from an
external test crate. Re-home as `#[cfg(test)] mod tests` blocks beside the
implementation they exercise (so `pub(crate)` access is legal):

- `tests/integration/m1_substrate_test.rs` — substrate construction
  invariants. Belongs next to `dag/builder.rs` (or whichever module owns
  the substrate predicates it asserts).
- `tests/integration/m1_3_lens_cost_test.rs` — structural cost-lens
  cases over hand-built Dags. Belongs next to the cost lens impl.
- `tests/integration/m1_3_lens_unused_parameters_test.rs` — structural
  unused-parameter lens cases over hand-built Dags. Belongs next to
  `lens_unused_parameters.rs`.

For each file, split case-by-case:

- "given this Dag, this lens reports X" → in-crate unit test.
- "compile this source, observe emitted Rust / diagnostic / rustc result"
  → leave in `tests/integration/` (or `tests/boundary/`).

Coverage rule: every invariant currently asserted must still be asserted
after the move. Audit by name, not by file count.

## Integration receipts that should remain in `tests/`

Keep tests in `tests/integration/` (or `tests/boundary/`) only when their
claim is a cross-boundary receipt that an in-crate test cannot make
honestly:

- end-to-end source → compile → emit pipelines (anything driven from
  `compile_for_test` / parse fixtures, not from a hand-built Dag);
- emitted-Rust / emitted-Python / emitted-Go textual receipts;
- rustc-harness round-trips on emitted code;
- diagnostic-count ratchets and census tests
  (`sg0_census_test`, `sg6_hand_authored_census_test`, etc.);
- fixture regressions (`four_fixture_regression_test`,
  `thesis_validation_test`, lane2/lane3 stage tests).

These do not touch the builder shim and are unaffected.

## Acceptance check (run after #574 lands and the cleanup is applied)

- `rg 'dag_test_support'` → no hits.
- `rg 'pub fn' src/v3/compiler/src/dag/builder.rs` → no builder method is
  `pub` (only `pub(crate)` or private).
- The three named test files either no longer exist in
  `tests/integration/` or contain only boundary/emit cases.
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo clippy --all-targets -- -D warnings` clean.
- Diagnostic-ratchet tests unchanged.

## Out of scope (do not do during cleanup)

- No new builder features.
- No widening of any builder method's visibility "to make the move easier."
- No new test taxonomy / directory reshuffle beyond moving the three named
  files.
- No changes to lens or compiler semantics. If a relocated test starts
  failing, that is a signal — investigate, don't paper over.
