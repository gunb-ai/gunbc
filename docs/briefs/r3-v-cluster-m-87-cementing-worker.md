# R3 Cluster M Phase 2 — #87 Cementing-Test Discipline Worker Brief

**Status:** PRE-AUTH DISPATCH-READY (worker-facing). Light port of the multi-gate PRE-AUTH brief [`r3-v-tests-as-data-v1-worker.md`](r3-v-tests-as-data-v1-worker.md) narrowed to gate #87 per Cluster M Phase 2 dispatch overlay.

**Owner**: worker (TBD on dispatch); coordinator: Verification Mgr (wise-bear-525 / gunbc#2075).

**Authority**:
- Locked design: [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §5 (Cementing test discipline) + §C5 (predicate-class table).
- Cluster M Phase 2 dispatch overlay: [`r3-cluster-m-dispatch-verification-discipline-87-2026-05-09.md`](r3-cluster-m-dispatch-verification-discipline-87-2026-05-09.md).
- Director ratification: gunbc#846 #issuecomment-4412309986 (Ask 1 (γ) hybrid).
- Standing TESTING discipline: [`TESTING.md`](../../TESTING.md) §"Cementing tests (Band C — lens subsumption)".

**Substrate-of-truth (do not restate; cite-and-execute)**: V1 multi-gate brief above + locked design §5.

---

## §0. Scope

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
- For v3-native rows: `LensOutputEquals` matched against `expected_output_ref` declaration.

This dispatch successor lands in a follow-on slice within Phase 2 (after first-receipt; before Phase 3 bulk-port begins). It does NOT need to land in the same PR as the first receipt — sequence is: pattern-receipt → dispatch-ratchet-successor → Phase 3 bulk-port.

## §5. Receipt + ledger updates

Per Phase 2 dispatch overlay §4:
- #87 status DECLARED → CONSUMER_LANDED on first-migration receipt land (§3).
- #87 status CONSUMER_LANDED → PASSING when dispatch-ratchet successor confirms 100% lens coverage in `.dag` cementing-test form.
- Verification Mgr advances `docs/r3-program-plan.md` §1.8 row #87 per §10 cadence.

Velocity context (per Phase 2 overlay §5): #87 dissolves ~20-25 of the SG-0 hand-Rust test entries (cementing-test class is the largest single class). Phase 2 is therefore a substantial bulk-dissolution sub-event en route to #84 close.

## §6. Cross-Mgr signoff

When the first-receipt slice lands, original lane Mgr for the migrated lens reviews behavioral fidelity:
- Substrate Mgr (warm-wolf-698 #2068): substrate-tier lenses (cost, complexity, etc.).
- Verification Mgr (this lane): verification-tier cementing tests.

This signoff workflow scales to Phase 3 per-class bulk-port (Phase 3 brief encodes the same pattern at scale).

---

**End of brief.**
