# R3 Cluster M Phase 3 — Verification Bulk-Port Coordinator #84 Dispatch (2026-05-09)

**Owner**: Verification Mgr (wise-bear-525 / gunbc#2075) — coordinator role
**Authority**: PM-tier dispatch coordination per Director ratification at gunbc#846 #issuecomment-4412309986 (Director answered Ask 3 with "Verification Mgr coordinator"). Sequencing structure lives at [`docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md`](../audit/r3-cluster-m-sequencing-plan-2026-05-09.md) §5 (in-flight via concurrent PR #2361; this brief is the dispatch overlay — substantive content here is self-contained and grounded in live ledger authority below). This brief is a light-touch dispatch surface.

---

## §0. Scope

**Gate #84** `every_rust_test_ports_to_dag_or_generated` — Cluster M Phase 3. Verification Mgr coordinates **bulk-port** of the full `EXPECTED_HAND_AUTHORED_TEST` set to `.dag` TestClaim form, partitioned by dissolution-trigger class.

## §1. Director-ratified strict-zero close-condition (cite-and-execute)

Per Director ratification at gunbc#846 #issuecomment-4412309986 Ask 4: **strict-zero, no closure-allowed exceptions**.

- Director Option 2 timed-carries (`cross_target_coverage_carrier_test.rs`, `method_template_contract_test.rs`, etc.) are **NOT** folded into closure-allowed; they are blockers / non-close-risk until they migrate to testgen coverage
- Where testgen can't cover (e.g., reflected-Dag structural assertions over std/ row authorities) → **Cluster M scope expansion**, not a closure-allowed carve
- **Bulk-port scope = all entries in `EXPECTED_HAND_AUTHORED_TEST`** at PR-merge time (live authority: [`src/v3/compiler/tests/integration/sg0_census_test.rs`](../../src/v3/compiler/tests/integration/sg0_census_test.rs); count is `wc -l`-derivable from the array literal — **not hardcoded here** to avoid duplicate-authority drift). This is post #85/#86/#87 substrate landing.
- Cluster M's job is making testgen capable enough to cover them all

This is the load-bearing answer per `feedback_pb_zero_is_r3_close_target`: cascade promotion 2026-04-25 target is `0 hand-Rust + 0 TESTING residual`.

## §2. Coordinator role (vs distributed per-lane)

Per Director ratification: Verification Mgr coordinates centrally; lane Mgrs review their own ported tests for behavioral fidelity (signoff, not authoring). Reasoning: avoids per-lane TestClaim-authoring learning curve; concentrates testgen-capability development in one place.

**Lane Mgr signoff workflow**:
- Verification dispatches per-class workers; workers author bulk migrations
- For each migration class, original lane Mgr reviews behavioral fidelity (e.g., m1/m2 boundary tests reviewed by their respective lane Mgrs)
- Lane Mgr signoff is required before each migration class is considered closed

## §3. Dispatch trigger

Verification Mgr dispatches Phase 3 **on Phase 1 + Phase 2 lock** — i.e., #85 + #86 substrate carriers landed (post-Substrate Mgr canvas ratification + worker dispatch) AND #87 cementing-test discipline pattern landed (post-Phase 2 first-migration receipt).

Phase 3 is parallel-dispatchable per-class within itself (per operator "staffing is not a concern"); coordinator role manages cross-class dependencies + lane-Mgr signoff workflow.

## §4. Per-class brief queue

Per Cluster M sequencing plan §5.2 (estimated from velocity-walk audit §2.1):

| Class | Approx count | Worker brief scope |
|---|---|---|
| Cementing-test family (post-#87 pattern) | ~20-25 | Apply #87 pattern to remaining lens cementing tests |
| Reflected-Dag structural assertion family | ~25-30 | Migrate to ProgramGenerator-driven TestClaims (#86 consumer) |
| Generic DimensionReport / runner-discipline family | ~20-25 | Migrate to runner-side TestClaim (post-#87) |
| Boundary tests (m1_*, m2_*, etc.) | ~10 | Each migrates per testgen-coverage discipline |
| R1C-D / R1C-E wrappers | ~3 | R1-close residuals; may be eligible NOW |
| L4/L7/L5 skeleton | ~5 | Verification Cluster G overlap; may bundle |

Each class is a parallel-dispatchable worker batch. Verification Mgr authors per-class briefs as Phase 3 begins.

## §5. Receipt

- #84 status moves DECLARED → CONSUMER_LANDED on first migration class fully ported
- #84 status moves CONSUMER_LANDED → PASSING when SG-0 census `EXPECTED_HAND_AUTHORED_TEST` count = 0 (strict; no exception fold per Director Ask 4)
- §1.8 ledger Status updated by Verification Mgr per §10 cadence
- Daily SG-0 census count tracked at [`docs/audit/r3-sg0-trajectory-tracker.md`](../audit/r3-sg0-trajectory-tracker.md) §3 history table

## §6. Velocity context

Per [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](../audit/r3-pb0-velocity-walk-2026-05-09.md): Phase 3 dissolves the bulk of the SG-0 test side (~50-65 remaining test entries after Phase 2's ~20-25 reduction). #84 closure is the load-bearing event for PB-0 thesis test-side closure.

Phase 3 timing: weeks 4-8 of the 8-12 week R3 window per Director's bulk-event sequencing in #846 #issuecomment-4412008376 §C. Parallel-dispatchable per-class; total close fits R3 horizon under operator "staffing is not a concern" directive.

---

**End of brief.**
