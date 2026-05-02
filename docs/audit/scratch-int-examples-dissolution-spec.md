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
| `BoundDeclaration = StaticBound(Interval<Int>) \| PlatformDependent` | **Yes, landed in #1449** | `src/v3/std/substrate.dag` declares the carrier. Ground-truth at HEAD: `StaticBound` wraps the shared `Interval<Int>` parent; `Interval<D>` remains `BoundedInterval { lower: D, width: IntervalWidth } \| Unbounded`. #1449 did not introduce an `ExactInterval { lo, hi }` shape. |
| Program syntax / lowering for `Int(lo..hi)` to `BoundDeclaration` | No for this fold | #1449 explicitly did not add parser/lowerer syntax for `Int(lo..hi)`, `Int(any)`, or `Int(platform)`. Design-doc `(lo..hi)` syntax must lower into `StaticBound(BoundedInterval { lower, width })` before Grounding can read it structurally. |
| Per-target integer inhabitance rows with algebra + bound facts | Partial, landed in #1459 | `src/v3/std/emit_model.dag` declares `TargetIntegerTypeInhabitance { language, kernel_integer, algebra, bound, type_realization }` and `TargetIntegerInhabitanceBound = BoundUnspecified | StaticBoundFact(IntInterval)`. `src/v3/spec/{rust,python,go}.dag` includes Example 8 i32-range rows for Rust/Python/Go. Rust also has `u32` and unspecified signed rows, and Python has an unbounded row. Full dissolution remains blocked on program-bound parse/lower and algebra intent; Examples 1/2/5/6 are not all dispatchable from real program facts yet. Owner: Substrate / LanguageSpec population for remaining rows as needed. |
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
    pub bound: TargetIntegerInhabitanceBoundRef,
    pub realization: TargetInhabitanceRef,
}
```

The exact Rust API can differ, but the authority shape should be:

- `target` selects Rust / Python / Go language rows.
- `type_inhabitances` is extracted from `src/v3/spec/{target}.dag` LanguageSpec
  data, not authored in Rust.
- `algebra` identifies the algebra family used for candidate selection.
- `bound` carries the #1459 row-level `TargetIntegerInhabitanceBound` fact:
  `BoundUnspecified` or `StaticBoundFact(IntInterval)`. The static-bound payload
  uses the same `Interval<Int>` shape as `BoundDeclaration`; the row does not
  embed `BoundDeclaration` directly.
- `realization` is the target carrier emitted after a unique candidate is found.

This projection is a read model over substrate rows. It should not invent target
choices or fallback defaults.

## Cross-Cutting Fold Predicate

Example 8 requires a single predicate:

```text
match_bound(
  program_bound: BoundDeclaration,
  target_bound: TargetIntegerInhabitanceBound
) -> MatchResult
```

Expected semantics:

- `StaticBoundFact(Interval<Int>)` on a target row matches only the same program
  `BoundDeclaration::StaticBound(Interval<Int>)` for emission. At HEAD this
  means exact equality on `BoundedInterval { lower, width }` or the shared
  `Unbounded` variant; there is no `ExactInterval { lo, hi }` carrier.
- `BoundUnspecified` is evidence for missing-bound diagnostics; it must not
  satisfy exact-bound Example 8 selection.
- `PlatformDependent` is distinct from static intervals; it cannot be silently
  treated as unbounded or as a host-machine integer.
- ordering / nearest-wider relations are diagnostic-only and must not select an
  emission target.

This predicate shape is now implementable for a synthetic Example 8 program
bound over the carriers landed in #1449 and #1459. It is not yet dispatchable
end-to-end for real source programs: program bounds are not lowered into
`BoundDeclaration`, and algebra intent remains unavailable for the other
examples.

## Slice Sequencing

1. **Slice A: substrate bound carrier.**  
   Owner: Substrate Manager (`#1130`). **Done in #1449.** `BoundDeclaration`
   is declared as `StaticBound(Interval<Int>) | PlatformDependent`; the payload
   uses the existing generic `Interval<D>` parent with
   `BoundedInterval { lower, width } | Unbounded`.

2. **Slice B: per-target integer inhabitance rows.**  
   Owner: Substrate / LanguageSpec population. Add Rust, Python, and Go integer
   inhabitance rows carrying algebra and `BoundDeclaration` facts. Include the
   rows needed by Examples 1, 2, 5, 6, and 8 first. This is also outside
   Grounding's authority unless explicitly delegated by Substrate. **Partial in
   #1459:** target rows use `TargetIntegerInhabitanceBound`, not
   `BoundDeclaration` directly, and Example 8's cross-target i32-range rows are
   present.

3. **Slice B.5: program-bound parse/lower and algebra intent projection.**  
   Owner: Substrate / parse-lower, coordinated by Substrate Manager. Lower
   `Int(lo..hi)`, unbounded/static-any, and platform-dependent forms into
   `BoundDeclaration`, and expose Semiring vs OrderedRing intent or an
   ambiguity diagnostic. Still open after #1449.

4. **Slice C: declared projection reader.**  
   Owner: Grounding after the remaining Slice B/B.5 prerequisites land. A
   partial Example 8-only Slice C is unblocked by #1459 because its target rows
   now exist; it should keep Examples 1/2/5/6 on the scratch path. Full Slice C
   replaces `ScratchIntExamples` with a declared projection extracted from the
   `Dag`, keeps `Undeclared` fail-closed, and adds tests that construct or load
   the minimal LanguageSpec rows and assert the same outputs currently covered
   by scratch examples.

5. **Slice D: remove scratch driver.**  
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

- Program-bound parse/lower into the landed `BoundDeclaration` carrier.
- Remaining per-target integer inhabitance row population beyond the #1459
  Example 8 set.
- Real algebra disambiguation for Example 5.
- Full cross-target selection from real lowered program facts.

Grounding should STOP+PING `#1130` rather than self-authoring substrate facts if
an implementation slice needs any of the blocked items above.
