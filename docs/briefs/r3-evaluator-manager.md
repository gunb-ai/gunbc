# R3 Evaluator Manager Brief

**Status:** active R3 manager lane, re-spawned 2026-05-14 after operator §4 sub-item 5 ratification and the `dashboard-ops --shape` flag landing in PR #3041. This brief is the R3 manager-facing program surface; the R2 lineage remains in [`r2-evaluator-manager.md`](r2-evaluator-manager.md).

## Authority

- R3 close plan: [`docs/r3-actual-close-plan.md`](../r3-actual-close-plan.md) Gap 3 and §4 item 5.
- R2 closure ledger source of truth: [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) Evaluator Manager table.
- R3 evaluator implementation boundary: [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md).
- Current sequencing audit: [`docs/audit/r3-gap3-fixed-point-precondition-coordination-2026-05-13.md`](../audit/r3-gap3-fixed-point-precondition-coordination-2026-05-13.md).
- Current ratchet: [`scripts/check-r2-evaluator-ledger-refresh.sh`](../../scripts/check-r2-evaluator-ledger-refresh.sh).

## Manager Rule

Do not run `dashboard-ops replan` or dispatch child work until the dashboard backend accepts composite-shaped work-item creation. This lane can prepare manager-level artifacts and PRs while dispatch is held. Worker dispatch resumes only after Director authorization with explicit composite shape.

## R3 Close Scope

This lane owns the R2-Evaluator joint precondition under Gap 3 (`pb_self_compile_fixed_point`). The precondition closes only when all five existing R2 Evaluator closure-ledger cells are green at HEAD:

| Gate | Current R3 read | R3 manager action |
|---|---|---|
| `runtime_value_model_structural` | green | Monitor for drift only. |
| `body_evaluator_structural` | green | Monitor for drift only. |
| `lens_application_complete_reflection` | in-flight | Dispatch/own remaining complete-reflection + real lens-over-`Dag` / generic fold closure work. |
| `witness_construction_structural` | in-flight | Dispatch/own complete witness materialization work, including non-stub `Violates` read-channel reporting. |
| `cross_target_equivalence_harness_structural` | green | Monitor for drift only; broader L5 corpus breadth remains R3 §1.8 gate #15, not this sub-lane. |

The current ledger refresh intentionally does not overclaim the two in-flight rows. Static E6-G1.a lens report production, descent proof consumption, bounded L7 witness receipts, and the L5 primitive harness are evidence, but Q-Reification / reflected-program authority and complete witness materialization remain the load-bearing tail.

## First Dispatches When Unblocked

1. **Lens application completion worker**: consume the Phase 5 handoff and Q-Reification boundary; produce complete reflection + real lens-over-`Dag` evidence without routing through host-side reflection shortcuts or widening `fold_lens<C>` before X1.b S1/S3 authority is present.
2. **Witness construction worker**: make witness materialization complete over the accepted evaluator surface, including a real `Violates` / diagnostic path rather than the E6-G1.a fail-closed empty-list stub.
3. **Ledger refresh follow-up**: after each worker lands, update the exact R2 closure-ledger row and keep `scripts/check-r2-evaluator-ledger-refresh.sh` bound to the row-level evidence.

## Cross-Manager Dependencies

- **Substrate Manager** owns substrate-shape authority for Q-Reification, generic runtime-callee authority, and any new fact carriers required by lens folding.
- **Verification Manager** owns R3 §1.8 gate execution and broader L4/L5/L7 close ceremony receipts.
- **Debt-Paydown / PB Manager** owns SG-0 and PB-0 retirement rows that consume evaluator readiness.
- **Grounding Manager** owns R2-Grounding T-Ground residuals that also block Gap 3 fixed-point dispatch.

## Close Signal

Signal Director when:

- `lens_application_complete_reflection` turns green in `docs/r2-closure-ledger.md`;
- `witness_construction_structural` turns green in `docs/r2-closure-ledger.md`;
- `scripts/check-r2-evaluator-ledger-refresh.sh` passes with all five rows green or with an explicitly ratified intermediate matrix;
- no dashboard REQUEST_CHANGES remain on the manager PR that updates the ledger.

Until then, Gap 3 remains sequencing-held and PB must not dispatch or claim `pb_self_compile_fixed_point_strong`.
