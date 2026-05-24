# Generative TestClaim Categories

> Part of: `src/v4/TASKS.md` T-19, `src/v4/lens/testgen.dag`, `src/v4/std/verification.dag`, `TESTING.md`, `INVARIANTS.md`.
>
> Purpose: define the wishlist of TestClaim families that can be generated from existing substrate facts without producing tautological tests.

## Rule

A generated TestClaim must add evidence not already proven by the source row that scheduled it.

Mechanical exclusion test: if the claim result can be proven from the scheduling row alone, without execution, an independent witness, a negative fixture, or a cross-target observation, the generator skips that row.

This excludes `f(x) = f(x)`, type-conformance for already typechecked inputs, and identity laws copied directly from an algebra declaration with no sample, witness, or falsification path. Uncertain rows fail closed by not generating a claim.

## Canonical Arms

`src/v4/lens/testgen.dag` owns the closed `TestgenConcept` coproduct. This doc does not add a second category authority; it records which universally generated claim families route through each existing arm.

| TestgenConcept arm | Source rows | Generated claim | Non-tautology basis | Mechanical skip |
|---|---|---|---|---|
| `LanguageBehaviorEquivalence` | Type-construction or modeled callable subject plus target language identity | Same typed input produces equivalent observed output across target models | Frozen I/O snapshot now; target-pair runtime observation once multi-target eval is live | Skip if the expected value is just the input row without a language-model observation |
| `AlgebraLaw` | `data carrier: Algebra<T>` rows and law symbols | Law expression normalizes to the independently declared expected witness | Law witness or falsifying negative fixture, not the algebra row itself | Reject `lhs == rhs` at the generator boundary |
| `DiagnosticExhaustiveness` | Diagnostic reasons and negative fixtures | Ill-formed input produces the declared typed diagnostic | Negative fixture, not a success-path restatement | Skip when there is no negative fixture or the expected diagnostic is only copied from the producer |
| `LensApplicability` | Lens plus program subject | Lens observation over a program produces or rejects with the expected witness | Observation fixture, property checker, or negative fixture | Skip if the row only reruns the lens and compares its own emitted fact |
| `BidirectionalRoundtrip` | Language production plus target identity | Encode/decode or parse/emit roundtrip preserves the independently declared value | Differential roundtrip through two directions | Skip identity productions with no distinct decode/encode step |
| `TypeConstruction` | Connective or behavior construction subjects | Constructed node matches the structural witness or rejects arity errors | Construction witness or negative arity fixture | Skip type-conformance rows already proven by parsing/lowering alone |
| `RefinementPreservation` | Refined value plus original base | Refinement preserves the accepted base value | Accepted refinement witness plus base projection | Skip if the refinement was not accepted or the projected base is only restated without the refined carrier |

Operator wishlist examples route through those seven arms rather than extending the arm set: coproduct exhaustiveness is a `DiagnosticExhaustiveness` or `LensApplicability` family depending on whether the first slice is a missing-branch diagnostic or a branch-coverage lens observation; refinement preservation uses its canonical `RefinementPreservation` arm; witness validity and idempotent operation conformance route through `LensApplicability` until a later substrate change proves a more specific arm is needed.

## Landed Slices

`LanguageBehaviorEquivalence` is live in `src/v4/lens/testgen.dag` via `testgen_emit_language_behavior_equivalence_claim` and `testgen_scheduled_language_behavior_generators`. The generated corpus is `src/v4/test/claim/generated/language_behavior_equivalence.dag`; it carries three runner receipts through `run_test_claim` / `run_test_claim_assert`.

`AlgebraLaw` now has a generator emission helper, `testgen_emit_algebra_law_claim`, and a first generated Nat corpus at `src/v4/test/claim/generated/algebra_law_conformance.dag`. The helper carries the canonical `AlgebraLawSubject` into both emitted equality terms; the sample rows reuse the modeled Nat algebra/law and operation/value symbols from `v4.std.nat` and keep the generated source side and expected-law witness side as separate nodes so the row is not `lhs == lhs`.

`src/v4/test/claim/generated/testgen_category_wishlist.dag` records pending or dispatched rows through the canonical `Generator` shape: `kind`, `t19_anchor`, `classification`, and `slot`. That file is a dispatch artifact, not a second authority for `TestgenConcept`; the closed seven-arm scheduling coproduct remains `src/v4/lens/testgen.dag`.

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

Recommended order by canonical arm: `LensApplicability` witness-validity instance, `DiagnosticExhaustiveness` coproduct-exhaustiveness instance, `LensApplicability` idempotent-operation instance, and `RefinementPreservation` generated-corpus broadening. Algebra-law broadening from the Nat sample to all algebra carriers can proceed in parallel under the existing `AlgebraLaw` arm.
