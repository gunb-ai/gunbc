# Partitioning `builtin_function_registry`: the denominator closes by relocation

## Why this document exists

`v1.compiler.infer_method` `builtin_function_registry` carries 131 names and maps each to a RETURN
TYPE. `std.primitive_identity` `primitive_signature` resolves a full parameter signature for a
primitive by keying into `std.algebra`'s `AlgebraFieldTemplate` rows. 20 of the registry's names
have such a row and resolve; **111 answer `SignatureNotGrounded`.**

While that 111 is a mixed population, `SignatureNotGrounded` is ambiguous between *"not a language
primitive"* and *"a language primitive nobody has modelled yet"*, and a coercion that fails closed
on it cannot distinguish a legitimate absence from an unmodelled one. Closing that ambiguity is the
precondition for the interpreter gaining the optional-into-required-parameter coercion its emitted
counterpart already has (→ [first-optional divergence census](first-optional-divergence-census.md)).

**The partition is by RELOCATION, not classification.** §3 holds that interface, realization and
policy are three facts and not one row, and that the dispatch selecting a realization is itself
realization. A lens's or a host transport's parameter shape is a realization fact; it does not
belong in a language-primitive signature authority at all. So the two populations are not two kinds
of one thing awaiting a discriminator — they are two different things sharing one table, and the
partition ends with the second population's rows leaving.

## Four candidate predicates, measured, and why none of them decides it alone

Recorded so the next reader does not re-derive them. Each was tested against the actual 111 rather
than reasoned about.

1. **Join on the interpreter's primitive-surface roster.** Fails by construction:
   `gunbc.v1_interpreter_primitive_surface` enumerates an arm for BOTH populations, so it
   classifies `doc_graph_orphan_count` and `parse_int` identically. It answers "is there an arm",
   which is true of every builtin.
2. **Does the name have a `.dag` `fn` declaration?** This is the `HostRealizedSeam` shape from
   `std.primitive_projection`, whose doc comment establishes seams *by reading the declaration, not
   by matching the name*. It identifies 8 of the 111 and leaves 103 undecided, and it
   false-positives on `string_contains`, which matches a declaration inside a witness test.
3. **Does the interpreter arm touch the host?** Fails in both directions on the real population.
   `parse_int`, `string_length`, `code_point` and `is_xid_start` look host-touching because their
   arms delegate to `v1_rt` helpers; `decl_facts` and `doc_graph_orphan_count` look pure because
   their host call is further down the arm than any fixed reading window.
4. **How many modules reference the name?** The strongest of the four and still not sufficient. It
   separates cleanly at the extremes — `string_contains` (451 referencing modules), `string_length`
   (94), `parse_int` (41) against `doc_graph_orphan_count` (1, in `src/v2/lens/doc_reachability`) —
   but breadth of USE is not ownership of the FACT: `filesystem_read` (63), `compile_dag_rust_emit_check`
   (71) and `shell_materialize_operation_argv` (10) are widely-used TRANSPORTS, and
   `scan_while` / `scan_to_eol` / `skip_horizontal_ws` are narrow LANGUAGE primitives whose only
   callers are inside the tokenizer.

**What the fourth predicate does supply, and it is the useful part, is the HOME.** Its value is not
the count but the module the references land in: `fallback_arm_census_facts` is
`src/v2/lens/fallback_arm_census`'s fact, `emit_host_run_transport` is `src/v2/compiler/emit_host`'s.
Naming the owning module is the §3 question ("a fact's home is its layer") asked mechanically. The
count is a symptom of the answer, never the criterion, and every row below carries its home.

## The criterion actually applied

For each name: **does the operation's meaning come from the LANGUAGE, or from a domain authority
that owns it?** A language primitive is one the substrate itself must be able to talk about
regardless of which lens, workflow or instrument exists — text and code-point manipulation, set
construction, hashing, identifier classification, lexer scanning. Everything else names a fact some
module owns, and its signature belongs at that module's declaration, which in most cases already
declares its parameters.

Both dispositions are adjudications, not lookups, and the residue is judgement — as it must be,
since all four mechanical predicates were measured to fail. What makes them auditable is that each
row states its home, so a disagreement is about one named module rather than about the rule.

## `LanguagePrimitive` — 24 names

Disposition: ground a signature through `std.primitive_identity`. Once the transports below have
left the registry, `SignatureNotGrounded` over this population means unambiguously "a language
primitive whose signature is not yet modelled" — a closeable gap, and the property the coercion
needs in order to fail closed honestly.

