# M0: 06_translate.dag function disposition table

> **Status:** WAVE 0 deliverable for parent review (dep-graph #1546 §1).
> **Session:** crisp-eagle-209 (closeout) · **Parent:** witty-raven-599 · **Work item:** node://adhoc-63bb0252-ace
> **Baseline census (HEAD):** 4912 lines · 171 `fn` · 21 `*_bounded` defs · 10 `*_go` defs · 18 `project_*` defs · 4 `fold_node(` call sites

Disposition key: `algebra-row` | `deleted-as-traversal` | `deleted-as-fuel` | `moves-to-target_model` | `kept+why`

§1-invalidation-c flags: functions that branch on facts not carried on `Node` / `TargetModel` rows — escalate before forcing into algebra.

---

## A. Public surface (must preserve)

| fn | disposition | notes |
|---|---|---|
| translate | kept+why | stage entry; orchestrates fold |
| translate_emitted_node | kept+why | cross-module witness helper |
| target_serialize_source_from_model | kept+why | emit consumer surface; body repoints to grammar-inverse fold |
| coerce_grounded_node | kept+why | coercion entry via std/coercion.dag |

---

## B. translate_algebra spine (new + repoint targets)

| fn | disposition | notes |
|---|---|---|
| translate_algebra | algebra-row | **NEW** — `fn(target) -> NodeFold<TargetValueExpression>`; per-connective data rows |
| translate_node | algebra-row | repoint → `fold_node(n, translate_algebra(target))` |
| translate_fold_step | algebra-row | absorbed into algebra `step` |
| translate_fold_init | algebra-row | absorbed into algebra `init` |
| translate_type_fold_init | algebra-row | Atom/Arrow/… init arm |
| translate_computation_fold_init | algebra-row | Computation init arm |
| translate_type_expression_tree | algebra-row | type-expr subtree fold; grammar-first short-circuit stays in step |
| translate_grammar_relation_row_match | kept+why | mode-1 grammar row gate; not a connective arm |

---

## C. Dual-path shim (SG-2 dissolve — delete with ProjectionAbsent arm)

| fn | disposition | notes |
|---|---|---|
| translate_node_mvp1 | deleted-as-traversal | orphaned when all targets carry projection row |
| translate_mvp1_fold_init | deleted-as-traversal | |
| translate_mvp1_coerce_from_grounding_or_evidence | algebra-row | E-9 bodied-arrow arm; merge into algebra init |
| translate_node_with_projection | deleted-as-traversal | unified translate_node |
| translate_node_computation | deleted-as-traversal | fold_node owns |
| target_bundle_child_optional_type_expr_projection | deleted-as-traversal | TypeExprProjectionPresence coproduct deleted |

---

## D. project_type_expression_* (target: ≤3 survivors)

| fn | disposition | connective | notes |
|---|---|---|---|
| project_type_expression_node | kept+why | — | **sole repoint:** `fold_node(node, translate_algebra(target))` |
| project_type_expression_node_bounded | deleted-as-fuel | — | |
| project_type_expression_connective_bounded | deleted-as-traversal | dispatch | → algebra connective rows |
| project_type_expression_conj_fields | algebra-row | Conj | WAVE 1 child |
| project_type_expression_disj_variants | algebra-row | Disj | WAVE 1 child |
| project_type_expression_field_slots | algebra-row | Conj | |
| project_type_expression_variant_slots | algebra-row | Disj | |
| project_type_expression_arrow_emitted_from_original_split | algebra-row | Arrow | |
| project_set_collection_type_node_impl | algebra-row | Cardinality | |
| project_set_collection_type_node | algebra-row | Cardinality | |
| project_free_monoid_collection_type_node | algebra-row | Cardinality | |
| project_type_expression_children | deleted-as-traversal | — | |
| project_type_expression_children_go | deleted-as-traversal | — | |
| project_type_expression_field_slots_go | deleted-as-traversal | — | |
| project_type_expression_variant_slots_go | deleted-as-traversal | — | |
| project_type_expression_subtree | deleted-as-traversal | — | |
| project_type_expression_children_in_node | deleted-as-traversal | — | |
| project_type_expression_children_in_node_go | deleted-as-traversal | — | |

---

## E. serialize_type_expr_* (token layout → target_model rows)

| fn | disposition | notes |
|---|---|---|
| serialize_type_expr_emitted_bounded | deleted-as-fuel | wrapper + bounded worker |
| serialize_type_expr_emitted_wire_bounded | deleted-as-fuel | |
| serialize_type_expr_generic_apply_bounded | deleted-as-fuel | moves-to-target_model for delimiter layout |
| serialize_type_expr_args_bounded | deleted-as-fuel | |
| serialize_type_expr_args_go | deleted-as-traversal | |
| serialize_type_expr_separated_bounded | deleted-as-fuel | |
| serialize_type_expr_separated_go | deleted-as-traversal | |
| serialize_type_expr_record_bounded | deleted-as-fuel | |
| serialize_type_expr_record_fields_bounded | deleted-as-fuel | |
| serialize_type_expr_record_fields_go | deleted-as-traversal | |
| serialize_type_expr_sum_bounded | deleted-as-fuel | |
| serialize_type_expr_arrow_bounded | deleted-as-fuel | |
| serialize_type_expr_arrow_named_bounded | deleted-as-fuel | |
| serialize_type_expr_arrow_param_bounded | deleted-as-fuel | |
| serialize_type_expr_arrow_params_go | deleted-as-traversal | |
| serialize_type_expr_arrow_inputs_bounded | deleted-as-fuel | |
| serialize_type_expr_arrow_inputs_go | deleted-as-traversal | |
| serialize_type_expr_boundary_bounded | deleted-as-fuel | |
| serialize_type_expr_boundary_atom_bounded | deleted-as-fuel | |
| serialize_type_expr_emitted_positional_head_tail | deleted-as-traversal | → node_query / fold_list |
| serialize_type_expr_record_field_label | moves-to-target_model | binding spellings row |
| serialize_type_expr_arrow_param_spelling | moves-to-target_model | |
| serialize_type_expr_arrow_input_wrapped | algebra-row | Arrow arm helper |

---

## F. Fuel triads (measure + bounded wrapper — delete all)

| fn | disposition | notes |
|---|---|---|
| node_subtree_count | kept+why | until all fuel deleted; then optional |
| target_translation_rules_budget | deleted-as-fuel | |
| translate_serialize_measure | deleted-as-fuel | |
| token_item_serialize_measure | deleted-as-fuel | |
| token_list_node_budget | deleted-as-fuel | |
| token_sequence_serialize_measure | deleted-as-fuel | |
| target_serialize_relation_row_from_model_bounded | deleted-as-fuel | grammar-inverse fold replaces |
| token_item_to_source_bounded | deleted-as-fuel | |
| token_item_to_source | deleted-as-fuel | wrapper deleted with bounded |
| token_sequence_to_source_bounded | deleted-as-fuel | |
| token_sequence_to_source | deleted-as-fuel | |
| target_serialize_source_from_model_bounded | deleted-as-fuel | |
| type_expr_arrow_split_from_types_bounded | deleted-as-fuel | |
| type_expr_arrow_split_from_types | deleted-as-traversal | → algebra Arrow arm |
| translate_project_arrow_input_types_go | deleted-as-traversal | |
| translate_project_arrow_split_types | algebra-row | Arrow ownership; fold absorbs |

---

## G. Grammar-inverse serialize (row interpreter — fold, not delete)

| fn | disposition | notes |
|---|---|---|
| target_serialize_relation_row_from_model_bounded | deleted-as-fuel | see F |
| target_serialize_bodied_arrow_from_model | algebra-row | bodied Arrow serialize; dissolve d) |
| grammar_relation_row_for_emitted | kept+why | find row in rules bundle |
| grammar_relation_row_emitted | kept+why | row field accessor |
| grammar_relation_row_tokens_root | kept+why | |
| grammar_relation_row_with_bodied_scaffold | kept+why | |
| grammar_relation_row_for_bodied_arrow_serialize | kept+why | |
| grammar_inverse_bodied_arrow_validated | kept+why | |
| grammar_inverse_source_validated | kept+why | |
| grammar_serialize_source_matches_row | kept+why | |
| grammar_tokens_node_targets | deleted-as-traversal | → conj_positional_targets / node_query |
| serialize_concrete_syntax_tokens_to_source | algebra-row | token list fold |
| token_spelling_from_model | moves-to-target_model | lex rule lookup |
| token_sequence_item_kind | kept+why | row item discriminant |
| concrete_syntax_token_from_node | kept+why | structural decode |

