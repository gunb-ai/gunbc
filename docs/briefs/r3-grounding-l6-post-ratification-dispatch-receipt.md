# R3 Grounding L6 Post-Ratification Dispatch Receipt

Status: Grounding implementation held until Substrate lands the `EmissionPathProjection` carrier.

## Verification

- `git-metadata`: available. Local branch `session/silent-badger-711` resolves at `43d57e50a815d6e4f1fd988d2761f4ce47e79904`.
- `src/v3/std/cross_target_coverage.dag`: absent at this HEAD, so the ratified carrier has not landed yet.
- Current substrate axes remain `TypeConnective` in `src/v3/std/substrate.dag` with six variants: `Atom`, `Conj`, `Disj`, `Arrow`, `Cardinality`, `Instantiation` (`src/v3/std/substrate.dag:214`). `Behavior` has five variants: `Value`, `Transform`, `Branch`, `Loop`, `Bind` (`src/v3/std/substrate.dag:464`).
- Current live L6 walker still uses hand-Rust axes and existing method-template lists: `src/v3/grounding_cross_target_meta/src/cells.rs` enumerates 6 x 5 x 3 cells; `coverage.rs` maps each non-empty per-target `MethodTemplateContract` list to `Cardinality x Transform x target`; `walker.rs` partitions all cells into present/missing diagnostics.

## Ratified Shape

Director ratified Option 2 from `docs/briefs/r3-substrate-l6-per-row-projection-routing-decision.md`: sibling projection carrier `EmissionPathProjection` keyed by `MethodTemplateContractKey`, with `MethodTemplateContract` rows left untouched.

Sub-question decisions now treated as locked for Grounding dispatch prep:

- 4.A = (a): place the L6 axes and projection carrier in `src/v3/std/cross_target_coverage.dag`.
- 4.B = `List<EmissionCell>` on each projection row.
- 4.C = target lives on `MethodTemplateContractKey`.
- 4.D = Substrate ships the carrier empty; Grounding owns row population in the follow-up.

The routing decision already records the handoff shape: Substrate declares `FormAxis`, `BehaviorAxis`, `ShapeATarget`, `EmissionCell`, and `EmissionPathProjection`; Grounding then populates the existing Phase 1 rows and converts `coverage.rs` to read per-row projection rows (`docs/briefs/r3-substrate-l6-per-row-projection-routing-decision.md:78`).

## Grounding Follow-Up Shape

After Substrate lands `src/v3/std/cross_target_coverage.dag` with the empty carrier:

1. Populate `EmissionPathProjection` rows for the existing Phase 1 `MethodTemplateContract` rows. Existing live audit says those rows cover `Cardinality x Transform x Shape A target`.
2. Convert `src/v3/grounding_cross_target_meta/src/coverage.rs` from list-non-empty coverage to per-row projection coverage by joining `MethodTemplateContractKey { target, dag_method }` to `EmissionPathProjection.row_identity` and unioning `cells`.
3. Keep `MethodTemplateContract` untouched.
4. Add tests for mixed-cell projection rows and fail-closed behavior when projection rows are missing, empty, malformed, or fail to join to a source row.

## Out Of Scope

- No Substrate carrier authoring in this Grounding receipt.
- No `MethodTemplateContract` migration.
- No `coverage.rs` or walker code changes before the carrier lands.
- No reuse of the old list-name to cell mapping once per-row projection is available.

## Test Note

Run `cargo test -p v3-grounding-cross-target-meta -- --nocapture` as the verification command for the current walker and for the follow-up conversion. The command was run for this receipt and passes at this HEAD.
