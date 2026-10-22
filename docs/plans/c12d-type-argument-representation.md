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
One row per function (the earlier grouped rows are replaced). Classes: A = WANTS_AUTHORED, T = SETTLED_TRANSPARENT, S = WANTS_SETTLED, U = UNAFFECTED. No UNSURE remains: each emit reader was resolved by reading its body. Derived by reading each function; unrun. The same grep over 04_infer/04_types/04_env/04_resolve found no `child_type_node(` caller missing from the tables above.

Source: docs/plans/c12d-type-argument-representation.md (Rule, Mechanical closure, List 2). Classes: A = WANTS_AUTHORED (identity/spelling), T = SETTLED_TRANSPARENT (type equality), S = WANTS_SETTLED (structure/kind/inhabitance), U = UNAFFECTED.

Facts established from bodies (04_types `child_type_node`): today `child_type_node(ch)` = `resolved_type(ch)` (the cached expansion) when `ch.inferred` is set and ch is not an Arrow; so today every alias type ARGUMENT is read as its expansion, and Rust text names the expansion for arguments. `__applied_type_args` has no writer anywhere, so every Present arm that reads it is dead. An Arrow's `inferred` is its return type (U). Emitted Rust alias policy: zero-param aliases are `pub type` items and may be peeled by `closed_alias_peel_verdict`; cross-module aliases need an import, so naming an alias in argument position is not decidable at the spelling site. Rust expands aliases, so for (i) identity/spelling the alias is needed only for import-surface collection and declaration emission (A rows), and for (ii) type equality / bound eligibility the expansion is needed (T, S rows). Decision for argument spelling: settle each argument before spelling (S) so bytes are unchanged.


### src/v1/trait_derive_emit.dag (all 120 fns)

| fn | class | reason |
|---|---|---|
| v1_type_expr_clone_impl_needs_param | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_declared_type_app_clone_impl_needs_param_loop | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_declared_arg_positions_need_clone_param | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_type_expr_wf_needs_clone_param | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_type_expr_clone_undecided_head | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_type_expr_head_is_known | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_type_expr_mentions_type_head | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_fn_generic_clone_bound_via_bounded_container_element | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_generic_param_used_as_collection_element | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_type_expr_ambiguous_declared_head | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_type_expr_components_ambiguous_declared_head | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_type_exprs_ambiguous_declared_head | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_type_expr_head_declared_identities | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_map_key_required_type_names | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_map_key_propagate_round | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_item_field_type_expr_wf_needs_clone_param | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_item_param_wf_needs_clone | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_fn_param_wf_needs_clone | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_item_clone_undecided_head | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_item_wf_propagated_clone_bounded_param_names | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_clone_bound_seed_for_item | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_clone_impl_seed_for_item | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_map_key_bound_seed_for_item | S | descends the written type's arguments via v1_wf_child_type_node/child_type_node and decides by the DECLARED head (resolve_type_decl_reference / container or algebra name). Rust expands aliases, so Clone/Ord/Eq-Hash eligibility is a property of the expanded type; an authored alias head would resolve to the alias item and lose the bound. Settle each argument before the head decision. |
| v1_type_expr_mentions_param_non_phantom | S | walks arguments to find a param occurrence that is non-phantom in the DECLARED head's parameter list (resolve_type_decl_reference); an alias-hidden occurrence must be seen through the expansion, so settle before the walk. |
| v1_declared_type_app_mentions_param_non_phantom_loop | S | walks arguments to find a param occurrence that is non-phantom in the DECLARED head's parameter list (resolve_type_decl_reference); an alias-hidden occurrence must be seen through the expansion, so settle before the walk. |
| v1_map_key_head_type_exprs | S | key child read via child_type_node, then keyed-map verdict from the algebra container name: a structure/kind question (is the settled head a keyed map), so the key and the receiver are settled first. |
| v1_type_expr_keyed_map_verdict | S | key child read via child_type_node, then keyed-map verdict from the algebra container name: a structure/kind question (is the settled head a keyed map), so the key and the receiver are settled first. |
| v1_keyed_map_verdict_from_algebra | S | key child read via child_type_node, then keyed-map verdict from the algebra container name: a structure/kind question (is the settled head a keyed map), so the key and the receiver are settled first. |
| v1_item_field_type_exprs | S | returns field type exprs fed to the S walkers; field wrappers are rewritten to authored refs, so the walkers settle. Pass-through itself, class follows its only consumers. |
| v1_enum_variant_field_type_exprs | S | returns field type exprs fed to the S walkers; field wrappers are rewritten to authored refs, so the walkers settle. Pass-through itself, class follows its only consumers. |
| v1_item_alias_hop_type_exprs | S | a bespoke one-hop settle of an alias item's RHS (no Rust checkpoint) so aliases of Clone/Ord-relevant types join the walk; still needed for alias identity arriving from fields, replace the hop by settled_type on the authored field ref (List 2). |
| v1_emit_struct_from_capability_table | S | feeds field type exprs from the item to the S walkers and selects derive/impl rows from the answers; no own argument read. |
| v1_emit_enum_supplemental_impls | S | feeds field type exprs from the item to the S walkers and selects derive/impl rows from the answers; no own argument read. |
| v1_type_expr_is_bare_param | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_type_expr_contains_param_name | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_type_expr_all_names | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_map_key_nested_names_in_type_expr | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_map_key_head_names_in_type_expr | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_freemonoid_element_params | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_set_element_params | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_freemonoid_param_in_fields | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_item_own_set_affected_param_names | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_ord_propagated_zip_loop | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_field_type_expr_ord_propagated_for_param | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_item_ord_propagated_param_names | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_fn_signature_set_element_param_names | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_field_type_expr_needs_clone_bound_for_param_narrow | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_item_type_param_needs_clone_bound_struct | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_generic_param_used_as_bare_value_param_type | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_generic_param_used_in_value_param_type_surface | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_type_expr_declared_identity | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_type_expr_map_key_declared_identities | A | asks about the WRITTEN spelling: is this occurrence the type parameter / which names occur / which declaration identity an authored head names. A type-parameter occurrence is an authored fact (a param named T inside an alias argument is not visible to the caller once expanded). Reads authored_name_at + DeclarationRef. |
| v1_wf_child_type_node | A | COMPENSATOR (List 2): re-derives the authored child when the expansion has no authored name (the `!= ""` fallback). After the change it is child_type_node(ch) and the fallback is dead; delete the function and call child_type_node. |
| rust_nominal_identity_carrier_shape_eligible | T | 'is this Symbol': authored name == "Symbol" with no children/params. An alias of Symbol must equal Symbol (type equality), so read settled_type(child_type_node(ch)) before the name test. |
| rust_symbol_wrapped_ord_carrier_shape_eligible | T | 'is this Symbol': authored name == "Symbol" with no children/params. An alias of Symbol must equal Symbol (type equality), so read settled_type(child_type_node(ch)) before the name test. |
| v1_callable_argument_and_return_type_exprs | U | resolved_type(type_expr) is the Callable's RETURN slot (an Arrow/Callable inferred is its codomain), not an alias expansion; unchanged. |
| v1_callable_type_expr_component_type_exprs | U | resolved_type(type_expr) is the Callable's RETURN slot (an Arrow/Callable inferred is its codomain), not an alias expansion; unchanged. |
| v1_arrow_type_expr_mentions_param | U | the .inferred read is the Arrow codomain; unchanged. |
| v1_coproduct_all_variants_nullary | U | tests whether variants have payload children (count), not a type argument. |
| trait_derive_emit_fn_clone_bound_keyed_carrier_module | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_trait_derive_refuse | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_repr_grounding_derive_elem_shape_from_coproduct_children | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_repr_grounding_derive_elem_shape_for_ord_carrier | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_with_map_key_requirement | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_map_key_round_add | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_map_key_fixpoint_loop | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_freemonoid_row_route | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_freemonoid_unroutable_row_refusal | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_freemonoid_hand_written_traits | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_freemonoid_filter_hand_written | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_freemonoid_supplemental_bound_spelling | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_item_header_bound_spellings | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_freemonoid_impl_type_params | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_freemonoid_bare_type_args | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_freemonoid_serde_bound_attr | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_set_unroutable_row_refusal | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_set_hand_written_traits | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_set_filter_hand_written | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_set_required_traits_for | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_set_supplemental_bound_spelling_for | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_set_serde_bound_attr | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_set_serde_bound_attr_for_traits | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_freemonoid_struct_debug_impl | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_freemonoid_struct_partial_eq_impl | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_freemonoid_enum_debug_impl | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_freemonoid_enum_partial_eq_impl | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_set_impl_type_params | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_set_enum_debug_impl | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_set_enum_partial_eq_impl | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_set_element_bound_spellings | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_set_struct_debug_impl | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_set_struct_partial_eq_impl | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_record_derive_traits_heap_for_construction | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_record_derive_traits_copy_for_construction | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_emit_struct_derives | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_emit_enum_derives | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_item_phantom_only_param_names | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_phantom_only_param_names_contains | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_declared_type_app_mentions_param_non_phantom | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_fn_phantom_only_generic_param_names | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_fn_generic_clone_bound_via_referenced_decl | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_type_param_needs_clone_bound | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_fn_param_type_needs_clone_bound | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_generic_params_needing_clone_bound | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_generic_param_cloned_by_returned_closure | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_item_generic_param_name_set | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_declared_type_app_clone_impl_needs_param | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_ambiguous_declared_head_refusal | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_clone_bound_round_add | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_clone_bound_round_for_item | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_clone_bound_fixpoint_loop | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_generic_declared_type_names | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_clone_bounded_type_params | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_clone_impl_required_type_params | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_item_clone_bounded_param_names | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_emit_type_params_with_bounds | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_map_key_bounded_type_params | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_item_map_key_bounded_param_names | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_emit_type_params_with_map_key_bounds | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |
| v1_emit_type_params_with_clone_bounds | U | reads no type argument: derive/impl/bound-spelling emitter, fixpoint round, or roster over names already computed (body greps show no child_type_node/resolved_type/.inferred; .children use is variant payload presence only). |

