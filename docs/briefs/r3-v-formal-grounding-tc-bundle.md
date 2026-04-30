# R3 Formal-Grounding Verification TC Bundle

**Status:** AUTHOR-NOW-FIRE-LATER management brief.

**Owning manager:** R3 Verification Manager.

## Scope

Track and harden the three formal-grounding TestClaims that bridge R2 substrate/evaluator work into R3 verification:

| Claim | Current state | R3 action |
|---|---|---|
| **TC1 eta-equivalence** | Fixture exists at `src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag` using `SubstrateResearchDeferredClaim`. | Replace the deferred marker with an executable eta-equivalence claim once the substrate/lens proof surface lands. |
| **TC2 Church-Rosser / evaluation-order independence** | Fixture exists at `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag`; R2 Evaluator manager names it as the slice-0 hook. | Strengthen to strict strategy-output equality over `DimensionReport<C>` once PB-Runtime and `T-Substrate-Lens-Primitive` support the proof. |
| **TC3 strong normalization** | Text-form only in `r3-pb-t-fixedpoint-worker.md` §"TC3 — Strong-normalization TestClaim". | Resolve the substrate gap for a theorem over the whole well-typed fragment, then author the executable proof/claim fixture. |

## Current Audit

- TC1 is present, fixture-scoped, and runner-gated to prevent `SubstrateResearchDeferredClaim` from becoming a general release-deferral escape hatch.
- TC2 is present and suite-named `tc2_evaluation_order_independence_suite`; it remains a deferred/evaluator-semantics hook.
- TC3 correctly has no `.dag` fixture yet. Current `TestPredicate` variants are per-program and cannot express universal quantification over every well-typed `.dag` program with bounded loops.

## TC3 Substrate Gap

TC3 requires a structural proof shape for:

```dag
every_typed_dag_program_terminates_in_bounded_steps
```

The likely proof obligation is structural induction over `Behavior` and `LoopBound` / bounded-lattice facts. That does not fit the existing per-program `TestClaim.source` model. R3 Verification must either:

- Compose TC3 from a newly landed substrate proof surface, if Substrate provides one before dispatch.
- Escalate substrate introduction via `INVARIANTS.md` §P1 for a quantifier-capable proof predicate or equivalent structural theorem carrier.

Do not encode TC3 as a corpus test and call it theorem coverage. Corpus termination checks can be supporting evidence, but they do not discharge the universal claim.

## Fire Conditions

- **TC1 fires** when the eta-equivalence proof can compare two equivalent `.dag` forms through the same lens and produce identical lens results without a deferred marker.
- **TC2 fires** when at least two executable evaluation strategies exist and produce identical `DimensionReport<C>` / observable outputs for the same program set.
- **TC3 fires** when B5 Loop construction-closure plus the substrate proof surface make the universal termination theorem expressible and checkable.

## STOP Conditions

- Reusing `ReleaseDeferredClaim` for TC1 / TC2 / TC3.
- Adding a new `TestPredicate` variant directly from this Verification brief without the substrate-fact introduction procedure.
- Treating a finite corpus as sufficient for TC3's universal theorem.
- Letting a deferred marker remain after its executable proof surface lands.

## Cross-Refs

- TC1 fixture: `src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag`
- TC2 fixture: `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag`
- TC3 source: [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) §"TC3 — Strong-normalization TestClaim"
- R2 evaluator acceptance hook: [`r2-evaluator-manager.md`](r2-evaluator-manager.md) §"Acceptance — .dag gates"
