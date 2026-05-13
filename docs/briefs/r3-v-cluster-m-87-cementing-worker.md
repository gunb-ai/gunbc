# R3 Cluster M Phase 2 — #87 Cementing-Test Discipline Worker Brief

**Status:** **PATTERN LANDED** — `docs/r3-program-plan.md` §1.8 gate **#87** `lens_cementing_test_discipline_complete` is **CONSUMER_LANDED + PASSING** at HEAD (receipt stack **PR #2639** + **PR #2757**; see §7). **Do not dispatch** new workers against the pre-land “first migration / dispatch successor” checklist; Phase 3 bulk-port consumes this pattern via [`r3-v-cluster-m-84-bulkport-coordinator.md`](r3-v-cluster-m-84-bulkport-coordinator.md). Historical intent below is retained for audit trail.

This file began as a PRE-AUTH worker-facing port of [`r3-v-tests-as-data-v1-worker.md`](r3-v-tests-as-data-v1-worker.md) narrowed to gate #87 per Cluster M Phase 2 dispatch overlay.

**Owner**: Verification Mgr (historical coordinator: wise-bear-525 / gunbc#2075); **no active worker dispatch** for #87 Phase 2 at HEAD.

**Authority**:
- Locked design: [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §5 (Cementing test discipline) + §C5 (predicate-class table).
- Cluster M Phase 2 dispatch overlay: [`r3-cluster-m-dispatch-verification-discipline-87-2026-05-09.md`](r3-cluster-m-dispatch-verification-discipline-87-2026-05-09.md).
- Director ratification: gunbc#846 #issuecomment-4412309986 (Ask 1 (γ) hybrid).
- Standing TESTING discipline: [`TESTING.md`](../../TESTING.md) §"Cementing tests (Band C — lens subsumption)".

**Substrate-of-truth (do not restate; cite-and-execute)**: V1 multi-gate brief above + locked design §5.

---

## §0. Scope

**Historical scope** (pre-land checklist; execution complete per Status banner + §7):

**Gate #87** `lens_cementing_test_discipline_complete`. Author the **discipline pattern** (`.dag` cementing-test shape), land the **first migration receipt** (smallest hand-Rust cementing test → `.dag` `TestClaim`), and prepare the **dispatch ratchet successor** that replaces `cementing_lens_registry_dispatch_test.rs`.

**Out of scope (STOP+PING)**:
- Inventing new `TestPredicate` variants — use existing `DifferentialEquals` (v2-counterpart lenses) / `LensOutputEquals` (v3-native lenses). Both are **🟡 Scaffold** per locked design §1 + `src/v3/std/verification.dag:175,184` inline annotations (named dissolution triggers: collapse with paired variants once substrate facets land). Per design §1 ("Rust-test migration may target either TERMINAL or 🟡 Scaffold variants; ports landing on a 🟡 Scaffold are inherently scoped by that variant's named dissolution trigger"), Scaffold-status is the correct migration substrate today; the cementing port forwards to the dissolved replacement when the trigger fires.
- Migrating cementing tests for lenses still at PROXY/STUB/PARTIAL in the lens-capability register — that's T-Lens-Behavioral-Parity scope; cementing-symmetry rule lands per-lens in same PR as the COMPLETE flip.
- Bulk-porting all cementing tests in this slice — that's Phase 3 #84 coordinator scope; this brief lands the **pattern + first receipt**, Phase 3 dispatches the bulk.

## §1. Dispatch trigger

**Independent of Cluster M Phase 1** (#85/#86 substrate canvases). Per Phase 2 dispatch overlay §2 authority correction (codex BLOCKING #4 on PR #2362 sha 60279789): cementing axis is orthogonal to property-based ProgramGenerator/Quantifier axis. Existing DB-15 `TestClaim` infrastructure + 🟡 Scaffold `DifferentialEquals`/`LensOutputEquals` predicates suffice (Scaffold per design §1 / verification.dag inline; targeting them is correct per design §1's explicit "Rust-test migration may target either TERMINAL or 🟡 Scaffold variants").

Phase 1 → Phase 2 coupling is **wrapper-level only**: when #85 `SuiteClaim` lands, existing `TestSuite.claims: List<TestClaim>` sites mechanically migrate to wrap claims in `Enumerated(...)`. Cementing TestClaims participate in that wrap migration as a follow-on; the pattern does not change.

## §2. Discipline pattern (the substantive deliverable)

Each `BEHAVIORALLY COMPLETE` lens in [`docs/v3-lens-capability-register.md`](../v3-lens-capability-register.md) gets a `.dag` cementing claim:

**Predicate selection axis**:
- **Real v2 counterpart** (register row names a non-`N/A` v2 lens): `DifferentialEquals { subject_ref: <v3_lens>, oracle_ref: <v2_lens>, input_ref: <fixture> }`.
- **v3-native** (`provenance`, `unused_parameters`, `variant_payload`, `structural_resolution`, `idempotency`, `named_function_count`, etc.): `LensOutputEquals { lens_ref: <v3_lens>, input_ref: <fixture>, expected_ref: <expected_carrier_decl> }` (per `src/v3/std/verification.dag:179-183`).

**Predicate axis distinctions** (do not mix):
- `BinaryDimensionReportEquals` is for Pattern-A DimensionReport comparisons (TC1/TC2/TC3 family) — different axis.
- `ProgramGenerator`/`Quantifier`/`QuantifiedTestClaim` (Phase 1 #85/#86) is for property-based "every program in family X satisfies P" claims — orthogonal to per-lens cementing. Cementing is single-fixture per claim.

**Claim shape** (mirror of locked design §5.2 sketch):
```dag
data cementing_<lens>_against_v2_oracle: TestClaim = {
  name: "<lens>_matches_v2_oracle_on_<minimal_fixture_name>",
  source: "<minimal .v3 source pinning the contract>",
  file_name: "<lens>_cementing.v3",
  predicate: DifferentialEquals {
    subject_ref: v3_<lens>,
    oracle_ref: v2_<lens>_oracle,
    input_ref: <lens>_cementing_source
  },
  requires: []
}
```

## §3. First migration target

Pick the smallest hand-Rust cementing test in `src/v3/compiler/tests/integration/cementing/` (worker grep at dispatch). Likely candidate: a single-fold or single-bind cementing module whose v2 oracle is already wired through `test_runner.rs::cost_of`-style dispatch.

**Acceptance for first-receipt slice**:
1. New `.dag` fixture file at `src/v3/compiler/tests/dag/cementing_<lens>.dag` declares the cementing `TestClaim` per §2 shape.
2. The corresponding hand-Rust file under `tests/integration/cementing/` is **deleted** (not stub-replaced) — receipt is a true port, not a parallel.
3. SG-0 census `EXPECTED_HAND_AUTHORED_TEST` decrements by 1 (the deleted file).
4. Test runner evaluates the new `.dag` claim green via BuildBuddy.
5. PR body: `SG-0 hand-path delta: -1`.

## §4. Dispatch-ratchet successor (post-first-receipt)

`cementing_lens_registry_dispatch_test.rs` enforces today: every register row with `BEHAVIORALLY COMPLETE` + real v2 counterpart has a matching `cementing/<stem>.rs`. The successor replaces module-list with claim-list:

- New `.dag` declaration `cementing_dispatch.dag` reads the lens-capability register projection.
- Verification: every register row matching `(BEHAVIORALLY COMPLETE, non-N/A v2 counterpart)` has a corresponding `.dag` `TestClaim` with `DifferentialEquals` predicate naming the row's v3 + v2 lens refs.
- For v3-native rows: `LensOutputEquals` matched against `expected_ref` declaration (per verification.dag field name).

This dispatch successor lands in a follow-on slice within Phase 2 (after first-receipt; before Phase 3 bulk-port begins). It does NOT need to land in the same PR as the first receipt — sequence is: pattern-receipt → dispatch-ratchet-successor → Phase 3 bulk-port.

## §5. Receipt + ledger updates

**Achieved (ledger)**: `docs/r3-program-plan.md` §1.8 row #87 documents **CONSUMER_LANDED + PASSING** with the regen enumeration + runner + `LensOutputEquals` / `DifferentialEquals` / frozen-oracle witness stack (not re-argued here).

Historical Phase 2 overlay milestones (for traceability):
- ~~DECLARED → CONSUMER_LANDED on first-migration receipt~~
- ~~CONSUMER_LANDED → PASSING when dispatch-ratchet successor confirms coverage~~ — superseded by the **§1.8** acceptance text tied to `regen.dag` + PR #2639 / #2757.

Velocity context (per Phase 2 overlay §5): bulk SG-0 shrink for the full cementing-test **class** continues under **#84** Phase 3, not by re-opening #87.

## §6. Cross-Mgr signoff

When the first-receipt slice lands, original lane Mgr for the migrated lens reviews behavioral fidelity:
- Substrate Mgr (warm-wolf-698 #2068): substrate-tier lenses (cost, complexity, etc.).
- Verification Mgr (this lane): verification-tier cementing tests.

This signoff workflow scales to Phase 3 per-class bulk-port (Phase 3 brief encodes the same pattern at scale).

## §7. As-shipped pattern index (HEAD)

Authoritative gate narrative: `docs/r3-program-plan.md` §1.8 row **#87** (grep at HEAD before citing Status).

**Dispatch + runner (single authority for which `.dag` modules the runner loads)**:
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs` — `R3_GATE_87_CEMENTING_REGEN_SUITES` table + `r3_gate_87_cementing_regen_lens_names_for_runner_table`.
- `src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs` — `r3_gate_87_cementing_regen_lens_suites_pass_through_runner` (PB-B-1 harness).

**Per-lens regen harnesses** (`TestSuite` in each file; naming convention `t_r3_gate_87_cementing_regen_<lens>.dag`):
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag` (10 lenses at HEAD; extend only with §1.8 / register + runner table updates in the same PR).

**Band-C dispatch receipt** (register + regen projection vs receipt list):
- `src/v3/compiler/tests/dag/cementing_dispatch.dag` — `CementingDispatchMatchesProjection` + `cementing_band_c_v2_complete_receipts`.
- `src/v3/compiler/src/cementing_dispatch.rs` — host-side evaluation / wiring shared with the `.dag` claim.

**Rust pin / behavioral contracts where `.dag` predicates stay intentionally narrower**:
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`.

**Design + ops prose** (cite-and-execute; not second authorities):
- `docs/design-tests-as-data-completeness.md` §5 / §C5 (cementing discipline).
- `TESTING.md` — *Cementing tests (Band C — lens subsumption)*.
- `docs/v3-lens-capability-register.md` — human-readable mirror of `lens_capability_register_rows`.

---

**End of brief.**
