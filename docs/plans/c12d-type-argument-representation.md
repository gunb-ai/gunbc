# C1.2d: type-argument nodes keep the authored reference

Opening commit of the representation PR (design note + reader census). Status: census classified, no production edit yet.

## Rule
**The one applied type.** An applied type is ONE node: the carrier holding its authored name, its `DeclarationRef`, and its authored arguments. A type-argument wrapper holds that carrier and nothing else.

**The cached expansion is not a value.** Today the carrier also holds `inferred: Resolved{expanded_node}`, the declaration body with slots substituted. That is a second representation of the same applied type, and readers (via `child_type_node` = `resolved_type(ch)`) hold it as if independent, which is how a stale or name-captured copy escapes. After this PR no reader may hold it. `settled_type` is its sole reader; it is at most a derived memo of (declaration, authored args), invalidated by any change to either, and substitution never rewrites it (drop, re-settle). Target end state: the field is removed from the node and any memo lives outside it.

**Question kind decides which view a reader takes** (one rule, no per-reader exceptions without a stated reason):
1. Nominal identity (brand, spelling, hashing, "was it written as X"): reads the authored reference. Brand/nominal aliases are opaque on purpose.
2. Type equality of two instantiations: settles through TRANSPARENT aliases only, stops at a brand alias, compares arguments recursively.
3. Structure, kind, inhabitance (container? arrow? coproduct? kernel scalar? read element or fields): calls `settled_type` first. An alias to a container is still a container.
4. Pass-through and builders read no structure.

Substitution keys on declaration-parameter identity, never spelling.

**Fifth control** (with the four REDs below): one applied alias; change only the declaration parameters' spelling and order, preserving declaration identities. Every structural consumer gets the identical settled result; identity consumers still see the authored reference.