| name | referencing modules | first non-test reference |
|---|---|---|
| `string_contains` | 450 | `dag/extdeps/astronomy/stellar_classification.dag` |
| `string_length` | 93 | `dag/extdeps/auth/jwt.dag` |
| `from_code_point` | 40 | `dag/extdeps/dns/domain_name.dag` |
| `parse_int` | 40 | `dag/extdeps/bmc/capability.dag` |
| `discriminant` | 39 | `dag/extdeps/git/git.dag` |
| `char_at` | 26 | `dag/extdeps/auth/jwt.dag` |
| `set_contains` | 20 | `dag/gunbc/emit_summary_map_consumer_partition.dag` |
| `code_point` | 19 | `dag/extdeps/auth/jwt.dag` |
| `empty_set` | 19 | `dag/gunbc/package_delivery.dag` |
| `set_insert` | 15 | `dag/gunbc/package_delivery.dag` |
| `sorted_map_keys` | 8 | `dag/gunbc/instruments/pr_containment_instrument.dag` |
| `atom_identity_hash` | 3 | `dag/std/content_hash.dag` |
| `chars_to_string` | 3 | `src/v1/00_core.dag` |
| `is_emoji_ident` | 3 | `src/v1/01_tokenize.dag` |
| `map_is_empty` | 3 | `src/v1/04_infer.dag` |
| `set_union` | 3 | `dag/std/authorization_profile.dag` |
| `hash_combine` | 2 | `dag/std/content_hash.dag` |
| `is_xid_continue` | 2 | `src/v1/01_tokenize.dag` |
| `is_xid_start` | 2 | `src/v1/01_tokenize.dag` |
| `record_source_chars_index_lookup` | 1 | `src/v1/01_tokenize.dag` |
| `scan_string_end` | 0 | `(no .dag reference)` |
| `scan_to_eol` | 0 | `(no .dag reference)` |
| `scan_while` | 0 | `(no .dag reference)` |
| `skip_horizontal_ws` | 0 | `(no .dag reference)` |

## `Transport` — 87 names

Disposition: the signature belongs with the realization authority named under *home*; the name
leaves the language registry. Where that module already declares the operation as a `.dag` `fn`,
the parameter list exists there and relocation supplies the signature at no authoring cost.

