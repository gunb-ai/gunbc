# Model-grounding EXTRACT

**Goal:** find bare `String`/`NonEmptyStr` fields that may duplicate existing concepts. **Method:** `v2.lens.grounding` folds the type-level concept index into candidates for semantic adjudication. **Validation:** whole-corpus extraction, explicit exclusions and retained refusal evidence; name coincidence alone never proves a grounding defect.

The carrier migration is [`gunbc.plans.reference_grounding_migration`](../../dag/gunbc/plans/reference_grounding_migration.dag). `enumerate_concepts()` includes records/coproducts, never variant names. `ConstructionJustification` remains `WallAfterGrounding { dissolves_to: SingleAuthority }`; `grounding_extract_closed_set_stall` owns the EXTRACT stall. `grounding_name_only_residual_boundary` is `OutsideModeledGuarantee`, not a live `RatchetForever` arm or the deterministic climb’s ceiling. DECIDE clears closed-set comparisons, consumer-cracked leaves and parse-input cases; meaning-dependent cases require CONFIRM. Open registries, grammar strings, opaque tokens and prose may legitimately stay strings despite matching concept names.

Whole-tree access requires fail-closed `concept_decl_facts(roots)` over `dag` + `src/v2`, using `medium_structure_project::parse_file`: `concept_decl_facts_live()` only reflects resolved entry closure; whole-tree resolve can yield `graph: None`. Marshal head type names for exact `String`/`NonEmptyStr` matching; an unrepresentable concept fails loudly, never `Err(_) => continue`.

- `build_concept_name_index` replaces quadratic scanning with O(unique names) lookup.
- Exclude meta self-coincidence only for substrate meta names `field`/`name`, and layer-DAG inversion (`std ← extdeps ← compiler ← workflow`). Role words such as `id` still reach adjudication.
- Surface sibling variants, `Unrecognized*`/`NonExternal*` flags and optional-decode siblings as evidence, never automatic exclusions.
- `GroundingWorklistEntry` retains `enclosing`, `field`, `declared_type`, resolved-qualified `coincides_with`, `target_kind`, `target_structure`, enclosing `qualified_name`, `layer_excluded`, `sibling_variant_names`, `has_unrecognized_sibling`, `sibling_decode_optional`. `candidates()` excludes layer-included=false rows; `worklist()` retains them for audit.

The historical haiku `GROUND`/`KEEP` judge (`starts_with` grading) was deleted as unconsumed on 2026-08-22; this plan does not claim a live CONFIRM implementation. Target-kind/structure enrichment was its precision lever. Evaluation history is [PR #5302](https://github.com/gunb-ai/gunbc/pull/5302) and the [pinned corpus](https://github.com/gunb-ai/gunbc/blob/d919d662ddf5/src/v2/lens/testdata/anemia_confirm_eval_corpus.json), not a retained test fixture.

Remove this document when EXTRACT finds no unadjudicated coinciding bare-string fields, groundedness is a walked graph property, and the host bridge dissolves into v2 self-host compile-graph access (#5364).
