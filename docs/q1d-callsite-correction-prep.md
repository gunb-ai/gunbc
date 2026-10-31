# Q1d false-registry correction prep: the call-site side (read-only, nothing run)

SCOPED OBSERVATION, not an inventory or a gate (DESIGN section 6): name-keyed syntactic extraction, misses aliased calls, not executed; first committed at revision 7a51287da5e1.

Base: origin/main as fetched 2026-10-31 for call sites and host arms; registry rows are `builtin_signature` in src/v1/04_method.dag. Authority for each row is the HOST realization (src/v1/stage0/src/v1_interpreter.rs `arm "free_call.<name>"`). Companion to q1d-registry-census-note @ 2440bc94b281. Call sites were enumerated by paren-balanced argument extraction over every `.dag` under dag/ and src/v2/ (comment lines skipped): this is a name-keyed syntactic census, so it also counts uses that shadow the name and misses aliased calls; none was observed. Not executed through the checker.

Method for "would refuse under Q1d": a call site refuses if the type of its argument at a position does not inhabit that position's DECLARED type in the current row. Types below are read from the call text and the enclosing fn signature, not measured.

## 1. compile_dag_rust_emit_check
- Host arm: `(source: Str, file_path: Str, includes: List<Str>, excludes: List<Str>) -> Bool` (expect_str, expect_str, expect_str_list, expect_str_list).
- Registry: `source, entry, expected, mode` all String. WRONG SIDE: the registry row. Names are stale and positions 3 and 4 are Lists.
- Call sites: 635, ALL arity 4, ALL positional. 633 pass a list literal at positions 3 and 4; the other two pass a variable (match_arm_pattern_identity_emission_witness.dag:28 `markers`, `[]`; src/v2/test/claim/manual/unary_prefix_values.dag:91 `includes`, `excludes`). Every site matches the HOST. So every one of the 635 would refuse under a Q1d check against the CURRENT row (List at String, twice per site); none refuses against the corrected row. 129 files (the per-site listing was a one-off with no consumer and is not kept; re-derive with the same extraction). Four are under src/v2/test.
- Corrected row text:
  `"compile_dag_rust_emit_check": BuiltinSignature { params: [BuiltinParam { name: "source", ty: NamedTemplate { name: "String" } }, BuiltinParam { name: "file_path", ty: NamedTemplate { name: "String" } }, BuiltinParam { name: "includes", ty: ContainerOf { source: Named { name: "List" }, element: NamedTemplate { name: "String" } } }, BuiltinParam { name: "excludes", ty: ContainerOf { source: Named { name: "List" }, element: NamedTemplate { name: "String" } } }], returns: bool_type },`
- Consequence: Q1d must land WITH this row, or the check refuses 635 correct sites.

## 2. compile_dag_multi_module_fixture
- Host arm: `(paths: List<Str>, contents: List<Str>, entry: Str)`.
- Registry: `(sources: List<String>, entry: String, function: String)`. WRONG SIDE: the registry row.
- Call sites: 1, dag/gunbc/instruments/multi_module_compile_fixture.dag:192 (positional: `fixture.sources |> map(path)`, `... |> map(content)`, `fixture.entry as String`). Matches the host. Would refuse under the current row (List<String> at String at position 2). Does not refuse against the corrected row.
- Corrected row: params `paths: List<String>`, `contents: List<String>`, `entry: String`; returns unchanged (`MultiModuleCompileFixtureOutcome`).

