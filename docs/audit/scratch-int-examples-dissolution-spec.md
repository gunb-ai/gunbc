# ScratchIntExamples Dissolution Spec

**Date:** 2026-05-01  
**Lane:** T-Ground-Coercion-Fold  
**Scope:** audit / dispatch spec only. No code changes.

## Summary

`src/v3/grounding_coercion_fold` now carries five Int-family worked examples
through the transitional `LanguageSpecProjection::ScratchIntExamples` driver:
Examples 1, 2, 5, 6, and 8 from `docs/design-emission-model.md`. The current
implementation is intentionally stub-shaped: each selector variant returns the
expected diagnostic or `TargetInhabitance` without reading program bounds,
algebra facts, or per-target inhabitance rows from the `Dag`.

The dissolution path is to replace `ScratchIntExamples` with a declared
LanguageSpec projection that the fold can read structurally. Grounding should
not self-author the missing substrate facts; the substrate-amendment work belongs
with Substrate Manager / `#1130`.

## Current Receipts

- `docs/design-emission-model.md` Examples 1, 2, 5, 6, and 8 define the intended
  fold behavior.
- `src/v3/grounding_coercion_fold/src/types.rs` marks both
  `ScratchIntExamples` and `TargetInhabitance` as YELLOW / transitional.
- `src/v3/grounding_coercion_fold/src/fold.rs` documents that the scratch path
  ignores `Dag` and lifetime facts and fixes synthetic `BindingId(0)`.
- Landed scratch-shape PRs: `#1241`, `#1289`, `#1291`, `#1433`, `#1435`.
- `#1431` added a lockstep ratchet ensuring lane-local `EmissionDiagnostic`
  mirrors remain a subset of the substrate diagnostic sum.

## Per-Example Structural Derivation

| Example | Current scratch result | Structural result after dissolution |
|---|---|---|
| 1. `Int` with no bound | `UnderRefined { unspecified_axis: "bound" }` | Parse the binding type as algebra intent plus no `BoundDeclaration`. Walk target integer inhabitance rows for the selected target / algebra family. More than one bound-distinct inhabitant remains, so the fold fails closed on the missing bound axis. |
| 2. `Int(0..2^32)` | `RustU32` | Parse the program bound as `BoundDeclaration::StaticBound(Interval<Int>)`. Walk Rust Semiring / unsigned integer inhabitance rows. Apply exact `match(program.bound, target.bound)`. `rust_u32` is the unique row whose static interval equals `(0..2^32)`. |
| 5. `Int(0..2^32)` with ambiguous algebra | `UnderRefined { unspecified_axis: "algebra" }` | Resolve the program type and bound, then attempt algebra selection before target inhabitance selection. If the facts leave both signed / OrderedRing and unsigned / Semiring families viable, fail closed on the algebra axis before choosing a target primitive. |
| 6. `Int(0..2^65)` / overlarge Int | `NoInhabitant` | Resolve the program algebra and `BoundDeclaration`. Walk the target integer inhabitance set and apply the same exact bound predicate. If no target row matches, return `NoInhabitant` with candidates considered and resolution hints. |
| 8. `Int(-2^31..2^31)` cross-target | `RustI32`, `PythonInt`, `GoInt32` | Use the same `match(program.bound, target.bound)` predicate for every target. Rust chooses the signed `i32` inhabitance row. Python chooses the arbitrary-precision `int` row only if its bound declaration is structurally compatible with the program bound. Go chooses the signed `int32` row. The predicate is one cross-target fold rule, not three hardcoded target branches. |

## Substrate-Fact Gap Analysis

| Requirement | Present at HEAD | Gap / owner |
|---|---:|---|
| `Interval<D>` shared parent | Yes, in `src/v3/std/substrate.dag` | Generic `Interval<D>` exists as `BoundedInterval { lower: D, width: IntervalWidth } | Unbounded`; it is not a concrete `Interval<Int>` row and is not yet consumed by Coercion-Fold inhabitance selection. |
| `BoundDeclaration = StaticBound(Interval<Int>) \| PlatformDependent` | No | Substrate amendment. Owner: Substrate Manager (`#1130`). Grounding should consume after it lands. |
| Program syntax / lowering for `Int(lo..hi)` to `BoundDeclaration` | Partial / not declared for this fold | Needs substrate and parse/lower authority work before Grounding can read it structurally. |
| Per-target integer inhabitance rows with algebra + bound facts | No for the required full family | `src/v3/spec/{rust,python,go}.dag` has broad `TypeRealization` rows such as `rust_int`, `python_int`, `go_int`. Rust also has a narrow `rust_uint8` / `UInt8` row. None of these rows carry the design-doc `BoundDeclaration` facts, and the needed family rows (`RustI32`, `RustU32`, `PythonInt`, `GoInt32`, etc.) are not declared. Owner: Substrate / LanguageSpec population. |
| Algebra resolution facts for signedness ambiguity | Not enough for Example 5 | Need declared relationship between program Int aliases/refinements and algebra families before the fold can fail closed on algebra structurally. |
| Inhabitance-search infrastructure | No production path | `fold_program_to_target` currently accepts a scratch projection and ignores the `Dag`. A real path needs typed extraction of LanguageSpec inhabitance rows and a pure candidate-filter pipeline. |
| Diagnostic payload richness | Partial | Substrate has `UnderRefined` / `NoInhabitant`, but the lane-local mirror currently carries only the minimal fields used by scratch tests. Candidate lists and hints can follow after structural selection exists. |

