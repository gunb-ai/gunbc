# Generative TestClaim Categories

> Part of: `src/v4/TASKS.md` T-19, `src/v4/lens/testgen.dag`, `src/v4/std/verification.dag`, `TESTING.md`, `INVARIANTS.md`.
>
> Purpose: define the wishlist of TestClaim families that can be generated from existing substrate facts without producing tautological tests.

## Rule

A generated TestClaim must add evidence not already proven by the source row that scheduled it.

Mechanical exclusion test: if the claim result can be proven from the scheduling row alone, without execution, an independent witness, a negative fixture, or a cross-target observation, the generator skips that row.

This excludes `f(x) = f(x)`, type-conformance for already typechecked inputs, and identity laws copied directly from an algebra declaration with no sample, witness, or falsification path. Uncertain rows fail closed by not generating a claim.

## Categories

| Category | Source rows | Generated claim | Non-tautology basis | Mechanical skip |
|---|---|---|---|---|
| Cross-target language behavior equivalence | Modeled callable or type-construction subject plus two target language identities | Same typed input produces equivalent observed output across target models | Frozen I/O snapshot now; target-pair runtime observation once multi-target eval is live | Skip if both sides are the same target and the expected value is just the input row |
| Algebra-law conformance | `data carrier: Algebra<T>` rows and law symbols | Law expression normalizes to the independently declared expected witness | Law witness or falsifying negative fixture, not the algebra row itself | Skip if lhs and rhs are structurally identical before law application |
| Coproduct exhaustiveness | Functions consuming a closed coproduct | Every variant has a handled branch or produces a typed diagnostic | Negative fixture for a missing variant or branch coverage witness | Skip open/user-extensible coproducts and functions whose body is unavailable |
| Refinement preservation | Functions with refined input and refined output | Refined input produces output satisfying the output refinement | Output witness checked independently of the function type | Skip if the output refinement is the same node as the input refinement with no transform |
| Witness validity | `Witness<T>` data rows | Witness payload satisfies the claimed property | Re-run the property checker over the payload, not the witness constructor | Skip witnesses whose payload is absent or whose property checker is unavailable |
| Idempotent operation conformance | Operations declared idempotent by algebra/effect inhabitance | Applying the operation twice equals applying it once | Behavioral double-application sample or algebra witness | Skip operations where idempotence is only asserted by a label and no operation body exists |

## Landed Slices

`LanguageBehaviorEquivalence` is live in `src/v4/lens/testgen.dag` via `testgen_emit_language_behavior_equivalence_claim` and `testgen_scheduled_language_behavior_generators`. The generated corpus is `src/v4/test/claim/generated/language_behavior_equivalence.dag`; it carries three runner receipts through `run_test_claim` / `run_test_claim_assert`.

`AlgebraLaw` now has a generator emission helper, `testgen_emit_algebra_law_claim`, and a first generated Nat corpus at `src/v4/test/claim/generated/algebra_law_conformance.dag`. The sample rows keep the generated source side and expected-law witness side as separate nodes so the row is not `lhs == lhs`.

`src/v4/test/claim/generated/testgen_category_wishlist.dag` records the remaining pending generator rows with an oracle basis and dispatch key. That file is a dispatch artifact, not a second authority for `TestgenConcept`; the closed scheduling coproduct remains `src/v4/lens/testgen.dag`.

## Worked Samples

Cross-target LBE samples:

- `T19ManualLbeConjDagSurface` checks a `Conj` type-node against the `.dag` language model surface snapshot.
- `T19ManualLbeDisjDagSurface` checks a `Disj` type-node against the same language-model authority.
- `T19ManualLbeTransformDagSurface` checks a `Transform` computation-node snapshot.

Algebra-law samples:

- `generated_nat_add_left_identity_claim` emits a Nat addition identity claim from the law symbol and expected witness node.
- `generated_nat_add_associativity_claim` emits a Nat addition associativity claim.
- `generated_nat_mul_annihilator_claim` emits a Nat multiplication zero-annihilator claim.

## Dispatch Plan

The next generator PRs should land one category at a time and include:

- the generator function in `src/v4/lens/testgen.dag` or a tighter existing authority if the category already has one;
- at least three generated TestClaim rows under `src/v4/test/claim/generated/`;
- a structural test or script guard proving the rows are generated through the category emission helper;
- an explicit tautology-skip path, preferably represented as a negative fixture or count witness.

Recommended order: witness validity, coproduct exhaustiveness, idempotent operation conformance, refinement preservation. Algebra-law broadening from the Nat sample to all algebra carriers can proceed in parallel with those category workers.
