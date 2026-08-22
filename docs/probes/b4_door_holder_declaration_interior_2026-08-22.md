# Boundary 4's door-holder moves off retention entirely: a declaration's interior is not a body-producer subject (2026-08-22)

| | |
|---|---|
| what establishes this | one two-arm execution over the same binary (the classifier branch removed between runs), one three-variant bisect, and four seven-boundary receipts |
| producer | `v2.workflow.product_receipt_transport` `run_seven_boundary_product_receipt`, executed through `gunbc run` on a BuildBuddy runner |
| subject | `src/v2/compiler/00_compile.dag`, the compiler's own 107-module closure |
| repository ref | **current `main` at dispatch time, plus the branch diff** — same constraint as the predecessor document's §4, and for the same reason |
| what it continues | [b4 door-holder: the statement-form let](b4_door_holder_statement_let_2026-08-21.md) |

---

## 1. The prediction held, to the diagnostic

The predecessor document, written before #8828 merged, predicted boundary 4 would then refuse with `7 parse_grammar_choice_overlap_residue / 37 body_lowering_reason_wrapper_retained_emitted / 1 normalized_tree_reason_wrapper_retention_not_normalized`. Re-measured post-#8828: exactly that, count 45. A receipt landing where a committed prediction placed it is the strongest evidence shape this lane has produced, and it is recorded before anything this document adds.

## 2. The first failing module is the subject's own root

`src/v2/compiler/00_compile.dag`. Established from the diagnostic ledger's atom renderings rather than from the count: the retained shells spell `TranslateTo`/`Eval`, `TranslateResult`/`EvalResult`, `CompileLens`/`CompileLensGate`/`CompileLensEnforce`/`CompileLensIntrospect`, the five stage fn types, and seven `gate:` field inits naming that module's seven `CompileLens` rows. Every symbol is declared there.

The 37 partition by emitted production identity as **30 type syntax** (`dag_surface_type_variant`, `dag_surface_type_alias_rhs`, `dag_surface_field_decl_block`, `dag_surface_fn_type`) and **7 `dag_surface_field_init`**. Their retention producer is the single one, `body_lower_wrapper_retained_shell`, reached through `body_lower_production_emitted`'s final else arm.

## 3. The root, and why it is a classification gap rather than an admission one

`body_lower_is_metadata_preserved_emitted` already preserves the OUTER declarations — `dag_surface_type_decl`, `dag_surface_data_decl`, `dag_surface_type_expr`, `dag_surface_qualified_name`, `dag_surface_param_list`, `dag_surface_generic_params`. Body lowering is a bottom-up fold over the whole module tree, so every node *inside* an already-preserved declaration re-entered the dispatch and fell to the unregistered-producer arm.

So type syntax surviving into an admitted `NormalizedTree` was already this repository's standing decision; what was missing was that the decision was written per node while the fold reaches inside. `admit_normalized_tree` is untouched.

## 4. The two-arm execution

One dispatch, one binary, the `body_lower_is_structure_preserved_emitted` branch deleted between the two runs (the `.dag` corpus is read at runtime, so no rebuild intervenes):

| source | with the branch | without it |
|---|---|---|
| `type T = A { x: Int } \| B` + a fn | `ACCEPTED` | `body_lowering_reason_wrapper_retained_emitted` |
| `type F = fn(Int) -> Int` + a fn | `ACCEPTED` | `body_lowering_reason_wrapper_retained_emitted` |
| `type R { a: Int }` + `data d: R = R { a: 1 }` + a fn | `ACCEPTED` | `body_lowering_reason_wrapper_retained_emitted` |

The right-hand column is also the reachability evidence for the retained arm: withdraw the classification and it fires immediately.

**A falsified first attempt is recorded because it cost a dispatch and would cost the next reader one.** The same three sources WITHOUT a function refuse at `namespace_graft_body_dissolved_refused` — before and after the repair alike. Written that way the rows asserted a string that could never be true, which is a row that cannot pass rather than a red.

## 5. The bisect, and what it settles

| variant | B4 count | partition |
|---|---|---|
| type syntax only | 15 | 7 overlap + 7 retained + 1 retention door |
| `field_init` only | 38 | 7 overlap + 30 retained + 1 retention door |
| all five | 8 | 7 overlap + 1 `parse_g0_tokens_remain` |

Exactly 30/7, no overlap, each half independently effective, **neither half alone opening the door**. That is what makes the two halves separately reversible as a measured fact rather than an intention.

## 6. Where the door is now, and one thing not to do with it

```
B4 Refused, count 8    7  parse_grammar_choice_overlap_residue
                       1  parse_g0_tokens_remain
   census: families 0, observations 0
```

Boundary 4 is now held by a **parse** diagnostic — a different stage from every door this lane has faced. B5, B6 and B7 remain `NotExercised`; the seven-boundary transport is still five-sevenths unexecuted and this document does not claim otherwise.

Leftover tokens after a parse is the shape that most tempts a repair at the consumption site. Every root this lane has found tonight fired one layer below where the diagnostic appeared. `v2.workflow.realization_sweep` `member_scan` exists precisely to attribute `parse_g0_tokens_remain` rows to a closure member — it drives each member through the same canonical assembly as a singleton ingest — and is the instrument to name the subject before anything is repaired.

## 7. One thing this run did NOT establish, named rather than left to be found

Reaching the receipt at all required the `OccurrenceId` producer repair that gunbc#8854 carries: `dag_int_literal_node_from_magnitude` / `_from_lexeme` stored a bare `OccurrenceId` into `Node.occurrence_id`, which is the `NodeOccurrenceId` coproduct, so with retention cleared the run died with `PatternMatchFailure` and wrote no receipt. That defect is `main`'s, inherited through the merge, and none of it is in this lane's diff. The executed before/after for it is a comment on that PR.
