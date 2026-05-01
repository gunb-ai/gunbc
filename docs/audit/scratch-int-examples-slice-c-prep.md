# ScratchIntExamples Slice C Prep

**Date:** 2026-05-01  
**Lane:** T-Ground-Coercion-Fold  
**Scope:** audit / implementation prep only. No code changes.

## Summary

This audit prepares the Grounding-owned Slice C named in
`docs/audit/scratch-int-examples-dissolution-spec.md`: replace the transitional
`LanguageSpecProjection::ScratchIntExamples` test driver with a declared
projection reader after Substrate lands Slice A (`BoundDeclaration`) and Slice B
(per-target integer inhabitance rows).

The implementation slice should be mechanical: read target inhabitance facts from
the `Dag`, run a pure candidate-search pipeline, and preserve the outcomes of
Examples 1, 2, 5, 6, and 8 without hardcoded selector variants.

Substrate amendments remain Substrate Manager territory (`#1130`). Grounding
consumes them after they land.

## API Surface

`LanguageSpecProjection` should stop being an example selector and become a
read model over declared LanguageSpec rows. A concrete Rust-side shape can be:

```rust
pub enum LanguageSpecProjection<'dag> {
    Undeclared,
    Declared(DeclaredLanguageSpecProjection<'dag>),
}

pub struct DeclaredLanguageSpecProjection<'dag> {
    pub target_language: DeclarationId,
    pub type_inhabitances: Vec<TargetTypeInhabitance<'dag>>,
}

pub struct TargetTypeInhabitance<'dag> {
    pub row_decl: DeclarationId,
    pub source_type: DeclarationId,
    pub algebra: DeclarationId,
    pub bound: BoundDeclarationView<'dag>,
    pub realization: TargetRealizationRef,
}

pub struct TargetRealizationRef {
    pub type_realization_decl: DeclarationId,
    pub carrier_decl: Option<DeclarationId>,
    pub carrier_name: String,
}
```

Field intent:

- `target_language` is DeclarationRef-keyed. It identifies the target
  `LanguageSpec` row (`rust_language`, `python_language`, `go_language`, etc.).
- `type_inhabitances` is value-owned inside the projection. It is extracted from
  the `Dag` once so `fold_program_to_target` can search candidates without
  repeatedly walking declarations.
- `row_decl`, `source_type`, and `algebra` stay DeclarationRef-keyed. They point
  back to substrate rows and algebra/type declarations; they are not copied into
  string keys.
- `bound` should borrow or view the substrate `BoundDeclaration` payload. If the
  compiler representation cannot borrow nested payloads safely, copy the small
  normalized value into `BoundDeclarationView`; do not copy the target decision.
- `realization` points at the target realization row and carries only the render
  handle needed after a unique candidate is found.

Candidate-search API:

```rust
impl<'dag> DeclaredLanguageSpecProjection<'dag> {
    pub fn candidates_for_type_and_algebra(
        &self,
        source_type: DeclarationId,
        algebra: DeclarationId,
    ) -> impl Iterator<Item = &TargetTypeInhabitance<'dag>>;
}
```

The projection should own candidate rows, not the `Dag`. It may borrow row
payload values from the `Dag` through `'dag` views, but all identity remains
`DeclarationId` / DeclarationRef-keyed. No string-name lookup should become a
second authority.

## Bound Match Predicate

After Substrate Slice A lands, Grounding should implement one cross-target
predicate:

```rust
pub enum BoundMatch {
    Matches,
    DiffersExact,
    DiffersKind,
}

pub fn match_bound(program: &BoundDeclarationView, target: &BoundDeclarationView) -> BoundMatch
```

Expected structural body:

1. `StaticBound(Unbounded)` on the target side is universal for static program
   bounds of the same algebra family. It matches any `StaticBound(_)` program
   bound. It does not match `PlatformDependent`.
2. `StaticBound(BoundedInterval { lower, width })` matches only if the program
   is also `StaticBound(BoundedInterval { lower, width })` with exact structural
   equality. There is no wider/narrower target selection for emission; ordering
   remains diagnostic-only.
3. `PlatformDependent` is kind-only. It matches only a program bound that is also
   `PlatformDependent`. It must not be silently interpreted as the host's current
   integer width and must not match a static interval.

This function is intentionally independent of Rust/Python/Go. Example 8's
cross-target behavior is the proof obligation: Rust `i32`, Python `int`, and Go
`int32` must all be selected by the same predicate over declared facts.

## Post-Dissolution Test Shape

Each current scratch test should translate to a declared-projection test. The
fixture may be a tiny in-memory projection assembled by test helpers, or a small
`.dag` fixture loaded through the normal bootstrap path once Substrate rows are
available.

Minimal fixture row shape:

```text
TargetTypeInhabitance {
  row_decl,
  source_type: std.Int,
  algebra: std.algebra.Semiring | std.algebra.OrderedRing,
  bound: BoundDeclaration,
  realization: TypeRealization row,
}
```

Translation table:

| Current scratch test | Declared-projection fixture | Expected Slice C assertion |
|---|---|---|
| `design_doc_example_1_unrefined_int_under_refined` | Rust target projection with multiple `OrderedRing` signed Int rows carrying distinct static bounds; program bound is absent / unspecified. | Candidate search finds multiple bound-distinct rows and returns `UnderRefined { unspecified_axis: "bound" }`. |
| `design_doc_example_2_bounded_int_emits_rust_u32` | Rust target projection with unsigned Semiring rows including `RustU32` at `StaticBound(Interval<Int>(0..2^32))`; program bound is the same static interval. | `candidates_for_type_and_algebra(Int, Semiring)` plus `match_bound` yields one candidate: the `RustU32` realization row. |
| `fold_dag_int_ambiguous_algebra_fails_closed` | Projection contains both signed OrderedRing and unsigned Semiring candidate families for the program type/bound; program facts do not determine which algebra family applies. | Fold fails before target selection with `UnderRefined { unspecified_axis: "algebra" }`. |
| `fold_dag_int_bound_exceeds_max_no_inhabitant` | Rust target projection has signed Int rows up to the largest declared static bound; program bound is outside all declared static rows, or has no exact match. | Candidate search returns zero matching rows and emits `NoInhabitant`. |
| `fold_dag_int_refined_cross_target_consistent_rust` | Rust target projection includes `RustI32` with `StaticBound(Interval<Int>(-2^31..2^31))`. | Unique candidate is the `RustI32` realization row. |
| `fold_dag_int_refined_cross_target_consistent_python` | Python target projection includes `PythonInt` with a compatible static/unbounded declaration as defined by Substrate Slice B. | Same `match_bound` predicate selects the `PythonInt` realization row. |
| `fold_dag_int_refined_cross_target_consistent_go` | Go target projection includes `GoInt32` with `StaticBound(Interval<Int>(-2^31..2^31))`. | Unique candidate is the `GoInt32` realization row. |

The tests should assert declaration identity for the selected realization, not
the current `TargetInhabitance` enum variant.

## TargetInhabitance Migration

`TargetInhabitance::{RustU32, RustI32, PythonInt, GoInt32}` is a lane-local
mirror of target rows. Keeping it after declared projection lands would preserve
a parallel authority: target identity would live both in `src/v3/spec/*.dag` and
in Rust enum variants.

Slice C should replace `TargetInhabitance` with a row reference:

```rust
pub struct SelectedTargetInhabitance {
    pub inhabitance_row: DeclarationId,
    pub realization: TargetRealizationRef,
}
```

If callers still need a stable display value, derive it from the selected row's
carrier / realization fields at render time. Do not add new Rust enum variants
for target primitives after the declared projection is available.

## Failure-Mode Coverage

Slice C should preserve the minimal diagnostic outcomes first, then expose the
payload gap explicitly:

- Example 1 should still return `UnderRefined { unspecified_axis: "bound" }`,
  but the structural candidate set should be available internally. A follow-up
  diagnostic-payload slice can add candidate lists and resolution hints.
- Example 5 should still return `UnderRefined { unspecified_axis: "algebra" }`.
  The test should prove this comes from ambiguous algebra facts, not a selector
  variant.
- Example 6 should still return `NoInhabitant`; the test should retain the
  candidate rows considered so richer diagnostics can later cite them.

Current substrate `EmissionDiagnostic` only carries `UnderRefined {
unspecified_axis: String }` and `NoInhabitant` for these paths. Candidate lists
and hints are named-deferred; Slice C should make the missing payload observable
by retaining candidate-search evidence in the fold internals or test helper, not
by fabricating payload fields locally.

## Substrate-Prerequisite Checklist

Slice C cannot start until these substrate landings are present:

| Prerequisite | Required shape | Owner |
|---|---|---|
| Bound carrier | `BoundDeclaration = StaticBound(Interval<Int>) | PlatformDependent`, where `Interval<D>` remains the shared parent already present in `src/v3/std/substrate.dag`. | Substrate Manager (`#1130`) |
| Program-bound projection | Parsed/lowered program `Int(...)` refinements expose a `BoundDeclaration` fact the fold can read. | Substrate / parse-lower owner, coordinated by Substrate Manager |
| Per-target integer inhabitance rows | Rust, Python, and Go LanguageSpec rows declare source type, algebra, target realization, and bound facts for the Int-family examples. | Substrate / LanguageSpec population |
| Algebra intent facts | Program type/refinement analysis can distinguish Semiring vs OrderedRing, or fail closed when ambiguous. | Substrate Manager, consumed by Grounding |
| Projection extraction path | Grounding can walk the `Dag` and extract declared inhabitance rows without string-name authority. | Grounding after the rows above land |

No additional substrate prerequisite surfaced beyond the queued A/B family plus
the program-bound/algebra projection details implied by those slices. If
Substrate lands A/B without a readable program-bound projection, Grounding should
STOP+PING `#1130` rather than adding a local parser or string convention.