## 3. contiguous_loop_elementwise_float_kernel
- Host arm: `a, b, c` read by expect_float_list_flex (List<Float>); `fma_contraction_policy` read by expect_fma_contraction_policy_wire (a wire value of the FmaContractionPolicy variant, std.numerical_contract), not plain Int; op_codes List<Int>.
- Registry: a/b/c `List<Int>`, policy `Int`. WRONG SIDE: the registry row (the record types the policy and buffers truthfully at both call sites).
- Call sites: 2. dag/extdeps/languages/simd/kernel.dag:97 (`a/b/c: List<Float>`, `fma_contraction_policy: FmaContractionPolicy`) and dag/gunbc/accelerator_demo/accelerator_demo_realize.dag:285 (`fma_contraction_policy: fma_contraction_...`, `a: a` ...). Both would refuse under the current row (Float lists at List<Int>, a variant at Int); neither against the corrected row.
- Corrected row: a/b/c `ContainerOf List Float`, `fma_contraction_policy` the nominal type of std.numerical_contract FmaContractionPolicy (use `nominal_type_ref(name: "std.numerical_contract.FmaContractionPolicy")` as the row already does for returns types; confirm the module path against the declaration at dag/std/numerical_contract.dag).

## 4. The three qualified-name rows
Rows: extdeps_external_authority_facts_for_qualified_name, extdeps_qualified_name_resolves_in_derived_module_set, extdeps_shape_transport_policy_facts_for_qualified_name. Declared `module: String`; host arms take a QualifiedName value (the resolves arm does `$positional.first()` as an opaque Value, no expect_str).
- WRONG SIDE: the registry row. Every call site already passes a `QualifiedName`: src/v2/lens/mandatory_tag/corpus_scan.dag `corpus_anchor_facts(module: QualifiedName)` (1 site, line 32); src/v2/lens/extdeps_shape_transport_policy.dag (8 sites: lines 121, 126, 131, 138, 144, 148, 160 for the facts row and 152 for the resolves row; enclosing fns take `module: QualifiedName`). All 9 would refuse under the current `String` (QualifiedName at String) and none against `QualifiedName`.
- Corrected rows: param `module` typed as the QualifiedName nominal (module path to confirm: `std.name`/`std.qualified_name` declaration of QualifiedName); the name is `QualifiedName`, the single authority, not a String nickname (DESIGN 3). OPEN: the registry's `NamedTemplate`/`nominal_type_ref` choice for a type from another module must be confirmed against how the existing rows spell cross-module nominals.

## 5. chars_to_string: the census note's arity doubt is resolved, NOT a mismatch
The host arm reads positional 0 as free-monoid items, positional 1 via `expect_int(... "chars_to_string start")` (clamped to 0..len) and positional 2 via `expect_int(... "chars_to_string end")` (clamped to start..len). Registry `(chars: List<Int>, start: Int, end: Int)` matches. Call sites 10, all arity 3 and all consistent (dag/std/host_text.dag:53, dag/gunbc/primitive_egress/dispositions_text.dag:39, src/v2/workflow/source_reference_repoint.dag:43 and :46, dag/test/claim/host_text_text_of_chars_round_trip.dag:56/60/64). No correction. The only wrinkle: `chars` arrives as a free-monoid/List<Int> value of code points, so a `List<Char>` call would depend on the Char-in-List<Int> container path (see int-at-nat-red q4 probe: List<Char> at List<Int> falls to the terminal arm).

## 6. emit_host_run_transport, emit_host_run_transport_cached
Placeholder rows (`AlgebraTypeVariable` params). 1 call site each, arities 5 and 6. A Q1d check cannot refuse against a type variable and says nothing; they are a different class (the registry states no type), not a false row. No corrected text drafted: the right text needs the emit_host transport value types, which are modeled in the host, not in the registry.

## 7. The ten unchecked rows
No call-site work: the note's list has no host arm to read, so the authority is unlocated; once located, the same join (arm positions against rows, then call-site extraction with /tmp-style scripts) applies. NOT DONE.

## Summary
All six registry-vs-host rows with a real mismatch (1, 2, 3, 4 x3) have the registry row wrong and every call site correct. A Q1d check against the CURRENT rows would refuse 635 + 1 + 2 + 9 = 647 call sites that are right, so the corrected rows are a precondition of landing Q1d, not a follow-up. chars_to_string is a false positive of the note (no mismatch). Not run; the census is syntactic.
