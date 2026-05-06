---
status: draft (pre-author; held until dispatch trigger)
authority parent: R3 Grounding Manager (#1745)
issue: #1876
schedule row: docs/r3-design-schedule-2026-05-06.md G3
trigger: executable LanguageSpec projection replacement landed + manager reauthorization
---

# R3 G3 — Coercion-Fold Scratch Retirement Worker Brief

## Status

This is a held worker brief only. Do not implement this slice from the brief
alone.

Dispatch is authorized only after both conditions are true:

1. An executable `LanguageSpec` projection replacement has landed.
2. R3 Grounding Manager reauthorizes G3 against that landed shape.

The current schedule marks G3 as held with trigger "executable LanguageSpec
projection" and scope
`LanguageSpecProjection::ScratchIntExamples` + `TargetInhabitance` retirement
in `src/v3/grounding_coercion_fold`.

## Context

`src/v3/grounding_coercion_fold` currently carries the Int-family design
examples through the transitional
`LanguageSpecProjection::ScratchIntExamples` path. The current public fold
boundary is:

```rust
fold_program_to_target(
    dag,
    lifetime_facts,
    language_spec,
) -> Result<BTreeMap<BindingId, TargetInhabitance>, EmissionDiagnostic>
```

`LanguageSpecProjection::Undeclared` fails closed with
`EmissionDiagnostic::FoldNotImplemented`. `ScratchIntExamples` then drives
Examples 1, 2, 5, 6, and 8 for a single synthetic binding after verifying the
bootstrap `Dag` carries declared `TargetIntegerTypeInhabitance` rows. Examples
2 and 8 already consume declared integer-row payloads in part; Examples 1, 5,
and 6 remain scratch-gated by missing program-bound / algebra-intent extraction.

The named dissolution authorities are:

- `docs/audit/scratch-int-examples-dissolution-spec.md`
- `docs/audit/scratch-int-examples-slice-c-prep.md`
- `docs/audit/scratch-int-examples-slice-d-target-inhabitance-retirement.md`
- `docs/briefs/t-ground-languagespec.md`

Those authorities agree on the invariant: Grounding consumes declared
LanguageSpec / target-inhabitance facts structurally. It must not self-author
missing substrate carriers, row populations, parser/lowering surfaces, or
target choices.

## Slice

Retire the scratch selector surface once the executable projection can replace
it end-to-end:

1. Replace `LanguageSpecProjection::ScratchIntExamples(IntScratchExample)` with
   the landed declared/executable `LanguageSpec` projection reader. Preserve the
   fail-closed no-projection state currently represented by
   `LanguageSpecProjection::Undeclared`; the fold must still report
   `FoldNotImplemented` or the landed equivalent typed no-projection diagnostic.
2. Delete `IntScratchExample` and the hardcoded
   `fold_design_doc_example_*` routing functions. The examples must be proved
   through declared projection rows and program facts, not enum variants.
3. Retire the lane-local `TargetInhabitance::{RustU32, RustI32, PythonInt,
   GoInt32}` mirror. Return a selected row/reference carrier derived from the
   declared projection, for example a `SelectedTargetInhabitance` containing the
   chosen inhabitance row identity plus realization reference. Do not introduce
   new Rust enum variants for target primitive choices.
4. Keep candidate selection structural and identity-keyed. Projection rows,
   source type, algebra, target language, bound, and realization identity must
   be represented by `DeclarationId` / typed row references or the landed
   equivalent. No string-name dispatch or heuristic target matching.
5. Run one fold predicate over all targets. Bound matching must use the landed
   `BoundDeclaration` / `TargetIntegerInhabitanceBound` shape and preserve the
   documented semantics:
   - target `StaticBoundFact(BoundedInterval { lower, width })` matches only
     the same static program interval;
   - target `StaticBoundFact(Unbounded)` is universal only for static program
     bounds of the same algebra family;
   - `BoundUnspecified` is under-refined evidence, not an emission match;
   - `PlatformDependent` does not silently match static target intervals.
6. Preserve typed fail-closed diagnostics. Missing projection rows, malformed
   projection rows, ambiguous algebra, absent required program bound, duplicate
   candidate identity, zero matching candidates, and multiple equally valid
   candidates must not produce a default target.

## Non-Goals

- Do not implement this brief before the dispatch trigger and manager
  reauthorization.
- Do not edit `src/v3/grounding_coercion_fold` while authoring this brief.
- Do not author or amend substrate carriers, per-target rows, parser/lowerer
  surfaces, or generated compiler SG-0 surfaces as part of G3 unless the
  manager explicitly re-scopes after the trigger.
- Do not broaden into G2, G5, T-Ground-Tests, T-Ground-Dissolve, or emit-shim
  work.
- Do not preserve scratch compatibility by adding another Rust example-selector
  enum.

## STOP Conditions

Stop and route back to R3 Grounding Manager before code changes if any of these
are true at dispatch HEAD:

- The executable `LanguageSpec` projection reader is absent, not loadable from
  the bootstrap `Dag`, or not public enough for Coercion-Fold to consume.
- Target inhabitance selection cannot be expressed through typed row identity
  and requires string dispatch, name-prefix matching, or target-specific
  heuristic branches.
- Program-bound or algebra-intent authorities are insufficient to replace
  Examples 1, 2, 5, 6, and 8. A narrower Example-8-only partial slice requires
  explicit manager reauthorization and must not delete the full scratch surface.
- Candidate uniqueness cannot be checked fail-closed.
- Preserving the no-projection `Undeclared` behavior requires weakening typed
  diagnostics or fabricating ordinary target-missing diagnostics.
- The implementation requires broad `src/v3/compiler/` SG-0 changes rather than
  consuming the landed projection.
- The landed projection conflicts with the T-Ground-LanguageSpec authority or
  leaves ownership of target-inhabitance facts ambiguous.

## Implementation Plan At Dispatch

1. Re-read the landed projection authority and verify the trigger. Record the
   commit / PR that supplied the executable projection in the PR body.
2. Audit current `src/v3/grounding_coercion_fold` for remaining scratch entry
   points and current callers of `fold_program_to_target`.
3. Add a declared projection read model in the crate only if the landed
   projection does not already supply one. Keep row identity typed; normalize
   only small value payloads such as interval bounds.
4. Implement candidate search by source type, target language, algebra, and
   bound. Return typed diagnostics for missing, ambiguous, malformed, and
   no-inhabitant cases.
5. Replace the fold output with selected declared-row identity and realization
   reference. Update local callers/tests mechanically.
6. Delete `IntScratchExample`, `ScratchIntExamples`, `TargetInhabitance`, and
   hardcoded design-example functions after structural tests cover the same
   behavior.
7. Keep the `Undeclared` / no-projection test as the first fail-closed ratchet.

## Acceptance Tests

The implementation PR should include focused tests proving the structural path
replaces the scratch surface:

- No-projection input still fails closed with `FoldNotImplemented` or the
  landed typed equivalent.
- Example 1a: absent or unspecified program bound over multiple bound-distinct
  declared candidates returns the typed bound-under-refinement diagnostic or
  landed equivalent.
- Example 1b: explicit `BoundDeclaration::StaticBound(Unbounded)` follows the
  declared bound predicate and must not be collapsed into the absent-bound
  diagnostic case.
- Example 2: `Int(0..2^32)` selects the declared Rust `u32` inhabitance row by
  exact static-bound and algebra match.
- Example 5: ambiguous algebra facts return
  the typed algebra-under-refinement diagnostic or landed equivalent before
  target selection.
- Example 6: overlarge `Int(0..2^65)` or no exact declared target row returns
  `NoInhabitant`.
- Example 8: Rust, Python, and Go all select their declared i32-range
  inhabitance rows via the same predicate, not target-specific branches.
- Malformed projection rows, duplicate row identities, duplicate equally valid
  candidates, missing realization references, and unjoined rows fail closed.
- A ratchet over `src/v3/grounding_coercion_fold/src` shows no surviving code
  references to `ScratchIntExamples`, `IntScratchExample`,
  the bare `TargetInhabitance` type authority, or
  `fold_design_doc_example_`. Exact identifier matching must exclude successor
  names such as `SelectedTargetInhabitance`.

## Verification

Run at minimum:

```bash
cargo fmt --all --check
cargo test -p v3-grounding-coercion-fold -- --nocapture
! rg -n '\b(ScratchIntExamples|IntScratchExample|TargetInhabitance|fold_design_doc_example_[A-Za-z0-9_]*)\b' src/v3/grounding_coercion_fold/src
```

If the landed executable projection requires bootstrap regeneration or parse
manifest refresh, run the repo-standard regen/check command named by the
failing test or local docs and include the command/output summary in the PR
body. Generated mirrors must be updated only through the established regen flow.

## PR Body Requirements

The implementation PR body must include:

- Trigger evidence: projection replacement PR/commit plus manager
  reauthorization link.
- Whether the slice is full scratch retirement or a manager-authorized narrower
  partial. Full retirement is the default required for this brief.
- The selected row/reference output shape replacing `TargetInhabitance`.
- Verification commands and results.
- Any STOP condition encountered and how it was resolved or escalated.
