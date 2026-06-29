# Model-grounding lens — whole-tree EXTRACT

> DESIGN.md §2 leaf-decomposition residue lens. Detector half of the [reference-grounding migration](reference-grounding-migration.md) effort; carrier half is tidy-badger-45. Sibling: [anemia-lens design history](anemia-lens.md) (PR #5302).

## 1. What the lens does

`v2.lens.grounding` is a pure fold over the type-level concept index that flags bare-`String` / `NonEmptyStr` fields whose **name** coincides with an existing concept that could ground them (e.g. `cursor: String` next to a `Cursor` type). It stores nothing and gates nothing alone — it narrows the corpus to a high-quality **candidate set** for a downstream CONFIRM (LLM) judge.

`enumerate_concepts()` is **type-level** (record/coproduct declarations), never coproduct-arm names. Coincidence is checked against concept **names** only, so `cursor`→a `Cursor` **arm** is structurally impossible as a false positive.

## 2. Construction justification (post-#5943 carrier)

The module carries `ConstructionJustification { class: RatchetForever }` — nullary, no free-text `undecidable_because` / `rationale` on the carrier (#5943 de-prosed the construction-justification ledger). The judgment below is load-bearing **design context** re-homed from the carrier (#5933, pre-#5964).

Whether a bare-string field **should be grounded** on an existing concept (it re-spells a type that already exists) or is legitimately a string (an open registry, a constraint/grammar string, an opaque token, or free-form prose) is **not decidable from the substrate**. Name-coincidence with a concept (Signal-A) is only a **necessary** candidate condition, never sufficient: `cursor`~`Cursor`, `mediaType`~open registry, `schedule`~`Schedule`(wrong-target) all coincide by name yet are correct as strings. Sufficiency requires semantic judgment of the field's meaning — the residual routes to a CONFIRM judge (LLM); a construction wall cannot make under-grounding unwritable any more than it can make non-optimality unwritable (Rice).

The decidable part (`is_bare_string` AND `name_coincides`) is a candidate signal only; the should-ground verdict is the undecidable residue. **Dissolves toward:** deterministic DECIDE clearing of provable cases (closed-set-by-comparison, consumer-cracks-the-leaf, parse-input fields) shrinks the residual sent to the judge, but the name-only residual remains `RatchetForever`.

## 3. Whole-tree EXTRACT (#5933 follow-up)

`concept_decl_facts_live()` reflects only the **resolved closure** of the run entry (~3 hits on `src/v2`-only). Production use requires `concept_decl_facts(roots)` — a fail-closed host builtin that walks `dsl` + `src/v2` via **parse-only** projection (`medium_structure_project::parse_file`), not whole-tree resolve (blocked: unresolved imports → `graph: None`).

**§5 cardinal rule:** never `Err(_) => continue` on marshal — that silently drops concepts whose compound fields (`List<T>`, …) cannot fully marshal. The host marshals **head type-names** only (the lens matches exact `"String"` / `"NonEmptyStr"`). Unrepresentable concepts fail **loud**.

## 4. Precision machinery

| Mechanism | Role |
| --- | --- |
| **Name index** | `build_concept_name_index` — O(unique names) lookup vs O(n²) linear scan |
| **Job A — layer exclusion** | Structural, bidirectional: meta self-coincidence (`FieldRef.field`~`Field` on substrate layers) + layer-DAG inversion (enclosing layer cannot import target layer per `std ← extdeps ← compiler ← workflow`). Replaces the hand-list `is_structural_name` **meta** half. |
| **Job B — role words** | `widget.id` etc. are **not** pre-filtered. Flow to CONFIRM / adjudication ledger (tidy-badger-45). |
| **Residue evidence** | Sibling coproduct variant names, `Unrecognized*` / `NonExternal*` / `Non*` flags, optional-decode siblings — **surfaced**, not pre-filtered. Verdict is the ledger's. |
| **CONFIRM judge** | `gunbc.tools.grounding_confirm` — haiku, `GROUND`/`KEEP`, `starts_with` grading. **Enrichment** (`target_kind` + `target_structure`) is the precision lever: bare ~4/6 wrong → enriched 6/6 on the haiku_confirm slice. Eval authority: `src/v2/lens/testdata/anemia_confirm_eval_corpus.json`. |

## 5. Worklist schema (feeds tidy-badger ITEM 4)

Each `GroundingWorklistEntry`:

- `enclosing`, `field`, `declared_type`
- `coincides_with` — **resolved qualified name** of the matched concept (identity, not bare name)
- `target_kind`, `target_structure`
- `qualified_name` — enclosing concept's qualified name
- `layer_excluded` — Job A already ran
- `sibling_variant_names`, `has_unrecognized_sibling`, `sibling_decode_optional` — fail-closed residue evidence

`candidates()` returns layer-included rows only; `worklist()` includes layer-excluded rows for audit.

## Dissolution trigger

Delete this doc when `v2.lens.grounding`'s EXTRACT pass returns empty over the corpus (no unadjudicated bare-String field coincides with a concept), groundedness is a walked graph property, and the host `concept_decl_facts(roots)` bridge dissolves into v2 self-host compile-graph access (#5364).
