# host_text crossing-site inventory (DISCOVERY ONLY)

Method: grep + inspection, not a resolver. This is an inventory, not acceptance. Ruling: tidy-ant-251 msg_68a9518b, per #15415.
Completeness evidence is the port's own compile: a crossing this list misses must refuse when the port lands. The grep count is not evidence (sleek-owl-397, via msg_673c6d23).

- Revision: origin/main 500d8491b2a8.
- Scope: the closure reached by walking the import lines from v2.compiler.source_root_eval (self-host), v2.cli.compile_cli (v2-native-cli) and v2.test.claim.parse.expression_bodied_fn_decl_parse (native-seven). Paths follow the source-layout rule; 277 files.
- Match: call syntax NAME( for split, replace, starts_with and ends_with, with string-literal contents and // comments removed first. Enclosing symbol: the nearest preceding top-level fn, data or type.
- Binding: a site is ambient when its file has no import of that name. All 118 are ambient. Ambient call heads are population 3 (builtin arguments, typed by nothing) of gunbc.rung_drop text_boundary_identity_wall, by that row's definition.

Unknowns:
- Populations 1, 2 and 4 are not classified; they need inferred types (a later phase).
- Not caught: a name passed as a value (no parenthesis), a qualified spelling, a multi-line string literal, a nested-fn symbol, and anything the import walk misses (for example, generated references).
- The closure is an import-line approximation, not the compiler's closure.

Producer: scratch/text-port/producer/closure.sh <roots>, then awk -f scratch/text-port/producer/sites.awk over the files it prints.

| file | enclosing symbol | line | name | binding | population |
|---|---|---|---|---|---|
| src/v2/compiler/source_root_eval.dag | native_lane_declared_module_name | 1466 | split | ambient | 3 |
| src/v2/compiler/source_root_eval.dag | native_lane_declared_module_name | 1467 | starts_with | ambient | 3 |
| src/v2/compiler/source_root_eval.dag | native_driver_parse_step | 1727 | starts_with | ambient | 3 |
| src/v2/workflow/v2_native_route.dag | native_route_package_text_module | 110 | split | ambient | 3 |
| dag/gunbc/native_route_universe.dag | native_route_module_package_text | 32 | split | ambient | 3 |
| src/v2/workflow/floor_discovery_row.dag | floor_path_strip_dot_slash_prefix | 46 | starts_with | ambient | 3 |
| src/v2/workflow/floor_discovery_row.dag | floor_entry_requires_wire_contract_sidecar | 54 | ends_with | ambient | 3 |
| src/v2/workflow/floor_discovery_source_authority.dag | floor_discovery_before_first_equals | 111 | split | ambient | 3 |
| src/v2/workflow/floor_discovery_source_authority.dag | floor_discovery_after_first_colon | 125 | split | ambient | 3 |
| src/v2/workflow/floor_discovery_source_authority.dag | floor_discovery_line_has_wire_contract_decl | 155 | starts_with | ambient | 3 |
| src/v2/workflow/floor_discovery_source_authority.dag | floor_discovery_content_has_wire_contract_decl | 170 | split | ambient | 3 |
| src/v2/workflow/floor_discovery_source_authority.dag | floor_discovery_line_is_disposition_row | 185 | starts_with | ambient | 3 |
| src/v2/workflow/floor_discovery_source_authority.dag | floor_discovery_line_is_disposition_row | 190 | starts_with | ambient | 3 |
| src/v2/workflow/floor_discovery_source_authority.dag | floor_discovery_line_is_disposition_row | 190 | starts_with | ambient | 3 |
| src/v2/workflow/floor_discovery_source_authority.dag | floor_discovery_after_first_equals | 209 | split | ambient | 3 |
| src/v2/workflow/floor_discovery_source_authority.dag | floor_discovery_last_dot_segment | 225 | split | ambient | 3 |
| src/v2/workflow/floor_discovery_source_authority.dag | floor_discovery_initializer_rest_is_strict_single_token | 241 | starts_with | ambient | 3 |
| src/v2/workflow/floor_discovery_source_authority.dag | floor_discovery_first_token_before_space | 258 | split | ambient | 3 |
| src/v2/workflow/floor_discovery_source_authority.dag | floor_discovery_resolve_entry_live_tree | 379 | split | ambient | 3 |
| dag/extdeps/bazel/label.dag | parse_package_segments | 101 | split | ambient | 3 |
| dag/extdeps/bazel/label.dag | parse_label_body | 156 | split | ambient | 3 |
| dag/extdeps/bazel/label.dag | parse_label | 196 | starts_with | ambient | 3 |
| dag/extdeps/bazel/label.dag | parse_label | 198 | starts_with | ambient | 3 |
| src/v2/compiler/type_declaration_use_census.dag | hash_callee | 692 | starts_with | ambient | 3 |
| dag/std/content_hash.dag | parse_content_hash_candidate | 342 | starts_with | ambient | 3 |
| dag/std/content_hash.dag | parse_content_hash_candidate | 343 | split | ambient | 3 |
| dag/std/content_hash.dag | parse_content_hash_candidate | 351 | starts_with | ambient | 3 |
| dag/std/content_hash.dag | parse_content_hash_candidate | 352 | split | ambient | 3 |
| src/v2/compiler/source_authority.dag | module_file | 1482 | split | ambient | 3 |
| src/v2/compiler/source_authority.dag | module_name_for_file | 1524 | replace | ambient | 3 |
| src/v2/compiler/source_authority.dag | derived_roster_namespace_leaf | 1573 | split | ambient | 3 |
| src/v2/compiler/source_authority.dag | derived_roster_sibling_of | 1599 | split | ambient | 3 |
| src/v2/compiler/source_authority.dag | derived_roster_line_row | 1620 | split | ambient | 3 |
| src/v2/compiler/source_authority.dag | derived_roster_line_row | 1624 | split | ambient | 3 |
| src/v2/compiler/source_authority.dag | derived_roster_sibling_rows | 1639 | split | ambient | 3 |
| src/v2/compiler/source_authority.dag | reference_prefixes | 1774 | split | ambient | 3 |
| src/v2/compiler/source_authority.dag | reference_member | 1861 | split | ambient | 3 |
| src/v2/compiler/source_authority.dag | reference_member | 1862 | split | ambient | 3 |
| src/v2/compiler/source_authority.dag | module_namespace | 1997 | split | ambient | 3 |
| src/v2/compiler/module_acquisition.dag | namespace_module_name | 1884 | replace | ambient | 3 |
| src/v2/compiler/module_acquisition.dag | namespace_top_level | 1979 | split | ambient | 3 |
| src/v2/compiler/self_host/seed_retention_frontier.dag | top_level_src_basename | 368 | replace | ambient | 3 |
| src/v2/compiler/self_host/seed_retention_frontier.dag | frontier_nonempty_lines | 813 | split | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/std/types.dag | commit_sha_text_holds | 207 | replace | ambient | 3 |
| dag/extdeps/filesystem/filesystem_io.dag | filesystem_established_absence_path | 751 | ends_with | ambient | 3 |
| dag/extdeps/uri.dag | uri_from_wire | 98 | split | ambient | 3 |
| dag/extdeps/uri.dag | uri_from_wire | 99 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | href_is_relative_reference | 119 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | href_is_relative_reference | 120 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | href_is_relative_reference | 121 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | href_is_relative_reference | 122 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 128 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 132 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 132 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 134 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 134 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 136 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 136 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 138 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 138 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 140 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 140 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 142 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 142 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 144 | starts_with | ambient | 3 |
| dag/extdeps/uri.dag | parse_href_scheme | 144 | starts_with | ambient | 3 |
| src/v2/std/anonymous_binder.dag | type_variable_origin | 93 | starts_with | ambient | 3 |
| src/v2/std/anonymous_binder.dag | type_variable_origin | 95 | starts_with | ambient | 3 |
| src/v2/std/anonymous_binder.dag | is_fold_carrier_binder | 149 | starts_with | ambient | 3 |
| src/v2/std/anonymous_binder.dag | is_anonymous_param_label | 160 | starts_with | ambient | 3 |
| dag/std/unicode/scalar.dag | ascii_lowercase | 87 | split | ambient | 3 |
| dag/std/unicode/scalar.dag | ascii_uppercase | 91 | split | ambient | 3 |
| dag/gunbc/source_layout.dag | layout_module_storage_path | 56 | starts_with | ambient | 3 |
| dag/gunbc/source_layout.dag | layout_module_storage_path | 58 | replace | ambient | 3 |
| dag/gunbc/source_layout.dag | layout_module_storage_path | 64 | replace | ambient | 3 |
| dag/gunbc/rust_emitted_edge.dag | module_to_filename | 56 | split | ambient | 3 |
| dag/gunbc/stage0/stage0_emitted_population_manifest.dag | emitted_population_paths_from_manifest | 81 | split | ambient | 3 |
| dag/gunbc/stage0/stage0_emitted_population_manifest.dag | emitted_population_paths_from_manifest | 86 | starts_with | ambient | 3 |
| dag/extdeps/languages/dag/syntax.dag | strip_line_comment_delimiter | 182 | starts_with | ambient | 3 |
| dag/extdeps/languages/dag/syntax.dag | strip_line_comment_delimiter | 184 | starts_with | ambient | 3 |
| dag/extdeps/languages/dag/syntax.dag | render_annotation_block | 202 | split | ambient | 3 |
| dag/extdeps/languages/json/emit.dag | json_escape_replace | 152 | split | ambient | 3 |
| dag/extdeps/languages/rust/emit.dag | rust_qualified_module_mod_basename | 440 | split | ambient | 3 |
| dag/extdeps/languages/json/grammar.dag | json_number_lexeme_valid | 126 | starts_with | ambient | 3 |
| dag/extdeps/languages/json/grammar.dag | json_number_lexeme_valid | 131 | starts_with | ambient | 3 |
| dag/extdeps/languages/json/grammar.dag | json_text_trim_trailing_lf | 176 | ends_with | ambient | 3 |
| dag/extdeps/languages/json/grammar.dag | json_text_string_scan_terminated | 344 | ends_with | ambient | 3 |
| dag/extdeps/languages/json/grammar.dag | json_text_string_scan_terminated | 347 | replace | ambient | 3 |
| dag/extdeps/languages/json/grammar.dag | json_text_string_scan_terminated | 347 | ends_with | ambient | 3 |
| dag/extdeps/ietf/http_semantics.dag | http_is_ascii_digits | 125 | replace | ambient | 3 |
| dag/extdeps/uri_path.dag | parse_segment_tokens | 89 | split | ambient | 3 |
| dag/extdeps/uri_path.dag | parse_segment_tokens | 102 | split | ambient | 3 |
| dag/extdeps/uri_path.dag | match_path_segments | 153 | split | ambient | 3 |
| dag/extdeps/uri_path.dag | match_path_template | 192 | split | ambient | 3 |
| dag/extdeps/uri_path.dag | parse_path_template | 205 | split | ambient | 3 |
| dag/extdeps/uri_path.dag | parse_path_template | 209 | split | ambient | 3 |
| dag/extdeps/uri_path.dag | uri_query_string | 246 | split | ambient | 3 |
| dag/extdeps/uri_path.dag | parse_query_template | 260 | split | ambient | 3 |
| dag/extdeps/uri_path.dag | uri_query_param | 280 | split | ambient | 3 |
| dag/extdeps/uri_path.dag | uri_query_param | 282 | split | ambient | 3 |
| dag/extdeps/uri_path.dag | uri_query_param | 286 | split | ambient | 3 |
| dag/extdeps/version/semver.dag | semver_core_of_dotted | 172 | split | ambient | 3 |
| dag/extdeps/version/semver.dag | semver_minimum_of_constraint | 210 | starts_with | ambient | 3 |
