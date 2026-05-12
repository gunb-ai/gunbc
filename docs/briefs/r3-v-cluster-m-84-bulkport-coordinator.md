# R3 Cluster M Phase 3 — #84 Bulk-Port Coordinator Framework

**Status:** V2 COORDINATOR FRAMEWORK. Six per-class stubs exist; Coordinator finalizes live inventories and pilot/bulk splits immediately before worker dispatch.

**Owner**: Verification Mgr (`clever-tern-670`) — coordinator role per Director ratification gunbc#846 #issuecomment-4412309986 (Ask 3).

**Authority**:
- Phase 3 dispatch overlay: [`r3-cluster-m-dispatch-verification-bulkport-84-2026-05-09.md`](r3-cluster-m-dispatch-verification-bulkport-84-2026-05-09.md).
- Strict-zero close-condition: Director Ask 4 (no closure-allowed exceptions; bulk-port = full `EXPECTED_HAND_AUTHORED_TEST`; testgen must cover).
- Locked design: [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §3 (migration audit) + §5 (cementing) + §6 (implementation order).
- Cluster M sequencing plan: [`docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md`](../audit/r3-cluster-m-sequencing-plan-2026-05-09.md) §5.

---

## §0. Coordinator scope (vs distributed per-lane)

Verification Mgr coordinates centrally; lane Mgrs review their own ported tests for behavioral fidelity (signoff, not authoring). Reasoning per Phase 3 overlay §2: avoids per-lane TestClaim-authoring learning curve; concentrates testgen-capability development in one place.

## §0.1 V2 / D2 ownership split

V2 owns SG-0 test rows that are cementable as `.dag` `TestClaim` or generated target-language test artifacts under #87 Phase 3 / #84. A concrete V2 migration PR must include the corresponding `EXPECTED_HAND_AUTHORED_TEST` decrement in the same PR as the replacement artifact.

D2 owns slow-test residuals, exemption/trajectory accounting, and SG-0 rows that resist cementing because they require structural/compiler fixes. Do not silently move resistant rows into V2 class scope. If D2 later unblocks a resistant row, it hands back to V2 with the blocker named.

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
| Cementing-test family (post-#87 pattern) | ~20-25 | `r3-v-cluster-m-84-class-cementing-bulkport-worker.md` | STUB; #87 pattern/dispatch receipt landed; Mgr finalizes live inventory + pilot/bulk split |
| Reflected-Dag structural assertion family | ~25-30 | `r3-v-cluster-m-84-class-reflected-dag-bulkport-worker.md` | DIRECTOR-SCAFFOLD; Mgr inventory + pilot selection pending |
| Generic DimensionReport / runner-discipline family | ~20-25 | `r3-v-cluster-m-84-class-generic-dimreport-bulkport-worker.md` | DIRECTOR-SCAFFOLD; Mgr inventory + framework-entry decision pending |
| Boundary tests (m1_*, m2_*, etc.) | ~10 | `r3-v-cluster-m-84-class-boundary-bulkport-worker.md` | STUB; Mgr maps per boundary milestone owner |
| R1C-D / R1C-E wrappers | ~3 | `r3-v-cluster-m-84-class-r1c-de-bulkport-worker.md` | STUB; pilot brief exists and can feed final class dispatch |
| L4/L7/L5 skeleton | ~5 | `r3-v-cluster-m-84-class-l4-l7-l5-bulkport-worker.md` | STUB; coordinate with Verification Cluster G Pattern-A owner |

Each class is a parallel-dispatchable worker batch. Coordinator (this brief's owner) finalizes the class stubs as Phase 3 begins; spawns workers; tracks SG-0 census decrement per class. The six linked files are the authoritative per-class framework surfaces; this coordinator table tracks only status and class-level routing.

### §2.1 Live SG-0 Classifier Bridge

The transient Rust classifier in `src/v3/compiler/tests/integration/sg0_census_test.rs` currently uses seven buckets. This coordinator intentionally keeps six worker stubs from the ratified Cluster M queue; the bridge from live buckets to stubs is:

| Live `TestsAsDataMigrationClass` bucket | Owning worker stub |
|---|---|
| `CementingV2Oracle` | Cementing-test family |
| `PropertyBased` | Reflected-Dag structural assertion family for generator/quantifier claims; L4/L7/L5 skeleton for TC strict-fire/deferred Pattern-A entries |
| `LensOutputEquality` | Generic DimensionReport / runner-discipline family unless the entry is a pure carrier-shape reflection, then Reflected-Dag |
| `BoundaryHostProcess` | Boundary tests |
| `CensusOrRatchet` | R1C-D/E wrappers for `r1c_*` rows; Reflected-Dag for SG/census/authority rows; Coordinator assigns outliers explicitly |
| `BehavioralObservation` | Boundary tests when the observation is target/runtime behavior; Generic DimensionReport when it is runner-verdict behavior |
| `CompileOrReject` | Boundary tests for milestone compile/reject contracts; L4/L7/L5 skeleton for Verification-corpus compile/reject claims |

This bridge is dispatch guidance, not a second census. The live path list and classifier remain the source of truth until #84 reaches zero and both disappear.

### §2.2 Class-stub finalization rules

Before a class stub can move from STUB/DIRECTOR-SCAFFOLD to PRE-AUTH DISPATCH-READY, the Coordinator fills:
- Complete live inventory from `EXPECTED_HAND_AUTHORED_TEST` at dispatch time.
- Per-entry migration shape: `.dag` TestClaim, generated target-language test, or STOP-and-PING substrate gap.
- First pilot target(s) and bulk-wave split.
- Lane-Mgr signoff owner(s) for behavioral fidelity.
- SG-0 decrement expectation and the exact census-removal rule.
- D2/V2 disposition for any row that resists cementing, with the blocker named.

Do not duplicate the live census count in these briefs. The only live count authority remains `src/v3/compiler/tests/integration/sg0_census_test.rs`.

## §3. Dispatch trigger

Phase 3 dispatches from the current #87 landed pattern/dispatch receipt plus Phase 3 coordinator readiness:
- #86 `ProgramGenerator` / `ProgramShape` substrate carrier is `CONSUMER_LANDED + PASSING`.
- #85 quantifier carriers are landed, but generated consumer wiring / `SuiteClaim` wrapper migration still controls full `CONSUMER_LANDED` status.
- #87 cementing-test discipline is `CONSUMER_LANDED + PASSING` in `docs/r3-program-plan.md` row #87.
- Coordinator completes live inventory, per-class finalization, and selected pilot/bulk split.

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
