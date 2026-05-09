# R3 Cluster M Phase 3 — #84 Bulk-Port Coordinator Skeleton

**Status:** SKELETON (per-class briefs authored as Phase 2 mid-flights). Coordinator-tier surface for Verification Mgr; not a worker dispatch artifact.

**Owner**: Verification Mgr (wise-bear-525 / gunbc#2075) — coordinator role per Director ratification gunbc#846 #issuecomment-4412309986 (Ask 3).

**Authority**:
- Phase 3 dispatch overlay: [`r3-cluster-m-dispatch-verification-bulkport-84-2026-05-09.md`](r3-cluster-m-dispatch-verification-bulkport-84-2026-05-09.md).
- Strict-zero close-condition: Director Ask 4 (no closure-allowed exceptions; bulk-port = full `EXPECTED_HAND_AUTHORED_TEST`; testgen must cover).
- Locked design: [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §3 (migration audit) + §5 (cementing) + §6 (implementation order).
- Cluster M sequencing plan: [`docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md`](../audit/r3-cluster-m-sequencing-plan-2026-05-09.md) §5.

---

## §0. Coordinator scope (vs distributed per-lane)

Verification Mgr coordinates centrally; lane Mgrs review their own ported tests for behavioral fidelity (signoff, not authoring). Reasoning per Phase 3 overlay §2: avoids per-lane TestClaim-authoring learning curve; concentrates testgen-capability development in one place.

## §1. Strict-zero close-condition (cite-and-execute)

Per Director Ask 4 ratification:
- **No closure-allowed exceptions for #84.** Bulk-port scope = all entries in `EXPECTED_HAND_AUTHORED_TEST` at PR-merge time (live authority: `src/v3/compiler/tests/integration/sg0_census_test.rs`; count `wc -l`-derivable from array literal — NOT hardcoded here, avoid duplicate-authority drift).
- Director Option 2 timed-carries (`cross_target_coverage_carrier_test.rs`, `method_template_contract_test.rs`, etc.) are **NOT** folded into closure-allowed; they are SG-0-budgeted slice-active ratchets that dissolve via Cluster M, not permanent exceptions.
- Where testgen can't cover (e.g., reflected-Dag structural assertions over std/ row authorities) → **Cluster M scope expansion**, not a closure-allowed carve.
- #84 PASSING gate: SG-0 census `EXPECTED_HAND_AUTHORED_TEST` count = 0 (strict).

## §2. Per-class brief queue (skeleton — fill in as Phase 2 mid-flights)

Per Phase 3 overlay §4 (estimated counts from velocity-walk audit §2.1):

| Class | Approx count | Worker brief filename | Status |
|---|---|---|---|
| Cementing-test family (post-#87 pattern) | ~20-25 | `r3-v-cluster-m-84-class-cementing-bulkport-worker.md` | TBD-author when #87 first-receipt lands |
| Reflected-Dag structural assertion family | ~25-30 | `r3-v-cluster-m-84-class-reflected-dag-bulkport-worker.md` | TBD-author when #86 ProgramGenerator carrier lands |
| Generic DimensionReport / runner-discipline family | ~20-25 | `r3-v-cluster-m-84-class-generic-dimreport-bulkport-worker.md` | TBD-author when #87 pattern + runner extensions stable |
| Boundary tests (m1_*, m2_*, etc.) | ~10 | `r3-v-cluster-m-84-class-boundary-bulkport-worker.md` | TBD-author per testgen-coverage discipline |
| R1C-D / R1C-E wrappers | ~3 | `r3-v-cluster-m-84-class-r1c-de-bulkport-worker.md` | May be eligible NOW; pre-Phase-2 dispatch candidate |
| L4/L7/L5 skeleton | ~5 | `r3-v-cluster-m-84-class-l4-l7-l5-bulkport-worker.md` | Verification Cluster G overlap; may bundle |

Each class is a parallel-dispatchable worker batch. Coordinator (this brief's owner) authors per-class briefs as Phase 3 begins; spawns workers; tracks SG-0 census decrement per class.

## §3. Dispatch trigger

Phase 3 dispatches **on Phase 1 + Phase 2 lock**:
- #85 + #86 substrate carriers landed (Substrate Mgr canvas ratification + worker dispatch).
- #87 cementing-test discipline pattern landed (Phase 2 first-migration receipt).

R1C-D/R1C-E wrappers may be eligible NOW (pre-Phase-1) as a pre-Phase-2 sanity dispatch — surface to Director when reviewing this skeleton.

Phase 3 is parallel-dispatchable per-class within itself (operator "staffing is not a concern" directive); coordinator manages cross-class dependencies + lane-Mgr signoff.

## §4. Lane-Mgr signoff workflow

Each per-class brief encodes:
1. Worker authors bulk migration for the class.
2. Original lane Mgr reviews behavioral fidelity (e.g., m1/m2 boundary tests reviewed by their respective lane Mgrs; cementing tests reviewed by Substrate/Verification per lens-tier).
3. Lane Mgr signoff required before each migration class is closed.
4. Coordinator (Verification Mgr) tracks per-class signoff state in this skeleton's §2 table; promotes status to CLOSED when SG-0 census reflects zero hand-Rust entries for that class.

## §5. Receipt + ledger updates

- #84 status DECLARED → CONSUMER_LANDED on first migration class fully ported (e.g., R1C-D/E wrappers if dispatched first).
- #84 status CONSUMER_LANDED → PASSING when SG-0 census `EXPECTED_HAND_AUTHORED_TEST` count = 0 (strict per §1).
- Verification Mgr advances `docs/r3-program-plan.md` §1.8 row #84 per §10 cadence.
- Daily SG-0 census count tracked at [`docs/audit/r3-sg0-trajectory-tracker.md`](../audit/r3-sg0-trajectory-tracker.md) §3.

## §6. Velocity context

Per [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](../audit/r3-pb0-velocity-walk-2026-05-09.md): Phase 3 dissolves the bulk of the SG-0 test side (~50-65 remaining test entries after Phase 2's ~20-25 reduction). #84 closure is the load-bearing event for PB-0 thesis test-side closure.

Phase 3 timing: weeks 4-8 of the 8-12 week R3 window per Director's bulk-event sequencing in #846 #issuecomment-4412008376 §C. Parallel-dispatchable per-class; total close fits R3 horizon under operator staffing directive.

## §7. Per-class brief authoring discipline

When authoring each per-class brief (§2 queue), follow the Phase 2 #87 worker-brief template at [`r3-v-cluster-m-87-cementing-worker.md`](r3-v-cluster-m-87-cementing-worker.md):
- §0 narrow scope to the class.
- §1 dispatch trigger (Phase 1/2 dependency state).
- §2 substantive deliverable (predicate-class mapping per locked design §3 / §C5).
- §3 first migration target (smallest exemplar in the class).
- §4 dispatch successor / bulk completion criteria.
- §5 receipt + ledger updates.
- §6 cross-Mgr signoff (which lane Mgr reviews this class).

Cite-and-execute discipline: do NOT restate locked design content; cite §C5 / §3 / §5 of `design-tests-as-data-completeness.md` and refer worker to those sections.

---

**End of skeleton.**