Rows: 120

### src/v1/04_lookup.dag

| fn | class | reason |
|---|---|---|
| lookup_field_type_node | S | field type of a product; its no-structure arm follows `n.inferred Resolved target` (a hand-rolled expansion). Becomes settled_type(n). |
| lookup_coproduct_common_field_node | U | walks declared fields/variants by name; returns the field node unchanged. |
| lookup_field_in_product | U | walks declared fields/variants by name; returns the field node unchanged. |
| map_lookup_result_type | U | walks declared fields/variants by name; returns the field node unchanged. |
| resolve_scrutinee_type_node_seen | S | IS a hand-rolled settle: walks `Resolved` until structure. Replace by settled_type + preserve_outer_optional_cardinality (List 2). |
| map_value_type_in_env | S | settle the receiver via resolve_scrutinee_type_node, then return raw children; after the change those children are authored arg carriers, so the return is an authored ref and every caller that reads structure must settle it (List 2). |
| map_key_type_in_env | S | settle the receiver via resolve_scrutinee_type_node, then return raw children; after the change those children are authored arg carriers, so the return is an authored ref and every caller that reads structure must settle it (List 2). |
| set_element_type_in_env | S | settle the receiver via resolve_scrutinee_type_node, then return raw children; after the change those children are authored arg carriers, so the return is an authored ref and every caller that reads structure must settle it (List 2). |
| field_summary_for_type | S | summarises the settled type's fields (structure). |
| product_field_result_type | S | `rt.params /> count > 0` is an arrow test; an alias-of-arrow breaks it, so settle rt first. |
| lookup_structural_method | S | head and connective of the receiver decide the method table; caller must pass the settled receiver. |
| declared_arg_types_for_method | S | takes an `expand_field_type` compensator callback (caller 04_infer.dag line 9705 passes expand_type_for_field_access(..).resolved); replaced by settled_type(callable); parameter deleted (List 2). |

Rows: 12

### src/v1/04_patterns.dag

| fn | class | reason |
|---|---|---|
| generic_use_slot_bindings | A | binds a generic use's slots by AUTHORED param spelling; after the change should key on declaration-parameter identity (rule: substitution keys on DeclarationRef parameter identity, not spelling). |
| instantiate_scrutinee_expansion | S | substitutes use-site arguments into the declaration; its `arg.inferred Resolved target` follow for a childless arg becomes child_type_node(arg) (authored) (List 2). |
| expand_scrut_from_decl | S | each re-implements settled_type + parameter substitution for the scrutinee; replace with settled_type(scrut_node) and delete (the decl.inferred alias RHS is declaration-level) (List 2). |
| expand_scrut_from_type_name | S | each re-implements settled_type + parameter substitution for the scrutinee; replace with settled_type(scrut_node) and delete (the decl.inferred alias RHS is declaration-level) (List 2). |
| expand_scrut_type_for_variant_lookup | S | each re-implements settled_type + parameter substitution for the scrutinee; replace with settled_type(scrut_node) and delete (the decl.inferred alias RHS is declaration-level) (List 2). |
| pattern_subject_from_node | S | recursion on `n.inferred Resolved target` only propagates PatternLookupBlocked; replace by a settled-error check (recursing into an authored self-ref is vacuous or non-terminating) (List 2). |
| lookup_variant_in_type | S | variant roster / inhabitance of the settled sum. |
| constructor_roster_for | S | variant roster / inhabitance of the settled sum. |
| is_uninhabited_sum | S | variant roster / inhabitance of the settled sum. |
| peel_never_identity_candidate | T | one-hop manual alias peel for the Never identity check = type equality with Never; becomes settled_type (List 2). |
| node_is_never_declaration | T | is the (settled) node the Never declaration: equality. |
| node_names_never_declaration | A | DeclarationRef / occurrence-identity join. |
| candidate_joins_never_identity | A | DeclarationRef / occurrence-identity join. |
| never_identity_node | U | builder / carrier / pass-through; reads no type argument expansion. |
| is_witness_type_name | U | builder / carrier / pass-through; reads no type argument expansion. |
| synthesize_optional_present_variant | U | builder / carrier / pass-through; reads no type argument expansion. |
| synthesize_witness_holds_variant | U | builder / carrier / pass-through; reads no type argument expansion. |
| synthesize_witness_violates_variant | U | builder / carrier / pass-through; reads no type argument expansion. |
| pattern_subject_from_inferred | U | builder / carrier / pass-through; reads no type argument expansion. |
| node_lookup_resolved | U | builder / carrier / pass-through; reads no type argument expansion. |
| node_lookup_failed | U | builder / carrier / pass-through; reads no type argument expansion. |
| lookup_result_subject | U | builder / carrier / pass-through; reads no type argument expansion. |
| pattern_binding_type | U | builder / carrier / pass-through; reads no type argument expansion. |
| variant_not_found_result | U | builder / carrier / pass-through; reads no type argument expansion. |
| lookup_field_in_variant | U | builder / carrier / pass-through; reads no type argument expansion. |
| find_variant_child_keyed | U | builder / carrier / pass-through; reads no type argument expansion. |
| constructor_fields | U | builder / carrier / pass-through; reads no type argument expansion. |
| resolve_scrutinee_type | U | builder / carrier / pass-through; reads no type argument expansion. |

Rows: 28

### src/v1/04_emit_info.dag

| fn | class | reason |
|---|---|---|
| collect_type_node_import_surface_occurrences | A | ('?' resolved) already walks applied arguments as authored nodes (children_are_applied_arguments) and additionally walks `peeled.inferred Resolved rt`. The import surface of `X<Alias>` must list Alias (the Rust text names it, or its peeled form needs the alias import to be in scope); the expansion's own deps come from the alias declaration. The Resolved walk is List 2 (delete). |
| build_field_type_map | A | records authored field type names and import_surface_names. |
| build_struct_field_summaries | S | field_value_shape_from_type_node: shape of the settled field type. |
| enum_field_type_consistent | T | node_type_equals across variants. |
| build_enum_field_summaries | S | shape (S) plus cross-variant consistency (T) via the two rows above. |
| build_type_summary | S | `rt.connective == Arrow` and field structure: settle before testing. |
| enum_variant_payload_has_fn | S | Arrow-through-alias test on a payload child. |
| enum_variant_payload_surface_names | A | names as authored. |
| type_summary_of_reference | A | names as authored. |
| add_emit_item_summary | S | `alias_rhs = resolved_type(item)` is the alias declaration's RHS (declaration level, unchanged); the variant payload `vc.inferred Resolved rt -> rt.connective == Arrow` is a structure test on a field wrapper and settles (List 2 minor). |
| is_tuple_type | U | connective/name test on a node already in hand. |

Rows: 11

### src/v1/04_items.dag

| fn | class | reason |
|---|---|---|
| inferred_to_outputs | S | `rt.connective`/Conj product test over the inferred result type then `child_type_node(child)` per field: field types become authored refs; the structure test needs the settled rt, the field type_expr stays authored (List 2 minor). |

Rows: 1

### src/v1/05_emit.dag (+04_types helpers it calls)

| fn | class | reason |
|---|---|---|
| render_node_type | S | ('?' resolved) prints the node's authored name and then renders ARGUMENTS via child_type_node (map/set/element/tuple parts via resolved_type(child)). Today every alias argument prints its EXPANSION; the emitted Rust is identical only if the alias is declared and in scope at that spot, which this function cannot decide (closed_alias_peel_verdict peels only zero-param aliases and cross-module aliases need an import). Alias spelling is not needed for identity or equality in Rust, so settle each argument before spelling: bytes preserved. |
| has_nested_records_node | S | product/coproduct structure of value/element child; settle child_type_node(child) before the test. |
| collect_type_names_from_node | A | collects authored names of the written type (ident_span) for import/type-name sets. |
| named_type_vars_in_inferred | U | collects type-variable ids from inferred TypeVariable/Resolved; walks `Resolved rt` into the expansion recursively; after the change an authored arg carries the expansion in its inferred so type variables inside alias bodies are not newly reached, and params are walked directly. Pass-through collector. |
| named_type_vars_in_node | U | collects type-variable ids from inferred TypeVariable/Resolved; walks `Resolved rt` into the expansion recursively; after the change an authored arg carries the expansion in its inferred so type variables inside alias bodies are not newly reached, and params are walked directly. Pass-through collector. |
| transparent_representation_root | T | manual alias peel: follows `n.inferred Resolved rt` and the alias item RHS to a representation root for cast identity (type equality). Becomes settled_type with the existing refinement, optional and grounded-carrier stops kept (List 2). |
| transparent_named_carrier_root | T | alias item hop via lookup_type_for + resolved_type(decl); the second manual peel; folds into settled_type (List 2). |
| cast_source_representation_identical | T | cast source type vs target type equality: `expr.inferred Resolved n` compared to the cast target; both through settled_type. |
| emit_typed_cast_shared | T | cast source type vs target type equality: `expr.inferred Resolved n` compared to the cast target; both through settled_type. |
| emit_typed_for_each_shared | S | for_each_element_type_node(resolved_type(collection)) reads the element child (child_type_node): the collection is settled, the element it returns is an authored ref bound to the loop variable (scope type), so the binder's consumers settle. |
| for_each_element_type_node | S | (04_types/05_emit) normalize_access_type_node(resolved_type(base)) then structure/String/keyed-map tests on the base: settle the base. |
| emit_typed_index_shared | S | (04_types/05_emit) normalize_access_type_node(resolved_type(base)) then structure/String/keyed-map tests on the base: settle the base. |
| emit_typed_slice_shared | S | (04_types/05_emit) normalize_access_type_node(resolved_type(base)) then structure/String/keyed-map tests on the base: settle the base. |