---

## H. Bodied-arrow / value-expression (#4627 dissolves a, d)

| fn | disposition | notes |
|---|---|---|
| translate_bodied_arrow_coerce_signature_slot_raw | algebra-row | Arrow signature slots |
| translate_bodied_arrow_coerce_signature_slots | algebra-row | |
| arrow_has_transform_body (import) | — | **dissolve d:** 4 dispatch sites → translation_rules |
| value_expression_to_concrete_tokens (import) | — | **dissolve a:** fold over TargetValueExpression |
| translate_shell_attach_value_expression | algebra-row | value tier |
| translate_apply_use_site_ownership_to_value_expression | algebra-row | |

---

## I. Coercion / bundle lookup (mostly kept; bespoke folds → find_witness)

| fn | disposition | notes |
|---|---|---|
| coercion_candidates_from_target_model | kept+why | |
| target_bundle_child | kept+why | |
| target_bundle_optional_child | kept+why | |
| target_bundle_child_lookup_step | kept+why | |
| target_selection_priority_from_model | kept+why | |
| conj_named_child | kept+why | |
| conj_optional_named_child | kept+why | |
| conj_positional_targets | deleted-as-traversal | thin Conj guard; inline at sites |
| optional_named_child_ignoring_positional | kept+why | |
| concrete_token_class_child | kept+why | |
| concrete_token_kind_child | kept+why | |

