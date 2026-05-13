# Gate #87 Lens Cementing Test Discipline Dispatch

Scope: `lens_cementing_test_discipline_complete`.

This brief decomposes the remaining test-discipline cementing work for the
lens-completeness invariant. It is a dispatch artifact, not a new authority:
the structural authorities remain `src/v3/std/verification.dag`,
`src/v3/compiler/tests/dag/cementing_dispatch.dag`, and
`src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`; the prose
mirror remains `docs/v3-lens-capability-register.md`; the reviewer checklist
remains `TESTING.md` under "Cementing tests (Band C -- lens subsumption)".

## Invariant To Preserve

A lens row may read `BEHAVIORALLY COMPLETE` only when the same PR lands the
behavioral receipt for the row's published contract. For a `LensRegistryEntry`
in `src/v3/compiler/regen.dag`, the receipt set must stay mechanically joined
across:

- `src/v3/compiler/regen.dag`
- `docs/v3-lens-capability-register.md`
- `src/v3/std/verification.dag` (`lens_capability_register_rows`)
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag`
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`
- temporary Rust receipts under `src/v3/compiler/tests/integration/cementing/`
  only when the `.dag` predicate carrier cannot yet express the published
  output shape

`CementingDispatchMatchesProjection` is the fail-closed merge gate for drift
between the register rows, `regen.dag`, receipt declarations, runner inventory,
and on-disk harnesses.

## Dispatch Items

### 1. Gate #87 Complete-Row Receipt Audit

Own the audit of every `LensCapabilityBehavioralComplete` row, split by v2
counterpart class.

Files to read:

- `src/v3/std/verification.dag`
- `docs/v3-lens-capability-register.md`
- `src/v3/compiler/regen.dag`
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`

Deliverable:

- A PR that either proves the existing receipt roster is exact, or fixes missing
  / stale triples so the register, dispatch fixture, runner table, and prose
  mirror agree.

Acceptance:

- `cargo test -p v3_compiler r3_gate_87_lens_cementing_regen_receipts_test`
- `cargo test -p v3_compiler t_pb_b_1_dag_runner_test::cementing_dispatch_suite_passes_through_runner`

### 2. `.dag` Harness Promotion For Temporary Rust Receipts

Own the migration path for temporary Rust Band-C receipts whose expected
published carriers are now authorable in `.dag` `TestClaim` form.

Files to inspect first:

- `src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs`
- `src/v3/compiler/tests/integration/cementing/cost_lens_symbolic_consumer_test.rs`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost_symbolic.dag`
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`

Deliverable:

- For each temporary Rust receipt, either replace it with a `.dag` harness and
  remove the Rust census row, or document the exact missing carrier/substrate
  blocker in the Rust module and keep the `TemporaryRustModule` receipt honest.

Acceptance:

- `cargo test -p v3_compiler r3_gate_87_lens_cementing_regen_receipts_test`
- `cargo test -p v3_compiler t_pb_b_1_dag_runner_test::cementing_dispatch_suite_passes_through_runner`
- The SG-0 expected-hand-authored-test census still passes if a Rust receipt
  remains.

### 3. Helper-Lens Narrow-Claim Cementing Closure

Own the helper / intentionally partial registry surfaces so narrow `.dag`
claims cannot be mistaken for behavioral completion.

Files to inspect first:

- `src/v3/lenses/infer_helpers.dag`
- `src/v3/lenses/lower_helpers.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_infer_helpers.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_lower_helpers.dag`
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`

Deliverable:

- A PR that keeps each helper row explicitly `PARTIAL` / `N/A`, tightens the
  harness comments or companion Rust contract pins where needed, and prevents
  helper compile/smoke claims from satisfying the Band-C complete-row roster.

Acceptance:

- `cargo test -p v3_compiler r3_gate_87_lens_cementing_regen_receipts_test`
- `cargo test -p v3_compiler t_pb_b_1_dag_runner_test::cementing_dispatch_suite_passes_through_runner`

### 4. Gate #87 Runner Drift Regression Tests

Own negative coverage for the dispatch projection and runner-table seam.

Files to inspect first:

- `src/v3/compiler/src/cementing_dispatch.rs`
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`
- `src/v3/compiler/tests/integration/common/wiring_scanner_test.rs`

Deliverable:

- A PR adding or tightening focused tests that fail when:
  - a complete real-v2 register row lacks a Band-C receipt;
  - a `.dag` receipt is absent from `R3_GATE_87_CEMENTING_REGEN_SUITES`;
  - a temporary Rust receipt is not wired through `tests/integration.rs`;
  - duplicate receipt triples appear in `cementing_dispatch.dag`.

Acceptance:

- `cargo test -p v3_compiler r3_gate_87_lens_cementing_regen_receipts_test`
- `cargo test -p v3_compiler cementing_dispatch`

### 5. Reviewer Checklist Consolidation

Own the human-facing reviewer path after the mechanical gates above are proven.

Files to inspect first:

- `TESTING.md`
- `docs/v3-lens-capability-register.md`
- `INVARIANTS.md`
- `src/v3/std/verification.dag`

Deliverable:

- A docs-only PR that removes any stale or duplicated reviewer instructions and
  leaves one short path for checking a future `BEHAVIORALLY COMPLETE` promotion:
  apply the Band-C same-PR checklist, then trust
  `CementingDispatchMatchesProjection` and the PB-B-1 runner receipt.

Acceptance:

- Documentation references point to the existing authorities above and do not
  introduce a second receipt roster.

## Suggested Execution Order

Run items 1 and 4 first: they define and harden the fail-closed surface. Item 2
can proceed in parallel after item 1 identifies which temporary Rust receipts
are still real blockers. Item 3 is independent as long as it does not promote
helper rows to behavioral completion. Item 5 should land after the code-facing
ratchets have settled.
