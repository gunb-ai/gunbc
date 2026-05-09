# ScratchIntExamples Slice D TargetInhabitance Retirement

**Date:** 2026-05-02  
**Lane:** T-Ground-Coercion-Fold  
**Scope:** audit / dispatch spec only. No code changes.

## Summary

PR #1486 made Example 8 the first Coercion-Fold output whose target choice is
mediated by declared substrate rows: the fold walks
`TargetIntegerTypeInhabitance` declarations, filters by target language, applies
the structural bound predicate, and then maps the selected realization back into
the lane-local `TargetInhabitance` enum. That last mapping is intentionally
transitional. Slice D is the cleanup that removes the local output mirror after
the remaining Examples 1, 2, 5, and 6 can use declared projection facts too.

This audit specifies that retirement path. It extends the Slice C prep in
`docs/audit/scratch-int-examples-slice-c-prep.md` and the original dissolution
spec in `docs/audit/scratch-int-examples-dissolution-spec.md`.

## Current TargetInhabitance State

`src/v3/grounding_coercion_fold/src/types.rs` currently declares:

```rust
pub enum TargetInhabitance {
    RustU32,
    RustI32,
    PythonInt,
    GoInt32,
}
```

The enum is documented as Practice 4 / YELLOW: a lane-local output mirror for
hardcoded scratch examples that retires with `ScratchIntExamples` when declared
LanguageSpec projection can compute target inhabitance structurally.

Current call sites:

| Location | Current use | Retirement action |
|---|---|---|
| `types.rs` | Defines the four enum variants. | Delete after `fold_program_to_target` returns selected row identity instead of enum variants. |
| `lib.rs` public exports | Re-exports `TargetInhabitance` with `IntScratchExample` and `LanguageSpecProjection`. | Export `SelectedTargetInhabitance` or the declared projection output type instead. |
| `lib.rs` crate docs | Names `TargetInhabitance::RustU32` for Example 2. | Rewrite docs to describe selected `TargetIntegerTypeInhabitance` row + `TypeRealization` identity. |
| `lib.rs` tests | `assert_single_binding_inhabitance` compares enum variants for Example 2 and Example 8. | Compare selected `inhabitance_row` and `realization.type_realization_decl`. |
| `fold.rs` output type | `fold_program_to_target` returns `BTreeMap<BindingId, TargetInhabitance>`. | Return `BTreeMap<BindingId, SelectedTargetInhabitance>`. |
| `fold.rs` scratch functions | Examples 1/2/5/6 are hardcoded-shaped and Example 8 maps selected rows to enum variants. | Delete scratch functions once every example uses declared facts. |

No non-Coercion-Fold consumer of `TargetInhabitance` surfaced in the HEAD audit.
The enum is local to the crate API and sibling tests; no cross-lane caller needs
a compatibility bridge beyond the Slice D PR boundary.

## Transitional Bridges

PR #1486 intentionally added two string-based bridges so Example 8 could remain
inside the existing output type without prematurely shipping Slice D.

### Realization-to-Enum Mapping

`target_inhabitance_from_type_realization` reads the selected
`type_realization` declaration name and maps it to a local enum variant:

| Declaration name | Local variant |
|---|---|
| `"rust_i32"` | `TargetInhabitance::RustI32` |
| `"python_int"` | `TargetInhabitance::PythonInt` |
| `"go_int32"` | `TargetInhabitance::GoInt32` |
| `"rust_u32"` | `TargetInhabitance::RustU32` |

This is a string-name authority bridge. It is acceptable only while the public
fold output is `TargetInhabitance`. Slice D deletes the helper; the selected
`type_realization` `DeclarationId` becomes part of the output instead of being
converted through a Rust enum.

### Target Language Lookup

`target_language_id` maps the scratch target selector to declaration names:

| Scratch target | Declaration name |
|---|---|
| `Rust` | `"rust_language"` |
| `Python` | `"python_language"` |
| `Go` | `"go_language"` |

This is also transitional. In the declared projection path, target language
identity should be supplied by the projection itself:

```rust
pub struct DeclaredLanguageSpecProjection {
    pub target_language: DeclarationId,
    pub type_inhabitances: Vec<TargetTypeInhabitance>,
}
```

The fold should filter candidates by `projection.target_language`, not by
looking up language row names locally.

## Replacement Design

Slice D should replace `TargetInhabitance` with selected row identity:

```rust
pub struct SelectedTargetInhabitance {
    pub inhabitance_row: DeclarationId,
    pub realization: TargetRealizationRef,
}

pub struct TargetRealizationRef {
    pub type_realization_decl: DeclarationId,
    pub carrier_decl: Option<DeclarationId>,
}
```

Field intent:

- `inhabitance_row` is the selected `TargetIntegerTypeInhabitance` declaration.
  This is the row whose `language`, `kernel_integer`, `algebra`, `bound`, and
  `type_realization` facts made it the unique inhabitant.
- `realization.type_realization_decl` is the target `TypeRealization` row, such
  as `rust_i32`, `python_int`, `go_int32`, or `rust_u32`.
