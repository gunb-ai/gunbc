# Main-based Optional consumer disposition census

Base: ed853cf430e. Discovery: lexical reads of return_cardinality/CardOptional in all v1 inference modules and emit_rust; imports, comments and string literals excluded. This replaces the prior cb952 census: #11277-only declarations are absent on this base.

| Declaration | Current disposition |
|---|---|
| `v1.compiler.infer_emit_info.field_value_shape_from_type_node` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer_env.qualify_borrowed_type_names` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer_env.node_with_children` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer_env.node_with_inferred` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.classify_field_recursion` | The production collect_item_inductive_fields route over module_items(module.module) is retained as syntax-only analysis: authored cardinality selects RecursionShape and output carries no type evidence. The helper can prefer field_node.inferred; the admission belongs to this pre-resolution call path, not arbitrary inputs. Divergence admitted in msg_9ea9a6bb; future syntax carrier owns this route. |
| `v1.compiler.infer.variant_reference_inferred_node` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.declared_field_is_required` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.conformance_ground_kernel_scalar` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.conformance_ground_element_collection` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.declared_type_kernel_inhabitance_mismatch` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.rejects_string_for_optional_coproduct_field` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.field_type_admits_bare_none` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.field_type_is_optional_coproduct` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.record_lit_instantiated_fields` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.match_arm_types_are_disjoint_coproducts` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.equality_operand_admission` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.structured_application_lit_is_declared_optional_variant` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.declared_type_inhabitance` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.nominal_product_head_name` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.applied_type_argument_conflicts` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.coproduct_payload_where_parent_required` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.direct_call_arg_type_mismatch` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.optional_cast_diags` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.annotate_pattern_parent_enums` | Deleted marker-derived parent synthesis; expand_scrut_type_for_variant_lookup supplies the instantiated owner identity. |
| `v1.compiler.infer.literal_boundary_elaboration` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.infer_expr_body` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.expand_alias_chain_for_field_access` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.infer_record_lit_structural` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer.annotate_descent` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.infer_property_values` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.infer_service_config_properties` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.infer_transport_node` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.infer_item` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.substitute_generics_apply` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.local_binding_for_item` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.transparent_alias_target_name` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.build_type_env_unresolved` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer.typecheck_module` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer_lookup.lookup_field_type_node` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer_lookup.field_summary_for_type` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer_lookup.map_lookup_result_type` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer_patterns.expand_scrut_type_for_variant_lookup` | Marker branch removed; follows ordinary instantiated owner/application path. |
| `v1.compiler.infer_patterns.lookup_variant_in_type` | Marker branch removed; follows ordinary instantiated owner/application path. |
| `v1.compiler.infer_patterns.resolve_scrutinee_type` | Marker branch removed; follows ordinary instantiated owner/application path. |
| `v1.compiler.infer_patterns.constructor_roster_for` | Marker branch removed; follows ordinary instantiated owner/application path. |
| `v1.compiler.infer_resolve.with_authored_identity` | Identity metadata retained; cardinality comes exclusively from the resolved structural input. |
| `v1.compiler.infer_resolve.substitute_type_slots_scoped` | Moved unchanged to infer_types, shared declaration substitution; preserves syntax/identity metadata. |
| `v1.compiler.infer_resolve.resolve_nominal_alias_rhs` | Authored Optional enters resolve_node; canonical Optional application is retained instead of recreating an uninstantiated RHS. |
| `v1.compiler.infer_resolve.resolve_node_bounded` | Single ingestion of authored T? before dispatch; emits applied Optional. Structural-field syntax is forwarded into this boundary; alias grounding forwards diagnostics. |
| `v1.compiler.infer_resolve.rendered_use_site_type` | Authored Optional returns canonical resolved result; other nominal metadata transport retained. |
| `v1.compiler.infer_resolve.resolve_item_types` | Authored return marker forwarded to resolve_node boundary; module-item metadata remains syntax. |
| `v1.compiler.infer_types.reground_alias_carrier_identity` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer_types.structural_carrier_template_name` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer_types.enrich_base_with_fields` | Retained identity/syntax transport or non-Optional Required check; no CardOptional construction or semantic marker test. |
| `v1.compiler.infer_types.node_type_shape` | Marker branch removed; follows ordinary instantiated owner/application path. |
| `v1.compiler.infer_types.node_type_compatible` | CardOptional branch deleted; no equivalence rule added. |
| `v1.compiler.infer_types.prefer_specific_type` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.infer_types.node_type_equals` | CardOptional branch deleted; no equivalence rule added. |
| `v1.compiler.infer_types.extract_optional_inner_node` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.rust_carrier_is_at_shared_layer` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.rust_carrier_optional_wrap` | Deleted; consumers use the instantiated Optional owner/application. |
| `v1.compiler.emit_rust.emit_inferred_type_leaf_name` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.collect_value_emit_type_surface_names` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.needs_box_wrapping` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.inferred_expr_is_optional` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.analyze_rc_match` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.explicit_record_struct_name` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.effective_variant_parent_from_resolved` | Marker branch removed; follows ordinary instantiated owner/application path. |
| `v1.compiler.emit_rust.emit_none_keyword_for_resolved_type` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.contextual_variant_parent_absent` | Marker branch removed; follows ordinary instantiated owner/application path. |
| `v1.compiler.emit_rust.rust_call_arg_fail_closed_unwrap` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.emit_rust_map_method_call` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.emit_typed_method_call` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.emit_typed_match` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.is_already_optional` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.emit_typed_record_lit` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.is_optional_typed_expr` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.type_node_is_ordered_element_run` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.is_string_typed_expr` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.emit_rust_tco_match` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.emit_dry_run_branch_from_props` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.emit_shell_call` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |
| `v1.compiler.emit_rust.emit_cli_param_type_node` | Rewired recognition/elimination to applied Optional owner and ordered argument; remaining metadata copies preserve identity/non-Optional cardinality. |

Observed main population: 83 declarations. This census is a review artifact, not an execution receipt.

Next-rung trigger: separate resolved/inferred evidence into a carrier that cannot contain CardOptional. Present cut is mechanically enforced by producer construction, not structural impossibility.

Bounded lookup divergence: lookup_field_type_node and map_lookup_result_type alone preserve idempotent flattening. Admit nesting there only once nested pattern elimination and emission are proven.
