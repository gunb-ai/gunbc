# EdgeLabel `Named` census — worktree quiet-koi-814 @ 60ef76cc5a

Subject: `v2.std.node` `EdgeLabel = Named { name: Symbol } | Positional`. Proposed shape: `Structural { label: StructuralEdgeLabel } | Authored { name: Symbol } | Positional`.

Instrument: `census.py` + `report.py` in this scratchpad (re-run: `python3 census.py rows.json && python3 report.py .` from repo root). Every `\bNamed\b` token in `*.dag` under `src/v2` and `dag/`, with `//` comments and string contents stripped, multi-line import lists excluded, and braces matched by depth (so nested payloads are handled).

## Premise correction: other types with a `Named` arm

Only three types in the corpus declare a `Named` variant:
- `v2.std.node` `EdgeLabel` (the subject)
- `std.algebra` `ContainerSource = SameAsReceiver | Named { name: String }`: 5 sites, all `Named { name: "List" }` in `dag/std/algebra.dag` string-method rows. Excluded.
- `extdeps.formats.spice` `Named { node: NamedNode }`: 22 sites (spice.dag plus spice fixtures and tests). Excluded.

`target_model.dag`, `testgen.dag`, `effects.dag`, `algebra.dag` (src/v2/std), and `body_lowering_fold.dag` have NO other `Named` type. All of their `Named {` hits are EdgeLabel (they import `Named` from `v2.std.node`). The excluded rows are listed at the end.

## Headline findings

1. **870 EdgeLabel sites.** 592 CONSTRUCT, 129 MATCH-on-name-text (95 compare against a known symbol, 34 are generic Symbol-parameter lookups), 49 bind the name and read it as a name, 48 arm-only, 41 inside constructor funnels, 7 pass-through relabels, 4 hash/canonical/wire sites.
2. **The structural/authored boundary is NOT carried at most Named sites; it lives in funnels and Symbol-keyed query APIs.** There are 29 per-language `*_named_edge(name: Symbol, …)` funnels (e.g. `rust_named_edge` 336 calls, `target_model_named_edge` 242, `dag_named_edge` 158, `ts_named_edge` 82). Most callers pass Symbol data rows, not `^literals`. There are also Symbol-keyed readers (`v2.std.node_query` `find_named_child` 248 calls/162 with a literal, `named_child_lookup` 17, `named_edge_target_lookup` 10, `v2.std.node` `name_occurrences` 34, `labeled_named_is`/`_is_not` 5). These are **not classified per caller** here. That population (about 1,500 funnel calls + about 310 query calls) is the explicit unclassified remainder, and splitting it is the bulk of the migration.
3. **`ReferenceSite.position` is a `QualifiedName` that mixes structural and authored segments.** `v2.compiler.reference_site_collector` `reference_sites_in_edge` snocs every Named label that is not metadata or identity-projection, so grammar productions like `dag_surface_import_decl` become path segments beside authored names. `v2.lens.module_graph` `site_is_import_syntax_mention` then tests membership of three `^dag_surface_import_*` symbols in that path. Under the new shape, position must carry `EdgeLabel` (or only Authored segments plus a separate structural context). A Symbol-only path cannot keep the distinction.
4. **"AUTHORED-modeled" literals (126 CONSTRUCT sites).** These are hand-built type nodes in `src/v2/std/algebra.dag`, `effects.dag`, `testgen.dag`, `target_model.dag`, `integer_value_set.dag`, `refinement_widening_predicate.dag`, `model_core.dag` and others. Their field and variant edges use `^magma_field_op`-style literals: they model *authored field names* of a modeled record, yet they are spelled as marker-like symbols. I proposed `becomes Authored`, but this is a judgment call. If those modeled types are ever meant to equal what ingest produces for an authored declaration, their names are wrong today (`magma_field_op` ≠ `op`). Flagged, not settled.
5. **Structural classification of a symbol is by a name heuristic** (`_edge`, `marker`, `_node_projection`, `dag_surface_*`, `grammar_*`, `formal_*`, `loop_*`, `arrow_*`, `match_arm_*`, …). The closed set below is therefore a candidate roster, not an authority. The real minting authorities are: the grammar productions' emitted identities (`v2.extdeps.languages.dag` rows and `v2.std.grammar` projections), `v2.compiler.02_parse` `parse_tree_projection_edge` (already a closed roster of 4), and the `v2.std.type_binder` markers (`type_params_marker`, `type_body_marker`, `type_alias_marker`, `cast_target_marker`, `type_annotation_marker`). Further sources are `v2.std.node` (`arrow_body_edge`, `arrow_signature_order_edge`, `loop_bound_edge`, `match_arm_pattern`, `match_arm_body`), `v2.std.node_query` (`positional_payload_field_name`, `pattern_wildcard_name`, `field_projection_base_name`/`_field_name`, `construct_tag_edge` tag), `v2.std.qualified_name` `declaration_reference_marker`, `v2.compiler.use_site_verdict` edges, the `v2.workflow.bootstrap` `bootstrap_projection_*_edge` symbols, and the `v2.std.compilers.target_model` `target_model_edge_*` symbols.

## The four named readers

| reader (actual symbol) | file | how it reads | disposition |
|---|---|---|---|
| `dag_surface_module_header_metadata_edge(name: Symbol)` (asked-for "header_metadata_edge") | src/v2/extdeps/languages/dag.dag | Symbol equality against `^dag_surface_module_header`, `^dag_surface_module_header_qualified_name`; 13 callers (symbol_index_fill `…`, namespace_graft, 03_name_resolve, reference_site_collector `reference_sites_in_edge`, 03_resolve x2, zero_metadata_test x6) — every caller first binds `Named { name: sym }` | becomes Structural arm match: take `StructuralEdgeLabel`, match the two arms; callers match `Structural { label }` instead of binding a Symbol |
| `dag_node_is_module_root_conj` | src/v2/extdeps/languages/dag.dag | fallback arm folds children: `Named { name: sym } => sym == ^dag_surface_module_header` | becomes Structural arm `dag_surface_module_header` (Authored arm → false) |
| `d1_edge_names_where_clause` (asked-for "edge_names_where_clause"; it lives in **src/v2/test/claim/parse/d1_declaration_grammar_parse_test.dag**, not module_graph.dag) | test | `Named { name: n } => n == ^dag_surface_where_refinement_clause` | becomes Structural arm `dag_surface_where_refinement_clause`; production minter is `v2.compiler.body_lowering_fold` (`label: Named { name: ^dag_surface_where_refinement_clause }`) |
| `site_is_import_syntax_mention(site: ReferenceSite)` | src/v2/lens/module_graph.dag | `contains(site.position, ^dag_surface_import_decl / ^dag_surface_import_block / ^dag_surface_import_decl_qualified_name)` — no Named match at all; depends on finding 3 | requires `ReferenceSite.position` to carry label arms; then becomes a Structural-arm membership test. Not a mechanical rename. |

Related closed roster already present: `v2.compiler.02_parse` `parse_tree_projection_edge` (4 grammar projection symbols; 5 callers). It is the precedent for `StructuralEdgeLabel` being a closed coproduct.

## Counts (EdgeLabel sites only)

Total EdgeLabel `Named` sites: 870


| Disposition | count |
|---|---|
| becomes Authored | 468 |
| becomes Structural arm X | 268 |
| arm-only rename | 55 |
| funnel split (caller-determined) | 41 |
| generic Symbol-keyed query (caller-determined) | 34 |
| hash-canonical change | 4 |

| Role | count |
|---|---|
| CONSTRUCT | 592 |
| MATCH-on-name-text(bound→compare) | 95 |
| MATCH-binds-name(read as name) | 49 |
| MATCH-arm-only | 48 |
| CONSTRUCT(funnel helper) | 41 |
| MATCH-on-name-text(bound→compare)/GENERIC-QUERY | 34 |
| CONSTRUCT(pass-through) | 7 |
| CANONICALIZE-ORDER | 1 |
| HASH | 1 |
| HASH(wire encode) | 1 |
| HASH(wire decode) | 1 |

| Symbol source | count |
|---|---|
| STRUCTURAL | 215 |
| TEST-FIXTURE | 182 |
| AUTHORED-modeled(literal field/variant name of a hand-built type node) | 126 |
| CALLER-DETERMINED | 75 |
| STRUCTURAL(via symbol-valued fn/data) | 53 |
| AUTHORED? | 49 |
| - | 48 |
| TEST-FIXTURE(var) | 48 |
| AUTHORED-modeled | 36 |
| AUTHORED | 27 |
| PASS-THROUGH | 7 |
| STRUCTURAL+AUTHORED | 4 |

## Candidate STRUCTURAL symbols (111 distinct, as observed at EdgeLabel sites)

