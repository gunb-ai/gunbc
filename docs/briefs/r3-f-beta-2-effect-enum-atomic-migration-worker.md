# R3 F-beta.2 Effect Enum Atomic Migration Worker

## Scope

Gate #82 (`effect_enumeration_lens_behaviorally_complete`) migrates the Lane 2
effect-enumeration carrier from the retired parallel `OperationEffect` record to
`v3.std.services::Operation` as the single operation authority.

This PR is the atomic implementation slice for the four internal phases named by
the R3 F-beta canvas:

- resource-threading through `Operation`
- `OperationEffect` / `WorkflowOperation` retirement
- lens-body rewrite for `WorkflowEffect.LinearEffect.ops`
- `CompositionVerdict` consumers over `Operation`

## P5 Receipt

**Receipt type:** explicit deferral to the existing R3 lane row and the
ROADMAP-adjacent carve-routing row that moved that lane into R3.

**Lane / rows:**

- `docs/r3-program-plan.md` §1.8 gate #82,
  `effect_enumeration_lens_behaviorally_complete`.
- `docs/r4-carve-out-routing.md` §"C2 —
  `effect_enumeration_lens_behaviorally_complete`" (dissolved
  carve-promotion-IN-R3 row), which names Cluster F sub-phases F-beta.1 /
  F-beta.2 and the locked `services.dag::Operation` carrier target.

**Why this satisfies `INVARIANTS.md` P5 mechanism (b):** the PR expands
hand-written Rust under `src/v3/` only as a co-located bootstrap bridge for an
authority move that removes the old `OperationEffect` substrate carrier and
threads the declared `Operation` carrier into the existing consumers. The net
steady-state direction is dissolution: operation identity now flows from
`services.dag::Operation` rather than from a second effect-specific record.

**Dissolution trigger:** remove the native `operation_effect_shape` /
`compose_operation_effects` mirror once `ArrowBody::Unparsed` lowering can
execute the `std.effects` bodies for `lane2_workflow_idempotency_report` and the
Stage 2e parallelism lens through the bootstrapped evaluator.

**Interim ratchets:**

- `OperationEffect` and `WorkflowOperation` are absent from the v3 effect carrier
  surface.
- `CallableRef` carries declaration identity, not a string operation name.
- keyless `PUT` / `PATCH` / `DELETE` classification fails closed as
  `KeylessFallback` instead of fabricating a resource key.
- `PairwiseNonCommute` exports the closed `.dag` reason string until typed
  non-commute evidence lands on `WorkflowParallelismReport`.
