# R3 Gate #87 Lens Cementing Test Discipline Dispatch

Date: 2026-05-13

Owner: Verification Manager session `vivid-gull-196`

## Scope

This is the dispatch packet for `lens_cementing_test_discipline_complete` after
the gate-#87 pattern landed. The current invariant is:

> every `LensRegistryEntry` in `src/v3/compiler/regen.dag` has a merge-visible
> Band-C cementing receipt, and drift between the regen registry, `.dag`
> runner inventory, dispatch projection, and Rust pin receipts fails closed.

The standing authorities remain:

- `docs/r3-structure.md` acceptance bullet `lens_cementing_test_discipline_complete`
- `docs/v3-lens-capability-register.md`
- `TESTING.md` section "Cementing tests (Band C -- lens subsumption)"
- `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`
- `src/v3/compiler/regen.dag`
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`

## Dispatch Items

### G87-D1: regen registry coverage smoke

Verify the live `src/v3/compiler/regen.dag` lens registry against
`R3_GATE_87_CEMENTING_REGEN_SUITES` and `cementing_dispatch.dag`.

Acceptance:

- `cargo test -p v3-compiler r3_gate_87_regen_lens_registry_names_match_fixture_inventory`
  passes.
- `cargo test -p v3-compiler cementing_dispatch_suite_passes_through_runner`
  passes.
- If a registry row is missing from the suite inventory, add the
  `t_r3_gate_87_cementing_regen_<lens>.dag` harness and suites-table row in
  the same PR.

### G87-D2: non-complete regen rows use explicit Band-C placeholders

Audit `cost_target_realization`, `effect_enumeration`, `parallelism`,
`infer_helpers`, and `lower_helpers` against the capability register. These rows
must either carry a behavior-bearing receipt or an explicit `Compiles`
placeholder with a named dissolution trigger and a paired Rust pin where the
published carrier is not yet authorable in `.dag` test data.

Acceptance:

- Each non-complete or helper row in `src/v3/compiler/regen.dag` has a matching
  `t_r3_gate_87_cementing_regen_<lens>.dag` file.
- Placeholder comments name the missing carrier or consumer prerequisite and
  the owning lane.
- `cargo test -p v3-compiler r3_gate_87` passes.

### G87-D3: complete v3-native rows carry behavioral pins

Audit `provenance`, `structural_resolution`, `unused_parameters`, and
`variant_payload` for v3-native Band-C receipts.

Acceptance:

- Existing `LensOutputEquals` `.dag` claims remain behavior-bearing where the
  expected carrier is authorable.
- Any `Compiles` placeholder has a paired Rust receipt in
  `r3_gate_87_lens_cementing_regen_receipts_test.rs` or the lens module tests.
- The audit updates `docs/briefs/r3-gate-87-lens-cementing-closure-audit.md`
  only if the live disposition changed.

### G87-D4: real-v2 counterpart rows keep frozen-oracle discipline

Audit `cost` / `complexity` and `cost_symbolic` receipts against the register's
v2-counterpart column.

Acceptance:

- `.dag` claims use `DifferentialEquals`, `LensOutputEquals`, or
  `SymbolicCostExprEquals` according to the currently authorable carrier.
- Temporary Rust frozen-oracle pins remain named and linked from the relevant
  `.dag` placeholder comments.
- No live v2 oracle consumer is introduced; frozen receipts remain compatible
  with `v2_oracle_no_remaining_test_consumers`.

### G87-D5: SG-0 hand-Rust census handoff for cementing residuals

Keep the post-#87 handoff table in
`docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md` aligned with
`src/v3/compiler/tests/integration/sg0_census_test.rs`.

Acceptance:

- Cementing residual rows name the predicate class, blocker, owner lane, and
  expected SG-0 census delta.
- Rows that are no longer Band-C lens-cementing work are explicitly marked as
  such rather than silently retained in the cementing class.
- Docs-only changes do not claim a census decrement.

## Dispatch Note

The dashboard CLI treats `dashboard-ops work-items create --help` as a literal
title and created a bogus child item named `--help` during this decomposition.
It is not part of the gate-#87 dispatch set and should be archived by the
manager/operator rather than worked.
