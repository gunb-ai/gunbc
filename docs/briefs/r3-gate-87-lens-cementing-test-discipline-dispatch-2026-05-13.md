# R3 Gate 87 Lens Cementing Test-Discipline Dispatch

Date: 2026-05-13

Owner: Verification Mgr lane, dispatched from `valiant-boar-233`.

Scope: decompose `lens_cementing_test_discipline_complete` into concrete child work that cements the lens-completeness invariant as test discipline. Gate #87 remains recorded as `CONSUMER_LANDED + PASSING` in `docs/r3-program-plan.md` row #87 for the `src/v3/compiler/regen.dag` corpus; this packet dispatches the remaining verification discipline around that landed pattern so workers do not reopen the gate under parallel authorities.

## Authorities

- `TESTING.md` section "Cementing tests (Band C - lens subsumption)" is the discipline rule.
- `docs/design-tests-as-data-completeness.md` section 5 / C5 is the predicate taxonomy.
- `docs/v3-lens-capability-register.md` is the human-readable register mirror.
- `src/v3/std/verification.dag` owns `TestPredicate` and `lens_capability_register_rows`.
- `src/v3/compiler/regen.dag` owns the gate-87 generated-lens enumeration.
- `src/v3/compiler/tests/dag/cementing_dispatch.dag` owns the Band-C receipt projection.
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs` owns the runner inventory.
- `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md` records the landed pattern consumed by these workers.

Do not create a second inventory of lens rows. If a row is complete, derive it from the register plus `regen.dag`; if it is residual hand-Rust cementing, derive it from `EXPECTED_HAND_AUTHORED_TEST` in `src/v3/compiler/tests/integration/sg0_census_test.rs` and the disposition table in the pattern brief.

## Child Items

### G87-A: Register / Regen Projection Audit

Goal: prove the lens-completeness projection is derived from one source path and that every complete `regen.dag` `LensRegistryEntry` has exactly one Band-C classification.

Concrete scope:
- Compare `src/v3/compiler/regen.dag` rows against `src/v3/std/verification.dag` `lens_capability_register_rows`.
- Confirm `docs/v3-lens-capability-register.md` mirrors the same behavioral status and v2-counterpart class.
- Update stale comments only where they would mislead future cementing workers.
- Do not add new predicate variants or migrate tests in this slice.

Acceptance:
- A PR or no-code closeout states the exact complete-row set, helper / partial exclusions, and drift findings.
- If code changes land, `cargo test -p v3-compiler r3_gate_87` is the minimum check.

### G87-B: Real-v2 Counterpart Cementing Receipts

Goal: keep every behaviorally complete lens with a real v2 counterpart cemented by executable equality against its frozen oracle or reviewed projection.

Concrete scope:
- Audit the `DifferentialEquals`, `SymbolicCostExprEquals`, and frozen-oracle Rust receipts for complete rows with real v2 counterpart status.
- For any gap, add or repair the `.dag` receipt under `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag` and update the runner inventory in the same PR.
- If a full carrier is still blocked, keep the temporary Rust receipt but name the owning blocker and dissolution trigger.

Acceptance:
- `cementing_dispatch.dag` and `R3_GATE_87_CEMENTING_REGEN_SUITES` agree with the real-v2 complete-row projection.
- No real-v2 complete row is represented only by a broad `Compiles` placeholder unless the blocker is explicit and paired with a Rust behavioral receipt.

### G87-C: V3-native / Helper Lens Cementing Receipts

Goal: preserve behavior for v3-native complete lenses and make helper / N/A rows explicitly non-behavioral where appropriate.

Concrete scope:
- Audit `LensOutputEquals` receipts for v3-native complete rows.
- Check helper-only or `N/A` rows (`infer_helpers`, `lower_helpers`, and related register entries) for explicit `Compiles` plus Rust pin receipt where the published contract is narrower than a full lens-output carrier.
- For `variant_payload`-style carriers that cannot yet be expressed as `.dag` expected values, keep the Rust unit receipt and name the expected-value blocker.

Acceptance:
- Every v3-native complete row has a behavioral receipt, either `.dag` `LensOutputEquals` or a named temporary Rust receipt with dissolution trigger.
- Helper rows are not counted as behavioral-complete gaps.

### G87-D: Dispatch Ratchet And SG-0 Handoff

Goal: keep the Band-C dispatch ratchet wired through the runner and hand residual Rust cementing rows to the #84 bulk-port lane without duplicating #87 scope.

Concrete scope:
- Verify `src/v3/compiler/tests/dag/cementing_dispatch.dag` is exercised by `t_pb_b_1_dag_runner_test::cementing_dispatch_suite_passes_through_runner`.
- Verify `src/v3/compiler/src/cementing_dispatch.rs` still rejects drift between register projection, `regen.dag`, and receipt list.
- Refresh the residual hand-Rust disposition table in `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md` only if the live `EXPECTED_HAND_AUTHORED_TEST` inventory changed.
- Route rows blocked by missing carrier expressiveness to their owning lane; do not relabel them as #87 incomplete.

Acceptance:
- The worker reports the current SG-0 hand-path delta opportunity for cementing rows and the exact blockers for rows that remain Rust-authored.
- Any executable change runs the narrow runner check for `cementing_dispatch_suite` or the broader `cargo test -p v3-compiler r3_gate_87` slice.

## Dispatch Rule

Each child owns one slice above. A child may close no-PR if the audit finds the slice already satisfied, but the closeout must cite the exact files checked and whether it found any actionable drift. If a child changes a `regen.dag` lens row, it must update the register mirror, per-lens `.dag` receipt, runner inventory, and `cementing_dispatch.dag` in the same PR.
