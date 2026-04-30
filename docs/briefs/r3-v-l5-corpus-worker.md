# R3 T-V-L5-Corpus Worker Brief

**Status:** STANDBY. Sequentially gated on `T-V-L4-L7-Direct` landing.

**Owning manager:** R3 Verification Manager.

## Scope

Build the L5 cross-target equivalence corpus:

`l5_cross_target_consistency` — for every materialized certification-corpus program, emitted Rust, Python, and Go programs produce algebraically equivalent runtime behavior.

L5 is corpus-driven and target-to-target. It does not compare target output to `.dag` evaluation directly; that is L4. L5 consumes the L4 corpus so the two verification surfaces stay aligned.

## Dispatch preconditions

- `T-V-L4-L7-Direct` has landed and owns a checked-in certification-corpus manifest.
- R2-Grounding Rust, Python, and Go are closed for Shape A emission.
- R2-Evaluator cross-target harness primitives support strict multi-target receipts, not only named scaffold claims.
- The target execution environment is deterministic enough for algebraic comparison, including normalized treatment of stdout, structured values, and effect receipts.

## Implementation outline

1. Reuse the L4 corpus manifest as the sole corpus authority.
2. Materialize one L5 row per corpus program with the target set frozen at dispatch.
3. Execute every target artifact under isolated namespaces and capture structured outputs/effect receipts.
4. Compare algebraic values, not byte-equal stdout. Float, effect, and diagnostic normalization rules must be explicit fixture data or declared harness policy.
5. Add coverage ratchets: every L4 corpus program has an L5 row once all target artifacts exist.

## Acceptance

- `l5_cross_target_consistency` evaluates Pass for each corpus row.
- Each failure identifies the program and the pair or set of targets that disagree.
- The corpus authority is shared with L4; L5 must not maintain a parallel hand-curated list that can drift.

## STOP conditions

- A target lacks grounded emission for a corpus program. Stop and return to the relevant Grounding owner rather than deleting the program from the corpus.
- The harness needs a generic "forward reference" staging predicate to land rows before target declarations exist. Existing `ReleaseDeferredClaim` and `SubstrateResearchDeferredClaim` are fixture-specific and must not be reused as generic staging.
- Effect or float semantics are ambiguous enough that comparison policy would become target-specific. Escalate the policy gap before authoring rows.

## Non-goals

- L4 emit/eval matching and L7 witnesses.
- L6 structural-form coverage.
- Per-target byte-identical fixed-point artifacts; that belongs to PB `T-FixedPoint`.

## Cross-refs

- Manager: [`r3-verification-manager.md`](r3-verification-manager.md).
- Upstream lane: [`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md).
- R3 authority: [`docs/r3-structure.md`](../r3-structure.md) `T-Verification-L5-Corpus`.
- Harness primitive history: [`r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md).
