# R3 Gate 87 Lens-Cementing Dispatch — 2026-05-13

**Owner:** Verification Mgr lane under `swift-raven-219`.

**Scope:** concrete child dispatch for `lens_cementing_test_discipline_complete`.
This does not reopen the landed gate-87 runner design. `docs/r3-program-plan.md`
row 87 is already `CONSUMER_LANDED + PASSING`; the work below keeps the
test-discipline cementing invariant live as adjacent lanes unblock stronger
data-authorable receipts.

**Single authorities to preserve:**

- `TESTING.md` "Cementing tests (Band C — lens subsumption)".
- `docs/v3-lens-capability-register.md` plus `src/v3/std/verification.dag`
  `lens_capability_register_rows`.
- `src/v3/compiler/regen.dag` `LensRegistryEntry` rows.
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`.
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`.
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`.
- `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md` for the
  current predicate taxonomy and hand-Rust disposition table.

## Dispatch Items

### G87-D1 — Regen Corpus Drift Audit

**Question:** does the live `regen.dag` registry still match the gate-87
runner and dispatch surfaces exactly?

**Work:**

1. Compare `LensRegistryEntry.name` values from `src/v3/compiler/regen.dag`
   against `R3_GATE_87_CEMENTING_REGEN_SUITES`.
2. Verify every `tests/dag/t_r3_gate_87_cementing_regen_*.dag` harness named
   by the table is exercised by `t_pb_b_1_dag_runner_test`.
3. Verify `cementing_dispatch.dag` contains only receipts implied by the
   `lens_capability_register_rows` projection and the runner table.

**Acceptance artifact:** either a docs-only note in this brief confirming zero
drift, or an implementation PR that updates all single-authority surfaces in
one change.

**Verification:** `cargo test -p v3-compiler r3_gate_87`.

### G87-D2 — Helper Placeholder Dissolution Readiness

**Question:** can any helper-only `Compiles` placeholder be strengthened to a
behavioral `LensOutputEquals` receipt now?

**Rows:** `infer_helpers`, `lower_helpers`, `variant_payload`.

**Work:**

1. Inspect the three gate-87 helper harnesses under
   `src/v3/compiler/tests/dag/`.
2. For each row, decide whether the published output carrier is now authorable
   as `.dag` expected data.
3. If unblocked, replace the `Compiles` placeholder with the strongest
   available behavioral claim, update `R3_GATE_87_CEMENTING_REGEN_SUITES`, and
   remove the paired Rust pin from
   `r3_gate_87_lens_cementing_regen_receipts_test.rs` in the same PR.
4. If still blocked, leave code unchanged and update the blocker text in the
   pattern brief only if the owning lane or blocker has changed.

**Acceptance artifact:** one PR for any strengthened row, or a no-code closeout
that names the exact still-blocking carrier.

**Verification:** `cargo test -p v3-compiler r3_gate_87`.

### G87-D3 — Rust Cementing Residual Classification Refresh

**Question:** is the hand-Rust cementing disposition table still complete and
correct against `EXPECTED_HAND_AUTHORED_TEST`?

**Work:**

1. Read `src/v3/compiler/tests/integration/sg0_census_test.rs` for every
   `tests/integration/cementing/*.rs` entry.
2. Cross-check each row against
   `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md` §3 and §3.1.
3. For each residual, keep exactly one classification: gate-87 Band-C
   residual, non-cementing infrastructure residual, or unrelated lane-owned
   receipt.
4. Patch the brief if a row was added, removed, ported, or unblocked.

**Acceptance artifact:** docs PR updating the classification table, or a
closeout note that the table remains current with command output.

**Verification:** `cargo test -p v3-compiler sg0_census`.

### G87-D4 — Real-V2 Counterpart Promotion Guard

**Question:** can a future `BEHAVIORALLY COMPLETE` promotion bypass Band-C
cementing?

**Work:**

1. Review the correspondence tests around the capability register and
   `LensCapabilityV2RealV2`.
2. Add or tighten a ratchet only if a `COMPLETE` + real-v2 row can evade
   `CementingDispatchMatchesProjection`.
3. Keep the guard data-driven through `lens_capability_register_rows`; do not
   add a parallel hand-maintained inventory.

**Acceptance artifact:** a small test/code PR if a gap exists, or a no-code
closeout that cites the existing guard path.

**Verification:** `cargo test -p v3-compiler lens_register_correspondence
cementing_dispatch_suite_passes_through_runner`.

### G87-D5 — Worker-Facing Checklist Lock

**Question:** are the human checklist and executable checks aligned enough for
future lens-completeness workers?

**Work:**

1. Compare `TESTING.md` same-PR checklist with the runner and dispatch code.
2. Ensure every checklist bullet maps to an executable failure path or an
   explicit reviewer responsibility.
3. Patch `TESTING.md` only for drift; avoid introducing a second authority.

**Acceptance artifact:** docs PR with checklist drift fixes, or closeout note
confirming no drift.

**Verification:** docs-only unless code changes are required; if code changes
land, run the narrow test named by the touched surface.

## Dispatch Order

Start D1 and D3 first: they establish the live inventory and avoid assigning
workers against stale row lists. D2 follows if D3 confirms helper placeholders
are still the only gate-87 placeholder class. D4 and D5 can run in parallel
after D1 because they inspect guard coverage and human checklist alignment
rather than editing the per-lens receipts directly.