**§1-invalidation-c candidate:** `grammar_relation_row_for_emitted` — hand fold over rules with BundleChild* accumulator; **dissolve b:** → std find_witness once row set is closed candidate set.

---

## J. Collection realization (Cardinality arms)

| fn | disposition | notes |
|---|---|---|
| target_collection_repr_from_node | moves-to-target_model | |
| target_collection_witness_from_node | moves-to-target_model | |
| target_collection_fold_list_from_node | deleted-as-traversal | |
| target_collection_choice_from_node | moves-to-target_model | |
| target_collection_fallback_from_node | moves-to-target_model | |
| target_collection_realization_from_bundle | kept+why | bundle decode |
| collection_realization_from_target | kept+why | |
| free_monoid_collection_realization_from_target | kept+why | |
| collection_realization_from_target_carrier | algebra-row | Cardinality |

---

## K. Ownership / use-site (coercion shell — fold candidate WAVE 2)

| fn | disposition | notes |
|---|---|---|
| translate_coerced_with_atom_realization | algebra-row | Atom arm |
| translate_coerced_with_atom_realization_at_use_site | algebra-row | |
| translate_coerced_shell | deleted-as-traversal | |
| translate_coerced_shell_base | deleted-as-traversal | |
| translate_coerced_shell_at_use_site | deleted-as-traversal | |
| translate_coerced_shell_at_use_site_from_source | algebra-row | **§1-invalidation-c:** branches on Instantiation vs other connectives — must become algebra row |
| translate_coerced_type_shell_from_source | algebra-row | |
| translate_use_site_ownership_source_carrier | kept+why | |
| translate_apply_use_site_ownership_to_type_shell | algebra-row | |
| translate_apply_use_site_ownership_to_projected_boundary | algebra-row | |
| translate_apply_use_site_ownership_to_projected_type | algebra-row | |
| translate_use_site_ownership_catalog_from_model | kept+why | |
| translate_reference_layer_tokens_from_target | moves-to-target_model | |
| translate_value_semantics_carriers_from_model | kept+why | |
| translate_sg_rc_bundle_edge_present | kept+why | |
| translate_sg_rc_bundle_apply_disposition | kept+why | |
| translate_outcome_is_catalog_lookup_miss | kept+why | |
| translate_outcome_is_signature_realization_miss | kept+why | |
| translate_boundary_source_carrier_node | moves-to-target_model | **dissolve c:** resolve-stage rows |
| function_boundary_site_to_ownership_use_site | kept+why | |
| translate_target_function_signature_realization_for_boundary | moves-to-target_model | |
| target_atom_realizations_catalog_from_model | kept+why | |
| translate_target_atom_realization_for_carrier | algebra-row | Atom realization lookup |
| translate_atom_realization_value_from_source_at_use_site | algebra-row | |