## Observed effects (each gets one RED, red on main, green at head)
1. `Pair<B, A>` permutation by name capture (swift-badger p7–p9, `generic_arity_n_descent_witness`).
2. Wrong-type field access on a generic call result.
3. Dropped alias args (`IntBox<String>`, review 84342's shape).
4. `src/v2/compiler/dependency_demand_census.dag` `demand_census_add_example` (census:1087): alias only in the type argument of the ACTUAL `xs` (`List<QualifiedName>`) compared at an if/else join refuses with `Container(List, QualifiedName) vs FreeMonoid<Coproduct(FreeMonoid)>`. Bisect: alias in `item` only is accepted; alias in no position is accepted.

## Reader census
Classes: WANTS_AUTHORED (reads identity/arguments), WANTS_SETTLED (reads structure, must call `settled_type`), UNAFFECTED, change site. Produced per file by reading each function body; unrun.

### 04_infer.dag
Classification of the 80 functions in `/tmp/ec804/readers_04_infer.dag.txt`. All paths are `src/v1/04_infer.dag` unless a function is named from another file. Nothing was edited.

**Funnel.** `child_type_node` (`04_types.dag`) is `resolved_type(ch)` unless the child is an Arrow or has no `inferred`. Almost every type-argument read below goes through it. After the change it returns the authored reference, so each caller that reads structure from it breaks unless it settles first.

**Caveats.**
- I classified from what each body reads. I did not run anything.
- Where a function only passes a type to a callee, the class describes what the callee reads. The callee is named.
- I classified `field_node` / `variant` field children (record and variant fields) as not being "type-argument" nodes unless the function reads them through `child_type_node`.

| fn | class | one-line reason |
|---|---|---|
| alias_chain_target_after_args | WANTS_AUTHORED | Builds the subst from `carrier.children` via `resolve_scrutinee_type_node` (already settles each arg), then `substitute_generics` on the alias body. |
| annotate_pattern_parent_enums | UNAFFECTED | Reads the scrutinee's whole resolved node (name, `connective == Disj`); no type-argument child. |
| application_argument_identities | WANTS_AUTHORED | `type_reference_identity(child_type_node(ch))`, pure identity. |
| application_type_expansion_head_name | UNAFFECTED | Declaration-level: takes the alias decl's `Resolved` RHS and calls `expand_nominal_alias_rhs` itself. |
| applied_type_argument_conflicts | WANTS_AUTHORED | Reads `authored_name_at(child_type_node(...))`, then `nominal_product_heads_agree`. |
| brand_grounds_transparently_to | UNAFFECTED | Declaration-level alias target name comparison. |
| builtin_text_argument_crossing_diags | UNAFFECTED | Only tests that an expression's `inferred` is `Resolved`. |
| callable_element_signature_mismatch | WANTS_SETTLED | Reads `connective == Arrow` on `child_type_node` of the element child (`type_node_is_arrow`), so an alias to an arrow is invisible. |
| callable_param_type | UNAFFECTED | Reads a param's own type expression `Resolved`, not a type-argument wrapper. |
| carried_structural_type_name | WANTS_SETTLED | Follows the `Resolved` chain (depth 16) to the terminal expansion's name; with authored refs it would stop at the alias name. It should use alias settling. |
| census_upgrade_service_item | UNAFFECTED | Rewrites service-op `Resolved` by re-resolving whole nodes. |
| census_upgrade_sig_binding | UNAFFECTED | Whole-node `resolve_node` of the return type. |
| census_upgrade_type_expr | UNAFFECTED | Recurses only over record/coproduct children; applied types go through `resolve_node` whole. |
| classify_field_recursion | WANTS_AUTHORED | Compares authored names of the field type and of `type_expr.children[i]` to the parent and recursive-type set (name identity). An alias to the parent would be missed. |
| classify_let_value | WANTS_SETTLED (reclassified by review) | Final arm asks "is it a container, what is its element": a structure question. An alias to a container must still classify as a container, so it calls `settled_type` before `is_container_type` / `rt.children[0]`. |
| conformance_ground_element_collection | WANTS_SETTLED | `conformance_ground_kernel_scalar(child_type_node(el))` needs no children, Required, and `is_kernel_type(authored name)`. `List<MyInt>` with `type MyInt = Int` passes today only because the arg is the expansion. |
| container_element_nominal_brand_mismatch | WANTS_AUTHORED | Brand check on authored names via `nominal_call_arg_brand_mismatch`; it keys on name difference and `alias_identity_in_judgment`. |
| coproduct_applied_type_arguments_conflict_scan | WANTS_AUTHORED | Authored name of `child_type_node(p)` against "Unit", then `applied_type_argument_conflicts`. |
| coproduct_variant_payload_admits_type_name | UNAFFECTED | Variant payload field type name plus `alias_identity_in_judgment`; field-level. |
| declaration_head_exposure | UNAFFECTED | Alias declaration's `Resolved` target fed to `exposure_view_for_node`. |
| declared_field_is_required | UNAFFECTED | Cardinality of a field type expression. |
| declared_type_kernel_inhabitance_mismatch_at_element | WANTS_AUTHORED | Hands `child_type_node` args to `declared_type_kernel_inhabitance_mismatch`, which calls `expand_nominal_alias_rhs` on both sides (self-settling). |
| derive_field_provenance | UNAFFECTED | Name of a pattern subject's `resolved_type`. |
| direct_call_formal_has_unbound_type_variable_at | WANTS_AUTHORED | Walks `Resolved`, `child_type_node` children and params looking for `TypeVariable`; it works on authored args that carry their own args. |
| direct_call_generic_type_argument_inhabitance_diags | WANTS_AUTHORED | Zips `declared_type.children` with `resolved_type(actual).children`, checks authored-name identity, and passes the args to `declared_type_obligation_diags`. It also requires equal arg counts, which fails today when args are lost. |
| elaborated_expr_node | UNAFFECTED | Node constructor. |
| equality_field_refusal | WANTS_AUTHORED | `child_type_node` of a field, an Arrow check, then `equality_operand_admission`, which settles. An alias to an arrow still refuses downstream. |
| equality_leaf_admission | WANTS_AUTHORED | Self-settles with `expand_nominal_alias_rhs` and compares `type_reference_identity` of the authored and expanded forms. It already needs the authored identity to differ. |
| equality_operand_admission | WANTS_AUTHORED | Reads `peeled.connective` directly. An authored alias ref has `NoConnective`, so it falls through to `equality_leaf_admission`, which expands it. Container children go through `child_type_node`. |
| equality_operand_is_optional | UNAFFECTED | Whole-node cardinality / "Optional" name check. |
| expected_collection_surface | WANTS_SETTLED | Hand-rolled alias chase through `exp.inferred Resolved` to find a `Map`/`Set`/`List`/`Outcome`/`Optional` head, up to `fuel` hops. It should call `settled_type`. |
| expected_decided_variant_value | UNAFFECTED | Builds a variant-value node. |
| field_substitution_carrier | UNSURE | When the field's type expression is a nameless, childless placeholder, it falls back to the cached `Resolved` expansion and substitutes that. That branch is a compensator for lost authored refs and goes dead once the authored ref is kept. Whether any case still relies on it needs a run. |
| fn_signature_fingerprint | WANTS_AUTHORED | Hashes `resolved_type_name(param)`, an identity name. An alias param type now hashes the alias name instead of the expansion name, so fingerprint values change. |
| grounded_arm_body | UNAFFECTED | Builds nodes from supplied types. |
| grounded_binop | UNAFFECTED | `resolved_type(left)` of an expression. |
| grounded_constructor_shapes | WANTS_SETTLED (reclassified by review) | Decides whether a field type is a grounded natural (inhabitance). A field typed by an alias of a grounded natural is grounded; reading the authored ref alone misses it, so settle first, then `declaration_is_grounded_natural`. |
| grounded_int_literal | UNAFFECTED | Literal constructor. |
| grounded_reflect | UNAFFECTED | Builds nodes; passes `carrier_type` whole to `infer_binop_type_node`. |
| grounded_signed_construct | UNAFFECTED | Builds nodes. |
| host_text_crossing_admitted | UNAFFECTED | Builds the crossing call node. |
| host_text_unfolded_at_code_point_boundary | UNAFFECTED | Only tests that `value.inferred` is `Resolved`. |
| infer_expr_body | see sub-sites below | |
| infer_item | UNAFFECTED | Declaration-level `Resolved` and `type_annotation`; return type read whole. |
| infer_map_literal | WANTS_SETTLED | Passes `map_node.children[1]` (the raw wrapper, not `child_type_node`) as `expected` to `infer_expr`; its readers (`record_lit_*`, `expected_collection_surface`) read structure from it. |
| infer_method_args_with_fold | UNAFFECTED | Forwards `element_type` as an opaque lambda param type. |
| infer_operand_literal | UNAFFECTED | The other operand's whole `Resolved` type is the expected type. |
| infer_record_lit_structural | WANTS_AUTHORED (mixed) | Orchestrator. Comparison sites already call `expand_nominal_alias_rhs` (about lines 11818 and 11819). It calls `substitute_generics` (about line 11763) with a name-keyed subst from `uninstantiated_owner_param_names`. It reads field `Resolved` as the carrier (about line 11741). It pulls the structure it needs from the `record_lit_*` helpers, so its settling needs are theirs. |
| infer_tier2b_builtin_with_kernel_diags | WANTS_AUTHORED | Receiver's whole `Resolved` goes to `map_value_type_in_env` / `map_key_type_in_env` / `set_element_type_in_env`. Those settle the container with `resolve_scrutinee_type_node`. The extracted element is re-embedded with `list_kernel_ty_from_element` or compared by `set_element_types_mismatch`. |
| kernel_optional_shape_refusal | WANTS_AUTHORED | Authored name of `child_type_node(f)` compared to the type parameter name. |
| lambda_arrow_param | UNAFFECTED | Wraps `resolved_type(param)` as a child. It is a producer. |
| nominal_ref_node | UNAFFECTED | Producer. Its output (name plus a `Resolved` leaf of the same name) is already the authored-reference shape. |
| optional_cast_diags | UNAFFECTED | Whole source-expression type. |
| payload_field_type_binder | WANTS_AUTHORED | `type_variable_binder_name` of `declaration_substitution_basis(child_type_node(f))`, a type-variable identity check. |
| peano_grounded_carrier_state | WANTS_AUTHORED | `type_reference_declaration_ref(child_type_node(f))` compared to `row.carrier`; a direct self-reference check. |
| populate_output_provenance | UNAFFECTED | Param type name via `resolved_type_name`; also `item.inferred`. |
| qualified_or_service_projection | UNAFFECTED | Builds the projection node; its only `Resolved` read is its own construction. |
| qualified_value_projection | UNAFFECTED | Reads a declaration's `Resolved` or annotation whole. |
| record_lit_alias_struct_expansion | UNAFFECTED | Declaration-level alias deref, then `expand_type_for_field_access` (settles). |
| record_lit_expanded_from_expected | WANTS_SETTLED | If `exp` isn't itself the Conj named `tn`, it follows `exp.inferred Resolved` and accepts it only if its name equals `tn`. That is a hand-rolled alias deref through the stored expansion. `exp` can be a type-argument wrapper (see `infer_map_literal`). |
| record_lit_expected_coproduct | WANTS_SETTLED | `child_type_node(elem)` then `lookup_type_for`, which is a declaration lookup that does not chase aliases (`04_env.dag`). It tests `connective == Disj`, so an authored alias ref to a coproduct yields none. |
| record_lit_instantiated_fields | WANTS_AUTHORED | Builds subst as `authored_name(decl.params[i]) -> child_type_node(arg)`. Authored args are right, but the key is the param name, not declaration-param identity. It also requires `params count == children count` (lost-arg workaround) and uses `field_substitution_carrier`. |
| resolve_pattern_subject | UNAFFECTED (delegates) | Calls `instantiate_scrutinee_expansion` (`04_patterns.dag`, ~line 67), which reads `arg.inferred Resolved` of each use-site type-argument directly. See the list below. |
| resolved_type_name | WANTS_AUTHORED | Authored name of the `Resolved` target, else of the first child. It is the identity name. |
| resource_declaration_identity | WANTS_AUTHORED | `declaration_ref_of_type_node` of a resource's `Resolved` type. |
| set_builtin_receiver_is_set | UNAFFECTED | `node_is_set_collection` on the receiver expression's whole type. |
| set_element_type_from_receiver | WANTS_AUTHORED | `child_type_node(set_element_type_in_env(...))`; the receiver is settled inside. The element goes to `set_element_types_mismatch` and `node_type_compatible`. |
| set_union_other_operand_is_resolved_non_set | UNAFFECTED | `node_is_set_collection` on the whole type. |
| side_is_established_non_collection_identity | WANTS_AUTHORED | Self-settles with `expand_nominal_alias_rhs` and exposure views. The where-refinement base name check uses `child_type_node(base)` by authored name only. |
| std_optional_payload_node | WANTS_AUTHORED | Returns `exp.children[0]` raw (the wrapper), else `child_type_node(field)`. The two branches are inconsistent today. |
| structural_from_expanded_type | WANTS_SETTLED | If `normed` is not Conj/Disj, it reads `normed.inferred Resolved target.connective` for Conj/Disj. A direct read of the stored alias expansion's structure. |
| substitute_generics_apply | the change site | Matches a bare node by `type_node_label` string and `TypeVariable.id`. It also recurses into and rewrites `n.inferred Resolved` (the cached expansion, `new_inferred`). Both are the pieces to re-key on declaration-param identity and stop rewriting. |
| transparent_alias_target_name | UNAFFECTED | Alias declaration's RHS leaf name. |
| type_node_mentions_any_name | WANTS_AUTHORED | Name mention search over children, params, and `Resolved` (fuel-bounded). Authored args carry the mention; the `Resolved` descent becomes redundant. |
| type_reference_identity | WANTS_AUTHORED | `node_declaration_identity` of the `Resolved` target. Today an alias wrapper yields the expansion head's identity (alias erased). After the change it yields the alias identity, which `equality_leaf_admission` relies on. |
| unfold_scalar_sequence_image | UNAFFECTED | Literal image constructor. |
| unify_binding_is_uninformative | WANTS_AUTHORED | Single-child recursion via `child_type_node`, testing for `TypeVariable`. |
| unify_callable_generics | UNAFFECTED (delegates) | Arrow `Resolved` is the return type. Everything goes to `unify_generics`. |
| unify_callable_param_pairs | UNAFFECTED (delegates) | `unify_generics(child_type_node(f), child_type_node(a))`. |
| unify_record_lit_field_walk | UNAFFECTED (delegates) | Field `Resolved` as formal and `resolved_type(actual)` as actual, into `unify_generics`. |
| validate_cast | UNAFFECTED | Source expression's `Resolved` name. |

**`unify_generics` (~line 14745) is not in the list but is the real reader behind the three `unify_*` rows above.** It recurses into `formal.children` and `actual.children` only when the counts are equal. An authored alias ref has zero children, so it will not bind against `List<T>`. It is WANTS_SETTLED on `actual`.

**`infer_expr_body` sub-sites that read type-argument children or `Resolved`** (approximate absolute line numbers in `04_infer.dag`):

- **ExprFieldAccess arm, ~8979 and ~9186.** `expand_type_for_field_access(base_rt)` / `(binding.resolved)`. Self-settling; fine.
- **ExprCall arm, ~9177 and ~9231.** `call_subst` comes from `unify_generics` over the actual arg types (type-argument children zipped), then `substitute_generics(s.inferred, call_subst)`. WANTS_SETTLED at the `unify_generics` boundary, plus the `substitute_generics` keying.
- **~9355-9372, `final_receiver`.** Copies `bridge_result_type.children` onto the receiver node and gates on `is_fully_resolved`. Reads the args as authored data.
- **Method call arm, ~9694-9705.** `resolve_method_receiver_type` (settles through `resolve_scrutinee_type_node`) and `declared_arg_types_for_method(expand_field_type: expand_type_for_field_access)`. Self-settling; fine.
- **Match arm, ~9780-9930.** `resolve_pattern_subject` then `instantiate_scrutinee_expansion`. WANTS_SETTLED/convert (see the list below).
- **ExprListLit arm, ~10085.** `child_type_node(col.children[0])` of the expected collection is the element `expected`, used for element inference and `declared_type_obligation_diags`. WANTS_SETTLED downstream, because record-literal elements go through `record_lit_expected_coproduct` and `record_lit_expanded_from_expected`.
- **ExprListLit arm, ~10121-10122.** The empty-list case takes `child_type_node(exp.children[0])` as the element type and builds the result with `list_kernel_ty_from_element`. WANTS_AUTHORED (embeds it as a type argument).
- **~10388, for-each.** `for_each_element_type_node(resolved_type(coll))` takes `child_type_node` of the single child after `normalize_access_type_node`. An alias-typed collection has no children, so this needs settling (WANTS_SETTLED). The function is in `04_types.dag` line 1699.
- **~10411-10480, index and slice.** `check_index_access_node` over whole base and index `Resolved` types. UNAFFECTED here; it is a whole-node reader.
- **~9574 and ~10247.** Callable return `Resolved` (the Arrow's `inferred` is its return type). UNAFFECTED.

**WANTS_SETTLED (these break or must convert):**
- `callable_element_signature_mismatch`: reads `Arrow` connective of a type argument.
- `carried_structural_type_name`: chases `Resolved` to the terminal expansion's name.
- `conformance_ground_element_collection`: needs the kernel-scalar name of an alias arg.
- `expected_collection_surface`: hand-rolled alias chase via `Resolved`.
- `infer_map_literal`: passes the raw value-arg wrapper as `expected`.
- `record_lit_expanded_from_expected`: follows `exp.inferred Resolved` to find the named record.
- `record_lit_expected_coproduct`: `lookup_type_for` does not chase aliases, so `Disj` is lost.
- `structural_from_expanded_type`: reads `Resolved target.connective`.
- Outside the list, same pattern:
  - `unify_generics`, which zips children only when counts match.
  - `instantiate_scrutinee_expansion` (`04_patterns.dag`), which reads `arg.inferred Resolved` of use-site args.
  - `for_each_element_type_node` (`04_types.dag`).
  - `infer_expr_body` sub-sites: ExprListLit `elem_expected`, ExprCall `call_subst`, match scrutinee.

**Change sites (not readers):**
- `substitute_generics_apply`: label-string keying and rewriting of `Resolved`.
- `record_lit_instantiated_fields`: param-name keying.
- `field_substitution_carrier`: the cached-`Resolved` fallback.
### 04_env.dag, 04_resolve.dag, 04_types.dag
**Method note.** I read every body in `04_env.dag`, `04_resolve.dag` and `04_types.dag`. I did not read `04_infer.dag`, because the list for it was not part of the task. The one-line reasons below each rest on a body I read. A few edge cases are flagged "edge".

One fact shapes most rows. `child_type_node` in `04_types.dag` is `resolved_type(ch)`, which is the stored `Resolved` target of the argument wrapper. It is the single accessor through which the type-argument expansion reaches almost every consumer. After the change it returns the authored reference. Any caller that then reads connective, children or fields of that result must settle first.

## 04_env.dag

| fn | class | reason |
|---|---|---|
| census_declaration_type_env | UNAFFECTED | Builds an empty `TypeEnv`. |
| qualify_borrowed_inferred | WANTS_AUTHORED | Qualifies names inside a `Resolved` tree. It must act on authored names only. |
| text_element_is_char | WANTS_SETTLED | Takes the `Resolved` of the first child and compares its declaration identity to `Char`. An alias of `Char` used as the argument would now read `NotText`, so it should compare after settling. |
| text_representation_by_identity_bounded | WANTS_AUTHORED | Walks the alias chain itself through `declaration_binding_of_ref` and `decl.inferred`, which is the alias declaration's right-hand side. It delegates the argument check to `text_element_is_char`. |
| transparent_alias_rhs | UNAFFECTED | Reads the alias declaration's own right-hand side, not a type-argument child. |
| type_reference_declaration | WANTS_AUTHORED | Reads the name, span and declaration of the node. Edge: it prefers the `Resolved` node over the authored one. Today that yields the alias target's identity, and after the change it would yield the alias's own identity. |
| type_reference_declaration_reading | WANTS_AUTHORED | Reads the authored identity first and falls back to `Resolved` only if that is unidentified. |
| type_reference_declaration_ref | WANTS_AUTHORED | Authored first, with fallbacks. |
| variant_lookup_structural | WANTS_SETTLED | For a childless node, returns the `Resolved` target in order to read variant structure. It should call `settled_type`. |

## 04_resolve.dag

| fn | class | reason |
|---|---|---|
| binder_marked_inferred | WANTS_AUTHORED | Stamps binder owner onto type variables inside the `Resolved` tree. This is correct only if it walks authored trees. |
| expand_nominal_alias_rhs_by_name | WANTS_SETTLED | It is the expansion producer. It reads the alias's `Resolved` target, resolves it and returns it with the authored identity. It should be replaced by lazy settling. |
| field_inferred_with_declaration | WANTS_AUTHORED | Stamps `DeclarationRef` onto references. |
| generic_declaration_application | WANTS_SETTLED | Producer. It builds `Resolved{expanded_node}` from `substitute_type_slots(decl.children, slot_bindings)`. The slot map is keyed by parameter name string. The stored expansion is what must go. |
| kind_admissible_inhabitant_name | WANTS_AUTHORED | Reads the name of the kind declaration's target. |
| reference_inferred_with_declaration | WANTS_AUTHORED | Stamps `DeclarationRef`. |
| rendered_use_site_type | WANTS_SETTLED | Producer. It keeps the authored name but sets `children: resolved.children`, which is the expansion's children, and sets `inferred: Resolved{resolved}`. This is where authored arguments are replaced. It should keep the authored children and store no expansion. |
| resolve_item_types | UNAFFECTED | Resolves the alias or parameter right-hand side of a declaration and stamps references. It reads no argument child. |
| resolve_optional_node | UNAFFECTED | Resolves a whole `InferredNode`. |
| stamp_collections_in_reference | WANTS_AUTHORED | Recurses into both children and `Resolved`. Edge: if no expansion is stored, only the authored tree is walked. |
| stamp_kernel_collection | UNAFFECTED | Marks the node's own `Resolved{self}`. Edge: whether kernel collections keep that self-reference is a design choice. |
| substitute_type_slots_scoped | WANTS_SETTLED | It rewrites the cached expansion (`new_inferred` recurses into `Resolved rt`) and keys slots by authored name string. Its `is_slot` test also requires `inferred == none`. It should stop rewriting `Resolved` and key on declaration-parameter identity. |

### resolve_node_bounded: arms that store or read a type argument's Resolved

| arm (anchor) | class | reason |
|---|---|---|
| Callable arm (`return_resolution`) | UNAFFECTED | The `Resolved` it reads is the callable's return type, a whole node. It then calls `rendered_use_site_type`. |
| Conj arm with `child.inferred != none` (`child_rt = resolved_type(child)`) | WANTS_SETTLED | Reads and re-stores each field's `Resolved`. It re-resolves the stored expansion and writes it back, so it cannot work from the authored reference. |
| User-generic arm, `arg_results` and `resolved_args` | WANTS_SETTLED | Wraps each argument in `rendered_use_site_type(authored, resolved)`. The result substitutes the expansion's children into the argument. It should keep the authored reference. |
| User-generic arm, `slot_bindings` | WANTS_SETTLED | Binds parameter name to the expanded argument and feeds `substitute_type_slots`. It should key on parameter identity and bind the authored argument. |
| User-generic arm, `is_parameterized_alias`: `alias_target` | WANTS_AUTHORED | Reads `decl.inferred` (the alias right-hand side) and substitutes slots into it. This is declaration-level. |
| User-generic arm, `is_parameterized_alias`: `expanded_node` and `resolved_node` | WANTS_SETTLED | Stores the alias expansion as `Resolved{expanded_node}` with children from `target_result`. This is the main stored-expansion write for aliases. |
| User-generic arm, non-alias branch (`generic_declaration_application`) | WANTS_SETTLED | Stores `Resolved{expanded_node}` built from substituted `decl.children`. |
| Map arm (`resolved_key_child` / `resolved_val_child`) | WANTS_SETTLED | Argument wrappers store `Resolved{key_resolved}` and `Resolved{val_resolved}`, where each value is the fully resolved key or value type from `child_type_node`. The wrapper should hold the authored reference. |
| Single-child container arm (`resolved_child`) | WANTS_SETTLED | The same wrapper construction with `Resolved{el_resolved}`. |
| Childless arm, `n_target_is_coproduct_container` | WANTS_SETTLED | Reads `n.inferred` `Resolved target` for connective, children, `is_container_type` and the declaration's `Disj` connective. That is alias-expansion structure. |
| Childless arm, `structurally_resolved` (`resolved.inferred` `Resolved target`) | WANTS_SETTLED | For a bare alias, it reads the target's structure (`target_is_coproduct_use`) and passes it to `preserve_nominal_brand_on_resolve`. This is the alias-expansion producer for non-generic references. |
| Childless arm, `n_is_error` / `n_is_type_var` | UNAFFECTED | Reads only whole-node `inferred` kind. |

## 04_types.dag

| fn | class | reason |
|---|---|---|
| algebra_child_or_placeholder | UNAFFECTED | Returns `child_type_node` as a whole-node pass-through. |
| algebra_method_field | UNAFFECTED | Builder that wraps a callable in `Resolved`. |
| algebra_value_field | UNAFFECTED | Builder that wraps a type in `Resolved`. |
| alias_identity_in_judgment | UNAFFECTED | Already settles through `alias_settled_agree`. |
| alias_names_identity_in_judgment | UNAFFECTED | Already settles through `alias_names_settled_agree`. |
| applied_argument_declared_agrees | UNAFFECTED | Already calls `settled_type` and then `alias_settled_agree`. |
| apply_type_substitution | WANTS_SETTLED (edge) | `ReceiverSelf` `is_bare` tests `is_kernel_type(authored name)` of the argument. An alias of a kernel type flips from not-bare to bare once the argument is authored. Everything else only passes arguments through or substitutes by template variable id. |
| arguments_agree_in_lockstep | SETTLED_TRANSPARENT (reclassified by review) | Drives pairwise argument equality; the callback it is given receives the authored arguments and the callback applies the equality rule, so the lockstep walk itself must not pre-expand. |
| bare_map_node | UNAFFECTED | Builder with type-variable wrappers. |
| bare_set_node | UNAFFECTED | Builder with type-variable wrappers. |
| callable_inferred | UNAFFECTED | `Resolved` here is the return type, a whole node. |
| callable_resolution_verdict | UNAFFECTED | The same return-type shape. |
| callable_return_type | UNAFFECTED | The same return-type shape. |
| callable_signatures_agree | UNAFFECTED | The same return-type shape. |
| child_type_node | UNAFFECTED | The accessor itself. Its result changes meaning from the expansion to the authored reference, and its callers are the readers that matter. |
| collection_carrier_node | WANTS_SETTLED | For a childless node, returns the `Resolved` target as the collection carrier. For an alias this is the expansion structure (its callers then read children or fields). |
| declaration_carried_through_resolution | CHANGE SITE (reclassified by review) | Its `Resolved`-chain fallback reads the cached expansion as an independent value (the second representation). Remove the fallback: the carrier's own `declaration` is the only identity; a node without one is a typed refusal. |
| for_each_element_type_node | UNAFFECTED | Returns the element as a whole node. Its callers read the structure. |
| infer_arithmetic_binop_type_node | UNAFFECTED | The `Resolved` it reads is the algebra field's callable type. |
| make_callable_type | UNAFFECTED | Builder. |
| make_container_type | UNAFFECTED | Builder. It stores whatever element node the caller passes. |
| make_kernel_record_field | UNAFFECTED | Builder. |
| make_map_type | UNAFFECTED | Builder. |
| make_tuple_type | UNAFFECTED | Builder. |
| method_receiver_element_node | UNAFFECTED | Returns the element as a whole node. |
| node_type_compatible | WANTS_SETTLED | Container, unit-like and optional branches read element structure from `child_type_node` with no settling (edge: an alias argument of a container or unit type). Only the leaf reaches `alias_identity_in_judgment`, which does settle. |
| node_type_deps | WANTS_AUTHORED | Descends `child.inferred` `Resolved rt`, so today it collects the expansion's names. After the change it should yield the authored name, and the alias's own dependencies come from the alias declaration. |
| node_type_equals_core | WANTS_SETTLED | Compares leaf, struct and applied structure of arguments through `child_type_node` with no alias settling. An alias argument versus its expansion would disagree. It should settle at entry. |
| node_type_shape | WANTS_AUTHORED | Produces a shape string. The `Named` or `Primitive` choice reads the `Resolved` leaf, and arguments are shaped through `child_type_node`. Edge: the string changes for alias arguments. |
| node_type_shape_argument_list | WANTS_AUTHORED | The same shape string for each argument. |
| optional_instance_at_payload_of | SETTLED_TRANSPARENT (reclassified by review) | Equality of two instantiations via `applied_argument_types_identical`; per the rule it settles transparent aliases and stops at a brand alias. |
| prefer_specific_type | WANTS_SETTLED (reclassified by review) | `is_unit_like` is an inhabitance/structure question; an alias of unit is unit-like. Not an edge case: settle the element before testing. |
| resolved_type | UNAFFECTED | The accessor itself. |
| type_reference_identity_key | WANTS_AUTHORED | Edge: it takes `Resolved rt` first. Today that gives the target's identity, and after the change the alias's own identity. |
| type_resolution_verdict | UNAFFECTED | Only recurses over `child_type_node`. |
| unify_template | WANTS_SETTLED | `ContainerOf`, `OptionalOf`, `TupleOf` and `ReceiverSelf` compare the container name of `concrete` and read `concrete.children`. An alias argument whose authored name is not the container, or that has no children, would no longer match. It should settle `concrete` before reading. |

## WANTS_SETTLED

**04_env.dag**
- `text_element_is_char`
- `variant_lookup_structural`

**04_resolve.dag**
- `expand_nominal_alias_rhs_by_name`
- `generic_declaration_application`
- `rendered_use_site_type`
- `substitute_type_slots_scoped`
- `resolve_node_bounded`, in these sub-arms:
  - Conj `child_rt`
  - User-generic `resolved_args` and `slot_bindings`
  - Alias `expanded_node` and `resolved_node`
  - Non-alias `generic_declaration_application`
  - Map and single-child `Resolved{...}` wrappers
  - `n_target_is_coproduct_container`
  - `structurally_resolved`

**04_types.dag**
- `apply_type_substitution` (the `is_bare` test, edge)
- `collection_carrier_node`
- `node_type_compatible`
- `node_type_equals_core`
- `unify_template`

Most of the `04_resolve.dag` entries are producers rather than readers. They write the stored expansion, so deleting the write is the change.

I created nothing in the repo. The only files I wrote are scratch extracts in `/tmp/ec804` (`b_env.txt`, `b_types.txt`, `b_resolve.txt` and `rnb.txt`).
## Identity relation (added after review; not in the original census)
`applied_argument_types_identical`, `applied_argument_heads_identical`, `arguments_agree_in_lockstep`, `optional_instance_at_payload_of` and `whole_declaration_carried` (04_types) form the type-equality relation. Class SETTLED_TRANSPARENT: heads settle through transparent aliases only and stop at a brand alias; arguments compare recursively; `whole_declaration_carried` reads only the carrier's own `declaration`. The review's missing reader was this relation.

## Mechanical closure over the remaining files
`child_type_node(` callers in 04_items, 04_lookup, 04_emit_info, 04_patterns, 05_emit, trait_derive_emit, 05_emit_rust are being classified by the same rule; the result is appended below when complete.
