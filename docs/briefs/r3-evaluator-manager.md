# R3 Evaluator Manager Brief

**Status:** active R3 manager lane, re-spawned 2026-05-14 UTC after operator §4 sub-item 5 ratification and the `dashboard-ops --shape` flag landing in PR #3041. This brief is the R3 manager-facing program surface; the R2 lineage remains in [`r2-evaluator-manager.md`](r2-evaluator-manager.md).

## Authority

- R3 close plan: [`docs/r3-actual-close-plan.md`](../r3-actual-close-plan.md) Gap 3 and §4 item 5.
- R2 closure ledger source of truth: [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) Evaluator Manager table.
- R2 closure ledger ownership protocol: [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) §"Signal protocol" and §"Authority discipline". R2 Release Manager remains the single ledger owner; this R3 lane prepares evidence and signals row transitions unless Director explicitly ratifies a refresh PR as the ledger-update vehicle.
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
3. **Ledger signal follow-up**: after each worker lands, send the exact row transition evidence through the R2 Release Manager signal/ack protocol, or land a Director-ratified ledger refresh PR that names that protocol exception. Keep `scripts/check-r2-evaluator-ledger-refresh.sh` bound to the acknowledged row-level evidence.

## Remaining Closure Predicates

`lens_application_complete_reflection` turns green only when all of these are true at HEAD:

- the evaluator consumes `Dag` as the reflected-program authority rather than a host-side registry or scalar declaration-reference shortcut;
- a lens-over-`Dag` path produces a real `DimensionReport<C>` through declared substrate values;
- the path covers complete reflection per [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md), not only the static E6-G1.a representative;
- any generic `fold_lens<C>` claim is backed by the required X1.b S1/S3 runtime-callee authority or by an explicit Director reroute.
- the result is signaled for the exact `lens_application_complete_reflection` row in [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md), with R2 Release Manager acknowledgement or explicit Director-ratified refresh authority.

`witness_construction_structural` turns green only when all of these are true at HEAD:

- `Witness::Inhabits` and `Witness::Violates` both materialize through evaluator-executed declared constructors;
- `Violates` carries a non-stub diagnostic path instead of the current E6-G1.a empty-list fail-closed receipt;
- algebraic-law witness rows stay tied to faithful law carriers and do not widen beyond the bounded Int receipts until substrate authority exists;
- the result is signaled for the exact `witness_construction_structural` row in [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md), with R2 Release Manager acknowledgement or explicit Director-ratified refresh authority, and keeps the row-bound ratchet current.

## Cross-Manager Dependencies

- **Substrate Manager** owns substrate-shape authority for Q-Reification, generic runtime-callee authority, and any new fact carriers required by lens folding.
- **Verification Manager** owns R3 §1.8 gate execution and broader L4/L5/L7 close ceremony receipts.
- **Debt-Paydown / PB Manager** owns SG-0 and PB-0 retirement rows that consume evaluator readiness.
- **Grounding Manager** owns R2-Grounding T-Ground residuals that also block Gap 3 fixed-point dispatch.

## Close Signal

Signal Director when:

- `lens_application_complete_reflection` has R2 Release Manager acknowledgement to turn green in `docs/r2-closure-ledger.md`;
- `witness_construction_structural` has R2 Release Manager acknowledgement to turn green in `docs/r2-closure-ledger.md`;
- `scripts/check-r2-evaluator-ledger-refresh.sh` is updated with the ratified matrix and passes;
- no dashboard REQUEST_CHANGES remain on the manager PR or acknowledged Release Manager signal that refreshes the ledger evidence.

Until then, Gap 3 remains sequencing-held and PB must not dispatch or claim `pb_self_compile_fixed_point_strong`.