---

## L. Type-expr shape decode (projection bundle readers)

| fn | disposition | notes |
|---|---|---|
| type_expression_projection_from_bundle | kept+why | |
| type_expression_projection_from_target | kept+why | |
| type_expr_shape_symbol | kept+why | |
| type_expr_optional_shape_symbol | kept+why | |
| type_expr_atom_shape_from_node | moves-to-target_model | shape already on projection row |
| type_expr_generic_apply_shape_from_node | moves-to-target_model | |
| type_expr_sum_shape_from_node | moves-to-target_model | |
| type_expr_arrow_shape_from_node | moves-to-target_model | |
| type_expr_atom_binding | kept+why | |
| type_expr_atom_binding_from_positional | deleted-as-traversal | |
| type_expr_carrier_identity | kept+why | |
| type_expr_spelling_for_atom | moves-to-target_model | binding_spellings row |
| node_atom_identity | kept+why | |
| lex_rules_literal | deleted-as-traversal | alias of lex_rules_literal_for_class |
| lex_rules_literal_for_class | moves-to-target_model | |
| lex_rule_literal_step | deleted-as-traversal | → target_model lex row fold |

---

## M. List/edge helpers (deleted-as-traversal)

| fn | disposition | notes |
|---|---|---|
| edge_list_head_and_tail | deleted-as-traversal | → std list_at / fold_list |
| list_head_and_tail | deleted-as-traversal | |
| serialize_source_tokens_fold_child | deleted-as-traversal | |
| serialize_source_fold_list_targets | deleted-as-traversal | |
| serialize_source_token_targets | deleted-as-traversal | |

---

## N. Diagnostics (all kept+why)

| fn | disposition |
|---|---|
| translate_target_bundle_malformed_diagnostic | kept+why |
| translate_target_bundle_duplicate_edge_diagnostic | kept+why |
| translate_sg_rc_bundle_partial_diagnostic | kept+why |
| translate_grammar_inverse_not_realized_diagnostic | kept+why |
| translate_grammar_relation_row_not_found_diagnostic | kept+why |
| translate_conj_invalid_edge_diagnostic | kept+why |
| translate_conj_named_child_missing_diagnostic | kept+why |
| translate_concrete_token_class_missing_diagnostic | kept+why |
| translate_concrete_token_kind_invalid_diagnostic | kept+why |
| translate_serialize_recursion_limit_exhausted_diagnostic | kept+why | deleted when fuel=0 |
| translate_lex_rule_literal_not_found_diagnostic | kept+why |
| translate_binding_spelling_not_found_diagnostic | kept+why |
| translate_type_expression_shape_missing_diagnostic | kept+why |
| translate_type_expr_kind_authority_invalid_diagnostic | kept+why |

---

## O. Summary counts

| disposition | count |
|---|---|
| algebra-row | 38 |
| deleted-as-traversal | 52 |
| deleted-as-fuel | 28 |
| moves-to-target_model | 22 |
| kept+why | 31 |
| **total** | **171** |

New artifacts: `translate_algebra` (+ connective row data), std `find_witness` for grammar row lookup (dissolve b).

---

## P. WAVE 1 fan-out (post parent M0 approval)

| child | connective | PR scope |
|---|---|---|
| W1-Atom | Atom | repoint init + delete Atom bounded arms |
| W1-Conj | Conj | field_slots + conj_fields |
| W1-Disj | Disj | variant_slots + disj_variants |
| W1-Arrow | Arrow | arrow split + bodied + ownership |
| W1-Cardinality | Cardinality | set/free-monoid collection |
| W1-Instantiation | Instantiation | generic_apply serialize |

---

## Q. §1-invalidation-c escalations (do not force)

1. `translate_coerced_shell_at_use_site_from_source` — branches on `Instantiation` connective not in algebra row set yet.
2. `grammar_relation_row_for_emitted` — bespoke unique-row fold; needs closed candidate set + find_witness (dissolve b).
3. `arrow_has_transform_body` at 4 sites — needs translation_rules dispatch row (dissolve d).
4. Fuel elimination (_bounded=0) — requires structural termination witness; coordinate with still-raven-546 algebra shape.