Rows: 13

### src/v1/05_emit_rust.dag (every fn containing child_type_node(, resolved_type(, .inferred, __applied_type_args, and the grouped families rust_*_ord_eligible, type_node_has_*, rust_empty_set_*, emit_*_from_children; 276 rows)

| fn | class | reason |
|---|---|---|
| render_rust_applied_via_emit_info | S | ('?' resolved) HEAD is spelled as authored (A) but ARGUMENTS are recursed through child_type_node, which today yields the alias EXPANSION, so today's Rust text names the expansion for an argument. Emitted Rust can only name the alias if it is a `pub type` in scope; closed_alias_peel_verdict peels only zero-param aliases, alias items from other modules need an import, and ParamAlias takes args. Neither alias identity nor equality is needed in Rust (Rust expands aliases), so settle each argument before spelling to keep bytes. The `__applied_type_args` arm is dead (delete). |
| render_rust_type | S | ('?' resolved) HEAD is spelled as authored (A) but ARGUMENTS are recursed through child_type_node, which today yields the alias EXPANSION, so today's Rust text names the expansion for an argument. Emitted Rust can only name the alias if it is a `pub type` in scope; closed_alias_peel_verdict peels only zero-param aliases, alias items from other modules need an import, and ParamAlias takes args. Neither alias identity nor equality is needed in Rust (Rust expands aliases), so settle each argument before spelling to keep bytes. The `__applied_type_args` arm is dead (delete). |
| render_rust_arrow_position | U | Arrow/Callable keep return in inferred, excluded from child_type_node; decides position only. |
| rust_type_node_is_arrow | U | reads connective only. |
| render_rust_arrow_type | U | builds fn type from params/inferred return; not via child_type_node. |
| render_rust_arrow_type_in_scope | U | same, arrow params and return; no alias-bearing children read. |
| rust_overlayless_alias_leaf_requires_peel | A | keyed on authored leaf name vs env binding (closed alias); identity question. Overlay-named; remains name-keyed. |
| render_rust_type_without_applied_binding | A | spells from n.name and arg children; names alias for zero-param, expansion for applied today; after, args authored so alias args are named. Dead __applied_type_args branch deleted. |
| rust_type_is_rc_wrapped | U | name-keyed table lookup. |
| is_host_freemonoid_vec_alias | U | name-keyed predicate. |
| is_host_diagnostics_carrier_alias | U | name-keyed predicate. |
| declaration_is_grounded_coproduct_native_alias | U | declaration-level, no type node children. |
| every_declarer_is_grounded_coproduct_native_alias | U | declaration index scan. |
| name_realizes_as_native_alias | U | name/import lookup. |
| summary_declares_grounded_coproduct_native_alias | U | summary field read. |
| rust_declaration_realizes_as_native_alias | U | declaration-keyed. |
| is_host_optional_carrier_type | S | decides carrier-ness by structure of the type (optional element); must see through a transparent alias to the carrier. Reads n.name + children; use settled_type. |
| is_host_diagnostics_carrier_type | S | same: kind of carrier decided on settled head. |
| render_rust_diagnostics_carrier_applied | U | builder from shared_types only. |
| render_rust_optional_carrier_inner | A | spells element via authored arg; spelling. |
| render_rust_optional_carrier_applied | A | spelling of authored arg. |
| render_rust_text_carrier | U | constant builder. |
| map_literal_key_is_string | S | inspects key child structure/kind (String key); alias of String must settle. |
| rust_opaque_kernel_alias_type_eligible | U | name-keyed. |
| rust_opaque_kernel_alias_carrier | U | name-keyed. |
| rust_opaque_kernel_alias_type_decl | U | name/module builder. |
| rust_ground_opaque_kernel_type_name | U | name-keyed. |
| rust_scalar_checkpoint_reference_base | U | decision-keyed. |
| rust_scalar_checkpoint_grounding_base | U | decision-keyed. |
| rust_scalar_checkpoint_spelling | U | decision-keyed spelling. |
| type_reference_provenance_in_env | A | resolves authored name via env lookup to provenance; identity of the written reference. |
| type_reference_realization_in_env | A | same, name-keyed realization. |
| rust_render_checkpoint_scalar_bare | S | decides scalar-ness of n; a transparent alias of a scalar must be seen as scalar for arity-0 strip. Reads n.name only today; needs settled head. |
| rust_host_text_carrier_elem_name | S | reads element child to decide Char/String carrier; kind question, settle. |
| is_host_text_carrier_type | S | structure/kind of text carrier. |
| rust_seed_host_freemonoid_empty | U | name-keyed. |
| rust_named_type_base | U | decision-keyed. |
| rust_applied_type_base | U | decision-keyed. |
| rust_witness_parent_leaf | U | name-keyed. |
| rust_peel_one_rc_type_node | S | peels Rc wrapper by child structure; alias to shared type must settle. |
| rust_peel_all_rc_type_node | S | loops peel_one. |
| rust_witness_type_arg_admit_rendered | U | operates on rendered string. |
| rust_witness_applied_type_arg_node | A | selects the authored arg to spell in witness; spelling. |
| rust_witness_carrier_from_type_node | A | spells carrier from type node; identity. |
| rust_witness_type_arg_render | A | renders arg spelling. |
| rust_witness_type_arg_from_holds_value_field | A | spells field type arg. |
| rust_witness_carrier_from_witness_headed_type_node | A | spelling from authored head. |
| rust_witness_type_arg_from_expected_type | A | spelling. |
| rust_witness_type_arg_for_variant | A | spelling of resolved_type arg; authored. |
| rust_witness_variant_ctor_path | U | string builder. |
| rust_witness_ctor_refusal_expr | U | string builder. |
| is_parametric_opaque_type_by_name | U | name-keyed. |
| is_parametric_opaque_type_base | U | name-keyed. |
| closed_alias_verdict_do_not_peel | U | constant. |
| closed_alias_peel_verdict | A | looks up authored name binding and param count; identity of the authored alias. |
| closed_alias_peels_zero_param | A | wrapper of verdict. |
| alias_decl_arity_verdict | U | declaration arity. |
| alias_decl_arity_refusal_expr | U | refusal builder. |
| type_param_is_collection_element_in_values | S | walks value variant child structure for collection element use; kind question. |
| is_value_variant_type_arg | U | name-in-list. |
| type_leaf_name_for_collapse | A | reads authored leaf name. |
| emit_width_type_arg_identity_refusal | A | refusal about identity of authored arg. |
| rust_type_arg_identity_spelling | A | explicit identity/spelling of arg; wants authored name. |
| type_node_has_value_variant_arg | S | child structure scan (value variant arg detection); settle. |
| render_rust_phantom_opaque_applied_type_arg | A | spelling. |
| render_rust_phantom_opaque_applied_decl_arg | A | spelling. |
| render_rust_applied_type_arg | A | spelling of arg; names alias if authored (applied alias args now no longer dropped). |
| render_rust_applied_type | S | ('?' resolved) HEAD is spelled as authored (A) but ARGUMENTS are recursed through child_type_node, which today yields the alias EXPANSION, so today's Rust text names the expansion for an argument. Emitted Rust can only name the alias if it is a `pub type` in scope; closed_alias_peel_verdict peels only zero-param aliases, alias items from other modules need an import, and ParamAlias takes args. Neither alias identity nor equality is needed in Rust (Rust expands aliases), so settle each argument before spelling to keep bytes. The `__applied_type_args` arm is dead (delete). |
| render_rust_shared_type_if_needed | U | name/string keyed. |
| rust_carrier_realizes_as_machine_scalar | S | kind question: is the carrier a machine scalar; settle. |
| rust_carrier_is_at_shared_layer | S | structural layer question. |
| render_rust_shared_type_with_optional | A | spelling wrap. |
| render_rust_applied_type_shared | S | ('?' resolved) HEAD is spelled as authored (A) but ARGUMENTS are recursed through child_type_node, which today yields the alias EXPANSION, so today's Rust text names the expansion for an argument. Emitted Rust can only name the alias if it is a `pub type` in scope; closed_alias_peel_verdict peels only zero-param aliases, alias items from other modules need an import, and ParamAlias takes args. Neither alias identity nor equality is needed in Rust (Rust expands aliases), so settle each argument before spelling to keep bytes. The `__applied_type_args` arm is dead (delete). |
| render_rust_decl_type | S | ('?' resolved) HEAD is spelled as authored (A) but ARGUMENTS are recursed through child_type_node, which today yields the alias EXPANSION, so today's Rust text names the expansion for an argument. Emitted Rust can only name the alias if it is a `pub type` in scope; closed_alias_peel_verdict peels only zero-param aliases, alias items from other modules need an import, and ParamAlias takes args. Neither alias identity nor equality is needed in Rust (Rust expands aliases), so settle each argument before spelling to keep bytes. The `__applied_type_args` arm is dead (delete). |
| rust_fn_sig_preserves_authored_alias_leaf | A | explicitly about preserving authored alias leaf. |
| rust_carrier_optional_wrap | U | string wrap. |
| render_rust_fn_sig_type | S | ('?' resolved) HEAD is spelled as authored (A) but ARGUMENTS are recursed through child_type_node, which today yields the alias EXPANSION, so today's Rust text names the expansion for an argument. Emitted Rust can only name the alias if it is a `pub type` in scope; closed_alias_peel_verdict peels only zero-param aliases, alias items from other modules need an import, and ParamAlias takes args. Neither alias identity nor equality is needed in Rust (Rust expands aliases), so settle each argument before spelling to keep bytes. The `__applied_type_args` arm is dead (delete). |
| render_rust_fn_sig_type_applied_binding | A | sig spelling; dead __applied_type_args branch deleted. |
| rust_decl_type_container_arg_needs_resolved_overlay | U | compensator, deleted (overlay module removed). |
| render_rust_decl_type_container_arg | U | compensator, deleted; callers pass arg directly. |
| alias_rhs_container_arg | S | decides collection-ness of an alias RHS arg from its resolved expansion; becomes settled_type call. |
| rust_alias_rhs_applied_container_or_base | A | spells alias RHS head. |
| render_rust_alias_rhs_applied_arg | A | spelling. |
| render_rust_alias_rhs_type | S | ('?' resolved) HEAD is spelled as authored (A) but ARGUMENTS are recursed through child_type_node, which today yields the alias EXPANSION, so today's Rust text names the expansion for an argument. Emitted Rust can only name the alias if it is a `pub type` in scope; closed_alias_peel_verdict peels only zero-param aliases, alias items from other modules need an import, and ParamAlias takes args. Neither alias identity nor equality is needed in Rust (Rust expands aliases), so settle each argument before spelling to keep bytes. The `__applied_type_args` arm is dead (delete). |
| type_variable_node | U | builder. |
| is_type_variable | U | inferred check. |
| type_var_in_fn_generic_scope | U | name-in-list. |
| rust_read_rendered_type_ident | U | string scan. |
| rust_fold_rendered_type_has_spurious_generic_atom | U | string. |
| rust_fold_rendered_type_has_spurious_from_pos | U | string. |
| rust_fold_rendered_type_has_any_spurious_generic | U | string. |
| type_leaf_is_unbound_in_closure_scope | U | name-keyed. |
| type_node_has_closure_unbound_generic_atom | S | walks child structure to find unbound generic atoms; drop Resolved-walk arm, walk authored args (alias RHS is a separate declaration). |
| type_node_has_unbound_type_variable | S | same; drop Resolved recursion. |
| is_rust_value_type | S | kind question on head/children. |
| rust_nominal_ord_derives_for_shape | U | shape-keyed. |
| rust_nominal_ord_type_decl_ord_eligible | S | inhabitance of decl fields. |
| rust_nominal_ord_type_ref_eligible | S | element ref eligibility via structure; settle. |
| rust_nominal_ord_type_eligible | S | same. |
| resolve_wire_serde_policy_for_coproduct_seen | U | Reads the declaring item's own children by variant name; no type argument. |
| resolve_emission_coproduct_wire_policy | U | Same: policy lookup by declaration identity. |
| build_data_variant_wire_spellings | U | Spells variant names, not types. |
| collect_phantom_zst_markers | A | Names the marker type as written; emitted Rust names the authored marker today. |
| has_complex_variants | U | Counts variant payload presence only. |
| is_simple_disj | U | Counts variant payload presence only. |
| emit_inferred_type_leaf_name | A | Identity/spelling read of the leaf name; must stay the authored reference. |
| anonymous_record_lit_surface_name | A | Names the record literal's type as authored. |
| record_lit_field_type_hints | A | Hint type is spelled into emitted Rust; emitted Rust names the authored alias there. |
| record_lit_resolved_ctor_import_names | A | Import identity: the imported name is the authored one. |
| collect_value_ref_names | A | Identity read for import collection. |
| collect_item_type_surface_names | A | Identity read; collects names as authored. |
| collect_item_type_surface_occurrences | A | Same, occurrence grain. |
| collect_value_emit_type_surface_names | A | Identity read for import surface. |
| collect_item_realized_surface_names | A | Identity read for import surface. |
| local_coproduct_variant_names | U | Variant names of a local declaration. |
| find_variant_parent_in_module | U | Looks up a declaration by variant name. |
| type_item_has_rust_nominal_shell_authority | A | Decides on the declared item's authored reference. |
| alias_rhs_nominal_shell_has_rust_authority | A | Reads the alias RHS as authored (the PR's alias_rhs). |
| is_self_referential_opaque_type_resolved | A | Compares a name to the declaration's own name: identity. |
| is_parametric_opaque_type_decl_item | U | Reads declaration parameter count. |
| is_emittable_parametric_type_alias_item | A | Alias RHS as authored decides emittability. |
| import_name_resolves_to_host_realized_kernel_scalar | A | Name identity lookup against a host-realized table. |
| emit_typed_item | U | Dispatch on item kind; no argument read. |
| needs_box_wrapping | A | Compares a field type name to the enclosing type's name (self-reference): identity. |
| is_function_type_param | A | Name identity of a type parameter. |
| emit_item_clone_bound_refusal | A | Clone bound is by type-parameter name. |
| emit_type_def_from_connective | U | Pass-through to the struct/enum emitters. |
| type_node_mentions_name | A | Identity: asks whether a name occurs as authored. |
| render_rust_type_with_applied_binding_in_scope | U | ('?' resolved) thin delegate to render_rust_type; its find_property("__applied_type_args") Present arm is dead (nothing writes it), delete the arm and keep the Absent delegation. |
| render_rust_field_type_with_applied_binding | S | ('?' resolved) live arm renders resolved_type(field) / tests is_product_type(rt_child): structure of the settled field; the three-way `__applied_type_args` check (field, authored type expr, rt_child) is dead. Settle the field type, delete the overlay arms. |
| emit_struct_field_from_child | S | ('?' resolved) live arm renders resolved_type(field) / tests is_product_type(rt_child): structure of the settled field; the three-way `__applied_type_args` check (field, authored type expr, rt_child) is dead. Settle the field type, delete the overlay arms. |
| find_shared_enum_fields | A | Field type identity across variants; compares names. |
| emit_enum_shared_accessors | A | Renders the shared field's type name as authored. |
| enum_variant_field_type_node | U | Lookup; returns the node unchanged. |
| render_variant_payload_type | S | ('?' resolved) live arm renders resolved_type(field) / tests is_product_type(rt_child): structure of the settled field; the three-way `__applied_type_args` check (field, authored type expr, rt_child) is dead. Settle the field type, delete the overlay arms. |
| emit_variant_from_child | U | Dispatch; the overlay check at its `__applied_type_args` read is dead. |
| v1_fn_body_rc_match_clone_bound_param_names | A | Parameter-name identity. |
| v1_fn_body_equality_bound_param_names | A | Parameter-name identity. |
| v1_resolved_type_name_or_empty | A | Reads the authored name; the name reads a reference, not an expansion. |
| v1_call_forwarding_forwarded_param_names | U | Forwards parameter names only. |
| v1_callee_clone_bound_param_names | U | `resolved_type(callee_item)` is the fn item's own return type, passed through to bound analyses; reads no argument child |
| v1_arrow_arg_type_names | A | reads `authored_name_at` of arrow param type exprs: slot names, spelling |
| v1_fn_result_record_lits | U | walks expression children (record literals), not type nodes |
| v1_fn_result_fn_value_clone_forwarded_param_names | A | `resolve_type_decl_reference(ret)` identity; `ret.children` only fed to v1_record_decl_slot_arg_name (authored name of arg wrapper); `child_type_node(decl_field)` is an Arrow field so it returns `ch` unchanged (connective Arrow guard) |
| v1_call_forwarding_static_bound_param_names | U | expression walk |
| v1_call_site_static_forwarded_param_names | U | `resolved_type(callee_item)` only as head name for `type_has_fn_fields`; no argument read |
| v1_call_forwarding_bound_param_names | U | expression walk |
| v1_fn_map_key_bound_table | U | passes `resolved_type(item)` (fn return) to v1_fn_signature_map_key_bound_names; reads nothing itself |
| inferred_expr_is_optional | U | `return_cardinality` of the expression type |
| emit_rust_unit_discarding_optional_typed | U | expression walk, record-literal fields |
| emit_rust_unit_discarding_stmt | U | expression walk |
| emit_func_body | U | `extend_scope(resolved: resolved_type(v))` binds the whole type; no argument read |
| render_rust_param_sig_type | U | whole param type to render_rust_fn_sig_type, which owns argument reading (outside range) |
| emit_rust_param_type | U | Arrow return head (`n.inferred` of an Arrow is its return type, not an alias expansion) and delegates |
| needs_reference_node | U | head only: connective, `children /> count > 0`, is_rust_value_type/is_product_type of the param type; top-level type is not an argument wrapper |
| positional_payload_scrut_type | U | authored name of binding type head (`normalize_access_type_node`), no argument read |
| analyze_rc_match | U | cardinality and shared-layer head of scrutinee type |
| field_binding_with_pattern | U | record copy of a Node |
| explicit_record_struct_name | A | returns authored name of the record type (`nname`) when product/ident_span set; `children /> first` used only for the annotated-wrapper base, a head |
| effective_variant_parent | U | head of resolved scrutinee via effective_variant_parent_from_resolved |
| emit_none_keyword_for_resolved_type | U | cardinality only |
| emit_var_ref | U | binding kind and head |
| emit_typed_expr_base | U | expression dispatch |
| field_access_field_is_boxed | S | `resolved_type(field_child)` is a field wrapper's Resolved: structure question (`needs_box_wrapping` reads fields/recursion) and `type_reference_realization(field_ty, name)` host-scalar question: both are about the emitted Rust type, which expands the alias. Rust today gets the expansion; after the change the authored alias would make `needs_box_wrapping` and the host-numeric test see the alias name |
| type_expr_reaches_sealed_carrier | A | seal is a declaration property: `resolve_type_decl_reference(n)` by declaration identity; children recursed through `child_type_node`; an authored alias reference now resolves to the alias declaration whose member is reached by decl_member_type_nodes' alias arm, so no settle is needed |
| member_type_nodes | A | returns each field/variant-payload type carrier unread; consumers decide by identity |
| decl_member_type_nodes | A | alias arm reads `resolved_type(decl)` = alias declaration RHS (declaration-level, unchanged); else member_type_nodes |
| declaration_item_identity | A | declaration identity of a decl item |
| emit_typed_field_access_as | U | head/cardinality of base type, anonymous-record field enumeration (fields, not arguments); delegates structure to field_access_field_is_boxed (S) and type_reference_seal_reading |
| type_needs_rc | U | wrapper over type_needs_rc_seen |
| type_needs_rc_seen | S | (not on the doc's List 2) follows `resolved_type(normed)` along `normed.inferred` through alias expansions to reach product/coproduct structure with no seen-check; once args hold the authored ref the chain recurses on an authored self-ref. Becomes settled_type with a DeclarationRef-keyed seen set. |
| rust_empty_map_value_type_str | S | ('?' resolved) renders child_type_node(value/key child) via render_rust_type; same bytes need the settled child. |
| rust_empty_map_kv_type_str | S | ('?' resolved) renders child_type_node(value/key child) via render_rust_type; same bytes need the settled child. |
| type_node_carries_erased_type_argument | S | recursion `type_node_carries_erased_type_argument(t)` walks the children of `child_type_node(c)`: today the expansion's children, so an alias whose body contains a CompilerError argument is found. After: the alias has no children; must walk `settled_type(t).args`. The direct test `t.inferred` is CompilerError stays a whole-node read |
| type_node_child_is_type_variable | U | `child_type_node(c).inferred` TypeVariable: a type variable is never an alias; decided by resolve_node_bounded's childless n_is_type_var arm (04_resolve census: UNAFFECTED, keeps `inferred: TypeVariable` on the node), which the authored reference node still carries |
| type_node_carries_applicationless_parametric_head | A | asks "was a parametric head written without arguments" of each authored argument; its doc says the defect was `child_type_node` projecting an alias to a declaration node that dropped RHS args. The authored reference removes that false positive; the true bare-head case remains an authored-identity question (compensating part of the comment/dissolution is dead, the function is kept) |
| rust_type_renders_bare_parametric_head | A | `lookup_type_for(authored node)` arity vs rendered head spelling: identity of the declaration; an authored zero-param alias is not parametric so it answers false (the old case where the expansion lost args cannot arise) |
| type_node_declares_parameters | A | binding params count of the authored node's declaration |
| rust_empty_map_init_expr | U | composes the four checks above and `map_type.children /> any(type_node_child_is_type_variable)` |
| rust_btree_set_element_ord_eligible | S | name roster (String/Int/...) on `elem_node` plus childless gate and rust_nominal_ord_type_eligible: Ord eligibility is a property of the emitted Rust element type, which expands an alias; an authored alias name misses the roster. Settle `elem_node` first (the callers pass `child_type_node(...)` of a Set element) |
| rust_empty_set_element_type_str | S | Ord eligibility plus render_node_type of the element: both settled; the type-var/error test is whole-node |
| emit_rust_empty_set_expr | S | same eligibility; the refusal message prints the element name (authored, acceptable spelling change, would alter refusal text for alias elements) |
| emit_rust_expr_var | U | binding dispatch |
| emit_rust_expr_call | U | dispatch |
| emit_rust_expr_method_call | U | dispatch |
| emit_rust_expr_record_lit | U | head of the record literal's own type via expand_type_for_field_access, name/Conj/ident_span; no argument read |
| emit_rust_expr_string_interp | U | expression children |
| emit_rust_expr_block | U | expression children |
| emit_typed_expr | U | dispatch |
| rust_list_lit_holds_callables | U | expression children (is_lambda_expr) |
| rust_list_lit_callable_element_type | S | tests `child_type_node(first child).connective == Arrow`; an alias of an arrow type expands to Arrow today and would read NoConnective authored: kind question, settle (then render the settled arrow) |
| is_map_typed_expr | U | head of whole expression type (node_is_keyed_collection) |
| is_collection_typed_expr | U | head of whole expression type |
| discriminant_zero_field_variant_tag | U | expression record literal |
| emit_discriminant_call_lowering | U | dispatch |
| emit_discriminant_call_scrutinee_lowering | U | `resolved_type_name(arg)` head name, then declaration `lookup_type_by_name(...).children` are variants |
| emit_typed_call_expr | U | passes `Resolved ret_type` to rust_empty_map_init_expr / emit_rust_empty_set_expr (the S readers) |
| rust_param_type_is_type_variable | U | childless/paramless single-letter name on the declared param type head |
| rust_call_arg_is_function_value | U | lambda expr or head Arrow of whole arg type |
| rust_call_arg_fail_closed_unwrap | U | cardinality of whole arg type and param type |
| lambda_argument_scope | U | builds scope from lambda param nodes |
| emit_typed_call | U | `authored_name_at(resolved_type(base_arg))` head name for `with`; `render_rust_type(ret_type)` delegates arguments |
| emit_typed_for_each | U | element type from `for_each_element_type_node` (04_types U pass-through) bound as the loop variable's type, like an annotated binding; consumers settle |
| emit_typed_index | U | head of normalized base type (string-like / keyed) |
| emit_typed_slice | U | head of normalized base type |
| collection_element_type | S | `child_type_node(first child)` of the receiver collection type: error/type-var tests are whole-node, but type_node_has_closure_unbound_generic_atom (outside range) walks the node and render_rust_type renders it; emitted Rust names the expansion of an alias element today, so settle before walk and render |
| lambda_scope_from_children | U | binds `pn.inferred` whole type |
| lambda_param_type_strs | U | param type is a lambda param's whole inferred type (head error/type-var tests); argument reading is delegated to type_node_has_closure_unbound_generic_atom and render_rust_type |
| emit_typed_collection_lambda | U | delegates |
| emit_typed_fold_lambda | U | delegates |
| emit_rust_fold_method_call | S | `acc_type.children /> any(c => is_unit_like(c) // c.ident_span == none)` and the grandchild form decide "under-resolved" (inhabitance: an alias of unit is unit-like; `ident_span == none` is an anonymous-expansion tell that an authored reference never has) plus `acc_type_node.children /> any(type_node_child_is_type_variable)`; acc rendered through render_rust_type. Settle each argument before `is_unit_like`; keep `type_node_child_is_type_variable` whole-node |
| emit_rust_effectful_fold_loop | U | expression/lambda walk, acc string passed in |
| rust_expr_reaches_awaited_call | U | expression children |
| emit_rust_sort_by_method_call | S | element via `for_each_element_type_node`, then `elem.ident_span != none` gate and render_rust_type(elem): the gate (anonymous vs named) and the render are expansion-shaped today; settle the element before the gate and render |
| emit_rust_map_method_call | U | cardinality of receiver and lambda expression children |
| emit_rust_higher_order_method | U | lambda expression children |
| emit_rust_get_method_call | U | `is_map_typed_expr` head and receiver presence |
| emit_rust_with_method_call | U | authored name of receiver record type head |
| emit_typed_method_call | U | `resolved_type(receiver).return_cardinality`: head only. |
| emit_typed_match | U | Scrutinee `.inferred` read for authored name + `CardOptional`; passes `resolved_type(scrutinee)` whole to `optional_match_arm_sees_present_value`/`analyze_rc_match` (other parts classify those). |
| emit_typed_match_arm_body_str | U | Wraps `match_result_type` back into `Resolved{}` for `emit_var_ref`: pass-through. |
| rust_receiver_has_callable_method_field | S | Tests `field_type.connective == Arrow` on a struct field's type: a field typed by an alias of an arrow is an arrow, so settle the field type first. Receiver head name is authored identity. |
| rust_struct_field_type_node | U | Lookup by name, returns `rust_struct_field_type_node_from_container` result unread. |
| rust_record_field_needs_fn_rc | S | Same `connective == Arrow` structure test on the field type across candidates. |
| rust_struct_field_type_node_variant_aware | U | Two-way lookup returning the field node, reads no structure. |
| rust_struct_field_type_node_from_container | U | Returns `resolved_type(field_node)`; today the field type (expansion), after the authored reference. Pass-through, but see List 2: its S consumers must settle. |
| rust_record_field_needs_box | A | `needs_box_wrapping` is keyed on the authored (last-segment) name in `shared_types`/`recursive_types`, then `coerce_primitive_type(type_reference_realization(n))` by declaration, plus `children /> count` and optional cardinality: identity/head. Rendered_use_site keeps the authored name today, so the answer is unchanged. |
| wrap_rust_record_field_value | U | Composes the two predicates above plus builders; reads no node. |
| is_already_optional | U | Reads `return_cardinality` of expression/base types and variant names only; `field_access_base.inferred` is used for the base's authored name. |
| lookup_struct_field_type_name | A | Returns `authored_name_at(resolved_type(field_child))` guarded by `ident_span`/type-variable: the authored name of the field type. Today `rendered_use_site_type` already keeps the authored name, so identity is the same after. Only the type-variable test reads `field_type.inferred` head. |
| emit_field_value_with_context | S | `nested_rt = expand_type_for_field_access(resolved_type(field_value)).resolved` is an explicit expansion of the literal's type used as `struct_node` (its field children are read next); `expand_type_for_field_access` is the settle. `variant_belongs_to_enum(expected_type)` uses an authored name (A part). |
| record_field_expected_type | U | Looks up the field type node and returns it; no structure read. |
| ambiguous_anonymous_record_literal_diagnostics | U | `is_product_type(resolved)` and `ident_span == none` on the literal's own anonymous record type; hints use `record_lit_field_type_hints` (authored names, 04 part). |
| emit_typed_record_lit | S | Reads the literal's type: authored name/`ident_span`/cardinality (A), then `resolve_node(resolved_type).resolved` and its Conj `children` names to find the canonical struct (structure of a possibly aliased record), and field value types as authored names. The Conj-children read is the settle. |
| emit_stored_record_field | U | Passes `resolved_type` through as `struct_node`; reads none itself. |
| plain_shared_struct_literal_name | S | `expand_type_for_field_access(resolved_type(lit)).resolved` then compares its authored name to `tn`: explicit expansion, becomes `settled_type`. |
| dying_literal_plan | S | Two `expand_type_for_field_access(resolved_type(..)).resolved` calls (outer `rt`, nested `inner_rt`) passed on as record structure to `emit_stored_record_field`; `lit.children` are expression children. |
| rust_operand_realization | U | Hands `resolved_type(operand)` to `_of_type`. |
| rust_operand_alias_or_refinement_continue | A | The hop reads `rt.inferred Resolved r`, tests `is_type_alias_item(r)` and returns `resolved_type(r)` (the alias declaration's RHS). It must cross refinement/brand aliases (`type Char = Int where unicode_scalar, brand("Char")`) so it is NOT `settled_type` (which stops at a brand alias): it is a declaration hop keyed on the DeclarationRef. See List 2. The `where`-refinement arm reads the parser's one-child Conj wrapper (declaration-level, not a type argument). |
| rust_operand_realization_of_type | A | Reads provenance, authored name, `is_container_type(name)`, host-carrier head, `type_reference_declaration_ref` (the DeclarationRef), `child_count` for a facts record; `rt.inferred` only for the type-variable test. All identity. |
| rust_operand_realization_of_stamped_declaration | U | Symbol-index lookup by declaration identity, then calls `_of_type`. |
| expr_realizes_as_refusing_int | A | `normalize_access_type_node(rt)` authored name, `type_reference_realization(n)`, and `normed.declaration` against kernel grounding rows: declaration identity; the doc comment says "by declaration, never spelling". |
| is_optional_typed_expr | U | `return_cardinality` only. |
| is_list_typed_expr | U | `type_node_is_ordered_element_run(rt)`: authored container name, `children /> count`, set/string tests on the expression's own type; no argument read. |
| is_string_typed_expr | U | `is_rust_string_like(inner)` on head after `with_required_cardinality`. |
| is_host_text_typed_expr | U | `is_host_text_carrier_type(inner)` head/provenance. |
| emit_rust_tco_non_self_call | U | Passes `frame.expr.inferred` through to `emit_typed_call`. |
| tco_match_scrut_type_name | U | Authored name + cardinality of the scrutinee type. |
| emit_rust_tco_match | U | Passes `resolved_type(s)` whole to `optional_match_arm_sees_present_value`; `tco_scrut_type` is the head name. |
| emit_rust_tco_let | U | `extend_scope(resolved: resolved_type(v))`: pass-through. |
| emit_operation_method | U | `render_rust_type(resolved_type(op_node))` and `inferred: resolved_type(op_node)` passed on; renders the operation's whole output type (renderer rows own the alias-vs-expansion answer). |
| emit_dry_run_branch_from_props | S | `inferred.connective == Conj && inferred.children /> count > 1` reads the operation output record's fields (structure; an aliased record output must be settled), then `ch.inferred Resolved tn` and `authored_name_at(tn)` as `dag_type_name` per field (A part), and `wire_ty = tn.inferred Resolved rt` of the wire declaration. |
| emit_rest_call | U | `render_rust_type(resolved_type(op_node))`, pass-through to the renderer. |
| emit_rest_http_method | U | Tests only that `m.inferred` is Present; value is the expression name. |
| has_from_key_fields | S | `rt = resolved_type(op_node)`, `rt.connective == Conj`, `rt.children` filtered by `child_from_key`: fields of the output record. |
| operation_response_200_resolved_type | U | Returns the `Resolved` node of the value or a by-name lookup; no structure read. |
| wire_child_for_segment | S | `normalize_access_type_node(wire_node).children` filtered by field name / `from_key`: record fields of the wire type, which may be named through an alias. |
| advance_wire_path_projection | U | `resolved_type(ch)` of the matched field child and `for_each_element_type_node`: returns the node without testing it. |
| emit_wire_struct_path_chain | U | Fold over the above. |
| emit_typed_wire_field_assign | U | Builds a string from the projection. |
| is_json_wire_declaration_type | A | `authored_name_at(normalize_access_type_node(t))` compared to the JSON declaration names: identity. |
| emit_from_key_extraction | S | Same as `has_from_key_fields`/dry-run: `rt.children` of the output record, plus `wire_ty = tn.inferred Resolved rt` for the wire declaration; field `dag_type_name` via authored name (A part). |
| argv_follow_inferred | U | Returns `Resolved` node or the annotation; no test. |
| argv_element_type_after_alias_hop | U | Starts `hop_argv_element_type` from the argument's `Resolved`. |
| hop_argv_element_type | S | Decides whether an argv element type is an ordered-element run/string-like by hopping alias declarations (via `rust_operand_alias_or_refinement_continue` and `resolve_type_decl_reference`): a structure question answered by a hand-rolled alias settle. Keep the declaration-keyed hop (it must cross brand aliases like `NonEmptyStr`); no `child_type_node` read. |
| shell_argv_element_is_word_list | U | `type_node_is_ordered_element_run` on the hop result. |
| shell_argv_element_is_string_word | U | `is_rust_string_like` on the hop result. |
| emit_capability_method | S | Anonymous Conj output (`ident_span == none`): `out.children /> map(f => render_rust_fn_sig_type(n: child_type_node(f)))`. Today each field's expansion is rendered; to keep bytes identical settle the field type before rendering (or hand the authored reference to a renderer that peels transparent aliases, `closed_alias_peel_verdict`). |
| emit_data_def | A | Spelling/brand of the annotation: `ann_name`, `annotation_type_node.children /> count`, `preserves_declared_brand`, named-refinement test; the fallback `value.inferred Resolved rt` is the expression's whole type given to the renderer. |
| value_inferred_type_is_rc_wrapped | U | `rust_carrier_is_at_shared_layer(rt)`: head. |
| emit_data_def_body | U | Passes `value.inferred` to `effective_variant_parent` and calls the predicate above. |
| rust_test_signature_comment | A | `emit_node_type(projection.inferred)` into a doc comment: spelling only, nothing compiled; the generic printer emits the authored name today. |
| render_rust_type_with_applied_binding | U | delegate. |
| emit_struct_from_children | U | dispatch to emit_struct_field_from_child / emit_variant_from_child / derive selection; no own type read. |
| emit_enum_from_children | U | dispatch to emit_struct_field_from_child / emit_variant_from_child / derive selection; no own type read. |
| rust_field_carrier_final_type | A | decides Box/Rc wrapping from the field node's name and type_reference_realization on the authored reference; wrapper shape is identity of the carrier. |

Rows: 276

### Resolved '?' rows (doc lines 335-380)

| doc row | final | decided by |
|---|---|---|
| collect_type_node_import_surface_occurrences | A | ('?' resolved) already walks applied arguments as authored nodes (children_are_applied_arguments) and additionally walks `peeled.inferred Resolved rt`. The import surface of `X<Alias>` must list Alias (the Rust text names it, or its peeled form needs the alias import to be in scope); the expansion's own deps come from the alias declaration. The Resolved walk is List 2 (delete). |
| render_node_type | S | ('?' resolved) prints the node's authored name and then renders ARGUMENTS via child_type_node (map/set/element/tuple parts via resolved_type(child)). Today every alias argument prints its EXPANSION; the emitted Rust is identical only if the alias is declared and in scope at that spot, which this function cannot decide (closed_alias_peel_verdict peels only zero-param aliases and cross-module aliases need an import). Alias spelling is not needed for identity or equality in Rust, so settle each argument before spelling: bytes preserved. |
| render_rust_type | S | ('?' resolved) HEAD is spelled as authored (A) but ARGUMENTS are recursed through child_type_node, which today yields the alias EXPANSION, so today's Rust text names the expansion for an argument. Emitted Rust can only name the alias if it is a `pub type` in scope; closed_alias_peel_verdict peels only zero-param aliases, alias items from other modules need an import, and ParamAlias takes args. Neither alias identity nor equality is needed in Rust (Rust expands aliases), so settle each argument before spelling to keep bytes. The `__applied_type_args` arm is dead (delete). |
| render_rust_decl_type | S | ('?' resolved) HEAD is spelled as authored (A) but ARGUMENTS are recursed through child_type_node, which today yields the alias EXPANSION, so today's Rust text names the expansion for an argument. Emitted Rust can only name the alias if it is a `pub type` in scope; closed_alias_peel_verdict peels only zero-param aliases, alias items from other modules need an import, and ParamAlias takes args. Neither alias identity nor equality is needed in Rust (Rust expands aliases), so settle each argument before spelling to keep bytes. The `__applied_type_args` arm is dead (delete). |
| render_rust_fn_sig_type | S | ('?' resolved) HEAD is spelled as authored (A) but ARGUMENTS are recursed through child_type_node, which today yields the alias EXPANSION, so today's Rust text names the expansion for an argument. Emitted Rust can only name the alias if it is a `pub type` in scope; closed_alias_peel_verdict peels only zero-param aliases, alias items from other modules need an import, and ParamAlias takes args. Neither alias identity nor equality is needed in Rust (Rust expands aliases), so settle each argument before spelling to keep bytes. The `__applied_type_args` arm is dead (delete). |
| render_rust_applied_type | S | ('?' resolved) HEAD is spelled as authored (A) but ARGUMENTS are recursed through child_type_node, which today yields the alias EXPANSION, so today's Rust text names the expansion for an argument. Emitted Rust can only name the alias if it is a `pub type` in scope; closed_alias_peel_verdict peels only zero-param aliases, alias items from other modules need an import, and ParamAlias takes args. Neither alias identity nor equality is needed in Rust (Rust expands aliases), so settle each argument before spelling to keep bytes. The `__applied_type_args` arm is dead (delete). |
| render_rust_alias_rhs_type | S | ('?' resolved) HEAD is spelled as authored (A) but ARGUMENTS are recursed through child_type_node, which today yields the alias EXPANSION, so today's Rust text names the expansion for an argument. Emitted Rust can only name the alias if it is a `pub type` in scope; closed_alias_peel_verdict peels only zero-param aliases, alias items from other modules need an import, and ParamAlias takes args. Neither alias identity nor equality is needed in Rust (Rust expands aliases), so settle each argument before spelling to keep bytes. The `__applied_type_args` arm is dead (delete). |
| render_rust_type_with_applied_binding_in_scope | U | ('?' resolved) thin delegate to render_rust_type; its find_property("__applied_type_args") Present arm is dead (nothing writes it), delete the arm and keep the Absent delegation. |
| render_rust_field_type_with_applied_binding | S | ('?' resolved) live arm renders resolved_type(field) / tests is_product_type(rt_child): structure of the settled field; the three-way `__applied_type_args` check (field, authored type expr, rt_child) is dead. Settle the field type, delete the overlay arms. |
| emit_struct_field_from_child | S | ('?' resolved) live arm renders resolved_type(field) / tests is_product_type(rt_child): structure of the settled field; the three-way `__applied_type_args` check (field, authored type expr, rt_child) is dead. Settle the field type, delete the overlay arms. |
| render_variant_payload_type | S | ('?' resolved) live arm renders resolved_type(field) / tests is_product_type(rt_child): structure of the settled field; the three-way `__applied_type_args` check (field, authored type expr, rt_child) is dead. Settle the field type, delete the overlay arms. |
| rust_empty_map_value_type_str | S | ('?' resolved) renders child_type_node(value/key child) via render_rust_type; same bytes need the settled child. |
| rust_empty_map_kv_type_str | S | ('?' resolved) renders child_type_node(value/key child) via render_rust_type; same bytes need the settled child. |

### List 2 conversions

| site | reads now | reads after | forced caller changes |
|---|---|---|---|
| 04_emit_rust: every `__applied_type_args` Present arm (render_rust_type, render_rust_type_without_applied_binding, render_rust_type_with_applied_binding_in_scope, render_rust_field_type_with_applied_binding, emit_struct_field_from_child three-way check, emit_variant_from_child, render_rust_decl_type_container_arg, needs_resolved_overlay) | find_property(props, "__applied_type_args") on node / authored type expr / rt_child; never Present because nothing writes the property | deleted (overlay dead); render_rust_decl_type_container_arg and needs_resolved_overlay deleted whole | none for the delete; callers of the two deleted fns pass the arg straight to render_rust_type |
| 04_lookup resolve_scrutinee_type_node_seen | hand-rolled walk of `inferred Resolved` until structure | settled_type(n) then preserve_outer_optional_cardinality | map_/set_ type_in_env callers; seen-guard moves to settled_type |
| 04_lookup map_value_type_in_env / map_key_type_in_env / set_element_type_in_env | settle receiver then return raw child (= expansion today) | settle receiver then return the authored arg carrier | every caller reading structure of the result settles: 04_infer index/for-each/method result sites (not read here; enumerate by grep before cut) |
| 04_lookup lookup_field_type_node | no-structure arm `n.inferred Resolved target` | settled_type(n) | lookup_field callers unchanged |
| 04_lookup declared_arg_types_for_method | takes `expand_field_type` callback = expand_type_for_field_access(..).resolved | settled_type(callable); callback param deleted | sole caller 04_infer.dag line 9705 drops the lambda argument; expand_type_for_field_access dead if no other caller |
| 04_lookup product_field_result_type | `rt.params /> count > 0` on cached rt | same test on settled_type(rt) | none |
| 04_patterns instantiate_scrutinee_expansion | `arg.inferred Resolved target` when arg childless | child_type_node(arg) (authored ref) | expand_scrut_* callers |
| 04_patterns expand_scrut_from_decl / expand_scrut_from_type_name / expand_scrut_type_for_variant_lookup | each re-implements settled_type + substitution | delete; callers call settled_type(scrut_node) | lookup_variant_in_type, constructor_roster_for, is_uninhabited_sum take the settled scrutinee |
| 04_patterns pattern_subject_from_node | recursion on `n.inferred Resolved target` propagating PatternLookupBlocked | settled-error check on settled_type(n) | none |
| 04_patterns peel_never_identity_candidate | manual one-hop `Resolved target` peel | settled_type; compare DeclarationRef with Never | node_is_never_declaration, candidate_joins_never_identity |
| 04_patterns generic_use_slot_bindings | binds by authored param spelling | binds by declaration-parameter identity | its substitution consumers |
| trait_derive_emit v1_wf_child_type_node | re-derives authored child when expansion lacks an authored name | delete; use child_type_node(ch) | ~20 walkers listed S/A in the trait_derive table rename the call |
| trait_derive_emit v1_item_alias_hop_type_exprs | one-hop alias RHS settle for alias items with no Rust checkpoint | settled_type on the authored field ref (declaration-level alias hop stays where the item itself is an alias) | v1_item_field_type_exprs consumers |
| trait_derive_emit rust_nominal_identity_carrier_shape_eligible / rust_symbol_wrapped_ord_carrier_shape_eligible | authored name == Symbol on child_type_node result (expansion today) | same test on settled_type(child_type_node(ch)) | v1_repr_grounding_derive_elem_shape_for_ord_carrier unchanged |
| 04_emit_info collect_type_node_import_surface_occurrences | also walks `peeled.inferred Resolved rt` (expansion) | walk only the authored node and its authored args; delete the Resolved walk | import sets for aliased arguments now list the alias; verify the alias import implies expansion deps (alias declaration owns them); callers see no signature change |
| 04_emit_info add_emit_item_summary / enum_variant_payload_has_fn | `vc.inferred Resolved rt -> rt.connective == Arrow` | same test on settled_type(vc) | none |
| 04_items inferred_to_outputs | rt = inferred result type; child_type_node(child) as field type_expr | product test on settled_type(rt); field type_expr stays authored ref | field consumers settle on read |
| 05_emit render_node_type + 05_emit has_nested_records_node | child_type_node / resolved_type(child) = expansion | settled_type(child_type_node(child)) before spelling/test | none (bytes preserved) |
| 05_emit transparent_representation_root / transparent_named_carrier_root | manual `Resolved rt` peel then alias item hop via resolved_type(decl) | settled_type(n) with refinement/optional/grounded-carrier stops retained | cast_representation_identical, emit_typed_cast_shared |
| 05_emit emit_typed_for_each_shared / index / slice | resolved_type(collection/base) then element child via child_type_node | settled_type(collection/base); element is an authored ref bound in scope | loop-variable consumers settle |
| 05_emit_rust rust_btree_set_element_ord_eligible caller in render_rust_type_without_applied_binding | `elem_node = child_type_node(elem_child)`; `elem_node.inferred` type-variable test | type-variable test must read the authored ref's own inferred (a TypeVariable arg stays a TypeVariable, not error_type); then settled_type for ord eligibility | rust_btree_set_element_ord_eligible and rust_set_element_ineligibility_reason |

### List 2 rows from the 05_emit_rust range forks

| site | reads now | reads after | forced caller changes |
|---|---|---|---|
| __applied_type_args readers (render_rust_type, render_rust_type_without_applied_binding, render_rust_applied_type, render_rust_fn_sig_type_applied_binding) | property overlay never written anywhere; Present branches dead | deleted | none |
| render_rust_decl_type_container_arg + rust_decl_type_container_arg_needs_resolved_overlay | child_type_node(arg) when overlay admitted | arg itself | delete gunbc.rust_decl_type_overlay, import at line 106, stage0 mirror src_v1_stage0 gunbc_rust_decl_type_overlay.rs, root4_measure_missing_generics_witness use |
| alias_rhs_container_arg | match arg.inferred Resolved rt, collection check | settled_type(arg, source_indices, env, fuel), collection check on result | render_rust_alias_rhs_type passes scope.type_env and fuel |
| type_node_has_closure_unbound_generic_atom / type_node_has_unbound_type_variable | Resolved-walk arm recursing into inferred | arm dropped; authored args walked | none |
| is_host_*_carrier_type, map_literal_key_is_string, rust_peel_*, rust_carrier_* (S rows) | n.name/children as authored-with-expansion-children | settled_type(n) first, then same read | callers need env/fuel; thread scope.type_env where absent |
| render_variant_payload_type (args) | `child_type_node(c)` = resolved expansion for inferred non-Arrow children | `settled_type(child_type_node(c))` | none; keeps bytes |
| render_rust_field_type_with_applied_binding (generic arm) | `resolved_type(field)` | `settled_type(field)` | none |
| emit_struct_field_from_child (structure arm) | `rt_child` from `resolved_type` | `settled_type` of the child | none |
| the three `__applied_type_args` Present arms (8855, 8440-8446, 8487-8491) | dead | delete | none |
| 04_emit_info.dag `collect_type_node_import_surface_occurrences` (outside range) | also recurses into `peeled.inferred Resolved rt` (expansion) | needs both authored and settled passes so imports cover the alias name and any expanded names | its callers see more import names |
| type_needs_rc_seen (not on the note's List 2; found here) | `resolved_type(normed)` along `normed.inferred`: the alias expansion chain to reach product/coproduct structure | `settled_type(normed)` then the declaration body's structure (product/coproduct) via the settled head `DeclarationRef`; recursion guard keyed by DeclarationRef instead of authored `canonical` name; the `else` arm (childless, no inferred) unchanged | callers `type_needs_rc` (all emit sites that decide Rc wrapping) unchanged in signature; needs `env: TypeEnv` (signature currently carries only `source_indices`) so settled_type can run: forces threading env through type_needs_rc/type_needs_rc_seen and their callers |
| type_node_carries_erased_type_argument | `child_type_node(c)` = expansion; recursion walks the expansion's children for a CompilerError | direct test `t.inferred` is CompilerError on the authored node; recursion over `settled_type(t).args` (and the authored `t.children` when t is not an alias) | needs `env` and `source_indices`; sole caller rust_empty_map_init_expr (which already has emit_info -> `fn_type_env`) |
| type_node_child_is_type_variable | `child_type_node(c).inferred` TypeVariable | identical read on the authored carrier `child_type_node(c)` (type variables are not aliases); no settle | none; verify only that the authored reference node keeps `inferred: TypeVariable` (resolve n_is_type_var arm) |
| rust_empty_map_value_type_str / rust_empty_map_kv_type_str | render_rust_type of the expansion node | render_rust_type of the SETTLED view (SETTLE-RENDER) | rust_empty_map_init_expr (emit_info available); render_rust_type must accept the settled head+args or a materialised settled Node |
| rust_empty_set_element_type_str / emit_rust_empty_set_expr / rust_btree_set_element_ord_eligible | `child_type_node(elem_child)` expansion for type-var/error tests, ord roster, render_node_type | type-var/error tests on authored carrier; ord eligibility and render on the settled view | rust_btree_set_element_ord_eligible is also called from render_rust_type_without_applied_binding and rust_set_element_ineligibility_reason (outside range): all three callers pass a settled element, so the settle belongs inside rust_btree_set_element_ord_eligible once, not at each caller |
| collection_element_type | `child_type_node(first child)` expansion into closure-unbound walk and render | error/type-var tests on authored carrier; unbound-generic walk and render on settled view | callers emit_rust_fold_method_call (via elem_type_str), emit_rust_sort_by_method_call-like paths pass `type_env`; already supplied |
| field_access_field_is_boxed | `resolved_type(field_child)` expansion into `type_reference_realization` and `needs_box_wrapping` | authored `field_ty` for naming; `settled_type(field_ty)` head/args for `needs_box_wrapping` and host-numeric realization of the settled head | none beyond access to scope.type_env (already used) |
| type_node_carries_applicationless_parametric_head (compensator) | `child_type_node` projection that drops alias RHS args, hence the bare-head false positive | unchanged function body over authored children; the alias-dropped-args false positive disappears, so the "whatever children the projected node still carries" provision and the dissolution marker in its comment become dead text | none; render_rust_type must name an authored parametric head with its arguments (render_rust_applied_type path) |
| __applied_type_args readers in range | none (rg: no readers in 9201-14500; all are at 1571, 1813, 1964, 1999, 2313, 2333, 8414-8490, 8855, outside) | n/a | n/a |
| rust_operand_alias_or_refinement_continue | `rt.inferred Resolved r` (cached target of the alias reference), `is_type_alias_item(r)`, then `resolved_type(r)` (the alias declaration's RHS) | Do not use `settled_type` (it stops at brand aliases and this hop must cross `type Char = Int where ..., brand("Char")`). Resolve the declaration by DeclarationRef: `type_reference_declaration_ref(rt)` then the declaration node via the symbol index (the same lookup `rust_operand_realization_of_stamped_declaration` uses: `symbol_index_lookup(module_path.decl_name)`), test `is_type_alias_item`, return `resolved_type(decl)`. Delete the `rt.inferred` read. The `where`-refinement arm is unchanged. | `rust_operand_realization_of_type` already holds `d` from `type_reference_declaration_ref`, so pass `d` in (drop the second lookup); `hop_argv_element_type` (calls it with the argv type) needs no change beyond receiving the authored node. |
| rust_struct_field_type_node_from_container | `resolved_type(field_node)`: the field type's cached expansion | `resolved_type(field_node)` now the authored reference; keep it as a pass-through and settle in the consumers that test structure, not here | `rust_receiver_has_callable_method_field` and `rust_record_field_needs_fn_rc` (`connective == Arrow`) call `settled_type(field_type)` first; `rust_record_field_needs_box`/`needs_box_wrapping` need none (authored name). `record_field_expected_type` and `rust_struct_field_type_node_variant_aware` pass through. |
| lookup_struct_field_type_name | `authored_name_at(resolved_type(field_child))` plus type-variable test on its `.inferred` | Authored name from the field's authored reference (identity, unchanged); type-variable test stays on the reference's own head | none |
| emit_field_value_with_context, plain_shared_struct_literal_name, dying_literal_plan | `expand_type_for_field_access(resolved_type(lit/field_value)).resolved` used as a record's field container | Same call replaced by `settled_type` of the literal's type (record structure) | `emit_typed_record_lit`'s `resolve_node(resolved_type).resolved` (Conj children names) converts the same way; `emit_stored_record_field`/`record_field_expected_type` receive the settled record as `struct_node` unchanged. |
| emit_capability_method | `child_type_node(f)` of each anonymous-output field (the expansion) into `render_rust_fn_sig_type` | `settled_type(child_type_node(f))` before rendering, or pass the authored reference if the renderer peels transparent aliases | none |
| emit_dry_run_branch_from_props, emit_from_key_extraction, has_from_key_fields | `resolved_type(op_node).children` and `tn.inferred Resolved rt` as the wire/output record | `settled_type(resolved_type(op_node))` before `.connective`/`.children`; `wire_ty` = `settled_type(tn)` | `wire_child_for_segment` takes the settled wire node (it reads `.children` after `normalize_access_type_node`). |
| hop_argv_element_type | Alias hop via `rust_operand_alias_or_refinement_continue` then `resolve_type_decl_reference` + `argv_follow_inferred(decl)` | Unchanged after the continue-fn conversion above (declaration hop, not cache read) | none |

### Counts per class (all tables above, no '?')

A=97 T=9 S=115 U=240  total=461


### Confidence and open checks (stated honestly)
- Not every body was fully re-read: about 47 U rows (Part A) were classified from names and about 38 A/U splits in 05_emit_rust from earlier reads. They are provisional until each reader is converted; conversion of a reader re-reads its body first.
- Emit policy: today `child_type_node(ch)` is the expansion, so emitted Rust names the expansion for alias arguments. Argument-spelling functions (`render_node_type`, `render_rust_type`, `render_rust_decl_type`, `render_rust_fn_sig_type`, `render_rust_applied_type`, `render_rust_alias_rhs_type`) are S: settle before spelling so emitted bytes do not change. Naming the alias would need proof it is emitted and imported in scope.
- Nothing writes `__applied_type_args`, so every arm reading it is dead and is deleted.
- `type_needs_rc_seen` follows the expansion chain without a seen-guard and would not terminate on an authored self-reference: newly found List 2 site.
- Before cutting `map_/set_*_type_in_env`, enumerate every consumer by grep.
