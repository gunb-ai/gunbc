# Q1d: builtin_signature rows vs host realization (read-only census)

Method: for each `builtin_signature` row in v1.compiler.infer_method (src/v1/04_method.dag, 122 rows), compared declared param kinds with the accessor sequence of its `arm "free_call.<name>"` in src/v1/stage0/src/v1_interpreter.rs (87 rows have such an arm; 47 do not and are UNCHECKED here: dispatched elsewhere). Base e0dfd7aeb6d. Instrument: regex script, arms confirmed by reading.

## Disagreements (declared vs real)
- compile_dag_rust_emit_check: declared (source String, entry String, expected String, mode String); real host `emit_host::compile_dag_rust_emit_check(source: &str, file_path: &str, includes: &[String], excludes: &[String])` = String, String, List<String>, List<String>. Names entry/expected/mode also stale (file_path/includes/excludes).
- compile_dag_multi_module_fixture: declared (sources List<String>, entry String, function String); real arm reads paths List<String>, contents List<String>, entry String. ARITY and names differ (2 lists + String vs list + 2 Strings).
- contiguous_loop_elementwise_float_kernel: declared (op_codes List<Int>, fma_contraction_policy Int, a/b/c List<Int>); real a/b/c List<Float> (expect_float_list_flex); fma policy read by expect_fma_contraction_policy_wire (wire type, not plain Int; confirm which).
- extdeps_external_authority_facts_for_qualified_name, extdeps_qualified_name_resolves_in_derived_module_set, extdeps_shape_transport_policy_facts_for_qualified_name: declared `module: String`; real arm takes a QualifiedName value (free-monoid symbol list, via free_monoid_symbol_value_to_dotted_string / qualified_name_resolves_in_derived_module_set). Declared String is a nickname for QualifiedName.
- chars_to_string: declared (chars List<Int>, start Int, end Int); arm reads only positional 0 as free-monoid items (code points); start/end not read there (arity doubt, may be handled elsewhere).

## Checked and agreeing (matched via `[Value::Str]` pattern, not an accessor)
trace_mark, record_generated_artifact_drift_gate_failure_detail: String, agree.

## Not checked
47 rows with no `free_call.` arm (e.g. emit_host_run_transport*, observed_*, set_*, scan_*, hash_combine, census counters). Need a second look at their dispatch site; measure with the census.
