# Q1d: builtin_signature rows vs host realization (read-only census)

Method: identity join by name between the 122 `builtin_signature` rows (v1.compiler.infer_method, src/v1/04_method.dag) and the `arm "free_call.<name>"` arms in src/v1/stage0/src/v1_interpreter.rs (139 distinct arms; 27 arms have no registry row, out of scope). Base e0dfd7aeb6d.
Coverage (reconciled): 122 rows = 112 with a free_call arm + 10 with none. Of the 112, 75 use `expect_*($positional...)` accessors (mechanically compared); 37 use a `[Value::..]` slice pattern, a `$positional.first()` match, or a nullary/zero-arg body, and were read by hand (all agree except the opaque-typed rows below). My first message said 87/47, which wrongly counted arms with no registry row.

## Disagreements (declared vs real)
- compile_dag_rust_emit_check: declared (source String, entry String, expected String, mode String); real host `emit_host::compile_dag_rust_emit_check(source: &str, file_path: &str, includes: &[String], excludes: &[String])` = String, String, List<String>, List<String>. Names entry/expected/mode also stale (file_path/includes/excludes).
- compile_dag_multi_module_fixture: declared (sources List<String>, entry String, function String); real arm reads paths List<String>, contents List<String>, entry String. ARITY and names differ (2 lists + String vs list + 2 Strings).
- contiguous_loop_elementwise_float_kernel: declared (op_codes List<Int>, fma_contraction_policy Int, a/b/c List<Int>); real a/b/c List<Float> (expect_float_list_flex); fma policy read by expect_fma_contraction_policy_wire (wire type, not plain Int; confirm which).
- extdeps_external_authority_facts_for_qualified_name, extdeps_qualified_name_resolves_in_derived_module_set, extdeps_shape_transport_policy_facts_for_qualified_name: declared `module: String`; real arm takes a QualifiedName value (free-monoid symbol list, via free_monoid_symbol_value_to_dotted_string / qualified_name_resolves_in_derived_module_set). Declared String is a nickname for QualifiedName.
- chars_to_string: declared (chars List<Int>, start Int, end Int); arm reads only positional 0 as free-monoid items (code points); start/end not read there (arity doubt, may be handled elsewhere).

## Hand-read arms (37): agree, except
- emit_host_run_transport, emit_host_run_transport_cached: declared params are AlgebraTypeVariable placeholders (emit_host_admission/files/build/run/environment), so the registry states no real param type; arms hand positional values to eval_emit_host_run_transport*_builtin. Opaque, not a typed mismatch.
- discriminant, map_is_empty, sorted_map_keys, rc_ptr_eq, set_*: declared by type variable/container; agree with the Value::Variant/Map/Set patterns.
- trace_mark, record_generated_artifact_drift_gate_failure_detail: String, agree.

## UNCHECKED residual (10 rows, no free_call arm; enumerated)
- name_resolution_policy_is_namespace_only
- rc_vec_ptr_eq
- resolution_silent_pick_is_enabled
- resolution_silent_pick_record_fn_parent_first_hit
- resolution_silent_pick_record_global_bare_lcp_pick
- resolution_silent_pick_record_global_bare_lcp_tie
- scan_to_eol
- scan_while
- skip_horizontal_ws
- type_ref_hit_ne_bind_measure_active
These are dispatched by another route (not a `free_call.` arm); their host realization still needs to be located.
