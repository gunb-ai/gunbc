# R3 Formal-Grounding Verification TC Bundle

**Status:** TRACKING / author-now-fire-later. This brief records TC1 / TC2 / TC3 state for R3 Verification Manager ownership. It does not activate strict claims before their substrate and Evaluator prerequisites exist.

**Owning manager:** R3 Verification Manager.

## Scope

Track the formal-grounding TestClaim bundle:

| Claim | Current surface | State |
|---|---|---|
| **TC1 eta-equivalence** | `src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag` + `SubstrateResearchDeferredClaim` runner scope | Deferred fixture landed and scoped. Valid only in the TC1 fixture; not a generic staging mechanism. |
| **TC2 Church-Rosser / evaluation-order independence** | `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag` | Slice-0 hook landed. Strengthens to strict strategy-output equality after executable strategies and `DimensionReport<C>` evaluation exist. |
| **TC3 strong normalization** | Text-form in [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) §"TC3" | Declarative shape authored by PB; Verification owns the transition to a real substrate / runner encoding. Current substrate cannot express the universal meta-theorem. |

## Audit notes at authoring

- TC1 is live as a scoped deferred fixture and runner validation explicitly rejects use outside `tc1_substrate_lens_eta_equivalence_deferred.dag`.
- TC2 fixture and suite are live; strict activation remains gated on Evaluator strategy and PB-Runtime/lens primitive prerequisites.
- TC3 has no `.dag` fixture on main. This is intentional: the current `TestPredicate` variants are per-program, while TC3 quantifies over the well-typed fragment.
- `ReleaseDeferredClaim` and `SubstrateResearchDeferredClaim` are fixture-specific. They must not be repurposed as a generic forward-ref or meta-theorem staging mechanism.

## TC1 activation path

Strict TC1 activation requires:

1. T-Substrate-Lens-Primitive landed with the needed lens equality surface.
2. Lambda-calculus / substrate grounding authority landed for eta-equivalence.
3. Runner encoding that checks eta-equivalence as a real property rather than accepting a deferred marker.

Until then, keep the existing fixture scoped and fail-closed.

## TC2 activation path

Strict TC2 activation requires:

1. R2-Evaluator PR-A.3 strategy carriers implemented.
2. At least two executable strategies or strategy orders that can evaluate the same lens application.
3. A comparison surface over strict outputs, expected to be `DimensionReport<C>` equality once `Lens<C>` evaluation is live.

The existing deferred fixture remains the named hook: `evaluation_order_independent_lens_results`.

## TC3 activation path

Declarative claim:

```dag
test_claim every_typed_dag_program_terminates_in_bounded_steps {
  // For any well-typed .dag program with declared LoopBound,
  // evaluation terminates in O(loop_bound) reduction steps.
}
```

Activation requires:

1. B5 Loop construction-closure audit landed.
2. T-Substrate-Lens-Primitive landed.
3. A substrate encoding for the universal proof obligation, or an approved proof-checking runner surface. Existing per-program predicates are insufficient.

If a new predicate, quantifier carrier, or proof witness carrier is needed, Verification escalates through `INVARIANTS.md` §P1 instead of authoring it locally.

## STOP conditions

- Any attempt to use TC1/TC2 deferred fixtures as blanket approval for unrelated claims.
- Any TC3 implementation that tests only a finite corpus while naming the universal theorem as closed.
- Any new `TestPredicate` variant authored directly from this brief without substrate-introduction review.

## Cross-refs

- Manager: [`r3-verification-manager.md`](r3-verification-manager.md).
- TC1 fixture: [`../../src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag`](../../src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag).
- TC2 fixture: [`../../src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag`](../../src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag).
- TC3 source shape: [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) §"TC3".
- Evaluator manager: [`r2-evaluator-manager.md`](r2-evaluator-manager.md).
