# Optional producer replacement on main

Authority: existing v2.std.optional.Optional<T> = Absent | Present { value: T }.

| Declaration | Replacement |
|---|---|
| v1.compiler.infer_resolve.resolve_node_bounded | Ingest authored T? before dispatch, resolve its Required inner type, construct one canonical application. Alias grounding forwards diagnostics. |
| v1.compiler.infer_types.instantiate_algebra_type | OptionalOf produces the same application through make_optional_type. |
| v1.compiler.infer_types.apply_type_substitution | OptionalOf produces the same application after substitution. |
| v1.compiler.infer_types.infer_literal_node | LitNull produces Optional of its unconstrained element variable; no Unit/CardOptional coercion. |
| v1.compiler.infer_access.check_index_access_node | List and map indexing produce a canonical Optional of the determined element. No flatten here. |
| v1.compiler.infer_method.builtin_function_registry | Optional builtin returns use make_optional_type; no marker results. |
| v1.compiler.infer.infer_tier2b_builtin_with_kernel_diags | Optional builtin result uses the canonical constructor. |
| v1.compiler.infer_lookup.lookup_field_type_node | Canonical application; bounded idempotent lookup divergence as declared. |
| v1.compiler.infer_lookup.map_lookup_result_type | Canonical application; bounded idempotent lookup divergence as declared. |
| v1.compiler.infer.expand_alias_chain_for_field_access | Peel canonical Optional element, expand alias, reconstruct canonical application. |
| v1.compiler.infer.variant_reference_inferred_node | Main's actual bare-variant producer checks the selected Optional owner's arity and instantiates its payload. Other coproducts retain main's behavior. No variant_owner_application borrowed from #11277. |
| v1.compiler.infer.infer_record_lit_structural | Main's Present route uses the same checked Optional application; forwards producer diagnostics. No separate Present payload convention. |

Three former kernel declaration sites (build_type_env, build_type_env_unresolved, compiler_kernel_type_env) consume kernel_optional_type_node with an authored-shaped generic parameter T. optional_application_node and apply_optional_owner share the declaration substitution used by resolve_node_bounded: substitute_type_slots/substitute_type_slots_scoped moved unchanged into infer_types to avoid an import cycle. No second substitution algorithm and no bind_type_variables special case.

Marker restorations are removed from declared_return_type_node, field_substitution_carrier, resolve_pattern_subject, bind_local_func_conformance, census_declaration_bound_formals, resolve_scrutinee_type_node_seen, resolve_scrutinee_type and preserve_nominal_brand_on_resolve. with_authored_identity copies the resolved structural cardinality. resolve_nominal_alias_rhs and rendered_use_site_type retain the canonical Optional result. Resolved scrutinee lookup preserves the application; pattern elimination reads its instantiated owner. annotate_pattern_parent_enums reads that owner instead of inventing an Optional parent from cardinality.

Node still permits authored CardOptional: this cut claims mechanical prevention at accepted resolved/inferred producer boundaries, not structural impossibility. Typed refusals remain refusals. Syntax fields/identity transports retain authored metadata only as syntax; their type evidence comes from the canonical boundary. classify_field_recursion is the explicitly admitted syntax-analysis divergence, with its caller chain and next-rung trigger in the consumer census.

Falsifier: main committed seed -> gen1 -> install/rebuild -> gen2 -> fixed point -> native observation. If this needs #11277's generic/literal canonicalization, B is falsified and the ruling switches to composed A. No mixed representation, comparator equivalence or emitter bridge may rescue B. The twelve Optional joins identified by regenA.ei3TxS must also disappear after #11277 recomposes.