| symbol | sites | minted at (CONSTRUCT) | read at (MATCH) |
|---|---|---|---|
| `algebra_edge` | 2 | v2/std/algebra.dag `algebra_inhabitance_node` | v2/std/bounded_lattice_completeness.dag `infer_edge_consumes_bounded_lattice_partial` |
| `arrow_body_edge` | 28 | v2/compiler/03_body_producer.dag `attach_arrow_body`<br>v2/compiler/body_lowering_fold.dag `body_lower_data_decl_to_member`<br>v2/compiler/body_lowering_fold.dag `body_lower_fn_decl_arrow`<br>v2/extdeps/languages/dag.dag `dag_arrow_with_body_node` …+18 | v2/std/node.dag `arrow_signature_edges_conform`<br>v2/std/node.dag `edge_contributes_to_cost_fold`<br>v2/std/type_binder.dag `arrow_named_labels_conform`<br>v2/test/claim/execution/data_decl_lowering_grounding_test.dag `ddl_edge_is_body` |
| `arrow_body_form_record_field` | 1 | v2/test/claim/body_lowering/arrow_body_form_witness_test.dag `fn` |  |
| `arrow_signature_order_edge` | 4 | v2/std/compilers/body_lowering.dag `signature_order_edge`<br>v2/std/node.dag `arrow_signature_order_label` | v2/test/claim/declared_parameter_order_test.dag `dpo_order_atoms_are_bare`<br>v2/test/claim/declared_parameter_order_test.dag `dpo_with_order` |
| `bcn_not_a_marker` | 1 | v2/test/claim/body_cast_node_test.dag `fn` |  |
| `bootstrap_projection_hash_digest_slot_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_hash_pin_pair_projection_node` |  |
| `bootstrap_projection_hash_pin_slot_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_hash_pin_pair_projection_node` |  |
| `bootstrap_projection_runtime_identity_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_projection_closure_node` |  |
| `bootstrap_projection_snapshot_content_hash_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_projection_closure_node` |  |
| `bootstrap_projection_snapshot_language_model_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_projection_closure_node` |  |
| `bootstrap_projection_snapshot_root_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_projection_closure_node` |  |
| `bootstrap_projection_target_bundle_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_projection_closure_node` |  |
| `cast_target_marker` | 1 |  | v2/std/type_binder.dag `edge_is_cast_target` |
| `cast_target_marker()` | 1 | v2/std/type_binder.dag `cast_target_edge` |  |
| `coproduct_exhaustiveness_omitted_variant_edge` | 1 | v2/lens/testgen.dag `coproduct_exhaustiveness_input` |  |
| `dag_binding_param_a` | 2 | v2/extdeps/languages/dag.dag `dag_bool_binop_arrow_domain_node`<br>v2/extdeps/languages/dag.dag `dag_complement_arrow_domain_node` |  |
| `dag_binding_param_b` | 1 | v2/extdeps/languages/dag.dag `dag_bool_binop_arrow_domain_node` |  |
| `dag_binding_param_r` | 1 | v2/extdeps/languages/dag.dag `dag_field_access_arrow_domain_conj_node` |  |
| `dag_binding_param_x` | 3 | v2/test/claim/body_lowering/arrow_body_form_witness_test.dag `arrow_body_form_id_arrow_fixture`<br>v2/test/claim/manual/match_infer_fail_open_audit_test.dag `classical_not_int_match_arrow_fixture`<br>v2/test/claim/manual/match_infer_fail_open_audit_test.dag `classical_not_int_match_non_octet_arm_fixture` |  |
| `dag_surface_field_decl_block` | 1 | v2/compiler/body_lowering_fold.dag `body_lower_type_variant_children_with_where` |  |
| `dag_surface_import_decl` | 3 | v2/test/claim/execution/typescript_import_emit_by_execution_test.dag `ts_import_carrier_node`<br>v2/test/claim/execution/typescript_import_emit_by_execution_test.dag `ts_import_missing_qn_carrier_node`<br>v2/test/claim/reference_derived_graph_witness_test.dag `rd_import_syntax_mention` |  |
| `dag_surface_import_decl_block` | 2 | v2/test/claim/execution/typescript_import_emit_by_execution_test.dag `ts_import_carrier_node`<br>v2/test/claim/execution/typescript_import_emit_by_execution_test.dag `ts_import_missing_qn_carrier_node` |  |
| `dag_surface_import_decl_qualified_name` | 1 | v2/test/claim/execution/typescript_import_emit_by_execution_test.dag `ts_import_carrier_node` |  |
| `dag_surface_module_header` | 1 |  | v2/extdeps/languages/dag.dag `dag_node_is_module_root_conj` |
| `dag_surface_module_header_metadata_edge` | 3 |  | v2/compiler/03_resolve.dag `resolve_child_edge`<br>v2/compiler/reference_site_collector.dag `reference_sites_in_edge`<br>v2/compiler/symbol_index_fill.dag `symbol_index_fill_containment_edge` |
| `dag_surface_top_level_item` | 3 | v2/compiler/namespace_graft.dag `namespace_graft_reconstruct_declaration_forest`<br>v2/test/claim/symbol_index/containment_test.dag `fn`<br>v2/test/claim/symbol_index/containment_test.dag `si_alias_module_root_node` |  |
| `dag_surface_type_expr` | 1 | v2/compiler/body_lowering_fold.dag `body_lower_type_variant_children_with_where` |  |
| `dag_surface_where_refinement_clause` | 5 | v2/compiler/body_lowering_fold.dag `body_lower_kept_where_clause`<br>v2/compiler/body_lowering_fold.dag `body_lower_type_variant_children_with_where` | v2/compiler/body_lowering_fold.dag `body_lower_edge_names_where_clause`<br>v2/test/claim/parse/d1_declaration_grammar_parse_test.dag `d1_edge_names_where_clause`<br>v2/test/claim/parse/where_refinement_clause_parse_test.dag `where_refinement_edge_names_clause` |
| `declaration_reference_marker()` | 1 | v2/std/qualified_name.dag `declaration_reference_node` |  |
| `dependency_binds_to_edge` | 4 | v2/test/lens_structural_resolution/binds_to_resolved_test.dag `sr_decl`<br>v2/test/lens_structural_resolution/facts_lookup_miss.dag `sr_lookup_decl`<br>v2/test/lens_structural_resolution/unbound_symbol_at_use.dag `sr_unbound_decl`<br>v2/test/lens_structural_resolution/unresolved_infer_witness.dag `sr_infer_decl` |  |
| `dependency_module_import_edge` | 1 | v2/test/lens_structural_resolution/module_import_out_of_scope.dag `sr_mod_root` |  |
| `discriminant(v: ModelCoreFactAxisEncoding {})` | 12 | v2/extdeps/languages/llvm_ir.dag `llvm_float_facts_node`<br>v2/extdeps/languages/llvm_ir.dag `llvm_integer_facts_node`<br>v2/extdeps/languages/machine_code.dag `mc_integer_facts_node`<br>v2/test/claim/manual/find_witness_project_to_core_controls.dag `nested_project_to_core_node` …+4 |  |
| `discriminant(v: ModelCoreFactAxisOverflowDisposition {})` | 1 | v2/extdeps/languages/machine_code.dag `mc_integer_facts_node` |  |
| `discriminant(v: ModelCoreFactAxisSignedness {})` | 1 | v2/extdeps/languages/machine_code.dag `mc_integer_facts_node` |  |
| `discriminant(v: ModelCoreFactAxisSurfaceSpelling {})` | 9 | v2/extdeps/languages/llvm_ir.dag `llvm_float_facts_node`<br>v2/extdeps/languages/llvm_ir.dag `llvm_integer_facts_node`<br>v2/extdeps/languages/machine_code.dag `mc_integer_facts_node`<br>v2/test/claim/manual/find_witness_project_to_core_controls.dag `nested_project_to_core_node` …+4 |  |
| `discriminant(v: ModelCoreFactAxisWidth {})` | 2 | v2/extdeps/languages/llvm_ir.dag `llvm_integer_facts_node`<br>v2/extdeps/languages/machine_code.dag `mc_integer_facts_node` |  |
| `discriminant(v: v2.std.model_core.ModelCoreFactAxisEncoding {})` | 1 | v2/test/claim/manual/find_witness_project_to_core_controls.dag `surface_wrapped_core_node` |  |
| `effect_edge_symbol` | 1 | v2/test/lens_effect/effect_depends_on.dag `effect_root` |  |
| `field_projection_base_name` | 1 |  | v2/std/node_query.dag `field_projection_edge_base_optional` |
| `field_projection_base_name()` | 1 | v2/std/node_query.dag `field_projection_node` |  |
| `field_projection_field_name` | 1 |  | v2/std/node_query.dag `field_projection_edge_field_optional` |
| `field_projection_field_name()` | 1 | v2/std/node_query.dag `field_projection_node` |  |
| `float_literal_lexeme_field` | 1 | v2/extdeps/languages/dag.dag `dag_float_literal_node_from_lexeme` |  |
| `fold_elements` | 4 | v2/test/claim/execution/fold_assembly_round_trip_test.dag `corrupted_fold_form`<br>v2/test/claim/execution/fold_assembly_round_trip_test.dag `duplicate_template_form`<br>v2/test/claim/execution/fold_assembly_round_trip_test.dag `head_without_tail_elements_form`<br>v2/test/claim/execution/fold_assembly_round_trip_test.dag `missing_template_form` |  |
| `fold_list_node_head` | 3 | v2/std/algebra.dag `fold_list_node`<br>v2/test/claim/execution/fold_assembly_round_trip_test.dag `head_without_tail_elements_form` | v2/std/qualified_name.dag `qn_spine_role` |
| `fold_list_node_tail` | 1 | v2/std/algebra.dag `fold_list_node` |  |
| `fold_template_edge` | 4 | v2/test/claim/execution/fold_assembly_round_trip_test.dag `corrupted_fold_form`<br>v2/test/claim/execution/fold_assembly_round_trip_test.dag `duplicate_template_form`<br>v2/test/claim/execution/fold_assembly_round_trip_test.dag `head_without_tail_elements_form` |  |
| `free_monoid_type_fixpoint` | 1 | v2/std/algebra.dag `free_monoid_tail_type_fixpoint` |  |
| `grammar_production_captured_node_projection` | 5 | v2/compiler/body_lowering_fold.dag `body_lower_kept_where_clause`<br>v2/test/claim/body_lowering_qualified_path_test.dag `dotted_fixture_shell`<br>v2/test/claim/name_resolve/test_code_reference_wall_test.dag `wall_module_root`<br>v2/test/claim/namespace_graft/graft_shape_test.dag `ng_projection_edge` …+1 |  |
| `grammar_production_identity_node_projection` | 12 | dag/test/claim/body_lowering_rejection_propagation_test.dag `emitted_shell`<br>v2/compiler/body_lowering_fold.dag `body_lower_kept_where_clause`<br>v2/test/claim/name_resolve/test_code_reference_wall_test.dag `wall_module_body`<br>v2/test/claim/name_resolve/test_code_reference_wall_test.dag `wall_module_root` …+1 | v2/compiler/03_ingest.dag `operational_parse_tree_project_to_formal_captures`<br>v2/compiler/body_lowering_fold.dag `body_lower_is_identity_projection_edge`<br>v2/compiler/emit_produced.dag `produced_module_edge_is_production_marker`<br>v2/lens/complexity_accumulator_copy/analyze.dag `edge_skips_identifier_count` …+3 |
| `grammar_relation_field_emitted` | 3 | v2/test/claim/execution/emit_ingest_type_decl_round_trip_test.dag `type_decl_row_with_foreign_emitted_fixture`<br>v2/test/claim/manual/emit_ingest_same_language_row_fold_test.dag `emit_ingest_fold_row_with_emitted`<br>v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_row_node` |  |
| `grammar_relation_field_production` | 1 | v2/test/claim/execution/emit_ingest_type_decl_round_trip_test.dag `type_decl_row_with_foreign_emitted_fixture` |  |
| `grammar_relation_field_tokens` | 4 | v2/test/claim/execution/emit_ingest_type_decl_round_trip_test.dag `type_decl_row_with_foreign_emitted_fixture`<br>v2/test/claim/manual/emit_ingest_same_language_row_fold_test.dag `emit_ingest_fold_row_with_emitted`<br>v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_row_node` | v2/compiler/06_translate.dag `serialize_source_tokens_fold_child` |
| `grammar_relation_forward_row_select_skip_catalog_edge` | 1 |  | v2/std/grammar.dag `grammar_relation_row_forward_select` |
| `grammar_relation_rules_formal_productions` | 3 | v2/test/claim/execution/emit_ingest_type_decl_round_trip_test.dag `type_decl_translation_rules_with_row` | v2/std/grammar.dag `grammar_relation_row_reverse_parse_selection`<br>v2/std/grammar.dag `grammar_relation_rules_formal_productions_from_rows` |
| `grammar_sequence_left_node_projection` | 5 | v2/compiler/body_lowering_fold.dag `body_lowering_match_field_name_spine`<br>v2/compiler/body_lowering_fold.dag `body_lowering_match_spine_identity_absent_fixture_spine`<br>v2/test/claim/body_lowering_qualified_path_test.dag `dotted_fixture_seq` | v2/test/claim/long/accumulator_copy_fold_analysis_test.dag `edge_is_seq_left` |
| `grammar_sequence_right_node_projection` | 4 | v2/compiler/body_lowering_fold.dag `body_lowering_match_field_name_spine`<br>v2/compiler/body_lowering_fold.dag `body_lowering_match_spine_identity_absent_fixture_spine`<br>v2/test/claim/body_lowering_qualified_path_test.dag `dotted_fixture_seq` |  |
| `hollow_alias_nested_child_edge` | 1 | v2/test/lens_fact_density/hollow_alias_nested_rejected_test.dag `hollow_alias_nested_root` |  |
| `idempotency_edge_symbol` | 1 | v2/test/lens_idempotency/write_effect_test.dag `idempotency_root` |  |
| `inhabitant_edge` | 1 | v2/std/algebra.dag `algebra_inhabitance_node` |  |
| `integer_literal_magnitude_field` | 3 | v2/extdeps/languages/dag.dag `dag_int_literal_node_from_magnitude`<br>v2/test/claim/execution/data_decl_lowering_grounding_test.dag `fn` | v2/compiler/04_infer.dag `infer_literal_edge_diagnostics_derived` |
| `llvm_facts_field_std_projection` | 2 | v2/extdeps/languages/llvm_ir.dag `llvm_float_facts_node`<br>v2/extdeps/languages/llvm_ir.dag `llvm_integer_facts_node` |  |
| `loop_actual` | 1 | v2/test/claim/execution/long/loop_eval_by_execution_test.dag `loop_tree_root` |  |
| `loop_bound_edge` | 22 | v2/compiler/fold_lowering.dag `fold_call_seam_loop`<br>v2/extdeps/languages/dag.dag `dag_loop_body_from_literal`<br>v2/lens/cost/copied_port_citations.dag `citation_loop_body_linear_in`<br>v2/lens/enforcement/receipts.dag `cost_red_control_probe` …+17 | v2/std/node.dag `loop_body_edges_conform` |
| `loop_carrier_edge` | 2 | v2/compiler/fold_lowering.dag `fold_call_seam_loop`<br>v2/test/claim/fold_lowering_test.dag `with_duplicate_carrier_edge` |  |
| `loop_eval_actual` | 1 | v2/test/claim/manual/loop_demand_driven_eval_test.dag `loop_eval_tree_root` |  |
| `loop_eval_expected` | 1 | v2/test/claim/manual/loop_demand_driven_eval_test.dag `loop_eval_tree_root` |  |
| `loop_expected` | 1 | v2/test/claim/execution/long/loop_eval_by_execution_test.dag `loop_tree_root` |  |
| `loop_illegal_named_extra_atom` | 1 | v2/test/lens_cost/loop_illegal_named_test.dag `loop_illegal_named_input` |  |
| `marker` | 2 | v2/test/lens_effect/effect_depends_on.dag `lens_effect_marker_node` | v2/std/type_binder.dag `computation_named_labels_conform` |
| `match_arm_body` | 2 | v2/compiler/body_lowering_fold.dag `body_lower_match_arm_wire` | v2/std/compilers/body_lowering.dag `match_arm_view` |
| `match_arm_pattern` | 4 | v2/compiler/body_lowering_fold.dag `body_lower_match_arm_wire` | v2/compiler/03_resolve.dag `resolve_match_arm_walk`<br>v2/std/compilers/body_lowering.dag `match_arm_view`<br>v2/std/compilers/target_model.dag `target_value_expr_match_arm_wire_schema_valid` |
| `mc_facts_field_std_projection` | 1 | v2/extdeps/languages/machine_code.dag `mc_integer_facts_node` |  |
| `namespace_graft_is_metadata_edge` | 4 |  | v2/compiler/namespace_graft.dag `namespace_graft_collect_body_edge`<br>v2/compiler/namespace_graft.dag `namespace_graft_has_explicit_braced_container`<br>v2/test/claim/namespace_graft/graft_shape_test.dag `namespace_graft_grafted_tree_has_no_metadata_edges`<br>v2/test/claim/namespace_graft/normalize_graft_witness_helpers.dag `ng_norm_grafted_tree_has_no_metadata_edges` |
| `ownership_edge_symbol` | 1 | v2/test/lens_ownership/resource_dependency.dag `ownership_root` |  |
| `parallelism_edge_symbol` | 1 | v2/test/lens_parallelism/data_dependency_test.dag `parallelism_root` |  |
| `parse_tree_projection_edge` | 2 |  | v2/compiler/namespace_graft.dag `namespace_graft_is_transparent_sequence_spine`<br>v2/compiler/namespace_graft.dag `namespace_graft_keep_flattened_member_edge` |
| `pattern_wildcard_name` | 1 |  | v2/std/anonymous_binder.dag `anonymous_binder_mint_parameter_list` |
| `positional_payload_field_name()` | 2 | v2/compiler/body_lowering_fold.dag `body_lower_pattern_field_edges`<br>v2/compiler/body_lowering_fold.dag `body_lower_positional_payload` |  |
| `refinement_preservation_list_head_edge` | 1 | v2/lens/testgen.dag `refinement_preservation_list_node` |  |
| `refinement_preservation_list_tail_edge` | 1 | v2/lens/testgen.dag `refinement_preservation_list_node` |  |
| `rollup_edge_symbol` | 1 | v2/test/lens_unused_parameters/rollup_unused_declaration.dag `rollup_function` |  |
| `subsumption_first_leaf_edge` | 2 | v2/test/claim/manual/dissolution_subsumption_reverification.dag `expected_rust_language_model_emit_receipt_node`<br>v2/test/claim/manual/dissolution_subsumption_reverification.dag `subsumption_receipt_node` |  |
| `subsumption_mechanical_claim_edge` | 2 | v2/test/claim/manual/dissolution_subsumption_reverification.dag `expected_rust_language_model_emit_receipt_node`<br>v2/test/claim/manual/dissolution_subsumption_reverification.dag `subsumption_receipt_node` |  |
| `subsumption_root_fix_edge` | 2 | v2/test/claim/manual/dissolution_subsumption_reverification.dag `expected_rust_language_model_emit_receipt_node`<br>v2/test/claim/manual/dissolution_subsumption_reverification.dag `subsumption_receipt_node` |  |
| `subsumption_second_leaf_edge` | 2 | v2/test/claim/manual/dissolution_subsumption_reverification.dag `expected_rust_language_model_emit_receipt_node`<br>v2/test/claim/manual/dissolution_subsumption_reverification.dag `subsumption_receipt_node` |  |
| `target_model_edge_atom_realizations` | 1 |  | v2/std/compilers/target_model.dag `target_model_bundle_core_keep_edge` |
| `target_model_edge_collection_realization` | 1 | v2/extdeps/languages/rust_freemonoid_char_fixtures.dag `fmc_fixture_bundle_core` |  |
| `target_model_edge_declared_inhabitants` | 1 | v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_bundle_with_rules` |  |
| `target_model_edge_fidelity_quotient` | 2 | v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_bundle_with_rules`<br>v2/test/claim/manual/target_carriers_ambiguous_quotient_test.dag `fidelity_quotient_edge` |  |
| `target_model_edge_selection_policy` | 1 | v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_bundle_with_rules` |  |
| `target_model_edge_serialize_source` | 1 | v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_bundle_with_rules` |  |
| `target_model_edge_translation_rules` | 3 | v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_bundle_with_rules` | v2/extdeps/languages/rust_test_fixtures.dag `rust_bind_literal_target_model_bundle`<br>v2/extdeps/languages/rust_test_fixtures.dag `rust_classical_not_target_model_bundle_core` |
| `target_model_edge_type_expression_projection` | 1 | v2/extdeps/languages/rust_freemonoid_char_fixtures.dag `fmc_fixture_bundle_core` |  |
| `target_model_edge_use_site_ownership_realizations` | 1 |  | v2/std/compilers/target_model.dag `target_model_augment_use_site_ownership_catalog` |
| `target_model_edge_value_semantics_carriers` | 1 |  | v2/std/compilers/target_model.dag `target_model_augment_value_semantics_carriers` |
| `target_operator_realization_row_edge` | 4 | v2/test/claim/manual/target_model_operator_lookup_dissolution_test.dag `operator_catalog_with_malformed_and_valid_rows` | v2/std/compilers/target_model.dag `target_operator_realization_catalog_lookup`<br>v2/std/compilers/target_model.dag `target_operator_realization_catalog_wire_schema_valid` |
| `target_value_expr_effect_callee_row_edge` | 1 |  | v2/std/compilers/target_model.dag `decode_effect_apply_callee_rows` |
| `type_alias_marker` | 1 |  | v2/std/type_binder.dag `edge_is_type_alias` |
| `type_alias_marker()` | 1 | v2/std/type_binder.dag `type_alias_wrapper` |  |
| `type_annotation_marker` | 1 |  | v2/std/type_binder.dag `edge_is_type_annotation` |
| `type_annotation_marker()` | 1 | v2/std/type_binder.dag `type_annotation_edge` |  |
| `type_body_marker` | 1 |  | v2/std/type_binder.dag `edge_is_type_body` |
| `type_body_marker()` | 1 | v2/std/type_binder.dag `type_decl_wrapper` |  |
| `type_params_marker` | 1 |  | v2/std/type_binder.dag `edge_is_type_params` |
| `type_params_marker()` | 8 | v2/std/type_binder.dag `type_alias_wrapper`<br>v2/std/type_binder.dag `type_decl_wrapper`<br>v2/std/type_binder.dag `type_opaque_wrapper`<br>v2/std/type_binder.dag `type_params_edge` …+4 |  |
| `unused_parameters_edge_symbol` | 1 | v2/test/lens_unused_parameters/binds_to_edge.dag `unused_parameters_root` |  |
| `unused_parameters_non_use_edge_symbol` | 1 | v2/test/lens_unused_parameters/non_use_edges.dag `unused_parameters_non_use_root` |  |
| `use_site_verdict_edge` | 1 | v2/compiler/use_site_verdict.dag `attach_use_site_verdict` |  |
| `use_site_verdict_field_edge` | 1 | v2/compiler/use_site_verdict.dag `use_site_verdict_to_node` |  |
| `variant_marker` | 1 | v2/std/effects.dag `keyed_shape_node` |  |


## Constructor funnels (caller-determined; callers NOT classified here)

Callers pass either a `^literal` or a Symbol-valued data row/parameter. Proposed disposition: each funnel splits into `<lang>_structural_edge(label: StructuralEdgeLabel, …)` and `<lang>_authored_edge(name: Symbol, …)`, or takes `EdgeLabel`.

| funnel | calls | calls with a `^literal` arg |
|---|---|---|
| rust_named_edge | 336 | 41 |
| target_model_named_edge | 242 | 0 |
| dag_named_edge | 158 | 4 |
| go_named_edge | 98 | 24 |
| ts_named_edge | 82 | 14 |
| cpp_named_edge | 62 | 42 |
| java_named_edge | 58 | 20 |
| wasm_named_edge | 51 | 12 |
| kotlin_named_edge | 51 | 8 |
| grammar_named_edge | 45 | 36 |
| swift_named_edge | 44 | 6 |
| bash_named_edge | 34 | 0 |
| python_named_edge | 27 | 12 |
| node_offset_named_edge | 21 | 0 |
| ecmascript_named_edge | 21 | 12 |
| english_named_edge | 21 | 5 |
| verilog_named_edge | 20 | 3 |
| named_edge (fold_assembly) | 19 | 15 |
| lean_named_edge | 19 | 1 |
| c_named_edge | 17 | 12 |
| integer_named_edge | 9 | 0 |
| runtime_value_named_edge | 9 | 0 |
| language_model_named_edge | 5 | 0 |
| dag_round_trip_edge | 5 | 0 |
| namespace_graft_kw_ident_named_edge | 4 | 0 |
| bool_named_edge | 2 | 0 |
| c_decl_named_edge / swift_decl_named_edge / sugar_variant_to_disj_edge | 1 each | 0 |
| named_edge_shape | 0 | 0 |

Symbol-keyed readers (also caller-determined): `find_named_child` 248 (162 literal), `name_occurrences` 34 (4), `named_child_lookup` 17 (10), `dag_surface_module_header_metadata_edge` 13 (6), `named_edge_target_lookup` 10 (6), `parse_tree_projection_edge` 5 (0), `labeled_named_is` 3, `labeled_named_is_not` 2. There are also 176 `data x: Symbol = ^…` rows corpus-wide; an unknown subset of them flows into labels through these funnels, and that flow is not traced.

## Hash / canonical-order / persisted-value consumers (churn if edge-label hashing changes)

Hash authority: `v2.std.node` `canonical_hash_of_edge_label` (tags `^canonical_tag_named_edge` / `^canonical_tag_positional_edge`) → `content_hash_step` → `content_hash`, `content_hash_of_children`. Order authority: `label_sort_key` / `sort_labeled` via `canonicalize_labeled_for_content_hash` (disciplines LabeledEdges, ArrowBodyEdges, LoopBoundEdges, PositionalPlusOneNamedEdges, which key on `arrow_body_edge`, `arrow_signature_order_edge`, `loop_bound_edge` by name).

**Persisted / pinned (values would churn):**
- `dag/test/claim/node_hash_protocol_witness_test.dag`: 6 pinned digest vectors on named-edge nodes. `labeled_edges_discipline_matches_its_vector` "27b43e6a2a853e35", `arrow_body_discipline…` "838a07cf36bb337b", `loop_bound_discipline…` "e1ea7ccc630cdeeb" and `nested_assembly…` "84f0f2e91248ef84" churn. `positional_edges…` "c579b53fa78cbd3f" and `no_edges…` "8a32b46ce830ef4c" do not, provided the Positional tag is unchanged. The connective/behavior tag vectors do not churn either.
- `dag/gunbc/scm/object_store.dag` ObjectId = `content_hash_of_children` (`stored_edge_label_of`): every stored SCM object id with a named edge churns.
- `dag/gunbc/scm/object_table_json.dag` `encode_label` / `decode_named_label`: the wire tag `"named"` plus a free `name` lexeme is a format change. The decoder cannot intern an arbitrary lexeme into a closed Structural set without refusing. No committed JSON fixture contains a `"named"` label (checked `dag/test/fixture/scm_repository_load/*.json`: 0 hits), so there is no on-disk fixture churn in-tree.
- SCM witness tests that recompute ids: `dag/test/claim/scm/scm_object_store_witness_test.dag`, `scm_commit_closure_witness_test.dag`, `scm_commit_closure_json_v2_witness_test.dag` (their literal "0000000000000000" is an uncontained sentinel, not a content pin).
- `src/v2/compiler/05_eval.dag` `test_claim_evaluation_nodes_digest` / `correction_verdict_digest`: the test-claim cache key churns, but only as invalidation.
- `src/v2/compiler/02_parse.dag` `parse_grammar_digest`: the grammar digest feeds the seed interpreter's cross-parse memo key (`v1_interpreter` `parse_table_memo_scope_and_key`). Invalidation only.
- `src/v2/workflow/bootstrap.dag` `closure_hash` plus `bootstrap_projection_hash_pin_slot_edge` / `…digest_slot_edge`: the bootstrap projection hash pin churns if any pin value is committed. No literal digest was found in-tree.
- `dag/extdeps/realization/hermetic_fixture.dag` + `src/v2/extdeps/runtimes/v2_effect_io_pure.dag`: the locator identity is `content_hash(runtime_value_node_projection(arg))`, so recorded hermetic fixtures keyed by locator churn. Their on-disk store was not located in this census.
- `dag/gunbc/guarantee_probe_corpus.dag` `structural_content_hash`.

**In-memory equality/keys only (no persisted churn, but behaviour depends on tag discrimination):** `v2.std.materialize` (`Fnv1a64(content_hash)` dedup), `v2.lens.affected_set`, `v2.lens.interface_summary`, `v2.std.generic_instantiation`, `v2.compiler.06_translate`, `v2.std.compilers.compilation_unit`, `v2.std.compilers.target_model`, `v2.workflow.locality_affinity`, `v2.workflow.operand_flow`.
**Order witnesses:** `src/v2/test/claim/manual/content_hash_named_edge_order_test.dag`, `content_hash_loop_bound_edge_order_test.dag`, `src/v2/test/manual/parse_tree_content_hash_witness.dag`.
Full list of 68 files touching the hash entry points: `hashfiles.txt` in this scratchpad.

## Rust (src/v1/stage0) correspondence

No hand-written Rust constructs `EdgeLabel::Named`; v2 `.dag` is interpreted by the seed. The files that name the type reflectively are `src/v1/stage0/src/coproduct_reflection.rs` (2 `ctx.sym("EdgeLabel")` rows) and `src/v1/stage0/src/data_initializer_identity.rs` (1). There are no Rust `Named { name` sites for EdgeLabel: `std_algebra.rs` / `v1_compiler_infer_types.rs` hits are `ContainerSource`. `src/v1/stage0/src/resolved_graph_cache.rs` has its own `graph_row_content_hash` over payload bytes, which is not this hash. `std_content_hash.rs` / `v1_interpreter.rs` evaluate the `.dag` hash generically.

## Explicitly unclassified / low-confidence populations

- Funnel callers (about 1,500) and Symbol-keyed query callers (about 310): not classified per call (see tables).
- 49 `MATCH-binds-name(read as name)` rows are marked `AUTHORED?`. These reader bodies consume the Symbol (e.g. `all_names_distinct`, `arrow_domain_binder_labels`, `loop_edge_contributes_to_iteration_fold`, `encode_label`, `edge_label_role`). Each needs a read to decide whether structural labels also reach it.
- Role detection for MATCH vs CONSTRUCT is lexical (whether `=>` follows the pattern). `if let`/`is` forms, if any, are not recognized.
- Structural vs authored-modeled vs test-fixture for literals is a name/path heuristic (finding 5).
- Symbols minted only through grammar rows (`dag_surface_*`, 74 distinct `^dag_surface_*` spellings corpus-wide) appear in the Structural roster below only where they occur at a direct EdgeLabel site.

## Candidate STRUCTURAL symbols (111 distinct, as observed at EdgeLabel sites)

| symbol | sites | minted at (CONSTRUCT) | read at (MATCH) |
|---|---|---|---|
| `algebra_edge` | 2 | v2/std/algebra.dag `algebra_inhabitance_node` | v2/std/bounded_lattice_completeness.dag `infer_edge_consumes_bounded_lattice_partial` |
| `arrow_body_edge` | 28 | v2/compiler/03_body_producer.dag `attach_arrow_body`<br>v2/compiler/body_lowering_fold.dag `body_lower_data_decl_to_member`<br>v2/compiler/body_lowering_fold.dag `body_lower_fn_decl_arrow`<br>v2/extdeps/languages/dag.dag `dag_arrow_with_body_node` …+18 | v2/std/node.dag `arrow_signature_edges_conform`<br>v2/std/node.dag `edge_contributes_to_cost_fold`<br>v2/std/type_binder.dag `arrow_named_labels_conform`<br>v2/test/claim/execution/data_decl_lowering_grounding_test.dag `ddl_edge_is_body` |
| `arrow_body_form_record_field` | 1 | v2/test/claim/body_lowering/arrow_body_form_witness_test.dag `fn` |  |
| `arrow_signature_order_edge` | 4 | v2/std/compilers/body_lowering.dag `signature_order_edge`<br>v2/std/node.dag `arrow_signature_order_label` | v2/test/claim/declared_parameter_order_test.dag `dpo_order_atoms_are_bare`<br>v2/test/claim/declared_parameter_order_test.dag `dpo_with_order` |
| `bcn_not_a_marker` | 1 | v2/test/claim/body_cast_node_test.dag `fn` |  |
| `bootstrap_projection_hash_digest_slot_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_hash_pin_pair_projection_node` |  |
| `bootstrap_projection_hash_pin_slot_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_hash_pin_pair_projection_node` |  |
| `bootstrap_projection_runtime_identity_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_projection_closure_node` |  |
| `bootstrap_projection_snapshot_content_hash_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_projection_closure_node` |  |
| `bootstrap_projection_snapshot_language_model_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_projection_closure_node` |  |
| `bootstrap_projection_snapshot_root_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_projection_closure_node` |  |
| `bootstrap_projection_target_bundle_edge` | 1 | v2/workflow/bootstrap.dag `bootstrap_projection_closure_node` |  |
| `cast_target_marker` | 1 |  | v2/std/type_binder.dag `edge_is_cast_target` |
| `cast_target_marker()` | 1 | v2/std/type_binder.dag `cast_target_edge` |  |
| `coproduct_exhaustiveness_omitted_variant_edge` | 1 | v2/lens/testgen.dag `coproduct_exhaustiveness_input` |  |
| `dag_binding_param_a` | 2 | v2/extdeps/languages/dag.dag `dag_bool_binop_arrow_domain_node`<br>v2/extdeps/languages/dag.dag `dag_complement_arrow_domain_node` |  |
| `dag_binding_param_b` | 1 | v2/extdeps/languages/dag.dag `dag_bool_binop_arrow_domain_node` |  |
| `dag_binding_param_r` | 1 | v2/extdeps/languages/dag.dag `dag_field_access_arrow_domain_conj_node` |  |
| `dag_binding_param_x` | 3 | v2/test/claim/body_lowering/arrow_body_form_witness_test.dag `arrow_body_form_id_arrow_fixture`<br>v2/test/claim/manual/match_infer_fail_open_audit_test.dag `classical_not_int_match_arrow_fixture`<br>v2/test/claim/manual/match_infer_fail_open_audit_test.dag `classical_not_int_match_non_octet_arm_fixture` |  |
| `dag_surface_field_decl_block` | 1 | v2/compiler/body_lowering_fold.dag `body_lower_type_variant_children_with_where` |  |
| `dag_surface_import_decl` | 3 | v2/test/claim/execution/typescript_import_emit_by_execution_test.dag `ts_import_carrier_node`<br>v2/test/claim/execution/typescript_import_emit_by_execution_test.dag `ts_import_missing_qn_carrier_node`<br>v2/test/claim/reference_derived_graph_witness_test.dag `rd_import_syntax_mention` |  |
| `dag_surface_import_decl_block` | 2 | v2/test/claim/execution/typescript_import_emit_by_execution_test.dag `ts_import_carrier_node`<br>v2/test/claim/execution/typescript_import_emit_by_execution_test.dag `ts_import_missing_qn_carrier_node` |  |
| `dag_surface_import_decl_qualified_name` | 1 | v2/test/claim/execution/typescript_import_emit_by_execution_test.dag `ts_import_carrier_node` |  |
| `dag_surface_module_header` | 1 |  | v2/extdeps/languages/dag.dag `dag_node_is_module_root_conj` |
| `dag_surface_module_header_metadata_edge` | 3 |  | v2/compiler/03_resolve.dag `resolve_child_edge`<br>v2/compiler/reference_site_collector.dag `reference_sites_in_edge`<br>v2/compiler/symbol_index_fill.dag `symbol_index_fill_containment_edge` |
| `dag_surface_top_level_item` | 3 | v2/compiler/namespace_graft.dag `namespace_graft_reconstruct_declaration_forest`<br>v2/test/claim/symbol_index/containment_test.dag `fn`<br>v2/test/claim/symbol_index/containment_test.dag `si_alias_module_root_node` |  |
| `dag_surface_type_expr` | 1 | v2/compiler/body_lowering_fold.dag `body_lower_type_variant_children_with_where` |  |
| `dag_surface_where_refinement_clause` | 5 | v2/compiler/body_lowering_fold.dag `body_lower_kept_where_clause`<br>v2/compiler/body_lowering_fold.dag `body_lower_type_variant_children_with_where` | v2/compiler/body_lowering_fold.dag `body_lower_edge_names_where_clause`<br>v2/test/claim/parse/d1_declaration_grammar_parse_test.dag `d1_edge_names_where_clause`<br>v2/test/claim/parse/where_refinement_clause_parse_test.dag `where_refinement_edge_names_clause` |
| `declaration_reference_marker()` | 1 | v2/std/qualified_name.dag `declaration_reference_node` |  |
| `dependency_binds_to_edge` | 4 | v2/test/lens_structural_resolution/binds_to_resolved_test.dag `sr_decl`<br>v2/test/lens_structural_resolution/facts_lookup_miss.dag `sr_lookup_decl`<br>v2/test/lens_structural_resolution/unbound_symbol_at_use.dag `sr_unbound_decl`<br>v2/test/lens_structural_resolution/unresolved_infer_witness.dag `sr_infer_decl` |  |
| `dependency_module_import_edge` | 1 | v2/test/lens_structural_resolution/module_import_out_of_scope.dag `sr_mod_root` |  |
| `discriminant(v: ModelCoreFactAxisEncoding {})` | 12 | v2/extdeps/languages/llvm_ir.dag `llvm_float_facts_node`<br>v2/extdeps/languages/llvm_ir.dag `llvm_integer_facts_node`<br>v2/extdeps/languages/machine_code.dag `mc_integer_facts_node`<br>v2/test/claim/manual/find_witness_project_to_core_controls.dag `nested_project_to_core_node` …+4 |  |
| `discriminant(v: ModelCoreFactAxisOverflowDisposition {})` | 1 | v2/extdeps/languages/machine_code.dag `mc_integer_facts_node` |  |
| `discriminant(v: ModelCoreFactAxisSignedness {})` | 1 | v2/extdeps/languages/machine_code.dag `mc_integer_facts_node` |  |
| `discriminant(v: ModelCoreFactAxisSurfaceSpelling {})` | 9 | v2/extdeps/languages/llvm_ir.dag `llvm_float_facts_node`<br>v2/extdeps/languages/llvm_ir.dag `llvm_integer_facts_node`<br>v2/extdeps/languages/machine_code.dag `mc_integer_facts_node`<br>v2/test/claim/manual/find_witness_project_to_core_controls.dag `nested_project_to_core_node` …+4 |  |
| `discriminant(v: ModelCoreFactAxisWidth {})` | 2 | v2/extdeps/languages/llvm_ir.dag `llvm_integer_facts_node`<br>v2/extdeps/languages/machine_code.dag `mc_integer_facts_node` |  |
| `discriminant(v: v2.std.model_core.ModelCoreFactAxisEncoding {})` | 1 | v2/test/claim/manual/find_witness_project_to_core_controls.dag `surface_wrapped_core_node` |  |
| `effect_edge_symbol` | 1 | v2/test/lens_effect/effect_depends_on.dag `effect_root` |  |
| `field_projection_base_name` | 1 |  | v2/std/node_query.dag `field_projection_edge_base_optional` |
| `field_projection_base_name()` | 1 | v2/std/node_query.dag `field_projection_node` |  |
| `field_projection_field_name` | 1 |  | v2/std/node_query.dag `field_projection_edge_field_optional` |
| `field_projection_field_name()` | 1 | v2/std/node_query.dag `field_projection_node` |  |
| `float_literal_lexeme_field` | 1 | v2/extdeps/languages/dag.dag `dag_float_literal_node_from_lexeme` |  |
| `fold_elements` | 4 | v2/test/claim/execution/fold_assembly_round_trip_test.dag `corrupted_fold_form`<br>v2/test/claim/execution/fold_assembly_round_trip_test.dag `duplicate_template_form`<br>v2/test/claim/execution/fold_assembly_round_trip_test.dag `head_without_tail_elements_form`<br>v2/test/claim/execution/fold_assembly_round_trip_test.dag `missing_template_form` |  |
| `fold_list_node_head` | 3 | v2/std/algebra.dag `fold_list_node`<br>v2/test/claim/execution/fold_assembly_round_trip_test.dag `head_without_tail_elements_form` | v2/std/qualified_name.dag `qn_spine_role` |
| `fold_list_node_tail` | 1 | v2/std/algebra.dag `fold_list_node` |  |
| `fold_template_edge` | 4 | v2/test/claim/execution/fold_assembly_round_trip_test.dag `corrupted_fold_form`<br>v2/test/claim/execution/fold_assembly_round_trip_test.dag `duplicate_template_form`<br>v2/test/claim/execution/fold_assembly_round_trip_test.dag `head_without_tail_elements_form` |  |
| `free_monoid_type_fixpoint` | 1 | v2/std/algebra.dag `free_monoid_tail_type_fixpoint` |  |
| `grammar_production_captured_node_projection` | 5 | v2/compiler/body_lowering_fold.dag `body_lower_kept_where_clause`<br>v2/test/claim/body_lowering_qualified_path_test.dag `dotted_fixture_shell`<br>v2/test/claim/name_resolve/test_code_reference_wall_test.dag `wall_module_root`<br>v2/test/claim/namespace_graft/graft_shape_test.dag `ng_projection_edge` …+1 |  |
| `grammar_production_identity_node_projection` | 12 | dag/test/claim/body_lowering_rejection_propagation_test.dag `emitted_shell`<br>v2/compiler/body_lowering_fold.dag `body_lower_kept_where_clause`<br>v2/test/claim/name_resolve/test_code_reference_wall_test.dag `wall_module_body`<br>v2/test/claim/name_resolve/test_code_reference_wall_test.dag `wall_module_root` …+1 | v2/compiler/03_ingest.dag `operational_parse_tree_project_to_formal_captures`<br>v2/compiler/body_lowering_fold.dag `body_lower_is_identity_projection_edge`<br>v2/compiler/emit_produced.dag `produced_module_edge_is_production_marker`<br>v2/lens/complexity_accumulator_copy/analyze.dag `edge_skips_identifier_count` …+3 |
| `grammar_relation_field_emitted` | 3 | v2/test/claim/execution/emit_ingest_type_decl_round_trip_test.dag `type_decl_row_with_foreign_emitted_fixture`<br>v2/test/claim/manual/emit_ingest_same_language_row_fold_test.dag `emit_ingest_fold_row_with_emitted`<br>v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_row_node` |  |
| `grammar_relation_field_production` | 1 | v2/test/claim/execution/emit_ingest_type_decl_round_trip_test.dag `type_decl_row_with_foreign_emitted_fixture` |  |
| `grammar_relation_field_tokens` | 4 | v2/test/claim/execution/emit_ingest_type_decl_round_trip_test.dag `type_decl_row_with_foreign_emitted_fixture`<br>v2/test/claim/manual/emit_ingest_same_language_row_fold_test.dag `emit_ingest_fold_row_with_emitted`<br>v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_row_node` | v2/compiler/06_translate.dag `serialize_source_tokens_fold_child` |
| `grammar_relation_forward_row_select_skip_catalog_edge` | 1 |  | v2/std/grammar.dag `grammar_relation_row_forward_select` |
| `grammar_relation_rules_formal_productions` | 3 | v2/test/claim/execution/emit_ingest_type_decl_round_trip_test.dag `type_decl_translation_rules_with_row` | v2/std/grammar.dag `grammar_relation_row_reverse_parse_selection`<br>v2/std/grammar.dag `grammar_relation_rules_formal_productions_from_rows` |
| `grammar_sequence_left_node_projection` | 5 | v2/compiler/body_lowering_fold.dag `body_lowering_match_field_name_spine`<br>v2/compiler/body_lowering_fold.dag `body_lowering_match_spine_identity_absent_fixture_spine`<br>v2/test/claim/body_lowering_qualified_path_test.dag `dotted_fixture_seq` | v2/test/claim/long/accumulator_copy_fold_analysis_test.dag `edge_is_seq_left` |
| `grammar_sequence_right_node_projection` | 4 | v2/compiler/body_lowering_fold.dag `body_lowering_match_field_name_spine`<br>v2/compiler/body_lowering_fold.dag `body_lowering_match_spine_identity_absent_fixture_spine`<br>v2/test/claim/body_lowering_qualified_path_test.dag `dotted_fixture_seq` |  |
| `hollow_alias_nested_child_edge` | 1 | v2/test/lens_fact_density/hollow_alias_nested_rejected_test.dag `hollow_alias_nested_root` |  |
| `idempotency_edge_symbol` | 1 | v2/test/lens_idempotency/write_effect_test.dag `idempotency_root` |  |
| `inhabitant_edge` | 1 | v2/std/algebra.dag `algebra_inhabitance_node` |  |
| `integer_literal_magnitude_field` | 3 | v2/extdeps/languages/dag.dag `dag_int_literal_node_from_magnitude`<br>v2/test/claim/execution/data_decl_lowering_grounding_test.dag `fn` | v2/compiler/04_infer.dag `infer_literal_edge_diagnostics_derived` |
| `llvm_facts_field_std_projection` | 2 | v2/extdeps/languages/llvm_ir.dag `llvm_float_facts_node`<br>v2/extdeps/languages/llvm_ir.dag `llvm_integer_facts_node` |  |
| `loop_actual` | 1 | v2/test/claim/execution/long/loop_eval_by_execution_test.dag `loop_tree_root` |  |
| `loop_bound_edge` | 22 | v2/compiler/fold_lowering.dag `fold_call_seam_loop`<br>v2/extdeps/languages/dag.dag `dag_loop_body_from_literal`<br>v2/lens/cost/copied_port_citations.dag `citation_loop_body_linear_in`<br>v2/lens/enforcement/receipts.dag `cost_red_control_probe` …+17 | v2/std/node.dag `loop_body_edges_conform` |
| `loop_carrier_edge` | 2 | v2/compiler/fold_lowering.dag `fold_call_seam_loop`<br>v2/test/claim/fold_lowering_test.dag `with_duplicate_carrier_edge` |  |
| `loop_eval_actual` | 1 | v2/test/claim/manual/loop_demand_driven_eval_test.dag `loop_eval_tree_root` |  |
| `loop_eval_expected` | 1 | v2/test/claim/manual/loop_demand_driven_eval_test.dag `loop_eval_tree_root` |  |
| `loop_expected` | 1 | v2/test/claim/execution/long/loop_eval_by_execution_test.dag `loop_tree_root` |  |
| `loop_illegal_named_extra_atom` | 1 | v2/test/lens_cost/loop_illegal_named_test.dag `loop_illegal_named_input` |  |
| `marker` | 2 | v2/test/lens_effect/effect_depends_on.dag `lens_effect_marker_node` | v2/std/type_binder.dag `computation_named_labels_conform` |
| `match_arm_body` | 2 | v2/compiler/body_lowering_fold.dag `body_lower_match_arm_wire` | v2/std/compilers/body_lowering.dag `match_arm_view` |
| `match_arm_pattern` | 4 | v2/compiler/body_lowering_fold.dag `body_lower_match_arm_wire` | v2/compiler/03_resolve.dag `resolve_match_arm_walk`<br>v2/std/compilers/body_lowering.dag `match_arm_view`<br>v2/std/compilers/target_model.dag `target_value_expr_match_arm_wire_schema_valid` |
| `mc_facts_field_std_projection` | 1 | v2/extdeps/languages/machine_code.dag `mc_integer_facts_node` |  |
| `namespace_graft_is_metadata_edge` | 4 |  | v2/compiler/namespace_graft.dag `namespace_graft_collect_body_edge`<br>v2/compiler/namespace_graft.dag `namespace_graft_has_explicit_braced_container`<br>v2/test/claim/namespace_graft/graft_shape_test.dag `namespace_graft_grafted_tree_has_no_metadata_edges`<br>v2/test/claim/namespace_graft/normalize_graft_witness_helpers.dag `ng_norm_grafted_tree_has_no_metadata_edges` |
| `ownership_edge_symbol` | 1 | v2/test/lens_ownership/resource_dependency.dag `ownership_root` |  |
| `parallelism_edge_symbol` | 1 | v2/test/lens_parallelism/data_dependency_test.dag `parallelism_root` |  |
| `parse_tree_projection_edge` | 2 |  | v2/compiler/namespace_graft.dag `namespace_graft_is_transparent_sequence_spine`<br>v2/compiler/namespace_graft.dag `namespace_graft_keep_flattened_member_edge` |
| `pattern_wildcard_name` | 1 |  | v2/std/anonymous_binder.dag `anonymous_binder_mint_parameter_list` |
| `positional_payload_field_name()` | 2 | v2/compiler/body_lowering_fold.dag `body_lower_pattern_field_edges`<br>v2/compiler/body_lowering_fold.dag `body_lower_positional_payload` |  |
| `refinement_preservation_list_head_edge` | 1 | v2/lens/testgen.dag `refinement_preservation_list_node` |  |
| `refinement_preservation_list_tail_edge` | 1 | v2/lens/testgen.dag `refinement_preservation_list_node` |  |
| `rollup_edge_symbol` | 1 | v2/test/lens_unused_parameters/rollup_unused_declaration.dag `rollup_function` |  |
| `subsumption_first_leaf_edge` | 2 | v2/test/claim/manual/dissolution_subsumption_reverification.dag `expected_rust_language_model_emit_receipt_node`<br>v2/test/claim/manual/dissolution_subsumption_reverification.dag `subsumption_receipt_node` |  |
| `subsumption_mechanical_claim_edge` | 2 | v2/test/claim/manual/dissolution_subsumption_reverification.dag `expected_rust_language_model_emit_receipt_node`<br>v2/test/claim/manual/dissolution_subsumption_reverification.dag `subsumption_receipt_node` |  |
| `subsumption_root_fix_edge` | 2 | v2/test/claim/manual/dissolution_subsumption_reverification.dag `expected_rust_language_model_emit_receipt_node`<br>v2/test/claim/manual/dissolution_subsumption_reverification.dag `subsumption_receipt_node` |  |
| `subsumption_second_leaf_edge` | 2 | v2/test/claim/manual/dissolution_subsumption_reverification.dag `expected_rust_language_model_emit_receipt_node`<br>v2/test/claim/manual/dissolution_subsumption_reverification.dag `subsumption_receipt_node` |  |
| `target_model_edge_atom_realizations` | 1 |  | v2/std/compilers/target_model.dag `target_model_bundle_core_keep_edge` |
| `target_model_edge_collection_realization` | 1 | v2/extdeps/languages/rust_freemonoid_char_fixtures.dag `fmc_fixture_bundle_core` |  |
| `target_model_edge_declared_inhabitants` | 1 | v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_bundle_with_rules` |  |
| `target_model_edge_fidelity_quotient` | 2 | v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_bundle_with_rules`<br>v2/test/claim/manual/target_carriers_ambiguous_quotient_test.dag `fidelity_quotient_edge` |  |
| `target_model_edge_selection_policy` | 1 | v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_bundle_with_rules` |  |
| `target_model_edge_serialize_source` | 1 | v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_bundle_with_rules` |  |
| `target_model_edge_translation_rules` | 3 | v2/test/claim/manual/ingest_bridge_test.dag `ingest_fixture_bundle_with_rules` | v2/extdeps/languages/rust_test_fixtures.dag `rust_bind_literal_target_model_bundle`<br>v2/extdeps/languages/rust_test_fixtures.dag `rust_classical_not_target_model_bundle_core` |
| `target_model_edge_type_expression_projection` | 1 | v2/extdeps/languages/rust_freemonoid_char_fixtures.dag `fmc_fixture_bundle_core` |  |
| `target_model_edge_use_site_ownership_realizations` | 1 |  | v2/std/compilers/target_model.dag `target_model_augment_use_site_ownership_catalog` |
| `target_model_edge_value_semantics_carriers` | 1 |  | v2/std/compilers/target_model.dag `target_model_augment_value_semantics_carriers` |
| `target_operator_realization_row_edge` | 4 | v2/test/claim/manual/target_model_operator_lookup_dissolution_test.dag `operator_catalog_with_malformed_and_valid_rows` | v2/std/compilers/target_model.dag `target_operator_realization_catalog_lookup`<br>v2/std/compilers/target_model.dag `target_operator_realization_catalog_wire_schema_valid` |
| `target_value_expr_effect_callee_row_edge` | 1 |  | v2/std/compilers/target_model.dag `decode_effect_apply_callee_rows` |
| `type_alias_marker` | 1 |  | v2/std/type_binder.dag `edge_is_type_alias` |
| `type_alias_marker()` | 1 | v2/std/type_binder.dag `type_alias_wrapper` |  |
| `type_annotation_marker` | 1 |  | v2/std/type_binder.dag `edge_is_type_annotation` |
| `type_annotation_marker()` | 1 | v2/std/type_binder.dag `type_annotation_edge` |  |
| `type_body_marker` | 1 |  | v2/std/type_binder.dag `edge_is_type_body` |
| `type_body_marker()` | 1 | v2/std/type_binder.dag `type_decl_wrapper` |  |
| `type_params_marker` | 1 |  | v2/std/type_binder.dag `edge_is_type_params` |
| `type_params_marker()` | 8 | v2/std/type_binder.dag `type_alias_wrapper`<br>v2/std/type_binder.dag `type_decl_wrapper`<br>v2/std/type_binder.dag `type_opaque_wrapper`<br>v2/std/type_binder.dag `type_params_edge` …+4 |  |
| `unused_parameters_edge_symbol` | 1 | v2/test/lens_unused_parameters/binds_to_edge.dag `unused_parameters_root` |  |
| `unused_parameters_non_use_edge_symbol` | 1 | v2/test/lens_unused_parameters/non_use_edges.dag `unused_parameters_non_use_root` |  |
| `use_site_verdict_edge` | 1 | v2/compiler/use_site_verdict.dag `attach_use_site_verdict` |  |
| `use_site_verdict_field_edge` | 1 | v2/compiler/use_site_verdict.dag `use_site_verdict_to_node` |  |
| `variant_marker` | 1 | v2/std/effects.dag `keyed_shape_node` |  |

## Per-site rows

Columns: decl (kind `symbol`) · line (convenience only) · role · Symbol source · symbol/var · disposition


### dag/gunbc/scm/object_store.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `collision_probe_parent` | 1516 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^only_child` | becomes Authored |

### dag/gunbc/scm/object_table_json.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `encode_label` | 536 | HASH(wire encode) | STRUCTURAL+AUTHORED | `n` | hash-canonical change: SCM JSON wire tag "named" must split ("structural"/"authored"); format version bump |
| fn `decode_named_label` | 971 | HASH(wire decode) | STRUCTURAL+AUTHORED | `symbol_intern_lexeme(lexeme: lexeme)` | hash-canonical change: decoder must refuse/route by new tag; interning an arbitrary lexeme into Structural must refuse (closed set) |

### dag/gunbc/scm/role_requirement_integration.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `edge_label_role` | 275 | MATCH-binds-name(read as name) | AUTHORED? | `name` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `root_edge_for` | 631 | CONSTRUCT | AUTHORED | `role` | becomes Authored |

### dag/test/claim/body_lowering_rejection_propagation_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `emitted_shell` | 45 | CONSTRUCT | STRUCTURAL | `^grammar_production_identity_node_projection` | becomes Structural arm grammar_production_identity_node_projection |

### dag/test/claim/node_hash_protocol_witness_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `named` | 87 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### dag/test/claim/record_construction_census_witness_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `census_edge_is_construction` | 44 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `record_construction_spelling` | becomes Authored |

### dag/test/claim/scm/scm_commit_closure_json_v2_witness_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `closure_doc_named` | 196 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `scm_image_named_stored` | 786 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### dag/test/claim/scm/scm_commit_closure_witness_test.dag (7)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `parent_node` | 120 | CONSTRUCT | TEST-FIXTURE | `^only_child` | becomes Authored |
| fn `closure_two_child_parent_node` | 231 | CONSTRUCT | TEST-FIXTURE | `^alpha` | becomes Authored |
| fn `closure_two_child_parent_node` | 232 | CONSTRUCT | TEST-FIXTURE | `^beta` | becomes Authored |
| fn `closure_over_an_occupied_locator` | 645 | CONSTRUCT | TEST-FIXTURE | `^only_child` | becomes Authored |
| fn `insert_two_named` | 788 | CONSTRUCT(pass-through) | PASS-THROUGH | `first_label` | arm-only rename (carry EdgeLabel whole instead of re-wrapping a Symbol) |
| fn `insert_two_named` | 789 | CONSTRUCT(pass-through) | PASS-THROUGH | `second_label` | arm-only rename (carry EdgeLabel whole instead of re-wrapping a Symbol) |
| fn `closure_over_a_manifest_locator` | 1273 | CONSTRUCT | TEST-FIXTURE | `^only_child` | becomes Authored |

### dag/test/claim/scm/scm_load_standing_witness_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `ls_parent` | 70 | CONSTRUCT | TEST-FIXTURE | `^only_child` | becomes Authored |
| fn `ls_requirement_over_a_file` | 369 | CONSTRUCT | TEST-FIXTURE | `^ls_only_child` | becomes Authored |

### dag/test/claim/scm/scm_object_store_witness_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `named_edge` | 100 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### dag/test/claim/scm/scm_repository_envelope_witness_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `scm_env_named` | 151 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| test `fn` | 1708 | CONSTRUCT | TEST-FIXTURE | `^only_child` | becomes Authored |
| fn `scm_env_child_over` | 2041 | CONSTRUCT | TEST-FIXTURE | `^only_child` | becomes Authored |

### dag/test/claim/scm/scm_role_requirement_integration_witness_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `integration_witness_bind` | 82 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `integration_witness_role_over_a_file_fixture` | 754 | CONSTRUCT | TEST-FIXTURE | `^s` | becomes Authored |
| fn `integration_witness_role_over_a_node_fixture` | 783 | CONSTRUCT | TEST-FIXTURE | `^s` | becomes Authored |

### src/v2/compiler/00_compile.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `native_test_decl_in_children` | 1560 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |

### src/v2/compiler/03_body_producer.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `attach_arrow_body` | 43 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |

### src/v2/compiler/03_ingest.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `grammar_relation_binding_to_token_class_map` | 136 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `grammar_relation_token_class_set` | 168 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `operational_parse_tree_project_to_formal_captures` | 229 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `grammar_production_identity_node_projection` | becomes Structural arm match |

### src/v2/compiler/03_resolve.dag (5)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `try_edge_declared_binding` | 193 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `resolve_transform_children` | 1252 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `resolve_match_arm_walk` | 1446 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `match_arm_pattern` | becomes Structural arm match |
| fn `resolve_child_edge` | 1531 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `dag_surface_module_header_metadata_edge` | becomes Structural arm match |
| fn `resolve_arrow_node_in` | 1721 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |

### src/v2/compiler/04_infer.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `infer_conj_edge_named_type_for_binding` | 912 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `binding` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `infer_formals_from_domain` | 1635 | MATCH-binds-name(read as name) | AUTHORED? | `nme` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `infer_literal_edge_diagnostics_derived` | 2832 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `integer_literal_magnitude_field` | becomes Structural arm match |

### src/v2/compiler/05_eval.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `eval_param_labels_in_declared_order` | 1386 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `eval_transform_has_callee_reference` | 1466 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |

### src/v2/compiler/06_translate.dag (10)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `target_bundle_child_lookup_step` | 462 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `edge_name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `target_bundle_child_lookup_step` | 472 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `edge_name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `grammar_relation_row_with_bodied_scaffold` | 636 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `grammar_relation_row_with_bodied_scaffold` | 651 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `optional_named_child_ignoring_positional` | 732 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `edge_name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `optional_named_child_ignoring_positional` | 742 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `edge_name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `serialize_source_tokens_fold_child` | 857 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `grammar_relation_field_tokens` | becomes Structural arm match |
| fn `translate_algebra` | 2193 | MATCH-binds-name(read as name) | AUTHORED? | `label` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `translate_algebra` | 2238 | MATCH-binds-name(read as name) | AUTHORED? | `label` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `serialize_type_expr_record_field_label` | 2444 | MATCH-binds-name(read as name) | AUTHORED? | `label` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/compiler/body_lowering_fold.dag (33)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `body_lower_arrow_domain_named_bindings` | 354 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `body_lower_param_binding_symbols_from_param_list` | 378 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `body_lower_is_identity_projection_edge` | 567 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `grammar_production_identity_node_projection` | becomes Structural arm match |
| fn `body_lower_kept_where_clause` | 742 | CONSTRUCT | STRUCTURAL | `^grammar_production_identity_node_projection` | becomes Structural arm grammar_production_identity_node_projection |
| fn `body_lower_kept_where_clause` | 750 | CONSTRUCT | STRUCTURAL | `^grammar_production_captured_node_projection` | becomes Structural arm grammar_production_captured_node_projection |
| fn `body_lower_kept_where_clause` | 760 | CONSTRUCT | STRUCTURAL | `^dag_surface_where_refinement_clause` | becomes Structural arm dag_surface_where_refinement_clause |
| fn `body_lower_type_variant_children_with_where` | 774 | CONSTRUCT | STRUCTURAL | `^dag_surface_type_expr` | becomes Structural arm dag_surface_type_expr |
| fn `body_lower_type_variant_children_with_where` | 776 | CONSTRUCT | STRUCTURAL | `^dag_surface_where_refinement_clause` | becomes Structural arm dag_surface_where_refinement_clause |
| fn `body_lower_type_variant_children_with_where` | 785 | CONSTRUCT | STRUCTURAL | `^dag_surface_field_decl_block` | becomes Structural arm dag_surface_field_decl_block |
| fn `body_lower_field_decl_edge_optional` | 812 | CONSTRUCT | AUTHORED | `field_name` | becomes Authored |
| fn `body_lower_positional_payload` | 884 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `positional_payload_field_name()` | becomes Structural arm positional_payload_field_name() |
| fn `body_lower_typed_param_edge` | 1855 | CONSTRUCT | AUTHORED | `binding` | becomes Authored |
| fn `body_lower_generic_param_binder_edge` | 2279 | CONSTRUCT | AUTHORED | `name` | becomes Authored |
| fn `body_lower_fn_decl_named_member_wrap` | 3030 | CONSTRUCT | AUTHORED | `fn_name` | becomes Authored |
| fn `body_lower_data_decl_to_member` | 3259 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `body_lower_fn_decl_arrow` | 3308 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `body_lower_match_arm_wire` | 4820 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `match_arm_pattern` | becomes Structural arm match_arm_pattern |
| fn `body_lower_match_arm_wire` | 4821 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `match_arm_body` | becomes Structural arm match_arm_body |
| fn `body_lower_field_pattern_edge` | 4957 | CONSTRUCT | AUTHORED | `field_name` | becomes Authored |
| fn `body_lower_field_pattern_edge` | 4973 | CONSTRUCT | AUTHORED | `field_name` | becomes Authored |
| fn `body_lower_pattern_field_edges` | 4996 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `positional_payload_field_name()` | becomes Structural arm positional_payload_field_name() |
| fn `body_lowering_match_spine_identity_absent_fixture_spine` | 6956 | CONSTRUCT | STRUCTURAL | `^grammar_sequence_left_node_projection` | becomes Structural arm grammar_sequence_left_node_projection |
| fn `body_lowering_match_spine_identity_absent_fixture_spine` | 6960 | CONSTRUCT | STRUCTURAL | `^grammar_sequence_right_node_projection` | becomes Structural arm grammar_sequence_right_node_projection |
| fn `body_lowering_match_spine_identity_absent_fixture_spine` | 6969 | CONSTRUCT | STRUCTURAL | `^grammar_sequence_left_node_projection` | becomes Structural arm grammar_sequence_left_node_projection |
| fn `body_lowering_match_spine_identity_absent_fixture_spine` | 6973 | CONSTRUCT | STRUCTURAL | `^grammar_sequence_right_node_projection` | becomes Structural arm grammar_sequence_right_node_projection |
| fn `body_lowering_match_field_name_spine` | 6996 | CONSTRUCT | STRUCTURAL | `^grammar_sequence_left_node_projection` | becomes Structural arm grammar_sequence_left_node_projection |
| fn `body_lowering_match_field_name_spine` | 7000 | CONSTRUCT | STRUCTURAL | `^grammar_sequence_right_node_projection` | becomes Structural arm grammar_sequence_right_node_projection |
| fn `body_lower_field_init_edge` | 7185 | CONSTRUCT | AUTHORED | `field_name` | becomes Authored |
| fn `body_lower_edge_names_where_clause` | 7528 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `dag_surface_where_refinement_clause` | becomes Structural arm match |
| fn `body_lower_type_decl_member` | 7600 | CONSTRUCT | AUTHORED | `type_name` | becomes Authored |
| fn `body_lower_type_decl_declared` | 7713 | CONSTRUCT | AUTHORED | `type_name` | becomes Authored |
| fn `body_lower_operation` | 8051 | CONSTRUCT | AUTHORED | `op_name` | becomes Authored |
| fn `body_lower_service_node` | 8210 | CONSTRUCT | AUTHORED | `service_name` | becomes Authored |

### src/v2/compiler/emit_produced.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `emit_produced_decl_inner_arrow` | 51 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `produced_decl_edge_is_decl` | 103 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `produced_module_edge_is_production_marker` | 143 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `grammar_production_identity_node_projection` | becomes Structural arm match |
| fn `produced_module_member_is_bodied_decl` | 150 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |

### src/v2/compiler/fold_lowering.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `fold_call_seam_loop` | 328 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| fn `fold_call_seam_loop` | 329 | CONSTRUCT | STRUCTURAL | `^loop_carrier_edge` | becomes Structural arm loop_carrier_edge |

### src/v2/compiler/ingested_fixture_arrows.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `ingested_add_decl_node` | 222 | CONSTRUCT | AUTHORED | `fn_name` | becomes Authored |

### src/v2/compiler/namespace_graft.dag (10)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `namespace_graft_is_transparent_sequence_spine` | 115 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `parse_tree_projection_edge` | becomes Structural arm match |
| fn `namespace_graft_collect_body_edge` | 134 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `namespace_graft_is_metadata_edge` | becomes Structural arm match |
| fn `namespace_graft_reconstruct_declaration_forest` | 270 | CONSTRUCT | STRUCTURAL | `^dag_surface_top_level_item` | becomes Structural arm dag_surface_top_level_item |
| fn `namespace_graft_type_decl_named_edge` | 331 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `type_name` | split into structural/authored constructors; callers classified separately (see funnel table) |
| fn `namespace_graft_kw_ident_named_edge` | 350 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |
| fn `namespace_graft_keep_flattened_member_edge` | 471 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `parse_tree_projection_edge` | becomes Structural arm match |
| fn `namespace_graft_named_list_has_name` | 501 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `namespace_graft_flatten_minus_decl_names` | 511 | MATCH-binds-name(read as name) | AUTHORED? | `n` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `namespace_graft_wrap_segment` | 630 | CONSTRUCT | AUTHORED | `segment` | becomes Authored |
| fn `namespace_graft_has_explicit_braced_container` | 683 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `namespace_graft_is_metadata_edge` | becomes Structural arm match |

### src/v2/compiler/program_partition.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `partition_collect_user_semantic_type_edge` | 161 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/compiler/reference_conservation.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `rebuilt_spelling_pool` | 682 | MATCH-binds-name(read as name) | AUTHORED? | `name` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `named_child_optional` | 701 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `label` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `declaration_pools` | 733 | MATCH-binds-name(read as name) | AUTHORED? | `name` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/compiler/reference_site_collector.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `reference_sites_in_edge` | 148 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `dag_surface_module_header_metadata_edge` | becomes Structural arm match |

### src/v2/compiler/symbol_index_fill.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `symbol_index_fill_containment_edge` | 105 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `dag_surface_module_header_metadata_edge` | becomes Structural arm match |
| fn `symbol_index_fill_containment_node` | 176 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `disj_variant_counts_in_module` | 231 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `symbol_index_fill_unique_variant_aliases` | 380 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/compiler/use_site_verdict.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `use_site_verdict_to_node` | 52 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `use_site_verdict_field_edge` | becomes Structural arm use_site_verdict_field_edge |
| fn `attach_use_site_verdict` | 104 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `use_site_verdict_edge` | becomes Structural arm use_site_verdict_edge |

### src/v2/compiler/wrap_decision.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `wrap_decision_bundle_child_lookup_step` | 57 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `edge_name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `wrap_decision_bundle_child_lookup_step` | 67 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `edge_name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |

### src/v2/extdeps/languages/bash.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `bash_named_edge` | 259 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/c.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `c_named_edge` | 907 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/c_decl.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `c_decl_named_edge` | 95 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/cpp.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `cpp_named_edge` | 193 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/dag.dag (15)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `dag_named_edge` | 633 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |
| fn `dag_node_is_module_root_conj` | 3241 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `dag_surface_module_header` | becomes Structural arm match |
| fn `dag_loop_body_from_literal` | 3726 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| fn `dag_int_literal_node_from_magnitude` | 3846 | CONSTRUCT | STRUCTURAL | `^integer_literal_magnitude_field` | becomes Structural arm integer_literal_magnitude_field |
| fn `dag_float_literal_node_from_lexeme` | 3884 | CONSTRUCT | STRUCTURAL | `^float_literal_lexeme_field` | becomes Structural arm float_literal_lexeme_field |
| fn `dag_int_literal_magnitude_edge_target_optional` | 3905 | CONSTRUCT | AUTHORED | `field` | becomes Authored |
| fn `dag_arrow_with_body_node` | 4024 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `dag_arrow_domain_conj_node` | 4042 | CONSTRUCT | AUTHORED | `binding_id` | becomes Authored |
| fn `dag_complement_arrow_domain_node` | 4452 | CONSTRUCT | STRUCTURAL | `^dag_binding_param_a` | becomes Structural arm dag_binding_param_a |
| fn `dag_bool_binop_arrow_domain_node` | 4510 | CONSTRUCT | STRUCTURAL | `^dag_binding_param_a` | becomes Structural arm dag_binding_param_a |
| fn `dag_bool_binop_arrow_domain_node` | 4514 | CONSTRUCT | STRUCTURAL | `^dag_binding_param_b` | becomes Structural arm dag_binding_param_b |
| fn `dag_field_access_arrow_domain_conj_node` | 4611 | CONSTRUCT | STRUCTURAL | `^dag_binding_param_r` | becomes Structural arm dag_binding_param_r |
| fn `dag_round_trip_edge` | 5308 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |
| fn `namespace_graft_spine_segment_edge_optional` | 6167 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `namespace_graft_qualified_name_from_spine_node` | 6207 | MATCH-binds-name(read as name) | AUTHORED? | `seg` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/extdeps/languages/ecmascript.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `ecmascript_named_edge` | 240 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/english.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `english_named_edge` | 374 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/go.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `go_named_edge` | 413 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/java.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `java_named_edge` | 504 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/kotlin.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `kotlin_named_edge` | 343 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/lean.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `lean_named_edge` | 484 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/llvm_ir.dag (7)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `llvm_integer_facts_node` | 619 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisSurfaceSpelling {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisSurfaceSpelling {}) |
| fn `llvm_integer_facts_node` | 623 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisWidth {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisWidth {}) |
| fn `llvm_integer_facts_node` | 627 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisEncoding {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisEncoding {}) |
| fn `llvm_integer_facts_node` | 631 | CONSTRUCT | STRUCTURAL | `^llvm_facts_field_std_projection` | becomes Structural arm llvm_facts_field_std_projection |
| fn `llvm_float_facts_node` | 644 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisSurfaceSpelling {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisSurfaceSpelling {}) |
| fn `llvm_float_facts_node` | 648 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisEncoding {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisEncoding {}) |
| fn `llvm_float_facts_node` | 652 | CONSTRUCT | STRUCTURAL | `^llvm_facts_field_std_projection` | becomes Structural arm llvm_facts_field_std_projection |

### src/v2/extdeps/languages/machine_code.dag (6)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `mc_integer_facts_node` | 347 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisSurfaceSpelling {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisSurfaceSpelling {}) |
| fn `mc_integer_facts_node` | 351 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisSignedness {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisSignedness {}) |
| fn `mc_integer_facts_node` | 355 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisWidth {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisWidth {}) |
| fn `mc_integer_facts_node` | 359 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisOverflowDisposition {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisOverflowDisposition {}) |
| fn `mc_integer_facts_node` | 363 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisEncoding {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisEncoding {}) |
| fn `mc_integer_facts_node` | 367 | CONSTRUCT | STRUCTURAL | `^mc_facts_field_std_projection` | becomes Structural arm mc_facts_field_std_projection |

### src/v2/extdeps/languages/python.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `python_named_edge` | 255 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/rust.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `rust_named_edge` | 957 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |
| fn `rust_sg2_emitted_without_named_edge` | 4968 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `field` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |

### src/v2/extdeps/languages/rust_freemonoid_char_fixtures.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `fmc_fixture_bundle_core` | 35 | CONSTRUCT | STRUCTURAL | `^target_model_edge_type_expression_projection` | becomes Structural arm target_model_edge_type_expression_projection |
| fn `fmc_fixture_bundle_core` | 39 | CONSTRUCT | STRUCTURAL | `^target_model_edge_collection_realization` | becomes Structural arm target_model_edge_collection_realization |

### src/v2/extdeps/languages/rust_test_fixtures.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `rust_classical_not_target_model_bundle_core` | 1205 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `target_model_edge_translation_rules` | becomes Structural arm match |
| fn `rust_arrow_with_body_node` | 3804 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `rust_bind_literal_target_model_bundle` | 3938 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `target_model_edge_translation_rules` | becomes Structural arm match |

### src/v2/extdeps/languages/swift.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `swift_named_edge` | 364 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/swift_decl.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `swift_decl_named_edge` | 64 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/typescript.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `ts_named_edge` | 273 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/verilog.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `verilog_named_edge` | 1080 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/extdeps/languages/wasm.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `wasm_named_edge` | 337 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/lens/application.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `rebuild_node_with_child` | 222 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |

### src/v2/lens/complexity_accumulator_copy/analyze.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `edge_skips_identifier_count` | 82 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `grammar_production_identity_node_projection` | becomes Structural arm match |

### src/v2/lens/complexity_accumulator_copy/corpus_census.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `ingested_root_child_labels` | 406 | MATCH-binds-name(read as name) | AUTHORED? | `n` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/lens/cost/copied_port_citations.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `citation_loop_body_linear_in` | 150 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| fn `citation_arrow_two_param_loop` | 171 | CONSTRUCT | AUTHORED | `left_sym` | becomes Authored |
| fn `citation_arrow_two_param_loop` | 172 | CONSTRUCT | AUTHORED | `right_sym` | becomes Authored |
| fn `citation_arrow_two_param_loop` | 186 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |

### src/v2/lens/cost/copied_port_derivation.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `arrow_domain_param_nodes` | 77 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `arrow_domain_param_nodes` | 83 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |

### src/v2/lens/effect_reach.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `positional_child_targets` | 200 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |

### src/v2/lens/enforcement/receipts.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `cost_red_control_probe` | 193 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| data `cost_red_control_probe` | 194 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^enforcement_red_probe_extra` | becomes Authored |

### src/v2/lens/fact_density.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `named_fact_count` | 24 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |

### src/v2/lens/identity_captured_navigation/analyze.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `edge_is_identity_projection` | 79 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `grammar_production_identity_node_projection` | becomes Structural arm match |

### src/v2/lens/lifecycle_carrier.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `field_lifecycle_violation` | 117 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `name_is_lifecycle_field` | becomes Authored |

### src/v2/lens/live_read_classification.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `positional_child_targets` | 217 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |

### src/v2/lens/reference_derived_residency_reading.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `decl_first_unrecognised_atom` | 154 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `decl_has_keyed_population_field` | 193 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `decl_has_syntax_field` | 206 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |

### src/v2/lens/testgen.dag (41)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `refinement_preservation_list_node` | 480 | CONSTRUCT | STRUCTURAL | `^refinement_preservation_list_head_edge` | becomes Structural arm refinement_preservation_list_head_edge |
| fn `refinement_preservation_list_node` | 482 | CONSTRUCT | STRUCTURAL | `^refinement_preservation_list_tail_edge` | becomes Structural arm refinement_preservation_list_tail_edge |
| fn `algebra_law_subject_node` | 772 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^t19_algebra_law_algebra_field` | becomes Authored |
| fn `algebra_law_subject_node` | 773 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^t19_algebra_law_inhabitant_field` | becomes Authored |
| fn `algebra_law_subject_node` | 774 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^t19_algebra_law_law_field` | becomes Authored |
| fn `algebra_law_claim_term` | 784 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^t19_algebra_law_subject_field` | becomes Authored |
| fn `algebra_law_claim_term` | 785 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^t19_algebra_law_expression_field` | becomes Authored |
| fn `witness_validity_predicate_node` | 803 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_field_algebra` | becomes Authored |
| fn `witness_validity_predicate_node` | 805 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_field_preservation_rule` | becomes Authored |
| fn `witness_validity_witness_node` | 818 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_field_property` | becomes Authored |
| fn `witness_validity_witness_node` | 821 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_field_evidence` | becomes Authored |
| fn `witness_validity_subject_node` | 855 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_field_source_facts` | becomes Authored |
| fn `witness_validity_subject_node` | 856 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_field_candidate` | becomes Authored |
| fn `witness_validity_subject_node` | 858 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_field_predicate` | becomes Authored |
| fn `witness_validity_subject_node` | 862 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_field_witness` | becomes Authored |
| fn `witness_validity_subject_node` | 866 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_field_oracle_outcome` | becomes Authored |
| fn `witness_validity_correction_node` | 897 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_correction_tag_suggested` | becomes Authored |
| fn `witness_validity_correction_node` | 906 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_correction_tag_unavailable` | becomes Authored |
| fn `witness_validity_locus_node` | 940 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_locus_tag_textual` | becomes Authored |
| fn `witness_validity_locus_node` | 945 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_locus_textual_file` | becomes Authored |
| fn `witness_validity_locus_node` | 949 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_locus_textual_extent` | becomes Authored |
| fn `witness_validity_locus_node` | 967 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_locus_tag_node` | becomes Authored |
| fn `witness_validity_locus_node` | 980 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_locus_tag_source_port` | becomes Authored |
| fn `witness_validity_locus_node` | 984 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_locus_source_port_file` | becomes Authored |
| fn `witness_validity_locus_node` | 988 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_locus_source_port_extent` | becomes Authored |
| fn `witness_validity_locus_node` | 1003 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_locus_tag_invariant_port` | becomes Authored |
| fn `witness_validity_locus_node` | 1007 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_locus_invariant` | becomes Authored |
| fn `witness_validity_locus_node` | 1023 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_locus_tag_declaration` | becomes Authored |
| fn `witness_validity_diagnostic_node` | 1048 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_diag_field_reason` | becomes Authored |
| fn `witness_validity_diagnostic_node` | 1052 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_diag_field_at` | becomes Authored |
| fn `witness_validity_diagnostic_node` | 1056 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_diag_field_correction` | becomes Authored |
| fn `witness_validity_diagnostic_list_node` | 1081 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_list_cons_head` | becomes Authored |
| fn `witness_validity_diagnostic_list_node` | 1082 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_list_cons_tail` | becomes Authored |
| fn `witness_validity_non_empty_diagnostics_node` | 1102 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_diags_field_head` | becomes Authored |
| fn `witness_validity_non_empty_diagnostics_node` | 1106 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_diags_field_tail` | becomes Authored |
| fn `witness_validity_diagnostics_node` | 1132 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_diagnostics_tag_some` | becomes Authored |
| fn `witness_validity_outcome_node` | 1153 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_outcome_holds` | becomes Authored |
| fn `witness_validity_outcome_node` | 1158 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_outcome_accepted_value` | becomes Authored |
| fn `witness_validity_outcome_node` | 1162 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_outcome_accepted_diagnostics` | becomes Authored |
| fn `witness_validity_outcome_node` | 1182 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^witness_validity_outcome_violates` | becomes Authored |
| fn `coproduct_exhaustiveness_input` | 1651 | CONSTRUCT | STRUCTURAL | `^coproduct_exhaustiveness_omitted_variant_edge` | becomes Structural arm coproduct_exhaustiveness_omitted_variant_edge |

### src/v2/lens/unit_modeling.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `field_unit_violation` | 93 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `name_is_exempt` | becomes Authored |

### src/v2/std/algebra.dag (46)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `fold_list_node` | 126 | CONSTRUCT | STRUCTURAL | `^fold_list_node_head` | becomes Structural arm fold_list_node_head |
| fn `fold_list_node` | 127 | CONSTRUCT | STRUCTURAL | `^fold_list_node_tail` | becomes Structural arm fold_list_node_tail |
| fn `free_monoid_tail_type_fixpoint` | 146 | CONSTRUCT | STRUCTURAL | `^free_monoid_type_fixpoint` | becomes Structural arm free_monoid_type_fixpoint |
| fn `free_monoid_type_node` | 158 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^free_monoid_variant_empty` | becomes Authored |
| fn `free_monoid_type_node` | 160 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^free_monoid_variant_cons` | becomes Authored |
| fn `free_monoid_type_node` | 164 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^free_monoid_field_head` | becomes Authored |
| fn `free_monoid_type_node` | 166 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^free_monoid_field_tail` | becomes Authored |
| fn `ordering_type_node` | 205 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^ordering_variant_less` | becomes Authored |
| fn `ordering_type_node` | 206 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^ordering_variant_equal` | becomes Authored |
| fn `ordering_type_node` | 207 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^ordering_variant_greater` | becomes Authored |
| fn `magma_type_node` | 218 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^magma_field_op` | becomes Authored |
| fn `semigroup_type_node` | 230 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^semigroup_field_magma` | becomes Authored |
| fn `monoid_type_node` | 240 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^monoid_field_semigroup` | becomes Authored |
| fn `monoid_type_node` | 241 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^monoid_field_identity` | becomes Authored |
| fn `commutative_monoid_type_node` | 251 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^commutative_monoid_field_monoid` | becomes Authored |
| fn `group_type_node` | 261 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^group_field_monoid` | becomes Authored |
| fn `group_type_node` | 263 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^group_field_inverse` | becomes Authored |
| fn `abelian_group_type_node` | 275 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^abelian_group_field_group` | becomes Authored |
| fn `semiring_type_node` | 285 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^semiring_field_add` | becomes Authored |
| fn `semiring_type_node` | 286 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^semiring_field_mul` | becomes Authored |
| fn `commutative_semiring_type_node` | 296 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^commutative_semiring_field_semiring` | becomes Authored |
| fn `ring_type_node` | 306 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^ring_field_add` | becomes Authored |
| fn `ring_type_node` | 307 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^ring_field_mul` | becomes Authored |
| fn `commutative_ring_type_node` | 317 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^commutative_ring_field_ring` | becomes Authored |
| fn `ordered_ring_type_node` | 327 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^ordered_ring_field_ring` | becomes Authored |
| fn `ordered_ring_type_node` | 329 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^ordered_ring_field_compare` | becomes Authored |
| fn `field_type_node` | 341 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^field_field_commutative_ring` | becomes Authored |
| fn `field_type_node` | 342 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^field_field_reciprocal` | becomes Authored |
| fn `approximate_field_type_node` | 365 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^approximate_field_field_add` | becomes Authored |
| fn `approximate_field_type_node` | 368 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^approximate_field_field_zero` | becomes Authored |
| fn `approximate_field_type_node` | 370 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^approximate_field_field_negate` | becomes Authored |
| fn `approximate_field_type_node` | 374 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^approximate_field_field_mul` | becomes Authored |
| fn `approximate_field_type_node` | 377 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^approximate_field_field_one` | becomes Authored |
| fn `approximate_field_type_node` | 379 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^approximate_field_field_compare` | becomes Authored |
| fn `lattice_type_node` | 392 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^lattice_field_meet` | becomes Authored |
| fn `lattice_type_node` | 396 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^lattice_field_join` | becomes Authored |
| fn `bounded_lattice_type_node` | 408 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^bounded_lattice_field_lattice` | becomes Authored |
| fn `bounded_lattice_type_node` | 409 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^bounded_lattice_field_top` | becomes Authored |
| fn `bounded_lattice_type_node` | 410 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^bounded_lattice_field_bottom` | becomes Authored |
| fn `boolean_algebra_type_node` | 421 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^boolean_algebra_field_bounded_lattice` | becomes Authored |
| fn `boolean_algebra_type_node` | 425 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^boolean_algebra_field_complement` | becomes Authored |
| fn `algebra_inhabitance_node` | 437 | CONSTRUCT | STRUCTURAL | `^algebra_edge` | becomes Structural arm algebra_edge |
| fn `algebra_inhabitance_node` | 438 | CONSTRUCT | STRUCTURAL | `^inhabitant_edge` | becomes Structural arm inhabitant_edge |
| fn `bool_type_node` | 570 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^bool_variant_true` | becomes Authored |
| fn `bool_type_node` | 571 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^bool_variant_false` | becomes Authored |
| fn `equivalence_type_node` | 582 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^equivalence_field_eq` | becomes Authored |

### src/v2/std/anonymous_binder.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `anonymous_binder_mint_parameter_list` | 81 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `pattern_wildcard_name` | becomes Structural arm match |
| fn `anonymous_binder_mint_parameter_list` | 85 | CONSTRUCT(pass-through) | PASS-THROUGH | `label` | arm-only rename (carry EdgeLabel whole instead of re-wrapping a Symbol) |

### src/v2/std/bounded_lattice_completeness.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `infer_edge_consumes_bounded_lattice_partial` | 133 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `algebra_edge` | becomes Structural arm match |

### src/v2/std/coercion.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `coercion_homomorphism_evidence_for_node` | 152 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^coercion_homomorphism_evidence_source` | becomes Authored |
| fn `coercion_homomorphism_evidence_for_node` | 153 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^coercion_homomorphism_evidence_candidate` | becomes Authored |
| fn `coercion_homomorphism_evidence_for_node` | 155 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^coercion_homomorphism_evidence_find_witness` | becomes Authored |

### src/v2/std/compilers/body_lowering.dag (6)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `signature_order_edge` | 152 | CONSTRUCT | STRUCTURAL | `^arrow_signature_order_edge` | becomes Structural arm arrow_signature_order_edge |
| fn `lower_loop` | 272 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| fn `view_whole_positional_children` | 416 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `application_positional_targets` | 514 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `match_arm_view` | 720 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `match_arm_pattern` | becomes Structural arm match |
| fn `match_arm_view` | 722 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `match_arm_body` | becomes Structural arm match |

### src/v2/std/compilers/semantic_decl_emission.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `target_semantic_decl_field_surface_from_edge` | 1431 | MATCH-binds-name(read as name) | AUTHORED? | `fname` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `target_semantic_decl_tagged_union_variant_surface_from_edge` | 1928 | MATCH-binds-name(read as name) | AUTHORED? | `vname` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `target_semantic_decl_variant_surface_from_edge` | 1987 | MATCH-binds-name(read as name) | AUTHORED? | `vname` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/std/compilers/sugar.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `sugar_variant_to_disj_edge` | 260 | CONSTRUCT | AUTHORED | `name` | becomes Authored |

### src/v2/std/compilers/target_model.dag (63)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `produced_decl_param_segment_tokens` | 319 | MATCH-binds-name(read as name) | AUTHORED? | `pname` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `produced_decl_subject_from_decl` | 517 | MATCH-binds-name(read as name) | AUTHORED? | `fn_name` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `concrete_syntax_token_to_node` | 661 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^concrete_syntax_token_field_kind` | becomes Authored |
| fn `concrete_syntax_token_to_node` | 669 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^concrete_syntax_token_field_class` | becomes Authored |
| fn `concrete_syntax_token_to_node` | 684 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^concrete_syntax_token_field_kind` | becomes Authored |
| fn `concrete_syntax_token_to_node` | 692 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^concrete_syntax_token_field_class` | becomes Authored |
| fn `concrete_syntax_token_to_node` | 700 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^concrete_syntax_token_field_binding` | becomes Authored |
| fn `target_model_canonical_operation_atom_field_from_wire` | 1609 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `field_name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `target_model_canonical_operation_roster_edge` | 1770 | CONSTRUCT | AUTHORED | `name` | becomes Authored |
| fn `target_model_canonical_operation_wire_child_declared_type` | 1862 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_canonical_operation_field_predicate` | becomes Authored |
| fn `target_operator_shape_wire_schema_valid` | 2210 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_operator_shape_field_token` | becomes Authored |
| fn `target_operator_shape_token_from_wire` | 2237 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_operator_shape_field_token` | becomes Authored |
| fn `target_operator_realization_row_wire_schema_valid` | 2321 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_operator_realization_field_operation` | becomes Authored |
| fn `decode_effect_apply_callee_rows` | 2974 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `target_value_expr_effect_callee_row_edge` | becomes Structural arm match |
| fn `concrete_syntax_token_wire_schema_valid` | 4031 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `concrete_syntax_token_field_kind` | becomes Authored |
| fn `target_model_named_edge` | 4256 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |
| fn `target_value_expression_int_literal_lexeme_from_wire` | 4507 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expression_field_value` | becomes Authored |
| fn `target_value_expr_binding_ref_wire_schema_valid` | 4610 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_binding` | becomes Authored |
| fn `target_value_expr_binding_ref_binding_from_wire` | 4646 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_binding` | becomes Authored |
| fn `target_value_expr_primitive_apply_wire_schema_valid` | 4699 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_operation` | becomes Authored |
| fn `target_value_expr_primitive_apply_operand_bindings` | 4735 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_operation` | becomes Authored |
| fn `target_value_expr_effect_apply_wire_schema_valid` | 4837 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_effect_kind` | becomes Authored |
| fn `target_value_expr_effect_apply_kind_from_wire` | 4870 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_effect_kind` | becomes Authored |
| fn `target_value_expr_effect_apply_operand_bindings` | 4905 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_effect_kind` | becomes Authored |
| fn `target_value_expr_match_arm_edges_after_scrutinee` | 5192 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `target_operator_realization_catalog_wire_schema_valid` | 5386 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `target_operator_realization_row_edge` | becomes Structural arm match |
| fn `target_operator_realization_catalog_lookup` | 5443 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `target_operator_realization_row_edge` | becomes Structural arm match |
| fn `target_value_expr_primitive_apply_operation_from_wire` | 5486 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_operation` | becomes Authored |
| fn `target_value_expression_symbol_identity_from_wire` | 5728 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expression_field_identity` | becomes Authored |
| fn `target_value_expr_conditional_child_from_wire` | 5915 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `field` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `target_value_expr_record_construct_type_from_wire` | 6304 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_record_type` | becomes Authored |
| fn `target_value_expr_record_construct_field_slots_from_wire` | 6338 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_record_type` | becomes Authored |
| fn `target_value_expr_field_access_wire_schema_valid` | 6860 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_field_access_base` | becomes Authored |
| fn `target_value_expr_field_access_field_from_wire` | 6902 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_field_access_field` | becomes Authored |
| fn `target_value_expr_match_arm_wire_schema_valid` | 6994 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `match_arm_pattern` | becomes Structural arm match |
| fn `target_value_expr_match_wire_schema_valid` | 7043 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_match_scrutinee` | becomes Authored |
| fn `target_value_expr_match_arms_from_wire` | 7130 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `target_value_expression_bool_literal_value_from_wire` | 7260 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expression_field_value` | becomes Authored |
| fn `target_value_expr_callable_apply_callee_from_wire` | 7809 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_callee` | becomes Authored |
| fn `target_value_expr_callable_apply_args_from_wire` | 7843 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_callable_arg` | becomes Authored |
| fn `target_value_expr_closure_params_from_wire` | 7972 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expr_field_closure_param` | becomes Authored |
| fn `target_inhabitant_wire_primary_atom_identity` | 8235 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `target_model_bundle_core_keep_edge` | 9009 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `target_model_edge_atom_realizations` | becomes Structural arm match |
| fn `target_atom_realization_conj_child` | 9269 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `edge_name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `projection_bundle_has_child` | 9940 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `target_type_expr_surface_edge` | 9985 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^target_type_expr_field_surface` | becomes Authored |
| fn `target_type_expr_decode_kind_edge` | 10039 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_type_expr_field_kind` | becomes Authored |
| fn `target_type_expr_kind_edge` | 10088 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^target_type_expr_field_kind` | becomes Authored |
| fn `target_type_expr_field_slot_edges` | 10206 | CONSTRUCT(pass-through) | PASS-THROUGH | `field.label` | arm-only rename (carry EdgeLabel whole instead of re-wrapping a Symbol) |
| fn `target_type_expr_arrow_param_slot_edges` | 10215 | CONSTRUCT(pass-through) | PASS-THROUGH | `param.label` | arm-only rename (carry EdgeLabel whole instead of re-wrapping a Symbol) |
| fn `target_type_expr_variant_slot_edges` | 10224 | CONSTRUCT(pass-through) | PASS-THROUGH | `variant.label` | arm-only rename (carry EdgeLabel whole instead of re-wrapping a Symbol) |
| fn `target_type_expr_emitted_slot_type_nodes` | 10232 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_type_expr_field_surface` | becomes Authored |
| fn `target_type_expr_fold_named_metadata_edges` | 10262 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_type_expr_field_kind` | becomes Authored |
| fn `target_type_expr_emitted_labeled_slot_edges` | 10543 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_type_expr_field_surface` | becomes Authored |
| fn `target_type_expr_arrow_emitted_named` | 10605 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^target_type_expr_field_codomain` | becomes Authored |
| fn `target_type_expr_arrow_labeled_param_slots` | 10645 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_type_expr_field_surface` | becomes Authored |
| fn `target_use_site_ownership_catalog_lookup_step` | 12486 | MATCH-binds-name(read as name) | AUTHORED? | `display_name` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `target_model_bundle_replace_edge_target` | 12782 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `edge_name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `target_model_augment_use_site_ownership_catalog` | 12884 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `target_model_edge_use_site_ownership_realizations` | becomes Structural arm match |
| fn `target_model_augment_value_semantics_carriers` | 12934 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `target_model_edge_value_semantics_carriers` | becomes Structural arm match |
| fn `target_value_semantics_carrier_snoc` | 12997 | MATCH-binds-name(read as name) | AUTHORED? | `n` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `target_value_expr_int_literal_magnitude_node_optional` | 14098 | CONSTRUCT | AUTHORED | `field` | becomes Authored |
| fn `target_value_expr_record_construct_field_slots_from_edges` | 14373 | MATCH-binds-name(read as name) | AUTHORED? | `field_label` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/std/decl_facts_skeleton.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `skeleton_edge_is_construction_spelling` | 188 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `record_construction_spelling` | becomes Authored |

### src/v2/std/dependency.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `classify_node_edge_usage` | 182 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/std/effects.dag (19)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `key_source_node` | 134 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^key_source_path_param_field` | becomes Authored |
| fn `key_source_node` | 138 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^key_source_path_param_value_field` | becomes Authored |
| fn `key_source_node` | 149 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^key_source_input_field_field` | becomes Authored |
| fn `key_source_node` | 153 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^key_source_input_value_field` | becomes Authored |
| fn `keyed_shape_node` | 167 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `variant_marker` | becomes Structural arm variant_marker |
| fn `keyed_shape_node` | 171 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^idempotent_operation_key_source_field` | becomes Authored |
| fn `classified_idempotent_effect_node` | 203 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^effect_shape_is_idempotent_field` | becomes Authored |
| fn `classified_idempotent_effect_node` | 207 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^effect_shape_inner_field` | becomes Authored |
| fn `idempotent_operation_ref_node` | 220 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^idempotent_operation_callable_field` | becomes Authored |
| fn `idempotent_operation_witness_node` | 235 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^effect_shape_inner_field` | becomes Authored |
| fn `idempotent_operation_witness_node` | 239 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^idempotent_operation_callable_field` | becomes Authored |
| fn `idempotent_operation_witness_node` | 250 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^effect_shape_inner_field` | becomes Authored |
| fn `idempotent_operation_witness_node` | 254 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^idempotent_operation_callable_field` | becomes Authored |
| fn `idempotent_operation_witness_node` | 265 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^effect_shape_inner_field` | becomes Authored |
| fn `idempotent_operation_witness_node` | 269 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^idempotent_operation_callable_field` | becomes Authored |
| fn `idempotent_operation_witness_node` | 280 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^effect_shape_inner_field` | becomes Authored |
| fn `idempotent_operation_witness_node` | 286 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^idempotent_operation_callable_field` | becomes Authored |
| fn `idempotent_operation_apply_node` | 300 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^idempotent_apply_state_field` | becomes Authored |
| fn `idempotent_operation_apply_node` | 304 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^idempotent_apply_operation_field` | becomes Authored |

### src/v2/std/fold_assembly.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `named_edge` | 23 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/std/grammar.dag (8)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `grammar_named_edge` | 381 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |
| fn `formal_rhs_from_node` | 768 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `grammar_relation_rules_formal_productions_from_rows` | 877 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `grammar_relation_rules_formal_productions` | becomes Structural arm match |
| fn `formal_productions_from_catalog_node` | 939 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `conj_positional_children` | 1084 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `grammar_relation_row_reverse_parse_selection` | 2760 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `grammar_relation_rules_formal_productions` | becomes Structural arm match |
| fn `grammar_relation_row_forward_select` | 2835 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `grammar_relation_forward_row_select_skip_catalog_edge` | becomes Structural arm match |
| fn `grammar_node_carries_surface_identity` | 2966 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `surface_identity` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |

### src/v2/std/inhabitance.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `application_argument_does_not_inhabit_diagnostic` | 322 | CONSTRUCT | AUTHORED | `inhabitance_position_symbol(position: obligation.position)` | becomes Authored |
| fn `application_argument_does_not_inhabit_diagnostic` | 326 | CONSTRUCT | AUTHORED | `obligation.parameter_identity` | becomes Authored |
| fn `application_argument_does_not_inhabit_diagnostic` | 330 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^inhabitance_produced_type` | becomes Authored |
| fn `application_argument_does_not_inhabit_diagnostic` | 334 | CONSTRUCT | AUTHORED | `find_witness_reason` | becomes Authored |

### src/v2/std/integer.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `integer_named_edge` | 812 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/std/integer_value_set.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `integer_unbounded_value_set_node` | 39 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^integer_value_set_field_kind` | becomes Authored |
| fn `integer_interval_value_set_node` | 52 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^integer_value_set_field_kind` | becomes Authored |
| fn `integer_interval_value_set_node` | 56 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^integer_value_set_field_interval` | becomes Authored |

### src/v2/std/language_model.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `language_model_named_edge` | 30 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/std/logic.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `bool_named_edge` | 62 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/std/model_core.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `model_core_law_node` | 192 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^model_core_law_field_subject` | becomes Authored |
| fn `model_core_law_node` | 196 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^model_core_law_field_algebra` | becomes Authored |
| fn `model_core_law_node` | 200 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^model_core_law_field_kind` | becomes Authored |

### src/v2/std/node.dag (26)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `is_named` | 303 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `is_positional` | 310 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `name_occurrences` | 340 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `positional_child_count` | 349 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `loop_body_edges_conform` | 366 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `loop_bound_edge` | becomes Structural arm match |
| data `arrow_signature_order_label` | 388 | CONSTRUCT | STRUCTURAL | `^arrow_signature_order_edge` | becomes Structural arm arrow_signature_order_edge |
| fn `arrow_signature_edges_conform` | 394 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `arrow_body_edge` | becomes Structural arm match |
| fn `arrow_signature_edges_conform` | 403 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `arrow_body_edge` | becomes Structural arm match |
| fn `arrow_domain_binder_labels` | 537 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `arrow_first_positional_target` | 559 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `type_binder_conforms` | 575 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `id` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `named_edge_target_lookup` | 601 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `arrow_body_record_construct_conforms` | 673 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `loop_edge_contributes_to_iteration_fold` | 724 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `loop_edge_contributes_to_iteration_fold` | 729 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `edge_contributes_to_cost_fold` | 745 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `arrow_body_edge` | becomes Structural arm match |
| fn `edge_contributes_to_cost_fold` | 750 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `arrow_body_edge` | becomes Structural arm match |
| fn `edge_contributes_to_cost_fold` | 755 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `all_names_distinct` | 762 | MATCH-binds-name(read as name) | AUTHORED? | `n` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `match_arm_children_conform_skip_first` | 827 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `label_sort_key` | 900 | CANONICALIZE-ORDER | STRUCTURAL+AUTHORED | `sym` | hash-canonical change: define total order across Structural/Authored/Positional (sort key must discriminate arms) |
| fn `labeled_is_positional` | 912 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `labeled_named_is` | 918 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `wanted` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `labeled_named_is_not` | 925 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `canonical_hash_of_edge_label` | 1090 | HASH | STRUCTURAL+AUTHORED | `sym` | hash-canonical change: add distinct tag per arm (canonical_tag_structural_edge / canonical_tag_authored_edge) or keep ^canonical_tag_named_edge for Authored only |
| fn `node_offset_named_edge` | 1663 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/std/node_query.dag (15)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `declared_field_from_edge` | 215 | MATCH-binds-name(read as name) | AUTHORED? | `field_name` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `construct_tag_edge` | 271 | CONSTRUCT | AUTHORED | `tag` | becomes Authored |
| fn `construct_tag_optional` | 299 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `id` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `field_projection_node` | 371 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `field_projection_base_name()` | becomes Structural arm field_projection_base_name() |
| fn `field_projection_node` | 372 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `field_projection_field_name()` | becomes Structural arm field_projection_field_name() |
| fn `field_projection_edge_base_optional` | 385 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `field_projection_base_name` | becomes Structural arm match |
| fn `field_projection_edge_field_optional` | 397 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `field_projection_field_name` | becomes Structural arm match |
| fn `variant_declaration_node` | 449 | CONSTRUCT(pass-through) | PASS-THROUGH | `label` | arm-only rename (carry EdgeLabel whole instead of re-wrapping a Symbol) |
| fn `variant_declaration_edge_optional` | 463 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `node_positional_child_targets` | 569 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `node_labeled_child_edges` | 577 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `node_envelope_without_named_coordinate_go` | 594 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `coordinate` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `named_child_lookup` | 643 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `conj_ordered_named_param_binding_ids` | 719 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `repeated_name_edges` | 787 | MATCH-binds-name(read as name) | AUTHORED? | `n` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/std/node_shape.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `named_edge_shape` | 30 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/std/project_to_core_predicate.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `project_to_core_step` | 26 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `project_to_core_axis_is_surface` | becomes Authored |
| fn `project_to_core_has_core_fact_step` | 66 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `project_to_core_axis_is_surface` | becomes Authored |

### src/v2/std/qualified_name.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `qn_spine_role` | 87 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `fold_list_node_head` | becomes Structural arm match |
| fn `declaration_reference_node` | 341 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `declaration_reference_marker()` | becomes Structural arm declaration_reference_marker() |
| fn `declaration_reference_spine_optional` | 359 | MATCH-binds-name(read as name) | AUTHORED? | `label` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/std/refinement_widening_predicate.dag (5)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `refinement_widening_predicate_algebra_for_value_sets` | 36 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^refinement_widening_field_source` | becomes Authored |
| fn `refinement_widening_predicate_algebra_for_value_sets` | 40 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^refinement_widening_field_candidate` | becomes Authored |
| fn `refinement_widening_predicate_algebra_for_value_sets` | 44 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^refinement_widening_field_source_facts` | becomes Authored |
| fn `refinement_widening_predicate_algebra_for_value_sets` | 48 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^refinement_widening_field_source_value_set` | becomes Authored |
| fn `refinement_widening_predicate_algebra_for_value_sets` | 52 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^refinement_widening_field_candidate_value_set` | becomes Authored |

### src/v2/std/runtime.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `runtime_value_named_edge` | 125 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/std/type_binder.dag (16)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `edge_is_type_alias` | 54 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `type_alias_marker` | becomes Structural arm match |
| fn `edge_is_type_params` | 61 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `type_params_marker` | becomes Structural arm match |
| fn `edge_is_type_body` | 68 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `type_body_marker` | becomes Structural arm match |
| fn `type_params_edge` | 75 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `type_params_marker()` | becomes Structural arm type_params_marker() |
| fn `type_decl_wrapper` | 89 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `type_params_marker()` | becomes Structural arm type_params_marker() |
| fn `type_decl_wrapper` | 90 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `type_body_marker()` | becomes Structural arm type_body_marker() |
| fn `type_alias_wrapper` | 121 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `type_params_marker()` | becomes Structural arm type_params_marker() |
| fn `type_alias_wrapper` | 122 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `type_alias_marker()` | becomes Structural arm type_alias_marker() |
| fn `type_opaque_wrapper` | 134 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `type_params_marker()` | becomes Structural arm type_params_marker() |
| fn `binder_conj_names` | 208 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `cast_target_edge` | 225 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `cast_target_marker()` | becomes Structural arm cast_target_marker() |
| fn `edge_is_cast_target` | 230 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `cast_target_marker` | becomes Structural arm match |
| fn `type_annotation_edge` | 259 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `type_annotation_marker()` | becomes Structural arm type_annotation_marker() |
| fn `edge_is_type_annotation` | 270 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `type_annotation_marker` | becomes Structural arm match |
| fn `computation_named_labels_conform` | 306 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `marker` | becomes Structural arm match |
| fn `arrow_named_labels_conform` | 354 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `arrow_body_edge` | becomes Structural arm match |

### src/v2/test/algebra_laws/nat_semiring.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `algebra_law_subject` | 45 | CONSTRUCT | TEST-FIXTURE | `^algebra_law_subject_algebra_field` | becomes Authored |
| fn `algebra_law_subject` | 46 | CONSTRUCT | TEST-FIXTURE | `^algebra_law_subject_inhabitant_field` | becomes Authored |
| fn `algebra_law_subject` | 47 | CONSTRUCT | TEST-FIXTURE | `^algebra_law_subject_law_field` | becomes Authored |

### src/v2/test/claim/anonymous_param_binder_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `apb_domain_labels` | 107 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/test/claim/binding/selection_fold_two_implementations_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `binding_inhabitance_variant_edge` | 231 | CONSTRUCT | TEST-FIXTURE(var) | `type_sym` | becomes Authored (fixture) unless the var carries a marker |
| fn `binding_inhabitance_variant_edge` | 237 | CONSTRUCT | TEST-FIXTURE(var) | `variant_sym` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/body_cast_node_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| test `fn` | 510 | CONSTRUCT | STRUCTURAL | `^bcn_not_a_marker` | becomes Structural arm bcn_not_a_marker |

### src/v2/test/claim/body_lowering/arrow_body_form_witness_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `arrow_body_form_fixture_arrow_with_body` | 82 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `arrow_body_form_id_arrow_fixture` | 93 | CONSTRUCT | STRUCTURAL | `^dag_binding_param_x` | becomes Structural arm dag_binding_param_x |
| test `fn` | 173 | CONSTRUCT | STRUCTURAL | `^arrow_body_form_record_field` | becomes Structural arm arrow_body_form_record_field |

### src/v2/test/claim/body_lowering/atom_body_discriminator_helpers.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `atom_body_discriminator_walk_spine_to_body` | 143 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `atom_body_discriminator_top_level_arrow_lexeme_optional` | 167 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `atom_body_discriminator_top_level_named_arrow` | 198 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `atom_body_discriminator_named_binding_count` | 300 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |

### src/v2/test/claim/body_lowering/statement_let_bind_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `positional_target_at` | 136 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |

### src/v2/test/claim/body_lowering/wave1_gate1_b1_bare_call_helpers.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `wave1_gate1_b1_arrow_named_body_optional` | 49 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `wave1_gate1_b1_find_named_fn_arrow_body` | 64 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/test/claim/body_lowering_qualified_path_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `dotted_fixture_seq` | 111 | CONSTRUCT | STRUCTURAL | `^grammar_sequence_left_node_projection` | becomes Structural arm grammar_sequence_left_node_projection |
| fn `dotted_fixture_seq` | 112 | CONSTRUCT | STRUCTURAL | `^grammar_sequence_right_node_projection` | becomes Structural arm grammar_sequence_right_node_projection |
| fn `dotted_fixture_shell` | 180 | CONSTRUCT | STRUCTURAL | `^grammar_production_captured_node_projection` | becomes Structural arm grammar_production_captured_node_projection |

### src/v2/test/claim/body_lowering_well_formed_wall_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `well_formed_wall_malformed_arrow` | 39 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `well_formed_wall_atom_body_arrow` | 54 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `well_formed_wall_valid_arrow` | 69 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |

### src/v2/test/claim/boundary/english_ingest_fail_closed.dag (6)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `arbitrary_english_prose_input` | 13 | CONSTRUCT | TEST-FIXTURE | `^english_boundary_field` | becomes Authored |
| data `arbitrary_english_prose_input` | 17 | CONSTRUCT | TEST-FIXTURE | `^english_payload_field` | becomes Authored |
| data `english_shape_b_docs_input` | 28 | CONSTRUCT | TEST-FIXTURE | `^english_typed_value_field` | becomes Authored |
| data `english_shape_b_docs_input` | 32 | CONSTRUCT | TEST-FIXTURE | `^english_walker_field` | becomes Authored |
| data `english_shape_b_docs_output` | 43 | CONSTRUCT | TEST-FIXTURE | `^english_typed_value_field` | becomes Authored |
| data `english_shape_b_docs_output` | 47 | CONSTRUCT | TEST-FIXTURE | `^english_output_field` | becomes Authored |

### src/v2/test/claim/c_compilation_unit_witness_test.dag (7)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `witness_struct_field_by_value` | 90 | CONSTRUCT | TEST-FIXTURE | `^member` | becomes Authored |
| fn `witness_struct_field_by_pointer` | 102 | CONSTRUCT | TEST-FIXTURE | `^next` | becomes Authored |
| fn `witness_struct_field_by_pointer` | 107 | CONSTRUCT | TEST-FIXTURE | `^domain` | becomes Authored |
| fn `witness_struct_field_by_pointer` | 111 | CONSTRUCT | TEST-FIXTURE | `^codomain` | becomes Authored |
| fn `witness_point_struct_conj_body` | 126 | CONSTRUCT | TEST-FIXTURE | `^x` | becomes Authored |
| fn `witness_fn_taking_point_by_value` | 140 | CONSTRUCT | TEST-FIXTURE | `^domain` | becomes Authored |
| fn `witness_fn_taking_point_by_value` | 144 | CONSTRUCT | TEST-FIXTURE | `^codomain` | becomes Authored |

### src/v2/test/claim/claim_pipeline/normalize_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `spine_normalize_service_parse_tree` | 29 | CONSTRUCT | TEST-FIXTURE | `^spine_normalize_service_decl` | becomes Authored |
| fn `spine_normalize_nested_service_parse_tree` | 53 | CONSTRUCT | TEST-FIXTURE | `^spine_normalize_nested_field` | becomes Authored |

### src/v2/test/claim/claim_pipeline/resolve_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `spine_resolve_service_parse_tree` | 23 | CONSTRUCT | TEST-FIXTURE | `^spine_resolve_service_decl` | becomes Authored |
| data `spine_resolve_reject_module_subject` | 50 | CONSTRUCT | TEST-FIXTURE | `^spine_resolve_reject_decl` | becomes Authored |

### src/v2/test/claim/claim_verdict_rejection_cause_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `claim_verdict_cause_ill_formed` | 44 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |

### src/v2/test/claim/compilation_unit_witness_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `witness_point_struct_conj_body` | 77 | CONSTRUCT | TEST-FIXTURE | `^x` | becomes Authored |
| fn `witness_fn_taking_point_by_value` | 91 | CONSTRUCT | TEST-FIXTURE | `^domain` | becomes Authored |
| fn `witness_fn_taking_point_by_value` | 95 | CONSTRUCT | TEST-FIXTURE | `^codomain` | becomes Authored |

### src/v2/test/claim/compiler/infer_application_argument_inhabitance_witness_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `inhabitance_named_conj_domain` | 45 | CONSTRUCT | TEST-FIXTURE(var) | `parameter` | becomes Authored (fixture) unless the var carries a marker |
| fn `inhabitance_nominal_product_type_node` | 287 | CONSTRUCT | TEST-FIXTURE | `^inhabitance_field_a` | becomes Authored |

### src/v2/test/claim/complexity_gate/table_fixture_add_subject_producer.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `table_fixture_add_decl_node` | 30 | CONSTRUCT | TEST-FIXTURE(var) | `fn_name` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/computation_shapes/loop_linear_bound.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `loop_linear_bound_subject` | 50 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| data `loop_linear_bound_subject_rebuilt` | 89 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |

### src/v2/test/claim/declaration_graft_assemble_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `declaration_graft_label_present` | 74 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/test/claim/declared_parameter_order_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `dpo_with_order` | 159 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `arrow_signature_order_edge` | becomes Structural arm match |
| fn `dpo_canonical_domain` | 204 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `dpo_order_atoms_are_bare` | 357 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `arrow_signature_order_edge` | becomes Structural arm match |

### src/v2/test/claim/diagnostic_correction/show_correct_code.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `pair_node` | 13 | CONSTRUCT | TEST-FIXTURE | `^correction_left` | becomes Authored |
| fn `pair_node` | 14 | CONSTRUCT | TEST-FIXTURE | `^correction_right` | becomes Authored |

### src/v2/test/claim/emit/arrow_scope_atom_refuses_unresolved_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `probe_named` | 50 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/emit/c_semantic_decl_routing_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `cdr_field_edge` | 38 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `cdr_variant_edge` | 42 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `cdr_variant_edge_payload` | 47 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/emit/decl_emit_consolidated_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `dc_named_edge` | 71 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/test/claim/emit/fold_call_closure_fixture.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `fold_fixture_named_edge` | 14 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/test/claim/emit/rust_call_emit_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `rust_call_named_edge` | 63 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/test/claim/emit/semantic_decl_decoration_binding_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `bool_coproduct` | 21 | CONSTRUCT | TEST-FIXTURE | `^True` | becomes Authored |
| fn `bool_coproduct` | 22 | CONSTRUCT | TEST-FIXTURE | `^False` | becomes Authored |

### src/v2/test/claim/emit/semantic_decl_emit_substrings_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `sdr_variant_edge` | 18 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/emit/semantic_decl_routing_generic_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `sdrg_field_edge` | 31 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `sdrg_variant_edge` | 39 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/emit/semantic_decl_routing_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `sdr_variant_edge` | 30 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `sdr_field_edge` | 65 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/emit/semantic_decl_rules_parity_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `bool_coproduct` | 26 | CONSTRUCT | TEST-FIXTURE | `^True` | becomes Authored |
| fn `bool_coproduct` | 27 | CONSTRUCT | TEST-FIXTURE | `^False` | becomes Authored |

### src/v2/test/claim/emit/semantic_decl_serialize_parity_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `bool_coproduct` | 47 | CONSTRUCT | TEST-FIXTURE | `^True` | becomes Authored |
| fn `bool_coproduct` | 48 | CONSTRUCT | TEST-FIXTURE | `^False` | becomes Authored |

### src/v2/test/claim/emit/semantic_decl_target_refusal_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `sdtr_variant_edge` | 23 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/emit/std_logic_selfhost_witness_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `sl_named_edge` | 37 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/test/claim/emit/swift_semantic_decl_routing_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `sdr_field_edge` | 45 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `sdr_variant_edge` | 49 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `sdr_variant_edge_payload` | 54 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/execution/branch_eval_by_execution_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `branch_tree_root` | 351 | CONSTRUCT | TEST-FIXTURE | `^branch_actual` | becomes Authored |
| fn `branch_tree_root` | 352 | CONSTRUCT | TEST-FIXTURE | `^branch_expected` | becomes Authored |

### src/v2/test/claim/execution/c_type_decl_emit_host_run_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `ctd_field_edge` | 54 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `ctd_variant_edge` | 58 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/execution/data_decl_lowering_grounding_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `ddl_edge_is_body` | 192 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `arrow_body_edge` | becomes Structural arm match |
| fn `ddl_edge_names_value_member` | 218 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `ddl_value` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| test `fn` | 305 | CONSTRUCT | STRUCTURAL | `^integer_literal_magnitude_field` | becomes Structural arm integer_literal_magnitude_field |

### src/v2/test/claim/execution/emit_ingest_type_decl_round_trip_test.dag (5)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `type_decl_row_with_foreign_emitted_fixture` | 405 | CONSTRUCT | STRUCTURAL | `^grammar_relation_field_production` | becomes Structural arm grammar_relation_field_production |
| fn `type_decl_row_with_foreign_emitted_fixture` | 409 | CONSTRUCT | STRUCTURAL | `^grammar_relation_field_emitted` | becomes Structural arm grammar_relation_field_emitted |
| fn `type_decl_row_with_foreign_emitted_fixture` | 413 | CONSTRUCT | STRUCTURAL | `^grammar_relation_field_tokens` | becomes Structural arm grammar_relation_field_tokens |
| fn `type_decl_translation_rules_with_row` | 431 | CONSTRUCT | STRUCTURAL | `^grammar_relation_rules_formal_productions` | becomes Structural arm grammar_relation_rules_formal_productions |
| fn `type_decl_translation_rules_with_row` | 437 | CONSTRUCT | TEST-FIXTURE | `^type_decl_round_trip_foreign_binding` | becomes Authored |

### src/v2/test/claim/execution/fold_assembly_round_trip_test.dag (9)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `corrupted_fold_form` | 132 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `fold_template_edge` | becomes Structural arm fold_template_edge |
| fn `corrupted_fold_form` | 133 | CONSTRUCT | STRUCTURAL | `^fold_elements` | becomes Structural arm fold_elements |
| fn `missing_template_form` | 146 | CONSTRUCT | STRUCTURAL | `^fold_elements` | becomes Structural arm fold_elements |
| fn `duplicate_template_form` | 155 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `fold_template_edge` | becomes Structural arm fold_template_edge |
| fn `duplicate_template_form` | 156 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `fold_template_edge` | becomes Structural arm fold_template_edge |
| fn `duplicate_template_form` | 157 | CONSTRUCT | STRUCTURAL | `^fold_elements` | becomes Structural arm fold_elements |
| fn `head_without_tail_elements_form` | 166 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `fold_template_edge` | becomes Structural arm fold_template_edge |
| fn `head_without_tail_elements_form` | 167 | CONSTRUCT | STRUCTURAL | `^fold_elements` | becomes Structural arm fold_elements |
| fn `head_without_tail_elements_form` | 168 | CONSTRUCT | STRUCTURAL | `^fold_list_node_head` | becomes Structural arm fold_list_node_head |

### src/v2/test/claim/execution/infer_product_introduction_test.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `params_conj_children_match` | 114 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `x` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `params_conj_children_match` | 116 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `y` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| test `fn` | 245 | CONSTRUCT | TEST-FIXTURE | `^derived_field` | becomes Authored |
| test `fn` | 246 | CONSTRUCT | TEST-FIXTURE | `^frontier_field` | becomes Authored |

### src/v2/test/claim/execution/long/add_arrow_eval_by_execution_test.dag (6)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `add_arrow_eval_tree_root` | 392 | CONSTRUCT | TEST-FIXTURE | `^add_arrow_eval_actual` | becomes Authored |
| fn `add_arrow_eval_tree_root` | 393 | CONSTRUCT | TEST-FIXTURE | `^add_arrow_eval_expected` | becomes Authored |
| fn `add_arrow_swapped_add_arrow` | 531 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `add_arrow_excess_signature_arrow` | 545 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `add_arrow_excess_body_arrow` | 570 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `add_arrow_eval_lazy_arm_loop_node` | 830 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |

### src/v2/test/claim/execution/long/bind_eval_by_execution_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `bind_tree_root` | 279 | CONSTRUCT | TEST-FIXTURE | `^bind_actual` | becomes Authored |
| fn `bind_tree_root` | 280 | CONSTRUCT | TEST-FIXTURE | `^bind_expected` | becomes Authored |

### src/v2/test/claim/execution/long/loop_eval_by_execution_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `loop_tree_root` | 291 | CONSTRUCT | STRUCTURAL | `^loop_actual` | becomes Structural arm loop_actual |
| fn `loop_tree_root` | 292 | CONSTRUCT | STRUCTURAL | `^loop_expected` | becomes Structural arm loop_expected |

### src/v2/test/claim/execution/long/pick_ingested_structural_lowering_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `pick_eval_tree_root` | 317 | CONSTRUCT | TEST-FIXTURE | `^pick_eval_actual` | becomes Authored |
| fn `pick_eval_tree_root` | 318 | CONSTRUCT | TEST-FIXTURE | `^pick_eval_expected` | becomes Authored |

### src/v2/test/claim/execution/typescript_import_emit_by_execution_test.dag (5)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `ts_import_carrier_node` | 42 | CONSTRUCT | STRUCTURAL | `^dag_surface_import_decl` | becomes Structural arm dag_surface_import_decl |
| fn `ts_import_carrier_node` | 46 | CONSTRUCT | STRUCTURAL | `^dag_surface_import_decl_qualified_name` | becomes Structural arm dag_surface_import_decl_qualified_name |
| fn `ts_import_carrier_node` | 50 | CONSTRUCT | STRUCTURAL | `^dag_surface_import_decl_block` | becomes Structural arm dag_surface_import_decl_block |
| fn `ts_import_missing_qn_carrier_node` | 63 | CONSTRUCT | STRUCTURAL | `^dag_surface_import_decl` | becomes Structural arm dag_surface_import_decl |
| fn `ts_import_missing_qn_carrier_node` | 67 | CONSTRUCT | STRUCTURAL | `^dag_surface_import_decl_block` | becomes Structural arm dag_surface_import_decl_block |

### src/v2/test/claim/fold_lowering_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `with_duplicate_carrier_edge` | 186 | CONSTRUCT | STRUCTURAL | `^loop_carrier_edge` | becomes Structural arm loop_carrier_edge |

### src/v2/test/claim/generated/algebra_law_conformance.dag (6)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `algebra_law_generated_nat_value` | 42 | CONSTRUCT | TEST-FIXTURE | `^algebra_law_generated_operation_field` | becomes Authored |
| fn `algebra_law_generated_nat_value` | 43 | CONSTRUCT | TEST-FIXTURE | `^algebra_law_generated_nat_prev_field` | becomes Authored |
| fn `algebra_law_generated_binary` | 54 | CONSTRUCT | TEST-FIXTURE | `^algebra_law_generated_operation_field` | becomes Authored |
| fn `algebra_law_generated_binary` | 55 | CONSTRUCT | TEST-FIXTURE | `^algebra_law_generated_left_field` | becomes Authored |
| fn `algebra_law_generated_binary` | 56 | CONSTRUCT | TEST-FIXTURE | `^algebra_law_generated_right_field` | becomes Authored |
| fn `algebra_law_generated_binary` | 57 | CONSTRUCT | TEST-FIXTURE | `^algebra_law_generated_result_field` | becomes Authored |

### src/v2/test/claim/grounding_lens_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `field_edge` | 31 | CONSTRUCT | TEST-FIXTURE(var) | `field` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/impossible_bug/idempotency_contract.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `idempotency_contract` | 38 | CONSTRUCT | TEST-FIXTURE | `^idempotency_position_field` | becomes Authored |
| fn `idempotency_contract` | 39 | CONSTRUCT | TEST-FIXTURE | `^idempotency_body_field` | becomes Authored |

### src/v2/test/claim/impossible_bug/suboptimal_complexity.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `complexity_contract` | 39 | CONSTRUCT | TEST-FIXTURE | `^complexity_bound_field` | becomes Authored |
| fn `complexity_contract` | 40 | CONSTRUCT | TEST-FIXTURE | `^complexity_body_field` | becomes Authored |

### src/v2/test/claim/impossible_bug/transport_type_drift.dag (6)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `shared_wire_contract_input` | 12 | CONSTRUCT | TEST-FIXTURE | `^client_field` | becomes Authored |
| data `shared_wire_contract_input` | 13 | CONSTRUCT | TEST-FIXTURE | `^server_field` | becomes Authored |
| data `falsification_wire_contract_drift` | 21 | CONSTRUCT | TEST-FIXTURE | `^client_field` | becomes Authored |
| data `falsification_wire_contract_drift` | 22 | CONSTRUCT | TEST-FIXTURE | `^server_field` | becomes Authored |
| data `falsification_target_contract_drift` | 30 | CONSTRUCT | TEST-FIXTURE | `^client_field` | becomes Authored |
| data `falsification_target_contract_drift` | 31 | CONSTRUCT | TEST-FIXTURE | `^target_field` | becomes Authored |

### src/v2/test/claim/impossible_bug/unenumerated_effects.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `effect_derived_from_signature_input` | 49 | CONSTRUCT | TEST-FIXTURE | `^effect_read_subject_field` | becomes Authored |
| data `effect_derived_from_signature_input` | 53 | CONSTRUCT | TEST-FIXTURE | `^effect_write_subject_field` | becomes Authored |
| data `falsification_redundant_read` | 65 | CONSTRUCT | TEST-FIXTURE | `^effect_signature_field` | becomes Authored |
| data `falsification_redundant_read` | 66 | CONSTRUCT | TEST-FIXTURE | `^effect_body_field` | becomes Authored |

### src/v2/test/claim/impossible_bug/unhandled_diagnostic_paths.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `safety_subject` | 38 | CONSTRUCT | TEST-FIXTURE | `^operation_field` | becomes Authored |
| fn `safety_subject` | 39 | CONSTRUCT | TEST-FIXTURE | `^witness_field` | becomes Authored |

### src/v2/test/claim/infer_self_grounding_wall_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `wall_malformed_bool_with_payload` | 128 | CONSTRUCT | TEST-FIXTURE | `^wall_malformed_bool_payload_field` | becomes Authored |
| fn `wall_malformed_int_with_extra_payload` | 143 | CONSTRUCT | TEST-FIXTURE | `^wall_malformed_int_payload_field` | becomes Authored |

### src/v2/test/claim/integer/value_set_decode_reason_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `named_edge` | 27 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/test/claim/long/accumulator_copy_fold_analysis_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `edge_is_seq_left` | 259 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `grammar_sequence_left_node_projection` | becomes Structural arm match |

### src/v2/test/claim/long/door_real_module_probe_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `quad_module_root` | 49 | CONSTRUCT | TEST-FIXTURE(var) | `export_sym` | becomes Authored (fixture) unless the var carries a marker |
| fn `quad_subject_with_import_use` | 74 | CONSTRUCT | TEST-FIXTURE | `^list_append` | becomes Authored |

### src/v2/test/claim/loop_infer_iteration_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `loop_infer_two_stage_loop_node` | 30 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| fn `loop_infer_one_stage_loop_node` | 41 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |

### src/v2/test/claim/loop_multiplicity_measure_test.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `lmm_loop_with_measure` | 19 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| fn `lmm_conj_measure` | 39 | CONSTRUCT | TEST-FIXTURE | `^lmm_dim_left` | becomes Authored |
| fn `lmm_conj_measure` | 40 | CONSTRUCT | TEST-FIXTURE | `^lmm_dim_right` | becomes Authored |
| test `fn` | 179 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |

### src/v2/test/claim/manual/add_body_emit_typescript_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `add_body_ts_named_edge` | 109 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/test/claim/manual/add_body_producer_claim_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `add_body_arrow_with_atom_body_rejects` | 101 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |

### src/v2/test/claim/manual/add_body_value_expression_fold_typescript_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `add_body_fold_named_edge` | 73 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/test/claim/manual/bind_demand_driven_eval_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `bind_eval_arrow_with_body` | 331 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `bind_eval_tree_root` | 341 | CONSTRUCT | TEST-FIXTURE | `^bind_eval_actual` | becomes Authored |
| fn `bind_eval_tree_root` | 342 | CONSTRUCT | TEST-FIXTURE | `^bind_eval_expected` | becomes Authored |

### src/v2/test/claim/manual/body_lowering_normalize_add_test.dag (5)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `body_lowering_domain_named_count` | 438 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `body_lowering_normalized_operands_swapped` | 495 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `right` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `body_lowering_normalized_operands_swapped` | 497 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `left` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| test `fn` | 552 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `operand` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `body_lower_arrow_domain_first_binding` | 579 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/test/claim/manual/branch_lazy_arm_eval_acceptance_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `lazy_arm_arrow_with_body` | 482 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `lazy_arm_tree_root` | 492 | CONSTRUCT | TEST-FIXTURE | `^lazy_arm_actual` | becomes Authored |
| fn `lazy_arm_tree_root` | 493 | CONSTRUCT | TEST-FIXTURE | `^lazy_arm_expected` | becomes Authored |

### src/v2/test/claim/manual/content_hash_loop_bound_edge_order_test.dag (6)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `content_hash_loop_bound_last` | 60 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| fn `content_hash_loop_bound_first` | 73 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| fn `content_hash_loop_bound_middle` | 88 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| fn `content_hash_loop_bound_last_two_body` | 104 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| fn `content_hash_loop_with_extra_named` | 118 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| fn `content_hash_loop_with_extra_named` | 122 | CONSTRUCT | TEST-FIXTURE | `^content_hash_loop_extra_named_sym` | becomes Authored |

### src/v2/test/claim/manual/content_hash_named_edge_order_test.dag (6)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `content_hash_order_conj_ab` | 27 | CONSTRUCT | TEST-FIXTURE | `^content_hash_order_field_a` | becomes Authored |
| fn `content_hash_order_conj_ab` | 31 | CONSTRUCT | TEST-FIXTURE | `^content_hash_order_field_b` | becomes Authored |
| fn `content_hash_order_conj_ba` | 44 | CONSTRUCT | TEST-FIXTURE | `^content_hash_order_field_b` | becomes Authored |
| fn `content_hash_order_conj_ba` | 48 | CONSTRUCT | TEST-FIXTURE | `^content_hash_order_field_a` | becomes Authored |
| fn `content_hash_nested_conj_ab` | 66 | CONSTRUCT | TEST-FIXTURE | `^content_hash_order_field_a` | becomes Authored |
| fn `content_hash_nested_conj_ba` | 79 | CONSTRUCT | TEST-FIXTURE | `^content_hash_order_field_a` | becomes Authored |

### src/v2/test/claim/manual/dependency_recompute_plan.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `refinement_after_node` | 39 | CONSTRUCT | TEST-FIXTURE | `^refinement_new_field_symbol` | becomes Authored |
| data `dependency_projection_subgraph` | 68 | CONSTRUCT | TEST-FIXTURE | `^testclaim_lens_symbol` | becomes Authored |
| data `dependency_projection_subgraph` | 72 | CONSTRUCT | TEST-FIXTURE | `^boundary_test_lens_symbol` | becomes Authored |
| data `dependency_projection_subgraph` | 76 | CONSTRUCT | TEST-FIXTURE | `^target_test_projection_symbol` | becomes Authored |

### src/v2/test/claim/manual/dissolution_subsumption_reverification.dag (8)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `subsumption_receipt_node` | 157 | CONSTRUCT | STRUCTURAL | `^subsumption_root_fix_edge` | becomes Structural arm subsumption_root_fix_edge |
| fn `subsumption_receipt_node` | 161 | CONSTRUCT | STRUCTURAL | `^subsumption_first_leaf_edge` | becomes Structural arm subsumption_first_leaf_edge |
| fn `subsumption_receipt_node` | 165 | CONSTRUCT | STRUCTURAL | `^subsumption_second_leaf_edge` | becomes Structural arm subsumption_second_leaf_edge |
| fn `subsumption_receipt_node` | 169 | CONSTRUCT | STRUCTURAL | `^subsumption_mechanical_claim_edge` | becomes Structural arm subsumption_mechanical_claim_edge |
| fn `expected_rust_language_model_emit_receipt_node` | 185 | CONSTRUCT | STRUCTURAL | `^subsumption_root_fix_edge` | becomes Structural arm subsumption_root_fix_edge |
| fn `expected_rust_language_model_emit_receipt_node` | 189 | CONSTRUCT | STRUCTURAL | `^subsumption_first_leaf_edge` | becomes Structural arm subsumption_first_leaf_edge |
| fn `expected_rust_language_model_emit_receipt_node` | 193 | CONSTRUCT | STRUCTURAL | `^subsumption_second_leaf_edge` | becomes Structural arm subsumption_second_leaf_edge |
| fn `expected_rust_language_model_emit_receipt_node` | 197 | CONSTRUCT | STRUCTURAL | `^subsumption_mechanical_claim_edge` | becomes Structural arm subsumption_mechanical_claim_edge |

### src/v2/test/claim/manual/emit_ingest_same_language_row_fold_test.dag (5)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `emit_ingest_fold_row_with_emitted` | 15 | CONSTRUCT | STRUCTURAL | `^grammar_relation_field_emitted` | becomes Structural arm grammar_relation_field_emitted |
| fn `emit_ingest_fold_row_with_emitted` | 19 | CONSTRUCT | STRUCTURAL | `^grammar_relation_field_tokens` | becomes Structural arm grammar_relation_field_tokens |
| fn `emit_ingest_fold_rules_table` | 34 | CONSTRUCT | TEST-FIXTURE | `^emit_ingest_fold_row_label_dag` | becomes Authored |
| fn `emit_ingest_fold_rules_table` | 35 | CONSTRUCT | TEST-FIXTURE | `^emit_ingest_fold_row_label_python` | becomes Authored |
| fn `emit_ingest_fold_rules_all_rows_bijectivity_holds` | 100 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |

### src/v2/test/claim/manual/english_emit_add_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `english_emit_add_identity_edge_matches` | 223 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `grammar_production_identity_node_projection` | becomes Structural arm match |

### src/v2/test/claim/manual/fact_density_anchor.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `fact_density_anchor_fact_bundle` | 12 | CONSTRUCT | TEST-FIXTURE | `^fact_density_anchor_width_field` | becomes Authored |
| data `fact_density_anchor_fact_bundle` | 16 | CONSTRUCT | TEST-FIXTURE | `^fact_density_anchor_signedness_field` | becomes Authored |

### src/v2/test/claim/manual/find_witness_project_to_core_controls.dag (15)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `surface_only_node` | 87 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisSurfaceSpelling {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisSurfaceSpelling {}) |
| fn `surface_wrapped_core_node` | 110 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisSurfaceSpelling {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisSurfaceSpelling {}) |
| fn `surface_wrapped_core_node` | 115 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: v2.std.model_core.ModelCoreFactAxisEncoding {})` | becomes Structural arm discriminant(v: v2.std.model_core.ModelCoreFactAxisEncoding {}) |
| fn `surface_wrapped_core_node` | 119 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisSurfaceSpelling {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisSurfaceSpelling {}) |
| fn `nested_project_to_core_node` | 149 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisEncoding {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisEncoding {}) |
| fn `nested_project_to_core_node` | 154 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisEncoding {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisEncoding {}) |
| fn `nested_project_to_core_node` | 158 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisSurfaceSpelling {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisSurfaceSpelling {}) |
| data `witness_project_to_core_rejects_different_connective_same_core_children` | 202 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisEncoding {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisEncoding {}) |
| data `witness_project_to_core_rejects_different_connective_same_core_children` | 214 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisEncoding {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisEncoding {}) |
| data `witness_project_to_core_rejects_different_connective_same_core_children` | 225 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisEncoding {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisEncoding {}) |
| data `witness_project_to_core_rejects_different_connective_same_core_children` | 236 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisEncoding {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisEncoding {}) |
| fn `i32_value_set_interval_first` | 257 | CONSTRUCT | TEST-FIXTURE | `^integer_value_set_field_interval` | becomes Authored |
| fn `i32_value_set_interval_first` | 261 | CONSTRUCT | TEST-FIXTURE | `^integer_value_set_field_kind` | becomes Authored |
| fn `unbounded_value_set_with_extra_field` | 306 | CONSTRUCT | TEST-FIXTURE | `^integer_value_set_field_kind` | becomes Authored |
| fn `unbounded_value_set_with_extra_field` | 314 | CONSTRUCT | TEST-FIXTURE | `^integer_value_set_field_extra` | becomes Authored |

### src/v2/test/claim/manual/fold_call_closure_emit_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `fold_call_named_edge` | 63 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/test/claim/manual/fold_node_topdown.dag (5)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `topdown_one_child_tree` | 38 | CONSTRUCT | TEST-FIXTURE | `^topdown_child_a` | becomes Authored |
| fn `topdown_two_level_chain` | 48 | CONSTRUCT | TEST-FIXTURE | `^topdown_chain_child` | becomes Authored |
| fn `depth_edge_dispatch_algebra` | 71 | MATCH-on-name-text(bound→compare) | TEST-FIXTURE | `topdown_right_child` | becomes Authored |
| fn `topdown_edge_dispatch_root` | 84 | CONSTRUCT | TEST-FIXTURE | `^topdown_left_child` | becomes Authored |
| fn `topdown_edge_dispatch_root` | 85 | CONSTRUCT | TEST-FIXTURE | `^topdown_right_child` | becomes Authored |

### src/v2/test/claim/manual/infer_bounded_lattice_completeness_anchor_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `anchor_named_edge` | 41 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/test/claim/manual/infer_ground_add.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `fixture_add_resolved_tree` | 335 | CONSTRUCT | TEST-FIXTURE | `^fixture_add_name` | becomes Authored |

### src/v2/test/claim/manual/ingest_bridge_test.dag (10)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `ingest_fixture_row_node` | 119 | CONSTRUCT | STRUCTURAL | `^grammar_relation_field_emitted` | becomes Structural arm grammar_relation_field_emitted |
| fn `ingest_fixture_row_node` | 121 | CONSTRUCT | STRUCTURAL | `^grammar_relation_field_tokens` | becomes Structural arm grammar_relation_field_tokens |
| fn `ingest_fixture_bundle_with_rules` | 136 | CONSTRUCT | STRUCTURAL | `^target_model_edge_selection_policy` | becomes Structural arm target_model_edge_selection_policy |
| fn `ingest_fixture_bundle_with_rules` | 137 | CONSTRUCT | STRUCTURAL | `^target_model_edge_declared_inhabitants` | becomes Structural arm target_model_edge_declared_inhabitants |
| fn `ingest_fixture_bundle_with_rules` | 138 | CONSTRUCT | STRUCTURAL | `^target_model_edge_translation_rules` | becomes Structural arm target_model_edge_translation_rules |
| fn `ingest_fixture_bundle_with_rules` | 139 | CONSTRUCT | STRUCTURAL | `^target_model_edge_serialize_source` | becomes Structural arm target_model_edge_serialize_source |
| fn `ingest_fixture_bundle_with_rules` | 140 | CONSTRUCT | STRUCTURAL | `^target_model_edge_fidelity_quotient` | becomes Structural arm target_model_edge_fidelity_quotient |
| fn `ingest_fixture_model` | 162 | CONSTRUCT | TEST-FIXTURE | `^ingest_fixture_rule` | becomes Authored |
| fn `ingest_fixture_model_ambiguous` | 173 | CONSTRUCT | TEST-FIXTURE | `^ingest_fixture_rule` | becomes Authored |
| fn `ingest_fixture_model_ambiguous` | 174 | CONSTRUCT | TEST-FIXTURE | `^ingest_fixture_rule_alt` | becomes Authored |

### src/v2/test/claim/manual/loop_demand_driven_eval_test.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `loop_eval_loop_node` | 272 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| fn `loop_eval_arrow_with_body` | 300 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `loop_eval_tree_root` | 310 | CONSTRUCT | STRUCTURAL | `^loop_eval_actual` | becomes Structural arm loop_eval_actual |
| fn `loop_eval_tree_root` | 311 | CONSTRUCT | STRUCTURAL | `^loop_eval_expected` | becomes Structural arm loop_eval_expected |

### src/v2/test/claim/manual/match_infer_fail_open_audit_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `classical_not_int_match_arrow_fixture` | 166 | CONSTRUCT | STRUCTURAL | `^dag_binding_param_x` | becomes Structural arm dag_binding_param_x |
| fn `classical_not_int_match_non_octet_arm_fixture` | 211 | CONSTRUCT | STRUCTURAL | `^dag_binding_param_x` | becomes Structural arm dag_binding_param_x |

### src/v2/test/claim/manual/program_assembly_multi_file_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `pa_module_root` | 80 | CONSTRUCT | TEST-FIXTURE(var) | `export_sym` | becomes Authored (fixture) unless the var carries a marker |
| fn `pa_subject_with_import_use` | 105 | CONSTRUCT | TEST-FIXTURE | `^pa_import_use_sym` | becomes Authored |

### src/v2/test/claim/manual/resolve_compile_anchor_test.dag (5)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `anchor_resolve_multi_edge_conj_via_canonical_symbols` | 86 | CONSTRUCT | TEST-FIXTURE | `^anchor_decl_first` | becomes Authored |
| fn `anchor_resolve_multi_edge_conj_via_canonical_symbols` | 96 | CONSTRUCT | TEST-FIXTURE | `^anchor_decl_second` | becomes Authored |
| fn `anchor_resolve_bind_binder_use` | 122 | CONSTRUCT | TEST-FIXTURE | `^anchor_decl_first` | becomes Authored |
| fn `resolve_imported_module_root` | 196 | CONSTRUCT | TEST-FIXTURE | `^anchor_imported_decl` | becomes Authored |
| fn `resolve_subject_module_root_with_import_use` | 253 | CONSTRUCT | TEST-FIXTURE | `^anchor_subject_import_use` | becomes Authored |

### src/v2/test/claim/manual/scheduler_runnable_frontier_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `scheduler_fixture_graph` | 34 | CONSTRUCT | TEST-FIXTURE | `^scheduler_build_symbol` | becomes Authored |
| data `scheduler_fixture_graph` | 35 | CONSTRUCT | TEST-FIXTURE | `^scheduler_test_symbol` | becomes Authored |
| data `scheduler_fixture_graph` | 36 | CONSTRUCT | TEST-FIXTURE | `^scheduler_emit_symbol` | becomes Authored |

### src/v2/test/claim/manual/target_carriers_ambiguous_quotient_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `fidelity_quotient_edge` | 11 | CONSTRUCT | STRUCTURAL | `^target_model_edge_fidelity_quotient` | becomes Structural arm target_model_edge_fidelity_quotient |

### src/v2/test/claim/manual/target_model_operator_lookup_dissolution_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `operator_catalog_with_malformed_and_valid_rows` | 26 | CONSTRUCT | STRUCTURAL | `^target_operator_realization_row_edge` | becomes Structural arm target_operator_realization_row_edge |
| fn `operator_catalog_with_malformed_and_valid_rows` | 30 | CONSTRUCT | STRUCTURAL | `^target_operator_realization_row_edge` | becomes Structural arm target_operator_realization_row_edge |

### src/v2/test/claim/materialize/materialize_witness_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `conj2` | 34 | CONSTRUCT | TEST-FIXTURE | `^left` | becomes Authored |
| fn `conj2` | 35 | CONSTRUCT | TEST-FIXTURE | `^right` | becomes Authored |

### src/v2/test/claim/name_resolve/admission_fail_closed_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `fc_module_root_export` | 89 | CONSTRUCT | TEST-FIXTURE(var) | `export_sym` | becomes Authored (fixture) unless the var carries a marker |
| fn `fc_dual_import_subject_root` | 298 | CONSTRUCT | TEST-FIXTURE(var) | `use_sym` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/name_resolve/one_member_cost_probe_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `probe_one_member_root_export` | 57 | CONSTRUCT | TEST-FIXTURE | `^probe_export_sym` | becomes Authored |

### src/v2/test/claim/name_resolve/test_code_reference_wall_test.dag (10)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `wall_member` | 43 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `wall_member` | 50 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `wall_member_returning_its_param` | 60 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `wall_member_returning_its_param` | 66 | CONSTRUCT | TEST-FIXTURE(var) | `param` | becomes Authored (fixture) unless the var carries a marker |
| fn `wall_member_returning_its_param` | 70 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `wall_module_body` | 83 | CONSTRUCT | STRUCTURAL | `^grammar_production_identity_node_projection` | becomes Structural arm grammar_production_identity_node_projection |
| fn `wall_module_root` | 90 | CONSTRUCT | STRUCTURAL | `^grammar_production_identity_node_projection` | becomes Structural arm grammar_production_identity_node_projection |
| fn `wall_module_root` | 92 | CONSTRUCT | STRUCTURAL | `^grammar_production_captured_node_projection` | becomes Structural arm grammar_production_captured_node_projection |
| fn `wall_module_root` | 95 | CONSTRUCT | TEST-FIXTURE | `^wall` | becomes Authored |
| fn `wall_module_root` | 96 | CONSTRUCT | TEST-FIXTURE(var) | `leaf_segment` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/name_resolve_cross_tree_resolution_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `ct_module_root` | 96 | CONSTRUCT | TEST-FIXTURE(var) | `export_sym` | becomes Authored (fixture) unless the var carries a marker |
| fn `ct_subject_with_import_use` | 125 | CONSTRUCT | TEST-FIXTURE(var) | `import_use_sym` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/namespace_graft/collapsed_reader_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `cr_export_edge` | 55 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/namespace_graft/graft_shape_test.dag (7)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `ng_export_edge` | 79 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `namespace_graft_walk_spine_to_body` | 165 | MATCH-arm-only | - | `n/a` | arm-only rename (match both Structural and Authored, or the specific arm the reader means) |
| fn `namespace_graft_grafted_tree_has_no_metadata_edges` | 188 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `namespace_graft_is_metadata_edge` | becomes Structural arm match |
| data `ng_dual_position_root` | 238 | CONSTRUCT | TEST-FIXTURE | `^NgAlpha` | becomes Authored |
| fn `ng_projection_edge` | 281 | CONSTRUCT | STRUCTURAL | `^grammar_production_captured_node_projection` | becomes Structural arm grammar_production_captured_node_projection |
| fn `ng_edge_carries_type_decl_identity` | 330 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `grammar_production_identity_node_projection` | becomes Structural arm match |
| fn `ng_conj_member_edge` | 383 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/namespace_graft/marker_strip_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `ms_pre_graft_root` | 87 | CONSTRUCT | TEST-FIXTURE | `^MsPilotType` | becomes Authored |
| data `ms_pre_graft_root` | 92 | CONSTRUCT | TEST-FIXTURE | `^MsYes` | becomes Authored |
| data `ms_pre_graft_root` | 96 | CONSTRUCT | TEST-FIXTURE | `^MsNo` | becomes Authored |

### src/v2/test/claim/namespace_graft/normalize_graft_witness_helpers.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `ng_norm_grafted_tree_has_no_metadata_edges` | 80 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `namespace_graft_is_metadata_edge` | becomes Structural arm match |

### src/v2/test/claim/namespace_xl0/field_type_tree_presence_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `fixture_edge_label_lexemes` | 90 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/test/claim/namespace_xl0/repair_input_origin_denominator_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `xl0_declaration_of` | 55 | CONSTRUCT | TEST-FIXTURE(var) | `decl_sym` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/occurrence_role/occurrence_role_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `planted_unreadable_fn_decl` | 182 | CONSTRUCT | STRUCTURAL | `^grammar_production_identity_node_projection` | becomes Structural arm grammar_production_identity_node_projection |
| fn `planted_unreadable_fn_decl` | 183 | CONSTRUCT | STRUCTURAL | `^grammar_production_captured_node_projection` | becomes Structural arm grammar_production_captured_node_projection |

### src/v2/test/claim/parse/d1_declaration_grammar_parse_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `d1_edge_names_where_clause` | 67 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `dag_surface_where_refinement_clause` | becomes Structural arm match |
| fn `d1_edge_label_text` | 89 | MATCH-binds-name(read as name) | AUTHORED? | `n` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/test/claim/parse/parse_binding_fidelity_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `pbf_module_root` | 101 | CONSTRUCT | TEST-FIXTURE(var) | `export_sym` | becomes Authored (fixture) unless the var carries a marker |
| fn `pbf_subject_with_import_use` | 126 | CONSTRUCT | TEST-FIXTURE | `^pbf_import_use_sym` | becomes Authored |
| fn `pbf_subject_with_ambiguous_import_use` | 230 | CONSTRUCT | TEST-FIXTURE | `^pbf_ambiguous_export_sym` | becomes Authored |

### src/v2/test/claim/parse/variant_field_lowering_test.dag (3)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `vf_named_member_optional` | 116 | MATCH-binds-name(read as name) | AUTHORED? | `sym` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| test `fn` | 338 | MATCH-binds-name(read as name) | AUTHORED? | `nm` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `vf_pattern_binds` | 376 | MATCH-binds-name(read as name) | AUTHORED? | `nm` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/test/claim/parse/where_refinement_clause_parse_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `where_refinement_edge_names_clause` | 75 | MATCH-on-name-text(bound→compare) | STRUCTURAL | `dag_surface_where_refinement_clause` | becomes Structural arm match |
| fn `where_refinement_edge_label_text` | 97 | MATCH-binds-name(read as name) | AUTHORED? | `n` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |

### src/v2/test/claim/provenance/cross_file_provenance_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `cfp_declaration` | 48 | CONSTRUCT | TEST-FIXTURE(var) | `decl_sym` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/provenance/loaded_carrier_receipts_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `lcr_edge_seed` | 189 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `wanted` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |

### src/v2/test/claim/qualified_name/declaration_reference_marker_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| test `fn` | 76 | CONSTRUCT | TEST-FIXTURE | `^declaration_reference` | becomes Authored |

### src/v2/test/claim/reference_derived_graph_production_ingest_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `tree_named_edge_count_in_root` | 139 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `needle` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |

### src/v2/test/claim/reference_derived_graph_witness_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `rd_declaration` | 101 | CONSTRUCT | TEST-FIXTURE(var) | `decl_sym` | becomes Authored (fixture) unless the var carries a marker |
| fn `rd_import_syntax_mention` | 119 | CONSTRUCT | STRUCTURAL | `^dag_surface_import_decl` | becomes Structural arm dag_surface_import_decl |

### src/v2/test/claim/reference_derived_residency_reading_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `named_to` | 88 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/refinement_discharge_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `rdt_application` | 39 | CONSTRUCT | TEST-FIXTURE | `^rdt_param_x` | becomes Authored |

### src/v2/test/claim/resolve/qualified_module_projection_resolve_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `qmpr_provider_root` | 70 | CONSTRUCT | TEST-FIXTURE | `^ProviderType` | becomes Authored |

### src/v2/test/claim/spine/spine_receipt_witness_test.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `chain1` | 13 | CONSTRUCT | TEST-FIXTURE | `^only` | becomes Authored |
| fn `star3` | 21 | CONSTRUCT | TEST-FIXTURE | `^first` | becomes Authored |
| fn `star3` | 22 | CONSTRUCT | TEST-FIXTURE | `^second` | becomes Authored |
| fn `star3` | 23 | CONSTRUCT | TEST-FIXTURE | `^third` | becomes Authored |

### src/v2/test/claim/std/named_child_lookup_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `ncl_named` | 15 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |

### src/v2/test/claim/symbol_index/containment_test.dag (6)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `si_type_with_variants` | 43 | CONSTRUCT | TEST-FIXTURE(var) | `type_sym` | becomes Authored (fixture) unless the var carries a marker |
| fn `si_type_with_variants` | 51 | CONSTRUCT | TEST-FIXTURE(var) | `variant_sym` | becomes Authored (fixture) unless the var carries a marker |
| data `si_orphan_module_root_node` | 337 | CONSTRUCT | TEST-FIXTURE | `^OrphanPilotOnly` | becomes Authored |
| data `si_alias_module_root_node` | 490 | CONSTRUCT | TEST-FIXTURE | `^AliasTargetType` | becomes Authored |
| data `si_alias_module_root_node` | 494 | CONSTRUCT | STRUCTURAL | `^dag_surface_top_level_item` | becomes Structural arm dag_surface_top_level_item |
| test `fn` | 529 | CONSTRUCT | STRUCTURAL | `^dag_surface_top_level_item` | becomes Structural arm dag_surface_top_level_item |

### src/v2/test/claim/target_model_survivor_witness_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| test `fn` | 47 | MATCH-on-name-text(bound→compare) | AUTHORED-modeled | `target_value_expression_field_codepoint` | becomes Authored |

### src/v2/test/claim/type_param_binder_frame_test.dag (10)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `tpb_binder_edge` | 248 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `tpb_generic_arrow` | 261 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `type_params_marker()` | becomes Structural arm type_params_marker() |
| fn `tpb_generic_arrow` | 267 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |
| fn `tpb_formal` | 273 | CONSTRUCT | TEST-FIXTURE(var) | `name` | becomes Authored (fixture) unless the var carries a marker |
| fn `tpb_with_type_param` | 358 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `type_params_marker()` | becomes Structural arm type_params_marker() |
| fn `tpb_box_disj` | 495 | CONSTRUCT | TEST-FIXTURE | `^Full` | becomes Authored |
| fn `tpb_box_disj` | 496 | CONSTRUCT | TEST-FIXTURE | `^Empty` | becomes Authored |
| fn `tpb_box_member_mixed` | 516 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `type_params_marker()` | becomes Structural arm type_params_marker() |
| test `fn` | 788 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `type_params_marker()` | becomes Structural arm type_params_marker() |
| test `fn` | 789 | CONSTRUCT | TEST-FIXTURE | `^stray` | becomes Authored |

### src/v2/test/fixture/derivable_coercion_task_id.dag (7)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `task_id_semantic_core_node` | 18 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisEncoding {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisEncoding {}) |
| fn `task_id_encoding_core_node` | 34 | CONSTRUCT | TEST-FIXTURE | `^task_id_encoding_core_field_representation` | becomes Authored |
| fn `task_id_encoding_core_node` | 38 | CONSTRUCT | TEST-FIXTURE | `^task_id_encoding_core_field_carrier` | becomes Authored |
| fn `task_id_layer_inhabitant` | 51 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisEncoding {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisEncoding {}) |
| fn `task_id_layer_inhabitant` | 58 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisSurfaceSpelling {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisSurfaceSpelling {}) |
| fn `account_id_wrong_inhabitant_node` | 83 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisEncoding {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisEncoding {}) |
| fn `account_id_wrong_inhabitant_node` | 90 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `discriminant(v: ModelCoreFactAxisSurfaceSpelling {})` | becomes Structural arm discriminant(v: ModelCoreFactAxisSurfaceSpelling {}) |

### src/v2/test/fixture/rung_3_4_common.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `rung34_algebra_evidence_node` | 13 | CONSTRUCT | TEST-FIXTURE | `^rung34_algebra_evidence_axis` | becomes Authored |
| fn `rung34_roundtrip_input` | 52 | CONSTRUCT | TEST-FIXTURE | `^rung34_roundtrip_module_axis` | becomes Authored |

### src/v2/test/identity_captured_navigation_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `synth_named_edge` | 24 | CONSTRUCT(funnel helper) | CALLER-DETERMINED | `name` | split into structural/authored constructors; callers classified separately (see funnel table) |

### src/v2/test/lens_application/apply_lens_enforce_uses_projection_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `projection_test_root` | 49 | CONSTRUCT | TEST-FIXTURE | `^projection_section_step` | becomes Authored |

### src/v2/test/lens_application/required_lens_coverage_join_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `coverage_join_root` | 327 | CONSTRUCT | TEST-FIXTURE | `^ram_bytes` | becomes Authored |

### src/v2/test/lens_application/substitute_at_ambiguous_duplicate_name_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `substitute_ambiguous_duplicate_name_root` | 31 | CONSTRUCT | TEST-FIXTURE | `^substitute_ambiguous_dup_step` | becomes Authored |
| data `substitute_ambiguous_duplicate_name_root` | 35 | CONSTRUCT | TEST-FIXTURE | `^substitute_ambiguous_dup_step` | becomes Authored |

### src/v2/test/lens_application/substitute_at_depth_three_test.dag (6)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `depth_three_inner` | 25 | CONSTRUCT | TEST-FIXTURE | `^depth_three_step_c` | becomes Authored |
| data `depth_three_middle` | 36 | CONSTRUCT | TEST-FIXTURE | `^depth_three_step_b` | becomes Authored |
| data `depth_three_root` | 47 | CONSTRUCT | TEST-FIXTURE | `^depth_three_step_a` | becomes Authored |
| data `depth_three_expected_root` | 58 | CONSTRUCT | TEST-FIXTURE | `^depth_three_step_a` | becomes Authored |
| data `depth_three_expected_root` | 63 | CONSTRUCT | TEST-FIXTURE | `^depth_three_step_b` | becomes Authored |
| data `depth_three_expected_root` | 68 | CONSTRUCT | TEST-FIXTURE | `^depth_three_step_c` | becomes Authored |

### src/v2/test/lens_application/substitute_at_depth_two_test.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `substitute_middle` | 25 | CONSTRUCT | TEST-FIXTURE | `^substitute_step_b` | becomes Authored |
| data `substitute_root` | 36 | CONSTRUCT | TEST-FIXTURE | `^substitute_step_a` | becomes Authored |
| data `substitute_expected_root` | 47 | CONSTRUCT | TEST-FIXTURE | `^substitute_step_a` | becomes Authored |
| data `substitute_expected_root` | 52 | CONSTRUCT | TEST-FIXTURE | `^substitute_step_b` | becomes Authored |

### src/v2/test/lens_application/subterm_at_ambiguous_duplicate_name_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `ambiguous_duplicate_name_root` | 25 | CONSTRUCT | TEST-FIXTURE | `^ambiguous_dup_step` | becomes Authored |
| data `ambiguous_duplicate_name_root` | 29 | CONSTRUCT | TEST-FIXTURE | `^ambiguous_dup_step` | becomes Authored |

### src/v2/test/lens_cost/arrow_body_cost_domain_excluded.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `arrow_body_cost_arrow` | 59 | CONSTRUCT | STRUCTURAL | `^arrow_body_edge` | becomes Structural arm arrow_body_edge |

### src/v2/test/lens_cost/loop_illegal_named_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `loop_illegal_named_input` | 40 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |
| data `loop_illegal_named_input` | 41 | CONSTRUCT | STRUCTURAL | `^loop_illegal_named_extra_atom` | becomes Structural arm loop_illegal_named_extra_atom |

### src/v2/test/lens_cost/loop_iteration_floor_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `loop_iteration_floor_input` | 21 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |

### src/v2/test/lens_cost/loop_linear_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `loop_linear_input` | 43 | CONSTRUCT | STRUCTURAL | `^loop_bound_edge` | becomes Structural arm loop_bound_edge |

### src/v2/test/lens_effect/effect_depends_on.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `effect_root` | 67 | CONSTRUCT | STRUCTURAL | `^effect_edge_symbol` | becomes Structural arm effect_edge_symbol |
| fn `lens_effect_marker_node` | 131 | CONSTRUCT | STRUCTURAL(via symbol-valued fn/data) | `marker` | becomes Structural arm marker |

### src/v2/test/lens_fact_density/fact_bundle_compile_lens_passes_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `fact_bundle_cl_root` | 29 | CONSTRUCT | TEST-FIXTURE | `^fact_bundle_cl_field_name` | becomes Authored |

### src/v2/test/lens_fact_density/fact_bundle_named_test.dag (2)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `fact_bundle_input` | 33 | CONSTRUCT | TEST-FIXTURE | `^fact_bundle_width_field` | becomes Authored |
| data `fact_bundle_input` | 37 | CONSTRUCT | TEST-FIXTURE | `^fact_bundle_signedness_field` | becomes Authored |

### src/v2/test/lens_fact_density/hollow_alias_nested_rejected_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `hollow_alias_nested_root` | 27 | CONSTRUCT | STRUCTURAL | `^hollow_alias_nested_child_edge` | becomes Structural arm hollow_alias_nested_child_edge |

### src/v2/test/lens_idempotency/write_effect_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `idempotency_root` | 31 | CONSTRUCT | STRUCTURAL | `^idempotency_edge_symbol` | becomes Structural arm idempotency_edge_symbol |

### src/v2/test/lens_lifecycle_carrier/raw_prose_field_flagged_test.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `raw_dissolve_on_field_input` | 43 | CONSTRUCT | TEST-FIXTURE | `^dissolve_on` | becomes Authored |
| data `raw_dissolution_trigger_nonempty_input` | 55 | CONSTRUCT | TEST-FIXTURE | `^dissolution_trigger` | becomes Authored |
| data `typed_dissolution_field_input` | 67 | CONSTRUCT | TEST-FIXTURE | `^dissolution` | becomes Authored |
| data `ordinary_string_field_input` | 81 | CONSTRUCT | TEST-FIXTURE | `^reason` | becomes Authored |

### src/v2/test/lens_ownership/resource_dependency.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `ownership_root` | 31 | CONSTRUCT | STRUCTURAL | `^ownership_edge_symbol` | becomes Structural arm ownership_edge_symbol |

### src/v2/test/lens_parallelism/data_dependency_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `parallelism_root` | 30 | CONSTRUCT | STRUCTURAL | `^parallelism_edge_symbol` | becomes Structural arm parallelism_edge_symbol |

### src/v2/test/lens_structural_resolution/binds_to_resolved_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `sr_decl` | 21 | CONSTRUCT | STRUCTURAL | `^dependency_binds_to_edge` | becomes Structural arm dependency_binds_to_edge |

### src/v2/test/lens_structural_resolution/facts_lookup_miss.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `sr_lookup_decl` | 21 | CONSTRUCT | STRUCTURAL | `^dependency_binds_to_edge` | becomes Structural arm dependency_binds_to_edge |

### src/v2/test/lens_structural_resolution/module_import_out_of_scope.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `sr_mod_root` | 18 | CONSTRUCT | STRUCTURAL | `^dependency_module_import_edge` | becomes Structural arm dependency_module_import_edge |

### src/v2/test/lens_structural_resolution/unbound_symbol_at_use.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `sr_unbound_decl` | 20 | CONSTRUCT | STRUCTURAL | `^dependency_binds_to_edge` | becomes Structural arm dependency_binds_to_edge |

### src/v2/test/lens_structural_resolution/unresolved_infer_witness.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `sr_infer_decl` | 21 | CONSTRUCT | STRUCTURAL | `^dependency_binds_to_edge` | becomes Structural arm dependency_binds_to_edge |

### src/v2/test/lens_unit_modeling/bitwidth_flat_scalar_flagged_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `bitwidth_flat_scalar_input` | 30 | CONSTRUCT | TEST-FIXTURE | `^pointer_width` | becomes Authored |

### src/v2/test/lens_unit_modeling/bitwidth_modeled_passes_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `bitwidth_modeled_input` | 30 | CONSTRUCT | TEST-FIXTURE | `^pointer_width` | becomes Authored |

### src/v2/test/lens_unit_modeling/coordinate_index_not_flagged_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `coordinate_field_input` | 30 | CONSTRUCT | TEST-FIXTURE | `^index` | becomes Authored |

### src/v2/test/lens_unit_modeling/electrical_blocked_pending_flagged_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `electrical_field_input` | 30 | CONSTRUCT | TEST-FIXTURE | `^tdp_watts` | becomes Authored |

### src/v2/test/lens_unit_modeling/flat_scalar_unit_leaf_flagged_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `flat_scalar_unit_leaf_input` | 30 | CONSTRUCT | TEST-FIXTURE | `^ram_bytes` | becomes Authored |

### src/v2/test/lens_unit_modeling/label_version_not_flagged_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `label_field_input` | 30 | CONSTRUCT | TEST-FIXTURE | `^version` | becomes Authored |

### src/v2/test/lens_unit_modeling/modeled_unit_field_admitted_via_always_required_lenses_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `unit_ar_pass_root` | 40 | CONSTRUCT | TEST-FIXTURE | `^ram_bytes` | becomes Authored |

### src/v2/test/lens_unit_modeling/modeled_unit_field_passes_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `modeled_unit_field_input` | 30 | CONSTRUCT | TEST-FIXTURE | `^ram_bytes` | becomes Authored |

### src/v2/test/lens_unit_modeling/novel_unit_flagged_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `novel_unit_input` | 30 | CONSTRUCT | TEST-FIXTURE | `^throughput` | becomes Authored |

### src/v2/test/lens_unit_modeling/unit_modeling_blocked_via_always_required_lenses_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `unit_ar_root` | 40 | CONSTRUCT | TEST-FIXTURE | `^ram_bytes` | becomes Authored |

### src/v2/test/lens_unused_parameters/binds_to_edge.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `unused_parameters_root` | 30 | CONSTRUCT | STRUCTURAL | `^unused_parameters_edge_symbol` | becomes Structural arm unused_parameters_edge_symbol |

### src/v2/test/lens_unused_parameters/non_use_edges.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `unused_parameters_non_use_root` | 34 | CONSTRUCT | STRUCTURAL | `^unused_parameters_non_use_edge_symbol` | becomes Structural arm unused_parameters_non_use_edge_symbol |

### src/v2/test/lens_unused_parameters/rollup_unused_declaration.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| data `rollup_function` | 26 | CONSTRUCT | STRUCTURAL | `^rollup_edge_symbol` | becomes Structural arm rollup_edge_symbol |

### src/v2/test/lens_wiring_liveness/wiring_liveness_test.dag (1)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `wl_output_tree` | 42 | CONSTRUCT | TEST-FIXTURE | `^wiring_liveness_output_field` | becomes Authored |

### src/v2/workflow/bootstrap.dag (12)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `bootstrap_seed_capability_inventory_root` | 212 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^bootstrap_construct_source_language_subset` | becomes Authored |
| fn `bootstrap_seed_capability_inventory_root` | 216 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^bootstrap_construct_target_model_subset` | becomes Authored |
| fn `bootstrap_seed_capability_inventory_root` | 220 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^bootstrap_construct_runtime_model_subset` | becomes Authored |
| fn `bootstrap_seed_capability_inventory_root` | 224 | CONSTRUCT | AUTHORED-modeled(literal field/variant name of a hand-built type node) | `^bootstrap_construct_lowering_subset` | becomes Authored |
| fn `bootstrap_target_model_bundle_construct_fold_step` | 291 | MATCH-binds-name(read as name) | AUTHORED? | `label` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `bootstrap_hash_pin_pair_projection_node` | 410 | CONSTRUCT | STRUCTURAL | `^bootstrap_projection_hash_pin_slot_edge` | becomes Structural arm bootstrap_projection_hash_pin_slot_edge |
| fn `bootstrap_hash_pin_pair_projection_node` | 418 | CONSTRUCT | STRUCTURAL | `^bootstrap_projection_hash_digest_slot_edge` | becomes Structural arm bootstrap_projection_hash_digest_slot_edge |
| fn `bootstrap_projection_closure_node` | 511 | CONSTRUCT | STRUCTURAL | `^bootstrap_projection_snapshot_root_edge` | becomes Structural arm bootstrap_projection_snapshot_root_edge |
| fn `bootstrap_projection_closure_node` | 515 | CONSTRUCT | STRUCTURAL | `^bootstrap_projection_snapshot_language_model_edge` | becomes Structural arm bootstrap_projection_snapshot_language_model_edge |
| fn `bootstrap_projection_closure_node` | 519 | CONSTRUCT | STRUCTURAL | `^bootstrap_projection_snapshot_content_hash_edge` | becomes Structural arm bootstrap_projection_snapshot_content_hash_edge |
| fn `bootstrap_projection_closure_node` | 523 | CONSTRUCT | STRUCTURAL | `^bootstrap_projection_target_bundle_edge` | becomes Structural arm bootstrap_projection_target_bundle_edge |
| fn `bootstrap_projection_closure_node` | 527 | CONSTRUCT | STRUCTURAL | `^bootstrap_projection_runtime_identity_edge` | becomes Structural arm bootstrap_projection_runtime_identity_edge |

### src/v2/workflow/realization_attempt.dag (4)

| decl | line | role | source | symbol | disposition |
|---|---|---|---|---|---|
| fn `collect_arrows` | 133 | MATCH-binds-name(read as name) | AUTHORED? | `decl_name` | becomes Authored (reader consumes the name; must decide whether Structural labels were also reaching it) |
| fn `collect_arrows` | 138 | CONSTRUCT | AUTHORED | `decl_name` | becomes Authored |
| fn `decl_edges_named` | 237 | MATCH-on-name-text(bound→compare)/GENERIC-QUERY | CALLER-DETERMINED | `fn_name` | query API should take EdgeLabel (or split Structural/Authored lookups); callers classified at their call site |
| fn `decl_edges_named` | 242 | CONSTRUCT | AUTHORED | `n` | becomes Authored |
## Excluded non-EdgeLabel Named sites

| file | decl | line | owner |
|---|---|---|---|
| src/v2/std/node.dag | type `EdgeLabel` | 93 | TYPEDECL:EdgeLabel |
| src/v2/test/fixture/spice_rc_passive_deck.dag | data `spice_rc_passive_deck` | 10 | spice.Node(Named) |
| src/v2/test/fixture/spice_rc_passive_deck.dag | data `spice_rc_passive_deck` | 11 | spice.Node(Named) |
| src/v2/test/fixture/spice_rc_passive_deck.dag | data `spice_rc_passive_deck` | 18 | spice.Node(Named) |
| src/v2/test/fixture/spice_rc_tran_deck.dag | data `spice_rc_tran_deck` | 29 | spice.Node(Named) |
| src/v2/test/fixture/spice_rc_tran_deck.dag | data `spice_rc_tran_deck` | 38 | spice.Node(Named) |
| src/v2/test/fixture/spice_rc_tran_deck.dag | data `spice_rc_tran_deck` | 39 | spice.Node(Named) |
| src/v2/test/fixture/spice_rc_tran_deck.dag | data `spice_rc_tran_deck` | 47 | spice.Node(Named) |
| src/v2/test/fixture/spice_rc_tran_deck.dag | data `spice_rc_tran_deck` | 57 | spice.Node(Named) |
| src/v2/test/claim/extdeps/spice_element_letter_test.dag | fn `one_resistor` | 32 | spice.Node(Named) |
| src/v2/test/claim/extdeps/spice_element_letter_test.dag | fn `one_capacitor` | 49 | spice.Node(Named) |
| src/v2/test/claim/extdeps/spice_element_letter_test.dag | test `fn` | 101 | spice.Node(Named) |
| src/v2/test/claim/extdeps/spice_rc_passive_deck_claims_test.dag | fn `spice_rc_passive_deck_named_node_holds` | 10 | spice.Node(Named) |
| src/v2/test/extdeps/formats/spice_passive_projection_test.dag | test `fn` | 30 | spice.Node(Named) |
| src/v2/test/extdeps/formats/spice_passive_projection_test.dag | test `fn` | 43 | spice.Node(Named) |
| src/v2/test/extdeps/formats/spice_passive_projection_test.dag | test `fn` | 56 | spice.Node(Named) |
| src/v2/test/extdeps/formats/spice_passive_projection_test.dag | test `fn` | 69 | spice.Node(Named) |
| src/v2/extdeps/formats/spice_passive_projection.dag | fn `passive_terminal_from_node_ref` | 36 | spice.Node(Named) |
| src/v2/extdeps/formats/spice.dag | type `NodeRef` | 92 | spice.Node(Named) |
| src/v2/extdeps/formats/spice.dag | fn `spice_emit_node_ref` | 362 | spice.Node(Named) |
| src/v2/extdeps/formats/spice.dag | data `spice_probe_resistor_netlist` | 548 | spice.Node(Named) |
| src/v2/extdeps/formats/spice.dag | data `spice_probe_resistor_netlist` | 549 | spice.Node(Named) |
| dag/std/algebra.dag | type `ContainerSource` | 299 | TYPEDECL:ContainerSource |
| dag/std/algebra.dag | fn `free_monoid_scalar_templates` | 794 | algebra.ContainerSource |
| dag/std/algebra.dag | fn `free_monoid_scalar_templates` | 795 | algebra.ContainerSource |
| dag/std/algebra.dag | fn `free_monoid_scalar_templates` | 796 | algebra.ContainerSource |
| dag/std/algebra.dag | fn `finitely_supported_function_templates` | 855 | algebra.ContainerSource |
| dag/std/algebra.dag | fn `finitely_supported_function_templates` | 856 | algebra.ContainerSource |
| dag/gunbc/product/compute_board/spice_projection.dag | fn `component_node` | 129 | spice.Node(Named) |