- `carrier_decl` is optional because current `TypeRealization` rows may expose
  different carrier details by target. It is render data derived from the
  selected row, not a second selection authority.

Tests should compare declaration identity, not enum variants. For Example 8,
the assertions become:

| Target | Expected inhabitance row | Expected realization |
|---|---|---|
| Rust | `rust_integer_inhabit_i32_at_program_bound` | `rust_i32` |
| Python | `python_integer_inhabit_at_i32_program_bound` | `python_int` |
| Go | `go_integer_inhabit_i32_at_program_bound` | `go_int32` |

The fold may still provide display helpers for diagnostics, but those helpers
must derive labels from declaration metadata after selection. They must not
become another selector enum.

## Per-Example Translation Table

| Example | Current scratch arm | Post-Slice-D shape | Prerequisite |
|---|---|---|---|
| 1. Unrefined `Int` | `fold_design_doc_example_1_unrefined_int` returns `UnderRefined { unspecified_axis: "bound" }`. | Declared projection walks target integer candidates for the chosen target/algebra and finds multiple bound-distinct rows because program bound is absent. Diagnostic remains `UnderRefined("bound")`; candidate evidence is structural. | Program-bound absence/unspecified projection plus enough target rows to prove ambiguity. |
| 2. `Int(0..2^32)` / Semiring | `fold_design_doc_example_2_semiring_u32` returns `TargetInhabitance::RustU32`. | Fold reads program `BoundDeclaration::StaticBound(BoundedInterval { lower: 0, width: 2^32 - 1 })`, filters Rust Semiring candidates, selects the `rust_u32` inhabitance row, and returns `SelectedTargetInhabitance`. | Program-bound parse/lower and algebra intent facts. #1459 already has a Rust u32 row, but the program facts are not yet readable end-to-end. |
| 5. Ambiguous algebra | `fold_design_doc_example_5_ambiguous_algebra` returns `UnderRefined { unspecified_axis: "algebra" }`. | Fold observes viable Semiring and OrderedRing candidate families for the same program bound before target selection and fails closed on algebra. | Algebra intent/ambiguity facts and enough signed/unsigned rows to make the ambiguity structural. |
| 6. Bound exceeds max | `fold_design_doc_example_6_no_inhabitant` returns `NoInhabitant`. | Fold reads the oversized static bound, searches declared rows for the target/algebra, and finds zero exact matches. | Rows covering the target family maximum plus program-bound parse/lower for the oversized interval. |
| 8. Cross-target i32 range | Three Example 8 arms now read declared rows but map selected realization back to `TargetInhabitance`. | Keep the #1486 row walk and bound predicate, but return `SelectedTargetInhabitance { inhabitance_row, realization }` directly. | Already unblocked for Example 8 by #1459 and #1486; blocked only on the output type migration. |

## IntScratchExample Retirement Sequencing

`IntScratchExample` currently has seven variants:

- `DesignDocExample1UnrefinedInt`
- `DesignDocExample2BoundedU32`
- `DesignDocExample5AmbiguousAlgebra`
- `DesignDocExample6NoInhabitant`
- `DesignDocExample8Rust`
- `DesignDocExample8Python`
- `DesignDocExample8Go`

Structurally cleaner recommendation: retire `IntScratchExample` all at once in
full Slice D, after full Slice C can drive all worked examples from declared
facts. The enum is a closed example selector. Removing only some variants would
leave a hybrid API where some tests enter through declared projection and others
still enter through a selector, preserving the exact parallel-authority window
Slice D is meant to close.

A partial deprecation pattern is possible but should be avoided unless delivery
pressure requires it:

1. Keep `LanguageSpecProjection::ScratchIntExamples` for unmigrated examples.
2. Move migrated examples to `LanguageSpecProjection::Declared`.
3. Mark migrated selector variants unreachable in tests.
4. Delete the enum when the final example migrates.

That path increases transition surface and test matrix complexity. It is only
worth doing if Substrate lands prerequisites unevenly and Director wants
incremental production coverage before the full set is ready. Otherwise, land
Slice D as one PR after Examples 1, 2, 5, 6, and 8 all run through the declared
projection.

## Test Migration Plan

