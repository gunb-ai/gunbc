# R3 Gap 3 fixed-point precondition coordination (2026-05-13)

**Status:** coordination audit; no gate status changes.

**Scope:** `docs/r3-actual-close-plan.md` Gap 3 / §1.8 gate #16 (`pb_self_compile_fixed_point`) R3 strong-form sequencing. This note cross-checks the four joint preconditions named by `docs/briefs/r3-pb-t-fixedpoint-worker.md` against local source-of-truth docs at HEAD and records the next manager-facing actions.

## Verdict

Gate #16 remains **not dispatch-eligible** for the R3 strong suite. The P0 pins landed, but none of the four joint preconditions can be read as green from the local authorities:

| Precondition | Current read at HEAD | Source-of-truth pointer | Coordination action |
|---|---|---|---|
| R2-Evaluator landed and stable | **Blocked.** The 2026-05-14 refresh turns 3/5 Evaluator sub-lanes green (`runtime_value_model_structural`, `body_evaluator_structural`, `cross_target_equivalence_harness_structural`), but `lens_application_complete_reflection` and `witness_construction_structural` remain in-flight. | `docs/r2-closure-ledger.md` Evaluator Manager table; `docs/r3-actual-close-plan.md` Gap 3 | Continue the re-spawned Evaluator lane through the two remaining cells; do not let gate #16 consume the Evaluator precondition until all five rows are green. |
| R2-Grounding Rust + Python closed | **Blocked / unproven.** LanguageSpec remains in-flight in the R2 closure ledger, and Shape-A / Python+Go readiness is still cited as a blocker by L5/Evaluator readiness docs. | `docs/r2-closure-ledger.md` row `T-Ground-LanguageSpec`; `docs/briefs/r2-evaluator-closure-residuals.md`; `docs/r3-program-plan.md` rows #15/#16 | Grounding owner should produce a Rust+Python closure receipt that is ledger-consumable by T-FixedPoint. Do not let T-FixedPoint infer target coverage from scattered target-specific PRs. |
| T-LP / SG-0 closed | **Blocked.** T-LensProducer-Retirement remains not-started in the R2 ledger. Live SG-0 expected-list snapshot is nonzero: non-test 53, test 121, fragments 2. | `docs/r2-closure-ledger.md` PB continuation row; `src/v3/compiler/tests/integration/sg0_census_test.rs`; `docs/r3-program-plan.md` rows #5-#8 | PB/Debt-Paydown sequencing remains upstream of fixed point. Gate #16 must not author `pb_self_compile_fixed_point_strong` while #5-#8 are open. |
| Row-B materialization landed | **Blocked.** The rule is designed, but no dispatch-time frozen Row-B target set exists because the ledger read cannot yet happen. The current `FixedPointConverges` runner path still only supports `default_fixed_point_source` / `pipeline_stage_snapshots`, so non-Rust Row-B verification also needs the named extension from the fixed-point brief. | `docs/briefs/r3-pb-t-fixedpoint-worker.md` §§Acceptance / Single grounding gate; `src/v3/compiler/src/test_runner.rs` `FixedPointConverges` payload checks | Once the first three preconditions are green, PB should freeze Row-B rows from the single ledger read and extend `self_host_fixed_point` or a sibling step for per-target emission byte-stability before claiming the suite is executable. |

## Evaluator sub-lane ledger refresh target

Gap 3's dominant hidden blocker is the Evaluator precondition. The close predicate for this precondition is not a new §1.8 row set; it is the existing R2 closure ledger cells turning green at HEAD:

| R2 closure-ledger sub-lane | Current ledger status | Required close state for Gap 3 |
|---|---|---|
| `runtime_value_model_structural` | `green` | Closed by HEAD runtime `Value` carrier + evaluator execution evidence; monitor only |
| `body_evaluator_structural` | `green` | Closed by executable `evaluate_body` / `eval_node` coverage over all five `Behavior` variants |
| `lens_application_complete_reflection` | `in-flight` | Still needs complete reflection + real lens-over-`Dag` / generic fold evidence |
| `witness_construction_structural` | `in-flight` | Still needs complete read-channel / reflected-program witness materialization |
| `cross_target_equivalence_harness_structural` | `green` | Closed by L5 primitive harness; L5 corpus breadth remains its own R3 §1.8 gate |

Partial R3-era work should be mapped into these exact cells, not into parallel status labels. The current branch has audit trails (`docs/audit/r3-evaluator-pr-1275-1500-debt-sweep.md`, `docs/audit/r3-evaluator-pr-1500-1803-debt-sweep.md`, `docs/audit/r3-evaluator-pr-1804-onward-debt-sweep.md`) that help populate evidence, but they are not themselves the closure ledger.

## Sequencing rule for gate #16

1. Keep `src/v3/std/verification.dag` in P0-only state; no `pb_self_compile_fixed_point_strong` suite until the single dispatch ledger read is green.
2. Keep Evaluator ownership on the re-spawned R3 Evaluator Mgr lane until the two remaining cells close.
3. Keep the R2 closure ledger's Evaluator cells refreshed from HEAD before T-FixedPoint consumes them.
4. Obtain a Grounding Rust+Python closure receipt from the Grounding owner; do not substitute L5 or Shape-A planning prose.
5. Let PB/Debt-Paydown close T-LP/SG-0 (#5-#8) before PB starts the P3 fixed-point worker.
6. At P3 dispatch, freeze Row-B targets from the same ledger read that gates dispatch, then materialize Row A + Row B + Row C in the strong suite and extend the fixed-point verifier for non-Rust Row-B byte-stability.

## Message to parent / Director

Use this as the coordination payload if the dashboard is reachable:

> Gap 3 gate #16 remains sequencing-held. Local audit confirms all four joint preconditions are still non-green: Evaluator ledger has five open cells, Grounding Rust+Python has no ledger-consumable close receipt, T-LP/SG-0 is open with SG-0 snapshot 53 non-test / 121 test / 2 fragments, and Row-B cannot materialize until the dispatch ledger read. Recommended next action is Director/PM decision on Evaluator lane ownership, then ledger refresh against the five existing R2 closure cells; do not dispatch PB P3 or author `pb_self_compile_fixed_point_strong` yet.