| name | referencing modules | home |
|---|---|---|
| `non_fold_residue_roster_red_fixture_holds` | 0 | `(no .dag reference)` |
| `non_fold_residue_total_fold_green_fixture_holds` | 0 | `(no .dag reference)` |
| `witness_layer_roots_compile_clean_check` | 0 | `(no .dag reference)` |
| `witness_layer_roots_compile_clean_emit_check` | 0 | `(no .dag reference)` |
| `contiguous_loop_elementwise_float_kernel` | 2 | `dag/extdeps/languages/simd/kernel.dag` |
| `contiguous_loop_elementwise_kernel` | 2 | `dag/extdeps/languages/simd/kernel.dag` |
| `filesystem_read` | 62 | `dag/extdeps/llm/claude_agent_sdk_stream.dag` |
| `emit_host_native_cache_evict` | 1 | `dag/extdeps/realization/emit_on_demand_host.dag` |
| `decl_facts` | 27 | `dag/gunbc/bare_name_fork_lens.dag` |
| `compile_dag_diagnostic_census` | 28 | `dag/gunbc/compile_diagnostic_census.dag` |
| `compile_dag_rust_emit_check` | 71 | `dag/gunbc/compile_diagnostic_census.dag` |
| `shell_materialize_operation_argv` | 10 | `dag/gunbc/host/host_operation_exec.dag` |
| `witness_compile_clean_cli_floor_verdicts_agree` | 1 | `dag/gunbc/instruments/dag_compile_clean_cli_floor_agreement.dag` |
| `module_declaration_facts` | 2 | `dag/gunbc/instruments/dag_compile_clean_shard_roster.dag` |
| `install_or_consume_floor_compile_clean_gate_receipt` | 1 | `dag/gunbc/instruments/dag_compile_clean_transport.dag` |
| `consume_generated_artifact_drift_gate_receipt` | 1 | `dag/gunbc/instruments/generated_artifact_gate.dag` |
| `record_generated_artifact_drift_gate_clean` | 1 | `dag/gunbc/instruments/generated_artifact_gate.dag` |
| `record_generated_artifact_drift_gate_failure_detail` | 1 | `dag/gunbc/instruments/generated_artifact_gate.dag` |
| `compile_dag_multi_module_fixture` | 1 | `dag/gunbc/instruments/multi_module_compile_fixture.dag` |
| `test_migration_behavior_discovery_holds` | 1 | `dag/gunbc/legacy_test_behavior_disposition.dag` |
| `test_migration_legacy_behavior_ids` | 2 | `dag/gunbc/legacy_test_behavior_disposition.dag` |
| `test_migration_witness_behavior_ids` | 1 | `dag/gunbc/legacy_test_behavior_disposition.dag` |
| `data_decl_type_facts` | 3 | `dag/gunbc/lifecycle_survivor_scan.dag` |
| `namespace_structural_observation_admissions` | 2 | `dag/gunbc/namespace/namespace_structural_observations_production.dag` |
| `parse_roadmap_acceptance_event_history_jsonl` | 1 | `dag/gunbc/roadmap/roadmap_acceptance_history_carrier.dag` |
| `project_roadmap_acceptance_event_history_from_authority_text_host` | 1 | `dag/gunbc/roadmap/roadmap_acceptance_history_projection.dag` |
| `parse_stage0_cargo_manifest_bins` | 1 | `dag/gunbc/stage0/stage0_rust_host_observation.dag` |
| `observed_monotonic_nanos` | 2 | `dag/std/realization_measurement.dag` |
| `commit_witness_claim_pair_resolvable` | 1 | `dag/test/claim/commit_witness_claim_roster_witness_test.dag` |
| `doc_graph_admitted_root_count` | 1 | `dag/test/claim/doc_reachability_witness_test.dag` |
| `seed_runner_bool_false_failure_detail` | 1 | `dag/test/claim/long/extdeps_scope_placement_gate_loudness_witness_test.dag` |
| `shell_transport_operation_rows` | 1 | `dag/test/claim/operation_argv_corpus_witness_test.dag` |
| `observed_peak_resident_bytes` | 1 | `dag/test/claim/peak_resident_measured_witness_test.dag` |
| `name_resolution_policy_is_namespace_only` | 3 | `src/v1/04_env.dag` |
| `resolution_silent_pick_is_enabled` | 2 | `src/v1/04_env.dag` |
| `resolution_silent_pick_record_global_bare_lcp_pick` | 1 | `src/v1/04_env.dag` |
| `resolution_silent_pick_record_global_bare_lcp_tie` | 1 | `src/v1/04_env.dag` |
| `rc_ptr_eq` | 1 | `src/v1/04_infer.dag` |
| `rc_vec_ptr_eq` | 1 | `src/v1/04_infer.dag` |
| `type_ref_hit_ne_bind_measure_active` | 1 | `src/v1/04_resolve.dag` |
| `resolution_silent_pick_record_fn_parent_first_hit` | 1 | `src/v1/04_sigs.dag` |
| `trace_mark` | 1 | `src/v1/compile.dag` |
| `emit_host_run_transport` | 1 | `src/v2/compiler/emit_host.dag` |
| `emit_host_run_transport_cached` | 1 | `src/v2/compiler/emit_host.dag` |
| `toolchain_home_interference_probe` | 1 | `src/v2/extdeps/toolchain_interference.dag` |
| `complexity_linearity_syntactic_finding_count` | 1 | `src/v2/lens/complexity_linearity_audit.dag` |
| `complexity_linearity_syntactic_site_fired` | 1 | `src/v2/lens/complexity_linearity_audit.dag` |
| `complexity_linearity_wildcard_facts` | 1 | `src/v2/lens/complexity_linearity_audit.dag` |
| `doc_graph_dangling_link_count` | 2 | `src/v2/lens/doc_reachability.dag` |
| `doc_graph_doc_count` | 2 | `src/v2/lens/doc_reachability.dag` |
| `doc_graph_orphan_count` | 1 | `src/v2/lens/doc_reachability.dag` |
| `extdeps_qualified_name_resolves_in_derived_module_set` | 1 | `src/v2/lens/extdeps_shape_transport_policy.dag` |
| `extdeps_shape_transport_policy_facts_for_qualified_name` | 1 | `src/v2/lens/extdeps_shape_transport_policy.dag` |
| `fact_cardinality_decl_facts` | 1 | `src/v2/lens/fact_cardinality.dag` |
| `fallback_arm_census_class_count` | 1 | `src/v2/lens/fallback_arm_census.dag` |
| `fallback_arm_census_facts` | 1 | `src/v2/lens/fallback_arm_census.dag` |
| `fallback_arm_census_reconciliation_holds` | 2 | `src/v2/lens/fallback_arm_census.dag` |
| `fallback_arm_census_total` | 1 | `src/v2/lens/fallback_arm_census.dag` |
| `concept_decl_facts` | 2 | `src/v2/lens/grounding.dag` |
| `transport_script_position_facts_for_path` | 1 | `src/v2/lens/host_language_transport_script.dag` |
| `inert_carrier_declared_count` | 1 | `src/v2/lens/inert_carrier.dag` |
| `inert_carrier_names_live` | 1 | `src/v2/lens/inert_carrier.dag` |
| `export_signature_facts` | 2 | `src/v2/lens/interface_summary.dag` |
| `languages_consumer_census_data_decl_count` | 1 | `src/v2/lens/languages_consumer_census.dag` |
| `languages_consumer_census_external_consumer_count` | 1 | `src/v2/lens/languages_consumer_census.dag` |
| `languages_consumer_census_format_row_count` | 1 | `src/v2/lens/languages_consumer_census.dag` |
| `languages_consumer_census_has_external_consumer` | 1 | `src/v2/lens/languages_consumer_census.dag` |
| `languages_consumer_census_is_composition_only` | 1 | `src/v2/lens/languages_consumer_census.dag` |
| `languages_consumer_census_per_language_row_count` | 1 | `src/v2/lens/languages_consumer_census.dag` |
| `extdeps_external_authority_facts_for_qualified_name` | 1 | `src/v2/lens/mandatory_tag/corpus_scan.dag` |
| `extdeps_external_authority_live_clean_tree_holds` | 1 | `src/v2/lens/mandatory_tag/corpus_scan.dag` |
| `extdeps_external_authority_live_roster_module_count` | 1 | `src/v2/lens/mandatory_tag/corpus_scan.dag` |
| `dependency_resolution_facts` | 1 | `src/v2/lens/module_graph.dag` |
| `import_resolution_facts` | 1 | `src/v2/lens/module_graph.dag` |
| `reference_resolution_facts` | 1 | `src/v2/lens/module_graph.dag` |
| `census_corpus_roots_follow_layer_authority` | 1 | `src/v2/lens/non_fold_residue.dag` |
| `non_fold_residue_coproduct_universe_count` | 2 | `src/v2/lens/non_fold_residue.dag` |
| `non_fold_residue_count` | 1 | `src/v2/lens/non_fold_residue.dag` |
| `non_fold_residue_stale_roster_count` | 2 | `src/v2/lens/non_fold_residue.dag` |
| `non_fold_residue_unrostered_count` | 2 | `src/v2/lens/non_fold_residue.dag` |
| `test_migration_debt_module_names` | 1 | `src/v2/lens/test_migration_debt.dag` |
| `layer_import_facts` | 1 | `src/v2/std/layer_import_scan.dag` |
| `non_fold_residue_synthetic_unrostered_red_holds` | 1 | `src/v2/test/lens_non_fold_residue/non_fold_residue_test.dag` |
| `non_fold_residue_wildcard_red_fixture_holds` | 1 | `src/v2/test/lens_non_fold_residue/non_fold_residue_test.dag` |
| `observe_declared_import_closure_symbol_binding` | 1 | `src/v2/workflow/class_b_import_closure_probe.dag` |
| `class_b_import_closure_gate_not_affected_skip` | 1 | `src/v2/workflow/class_b_import_closure_transport.dag` |
| `commit_witness_claim_roster_unresolvable_count` | 1 | `src/v2/workflow/commit_witness_claim_roster.dag` |

## What this partition dissolves

Two classes, one relocation. Once the registry is no longer a signature authority for anything
`std.algebra` owns, the 20 overlapping names stop being two declarations of one operation — which
is the `coarser_parallel_authority` row in `gunbc.recurring_failure_mode`, filed from the measured
disagreement between the two carriers (`reverse` typed as `List<collection_element>` against
`ReceiverSelf`; `map_keys` and `map_values` sharing one element variable where the algebra rows
distinguish `ReceiverKey` from `ReceiverValue`; `concat` as `String` against `ReceiverSelf`; `get`
collapsing the List and Map readings). There is no second declaration left to disagree.

## Sequencing

This is a substrate program, not a step in a repair, and it is expected to span several PRs. This
document is the disposition census and carries no pipeline edit. Relocation lands per owning module,
so each PR is a readable diff against one authority.