| Current test | Current assertion | Slice D assertion |
|---|---|---|
| `fold_undeclared_projection_stays_fold_not_implemented` | `LanguageSpecProjection::Undeclared` returns `FoldNotImplemented`. | Keep unchanged unless `Undeclared` API shape changes; it does not depend on `TargetInhabitance`. |
| `scratch_int_examples_require_emit_model_inhabitance_rows` | Empty `Dag` plus scratch selector returns `UnderRefined("declared_TargetIntegerTypeInhabitance_rows")`. | Replace with declared-projection extraction test: empty `Dag` or empty projection returns the same under-refined row-source diagnostic. |
| `design_doc_example_1_unrefined_int_under_refined` | Scratch Example 1 returns `UnderRefined("bound")`. | Build/load projection with bound-distinct candidates; program fact has no bound; assert `UnderRefined("bound")` and candidate evidence count if exposed. |
| `design_doc_example_2_bounded_int_emits_rust_u32` | Enum result is `TargetInhabitance::RustU32`. | Assert selected inhabitance row is the Rust u32 static-bound row and realization is `rust_u32`. |
| `fold_dag_int_ambiguous_algebra_fails_closed` | Scratch Example 5 returns `UnderRefined("algebra")`. | Build/load projection with viable Semiring and OrderedRing families; assert the fold fails before realization selection with `UnderRefined("algebra")`. |
| `fold_dag_int_bound_exceeds_max_no_inhabitant` | Scratch Example 6 returns `NoInhabitant`. | Program bound exceeds declared maximum; assert zero matching rows and `NoInhabitant`. |
| `fold_dag_int_refined_cross_target_consistent_rust` | Enum result is `TargetInhabitance::RustI32`. | Assert `inhabitance_row == rust_integer_inhabit_i32_at_program_bound` and `realization == rust_i32`. |
| `fold_dag_int_refined_cross_target_consistent_python` | Enum result is `TargetInhabitance::PythonInt`. | Assert `inhabitance_row == python_integer_inhabit_at_i32_program_bound` and `realization == python_int`. |
| `fold_dag_int_refined_cross_target_consistent_go` | Enum result is `TargetInhabitance::GoInt32`. | Assert `inhabitance_row == go_integer_inhabit_i32_at_program_bound` and `realization == go_int32`. |
| `fold_dag_int_refined_cross_target_requires_matching_declared_bound` | Injects wrong bound and expects `NoInhabitant`, proving #1486 is not hardcoded. | Keep the same behavioral proof but assert it through declared projection test helpers. It should fail before any `SelectedTargetInhabitance` is produced. |

The helper `assert_single_binding_inhabitance` should become an
`assert_single_binding_selection` helper that takes expected declaration names or
resolved `DeclarationId`s. It should resolve names only in test setup; production
fold code should receive `DeclarationId`s from the projection.

## Substrate-Prerequisite Checklist

Slice D cannot ship until full Slice C is available. Required substrate and
Grounding prerequisites:

| Prerequisite | Status | Owner |
|---|---|---|
| Program-side bound carrier | Done in #1449: `BoundDeclaration = StaticBound(Interval<Int>) | PlatformDependent`. | Substrate |
| Target-side integer inhabitance carrier | Done in #1459: `TargetIntegerTypeInhabitance` with `TargetIntegerInhabitanceBound = BoundUnspecified | StaticBoundFact(IntInterval)`. | Substrate |
| Example 8 per-target rows | Done in #1459 for Rust/Python/Go i32-range rows. | Substrate / LanguageSpec population |
| Program-bound parse/lower | Open. Source `Int(lo..hi)` and related forms must lower into readable `BoundDeclaration` facts. | Substrate / parse-lower owner |
| Algebra intent facts | Open. Fold must distinguish Semiring, OrderedRing, and ambiguity structurally. | Substrate, consumed by Grounding |
| Remaining row coverage for Examples 1/2/5/6 | Partial. Rust u32 exists, but full proof fixtures require enough signed/unsigned/max-bound rows and real program facts. | Substrate / LanguageSpec population |
| Declared projection reader for all examples | Partial. #1486 covers Example 8 only and still maps to `TargetInhabitance`. | Grounding after substrate prerequisites land |
| Public output migration | Open. Replace `TargetInhabitance` with `SelectedTargetInhabitance`. | Grounding Slice D |

No additional substrate prerequisite surfaced from the TargetInhabitance audit.
The remaining blockers match the #1483 refresh: program-bound parse/lower,
algebra intent, and enough target rows for Examples 1, 2, 5, and 6.

## Slice D vs Partial Slice D

Recommended sequencing:

1. **Finish full Slice C first.** Examples 1, 2, 5, 6, and 8 should all execute
   through declared projection facts while the output is still temporarily
   `TargetInhabitance`. This keeps the behavioral migration separate from the
   public output type migration.
2. **Land Slice D as one Grounding PR.** Change the fold output to
   `SelectedTargetInhabitance`, delete `TargetInhabitance`, delete
   `IntScratchExample`, delete `fold_design_doc_example_*`, and update tests to
   assert row identity.
3. **Add a retirement ratchet.** A source-level ratchet should fail on new uses
   of `TargetInhabitance`, `IntScratchExample`, or `fold_design_doc_example_`
   outside archived audit docs. This mirrors the #1424 deferral-ratchet pattern
   and prevents the scratch selector from reappearing.

Partial Slice D is not preferred. It would preserve a mixed API with both
declared projection and scratch selectors, and it would still need the same final
cleanup. Use it only if Director explicitly asks for incremental output-type
retirement before full Slice C is ready.

## Receipts

- #1439: Slice C prep introduced `SelectedTargetInhabitance` as the intended
  replacement for the enum mirror.
- #1483: audit refresh narrowed Slice C after #1459 and named Example 8 as the
  only unblocked partial implementation.
- #1486: implemented Example 8 declared-row selection while keeping the enum
  output as a transitional bridge.
- `docs/design-emission-model.md`: Examples 1, 2, 5, 6, and 8 remain the worked
  example authority for expected fold behavior.
