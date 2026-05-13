# R3 Gate 87 Lens Cementing Test Discipline Dispatch

Date: 2026-05-13

Scope: decompose `lens_cementing_test_discipline_complete` into concrete worker items. This is the dispatch plan for cementing the lens-completeness invariant named in `docs/r3-structure.md` and operationalized by `TESTING.md` Band C.

## Gate Contract

Gate 87 is complete when every `LensRegistryEntry` in `src/v3/compiler/regen.dag` has a merge-visible cementing receipt:

- A `.dag` harness under `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag`.
- A runner-table row in `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`.
- A receipt ratchet in `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`.
- For real v2-counterpart, behaviorally complete rows: a Band-C receipt in `src/v3/compiler/tests/dag/cementing_dispatch.dag` that is accepted by `CementingDispatchMatchesProjection`.
- For v3-native, `N/A`, helper, or not-yet-authorable carrier surfaces: a narrow `.dag` claim plus an explicit dissolution trigger and, where needed, a Rust pin receipt.

The existing closure audit is `docs/briefs/r3-gate-87-lens-cementing-closure-audit.md`. Workers should update it only when they change evidence status; the single authority for executable gate inventory remains the runner table.

## Sub-Items

### 1. Inventory Ratchet Audit

Owner target: verification worker.

Files:

- `src/v3/compiler/regen.dag`
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag`

Work:

- Recompute the live `LensRegistryEntry.name` set from `src/v3/compiler/regen.dag`.
- Confirm it equals `r3_gate_87_cementing_regen_lens_names_for_runner_table()`.
- For every added or removed registry entry, update the runner table and add or retire the matching `.dag` harness in the same PR.

Acceptance:

- `r3_gate_87_regen_lens_registry_names_match_fixture_inventory` passes.
- No hand-maintained list other than `R3_GATE_87_CEMENTING_REGEN_SUITES` is introduced.

### 2. Real-v2 Band-C Dispatch Receipts

Owner target: verification worker.

Files:

- `src/v3/compiler/tests/dag/cementing_dispatch.dag`
- `src/v3/compiler/src/cementing_dispatch.rs`
- `src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs`
- `src/v3/compiler/tests/integration/cementing/*.rs`

Work:

- For every register row that is both `BEHAVIORALLY COMPLETE` and has a real v2 counterpart, ensure `cementing_band_c_v2_complete_receipts` contains exactly the required receipt stems.
- Prefer `.dag` `DifferentialEquals`/`LensOutputEquals` receipts. Keep temporary Rust receipts only when the expected carrier is not authorable as `.dag` data yet.
- When a temporary Rust receipt remains, document the concrete dissolution trigger in the Rust module or adjacent harness comment.

Acceptance:

- `cementing_dispatch_suite_passes_through_runner` passes.
- `CementingDispatchMatchesProjection` rejects drift between the capability register, `regen.dag`, receipt stems, and runner table.

### 3. V3-native and Helper Surface Receipts

Owner target: lens-specific verification worker.

Files:

- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_provenance.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_structural_resolution.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_unused_parameters.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_variant_payload.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_infer_helpers.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_lower_helpers.dag`
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`

Work:

- Keep v3-native lenses on `LensOutputEquals` when the public result carrier is authorable as `.dag` data.
- For helper-only or carrier-blocked lenses, keep `Compiles` receipts narrow and pair them with Rust behavior pins where the shipped API has behavior that cannot yet be represented in `.dag`.
- Each placeholder must name the exact future authoring capability that deletes it.

Acceptance:

- Every placeholder has a dissolution trigger.
- Rust pin tests make one structural claim per test and do not assert on stringified failure text.

### 4. Register-promotion Same-PR Checklist

Owner target: reviewer-support worker.

Files:

- `docs/v3-lens-capability-register.md`
- `TESTING.md`
- `src/v3/std/verification.dag`
- Gate-87 receipt files touched by the promoted lens.

Work:

- Before any row is promoted to `BEHAVIORALLY COMPLETE`, classify its v2-counterpart column as real v2, v3-native, `N/A`, or helper/partial.
- Apply the `TESTING.md` Band-C same-PR checklist mechanically: register row, receipt harness, runner table, dispatch receipt if applicable, Rust pin if required, and closure-audit status.
- Reject promotions that update prose without the executable receipt surface.

Acceptance:

- The PR that changes a `BEHAVIORALLY COMPLETE` status also changes the relevant gate-87 receipt files.
- The human-readable register and structural `lens_capability_register_rows` stay aligned.

### 5. Rust-to-.dag Dissolution Follow-through

Owner target: tests-as-data worker.

Files:

- `src/v3/compiler/tests/integration/cementing/*.rs`
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag`
- `src/v3/compiler/tests/integration/sg0_census_test.rs`

Work:

- Port temporary Rust cementing receipts to `.dag` TestClaims as the missing expected-carrier authoring support lands.
- In the same PR, remove the obsolete Rust module wiring and shrink the SG-0 hand-authored Rust-test census.
- Keep `cementing_dispatch.dag` as the receipt list authority for Band-C real-v2 complete rows.

Acceptance:

- No parallel Rust and `.dag` receipt remains for the same claim unless the Rust receipt documents a still-live carrier gap.
- The SG-0 census decreases whenever a Rust receipt dissolves.

## Dispatch Order

The first three sub-items can run in parallel because they touch mostly disjoint evidence slices. Sub-item 4 is a standing review gate for any future register-promotion PR. Sub-item 5 follows carrier-authoring work and should be dispatched opportunistically whenever a temporary Rust receipt becomes expressible as `.dag` data.

Minimum verification for this dispatch PR:

- `cargo test -p v3-compiler --test integration r3_gate_87_lens_cementing_regen_receipts_test`
- `cargo test -p v3-compiler --test integration t_pb_b_1_dag_runner_test::cementing_dispatch_suite_passes_through_runner`
