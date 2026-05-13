# R3 Gate #87 — Lens Cementing Test Discipline Dispatch

Dispatch date: 2026-05-13

Scope: decompose `lens_cementing_test_discipline_complete` into bounded child
items that cement the lens-completeness invariant across the gate-87 receipt
surface. This is a dispatch brief, not a new authority. The authorities remain
`TESTING.md` Band C, `docs/v3-lens-capability-register.md` Discipline rules 6-7,
`docs/design-tests-as-data-completeness.md` §5, and the acceptance row in
`docs/r3-structure.md`.

## Finish Line

Gate #87 is complete when every `LensRegistryEntry` enumerated by
`src/v3/compiler/regen.dag` has a matching gate-87 `.dag` cementing harness,
the runner table includes that harness, the Band-C dispatch projection matches
the real-v2 complete register slice, and every intentionally narrow `.dag`
receipt has a paired dissolution trigger or Rust behavioral pin per
`TESTING.md`.

Required verification for the integrated closeout:

```sh
cargo test -p v3-compiler r3_gate_87
cargo test -p v3-compiler r3_gate_87_regen_lens_registry_names_match_fixture_inventory
cargo test -p v3-compiler cementing_dispatch_suite_passes_through_runner
cargo test -p v3-compiler sg0_census
```

## Child Items

### 1. Parallelism Regen Harness Drift Fix

Owner type: implementation worker.

Problem: `lens_parallelism_entry` exists in `src/v3/compiler/regen.dag`, but
the gate-87 fixture inventory and runner table can lag it. This is the concrete
drift class described in `docs/briefs/r3-gate-87-parallelism-fixture-fix-worker.md`.

Deliverables:

- Add `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_parallelism.dag`
  with a behavior-bearing predicate if current `.dag` literals can express the
  output; otherwise use the narrowest allowed placeholder plus an explicit
  dissolution trigger and paired Rust pin.
- Add the suite row to
  `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`.
- Extend only the gate-87 receipt surface needed for parallelism. Do not touch
  unrelated gates #57-#59 or broad lens-application work.

Acceptance:

- `r3_gate_87_regen_lens_registry_names_match_fixture_inventory` passes.
- `r3_gate_87_cementing_regen_lens_suites_pass_through_runner` passes.

### 2. Band-C Predicate Strength Audit

Owner type: verification worker.

Problem: complete-lens receipt strength must match the Band-C class. Real-v2
complete rows should use frozen-oracle or differential receipts; v3-native
complete rows should use `LensOutputEquals` on minimal `Dag` shapes where the
carrier is authorable. Helper or intentionally partial rows may stay narrower
only with a dissolution trigger and Rust pin.

Deliverables:

- Audit every `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag`
  against the row class in `docs/v3-lens-capability-register.md`.
- Patch weak receipts when the carrier is now authorable as `.dag` data.
- For receipts that must remain narrow, ensure the paired Rust pin exists in
  `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`
  or a lens-local unit test named from the gate-87 audit.
- Update `docs/briefs/r3-gate-87-lens-cementing-closure-audit.md` only if the
  audit result changes.

Acceptance:

- No `BEHAVIORALLY COMPLETE` row relies on a bare `Compiles` claim unless the
  carrier authoring limitation and dissolution trigger are documented in-tree.
- `cargo test -p v3-compiler r3_gate_87` passes.

### 3. Dispatch Projection / Register Coherence

Owner type: implementation worker.

Problem: `cementing_dispatch.dag` is the `.dag` successor to the old Rust
dispatch ratchet. It must keep matching the projection of complete rows with a
real v2 counterpart and the shared host evaluator in `cementing_dispatch.rs`.

Deliverables:

- Re-check `src/v3/compiler/tests/dag/cementing_dispatch.dag`,
  `src/v3/compiler/src/cementing_dispatch.rs`, and
  `src/v3/std/verification.dag` register-row data for drift against
  `docs/v3-lens-capability-register.md`.
- Patch the projection data or host evaluator only when the current register
  projection fails closed incorrectly.
- Keep `cementing_band_c_v2_complete_receipts` limited to real-v2 complete
  rows; v3-native and helper rows stay covered by the per-lens regen harnesses.

Acceptance:

- `cementing_dispatch_suite_passes_through_runner` passes.
- `lens_register_correspondence_test` still passes for the register projection.

### 4. Same-PR Checklist Ratchet

Owner type: verification worker.

Problem: future `BEHAVIORALLY COMPLETE` promotions must fail closed when a PR
updates `src/v3/compiler/regen.dag` or the capability register without adding
the matching gate-87 receipt.

Deliverables:

- Inspect `TESTING.md` "Same-PR checklist — promoting a row to
  `BEHAVIORALLY COMPLETE`" against the executable ratchets in
  `r3_gate_87_lens_cementing_regen_receipts_test.rs`.
- Add or tighten tests only if a promotion can currently skip one of the
  checklist files without a failure.
- Do not introduce a new registry authority; keep `regen.dag`, `std.verification`
  register rows, and `cementing_dispatch.dag` as the existing authorities.

Acceptance:

- A missing `t_r3_gate_87_cementing_regen_<lens>.dag` or missing runner-table
  row fails with an actionable message.
- The checklist in `TESTING.md` and the test failure messages name the same
  edit set.

### 5. Closeout Integration

Owner type: this manager session after child PRs land or are declared
unneeded.

Deliverables:

- Re-run the integrated verification commands in this brief.
- Refresh this dispatch brief if any child changes the final inventory.
- Report to the parent with the PR URL and child disposition.

Acceptance:

- Dashboard review summary shows mergeable clean with the required approvals.
- The implementation PR from this session is merged, which closes the node.
