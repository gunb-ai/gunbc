Optional consumer census for the representation ruling (source head cb9529347fe; proposed payload-only draft does not alter these representation decisions).

Proposal: canonical inference result = applied `v2.std.optional.Optional<T>`, with its instantiated declaration payload. Authored `T?` remains parser input and is ingested at the resolved-type producer boundary. This is a replacement migration: no mixed-result comparator, emitter-site wrapping, or per-consumer exception. Recommend composing the complete producer/consumer cut in #11277, since its present seed cannot close its own corpus; do not publish an intermediate seed as green.

Discovery: `rg -n "return_cardinality|CardOptional" src/v1/04*.dag src/v1/05_emit_rust.dag`; entries below map actual reads (excluding imports, comments, string literals and constant Required writes) to enclosing declarations. This deliberately includes all inference-family modules, beyond the literal 04_infer glob. Copy-only reads are listed rather than silently excluded. Dispositions are proposals awaiting the gatekeeper/side-chat ruling, not implemented changes.

| Declaration | Disposition under applied Optional |
|---|---|
| `v1.compiler.infer_emit_info.field_value_shape_from_type_node` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer_env.qualify_borrowed_type_names` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer_env.node_with_children` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer_env.node_with_inferred` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer.classify_field_recursion` | Syntax-only analysis, retained: collect_fields_inductive consumes unresolved authored declarations before resolved field evidence exists; outputs RecursionShape/element spelling, never type evidence. Deliberate §3b divergence; next carrier split places this read on syntax by construction. |
| `v1.compiler.infer.variant_owner_application` | Retain nominal metadata copying; owner application is Required for Optional; reject malformed owner at the producer boundary. |
| `v1.compiler.infer.variant_reference_inferred_node` | Delete the CardOptional-vs-Required owner-selection convention after expected types are canonical; preserve owner identity. |
| `v1.compiler.infer.declared_field_is_required` | Rewire absence admission to the canonical Optional owner; preserve default-value condition. |
| `v1.compiler.infer.conformance_ground_kernel_scalar` | Rewire scalar admission to reject Optional applications; no broadening of conformance. |
| `v1.compiler.infer.conformance_ground_element_collection` | Rewire element-collection admission to reject Optional applications; no broadening of conformance. |
| `v1.compiler.infer.declared_type_kernel_inhabitance_mismatch` | Delete the dual-representation exclusion only as the canonical structure supplies the same domain boundary; retain existing kernel scope. |
| `v1.compiler.infer.rejects_string_for_optional_coproduct_field` | Rewire to the Optional element coproduct; keep the string refusal. |
| `v1.compiler.infer.field_type_admits_bare_none` | Rewire absence admission to applied Optional / declared absence variant. |
| `v1.compiler.infer.field_type_is_optional_coproduct` | Rewire to the element of applied Optional, then test coproduct shape. |
| `v1.compiler.infer.record_lit_instantiated_fields` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer.match_arm_types_are_disjoint_coproducts` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer.equality_operand_admission` | Delete CardOptional alternative; inspect canonical application argument. |
| `v1.compiler.infer.structured_application_lit_is_declared_optional_variant` | Rewire declared Optional recognition; validate owner and variant, not spelling alone. |
| `v1.compiler.infer.declared_type_inhabitance` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer.nominal_product_head_name` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer.applied_type_argument_conflicts` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer.coproduct_payload_where_parent_required` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer.direct_call_arg_type_mismatch` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer.optional_cast_diags` | Rewire both source and target recognition; retain OptionalCastNotEliminated refusal. |
| `v1.compiler.infer.annotate_pattern_parent_enums` | Delete marker-derived parent synthesis; read the instantiated Optional owner. |
| `v1.compiler.infer.literal_boundary_elaboration` | Rewire optional-destination recognition/element extraction at the canonical type boundary. |
| `v1.compiler.infer.infer_expr_body` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer.expand_alias_chain_for_field_access` | Delete marker reapplication; preserve the applied Optional wrapper while expanding its element. |
| `v1.compiler.infer.infer_record_lit_structural` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer.annotate_descent` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer.infer_property_values` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer.infer_service_config_properties` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer.infer_transport_node` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer.infer_item` | Retain authored declaration cardinality as input syntax; its inferred result must be canonical. |
| `v1.compiler.infer.refine_generic_constraint` | Retain exact cardinality disagreement refusal for non-Optional cardinalities; Optional constraints use the existing application-argument path. No comparator widening. |
| `v1.compiler.infer.substitute_generics_apply` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer.local_binding_for_item` | Rewire binding production to canonical inferred type; retain syntax copies only as syntax. |
| `v1.compiler.infer.transparent_alias_target_name` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer.build_type_env_unresolved` | Retain authored syntax metadata in unresolved declarations; canonicalize when producing resolved type evidence. |
| `v1.compiler.infer.typecheck_module` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer_lookup.lookup_field_type_node` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer_lookup.field_summary_for_type` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer_lookup.map_lookup_result_type` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer_patterns.expand_scrut_type_for_variant_lookup` | Delete optional-marker early return; follow the instantiated application payload through existing lookup. |
| `v1.compiler.infer_patterns.lookup_variant_in_type` | Delete synthetic marker-based Present/Absent fields; use the instantiated owner declaration. |
| `v1.compiler.infer_patterns.resolve_scrutinee_type` | Delete with_optional_cardinality reapplication; preserve application arguments during lookup. |
| `v1.compiler.infer_patterns.constructor_roster_for` | Delete marker-based fixed roster; read the instantiated Optional coproduct constructors. |
| `v1.compiler.infer_resolve.with_authored_identity` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer_resolve.substitute_type_slots_scoped` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer_resolve.resolve_nominal_alias_rhs` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer_resolve.resolve_node_bounded` | Ingest authored CardOptional into the canonical application; delete optional-marker reapplication to resolved results. |
| `v1.compiler.infer_resolve.rendered_use_site_type` | Retain authored cardinality only on syntax projection; resolved payload must already be canonical. |
| `v1.compiler.infer_resolve.resolve_item_types` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer_types.reground_alias_carrier_identity` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer_types.structural_carrier_template_name` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer_types.enrich_base_with_fields` | Retain syntax/identity transport; canonicalize upstream resolved Optional evidence and do not recreate CardOptional in an inferred type. |
| `v1.compiler.infer_types.node_type_shape` | Delete marker decoration; render the existing Optional application structure. |
| `v1.compiler.infer_types.node_type_compatible` | Delete marker representation branch after canonical producer cut; keep comparison of canonical application arguments, with no new acceptance rule. |
| `v1.compiler.infer_types.prefer_specific_type` | Rewire the Optional read to canonical applied-owner/element evidence; retain the existing admission/refusal scope. |
| `v1.compiler.infer_types.node_type_equals` | Delete marker representation branch after canonical producer cut; retain structural equality. |
| `v1.compiler.infer_types.extract_optional_inner_node` | Delete CardOptional peel; extract the single canonical application argument. |
| `v1.compiler.emit_rust.rust_carrier_is_at_shared_layer` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.rust_carrier_optional_wrap` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.emit_inferred_type_leaf_name` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.collect_value_emit_type_surface_names` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.needs_box_wrapping` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.inferred_expr_is_optional` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.analyze_rc_match` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.explicit_record_struct_name` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.effective_variant_parent_from_resolved` | Delete cardinality-based Optional parent fallback; use resolved owner identity. |
| `v1.compiler.emit_rust.emit_none_keyword_for_resolved_type` | Rewire type-directed None emission to canonical Optional identity; unresolved owner still refuses. |
| `v1.compiler.emit_rust.contextual_variant_parent_absent` | Delete cardinality-based Optional parent fallback; use resolved owner identity. |
| `v1.compiler.emit_rust.rust_call_arg_fail_closed_unwrap` | Rewire parameter/argument Optional recognition and element extraction; retain explicit elimination/refusal rules. |
| `v1.compiler.emit_rust.emit_rust_map_method_call` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.emit_typed_method_call` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.emit_typed_match` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.is_already_optional` | Rewire all five inferred/binding reads to canonical Optional; remove marker-specific coercion evidence. |
| `v1.compiler.emit_rust.emit_typed_record_lit` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.is_optional_typed_expr` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.type_node_is_ordered_element_run` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.is_string_typed_expr` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.emit_rust_tco_match` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.emit_dry_run_branch_from_props` | Rewire parameter Optional test to canonical type evidence; preserve required Some coercion behavior. |
| `v1.compiler.emit_rust.emit_shell_call` | Rewire Optional recognition to the applied owner and element; preserve the existing host wrapping/borrowing behavior without treating a Required Optional application as its element. |
| `v1.compiler.emit_rust.emit_cli_param_type_node` | Rewire optional CLI parameter recognition to canonical type evidence and element. |

The producer roster remains the one in [reduction receipt 5658653922](https://github.com/gunb-ai/gunbc/pull/11277#issuecomment-5658653922). The root helpers `with_optional_cardinality` / `preserve_outer_optional_cardinality` must cease producing inferred Optional results; authored syntax remains separate. Existing `Required`/other-cardinality transport is not permission to erase non-Optional cardinality.

New owner census from the accepted exact-arity condition: `build_type_env`, `build_type_env_unresolved`, and `compiler_kernel_type_env` each authored the kernel Optional with `params=[]` despite a generic Present value. Gatekeeper message msg_57ded911 approves declaring its parameter through one kernel owner producer and removing the separate Present fallback. This belongs to the payload correction; the representation migration remains unedited.