## Declared Projection Shape

The replacement for `ScratchIntExamples` should be a projection of declared
LanguageSpec facts, not another enum of example names. A dispatchable shape:

```rust
pub struct DeclaredLanguageSpecProjection {
    pub target: TargetLanguageRef,
    pub type_inhabitances: Vec<TargetTypeInhabitance>,
}

pub struct TargetTypeInhabitance {
    pub target_type: DeclarationRef,
    pub algebra: DeclarationRef,
    pub bound: Option<BoundDeclarationRef>,
    pub realization: TargetInhabitanceRef,
}
```

The exact Rust API can differ, but the authority shape should be:

- `target` selects Rust / Python / Go language rows.
- `type_inhabitances` is extracted from `src/v3/spec/{target}.dag` LanguageSpec
  data, not authored in Rust.
- `algebra` identifies the algebra family used for candidate selection.
- `bound` carries the `BoundDeclaration` fact when the inhabitance is bounded.
- `realization` is the target carrier emitted after a unique candidate is found.

This projection is a read model over substrate rows. It should not invent target
choices or fallback defaults.

## Cross-Cutting Fold Predicate

Example 8 requires a single predicate:

```text
match_bound(program_bound: BoundDeclaration, target_bound: BoundDeclaration) -> MatchResult
```

Expected semantics:

- `StaticBound(Interval<Int>)` matches only the same static interval for emission.
- `PlatformDependent` is distinct from static intervals; it cannot be silently
  treated as unbounded or as a host-machine integer.
- ordering / nearest-wider relations are diagnostic-only and must not select an
  emission target.

This predicate is not implementable as a pure function over current substrate
facts because `BoundDeclaration` is not declared and the per-target inhabitance
rows do not carry bound declarations. Grounding can implement the function body
only after Substrate lands those facts.

## Slice Sequencing

1. **Slice A: substrate bound carrier.**  
   Owner: Substrate Manager (`#1130`). Declare `BoundDeclaration` as
   `StaticBound(Interval<Int>) | PlatformDependent`, plus the structural
   constructors needed by program bounds. This is not Grounding-owned.

2. **Slice B: per-target integer inhabitance rows.**  
   Owner: Substrate / LanguageSpec population. Add Rust, Python, and Go integer
   inhabitance rows carrying algebra and `BoundDeclaration` facts. Include the
   rows needed by Examples 1, 2, 5, 6, and 8 first. This is also outside
   Grounding's authority unless explicitly delegated by Substrate.

3. **Slice C: declared projection reader.**  
   Owner: Grounding after Slices A/B land. Replace `ScratchIntExamples` with a
   declared projection extracted from the `Dag`. Keep `Undeclared`
   fail-closed. Add tests that construct or load the minimal LanguageSpec rows
   and assert the same outputs currently covered by scratch examples.

4. **Slice D: remove scratch driver.**  
   Owner: Grounding. Delete `IntScratchExample`, the hardcoded
   `fold_design_doc_example_*` functions, and the lane-local `TargetInhabitance`
   variants that are only mirrors of declared rows. Tests should assert the
   structural fold path, not selector variants.

## Dispatch Boundary

Unblocked today:

- Document this dissolution path.
- Keep scratch examples honest and bounded.
- Add tests around existing scratch behavior when requested.

Blocked on Substrate:

- Real `BoundDeclaration` matching.
- Per-target integer inhabitance row population.
- Real algebra disambiguation for Example 5.
- Cross-target Example 8 selection as a single structural predicate.

Grounding should STOP+PING `#1130` rather than self-authoring substrate facts if
an implementation slice needs any of the blocked items above.
