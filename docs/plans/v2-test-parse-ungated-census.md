# Census: ungated `v2.test.parse.*` modules

Census only. Nothing is moved, gated or deleted here. Requested by sharp-raven-357 through stern-bear-500 (grammar program).

## Finding being censused

`v2.workflow.required_floor` `required_gate_prefixes` admits only `test.claim.*` families. Every module whose header is `module v2.test.parse.<x>` sits outside the required floor's prepared universe, so its claims execute only when a PR's changed-witness selection reaches them. The census below shows the cost of that: **7 claims are FALSE on main@5e9c7ea18c and no required lane reports them.**

## Population

`git grep -l -E '^module v2\.test\.parse\.' origin/main` at `5e9c7ea18c5edc1e4989272f0cd119565ac2f098` lists 43 files, all under `src/v2/test/claim/parse/`. The grammar program owns 8 of them (bare_node_statement_route, brace_and_lambda_head_route, else_less_if_value_category, newline_dual_role_operator_parse, choice_lookahead_guard_test, choice_plan_test, grammar_validation_test, newline_dual_role_operator_mutation_test). That leaves **35 in scope**:

- 26 modules hold `test fn` claims (or, for `alias_decl_parse`, three zero-arity `-> Bool` `_holds` fns). All 26 were measured.
- 2 are support modules with no claims (`parse_binding_fidelity_support`, `supplied_token_stream_support`).
- 7 are foreign-language grammar modules (`go`, `kotlin`, `lean`, `python`, `rust`, `swift`, `typescript`). Their claims are `data claim_*: TestClaim = EqualsClaim {…}` rows, 14 in total. `claim_batch` refuses these as `does not declare requested function(s)` (invocation 5d346ed3), so they are **not executable on the floor's function-row route** and were not measured.

## The measuring run

- **Instrument:** `claim_batch` (`src/v1/stage0/src/bin/claim_batch.rs`). One process ran all 26 entry groups and 237 claims, explicit `--entry/--functions` rows, `--source-root dag --source-root src/v2`.
- **Run:** BuildBuddy invocation `ea96a65d-925d-4594-843c-960075bf8af1`, dispatched by `ctrl-build --remote` under `EstimatedMemory=28GB`. Inside the script: `git checkout --force -- .`, `git clean -fdq`, fetch and force-checkout of the pinned full sha, and a refusal unless `HEAD` equals the pin. A 240 s heartbeat ran throughout, and the full stdout was gzip+base64 framed out of the log.
- **Memory:** the run binds a child cgroup (`memory.max` = 24 GiB, the process joins it after the build). It does **not** use `GUNBC_MEMORY_BUDGET_BYTES` as the brief asked. That variable is a planning request that constrains no allocation, and on a runner where no observed limit backs it, it turns the typed `HostBudgetUnreadable` refusal into a silent SIGKILL. That is the escape hatch DESIGN §5 forbids. Peak RSS was 8.6 GiB after the first resolve.
- **Re-derive:** dispatch the same `claim_batch` row list at the pin. The list is the 26 files below × their `test fn` names, plus `alias_decl_parse`'s three `_holds` fns.
- **What `[witness] eval_steps` means here:** it is the claim's marginal steps. One shared fill of **6,207,706 eval steps / 19,104 cpu ms** was paid inside this batch (6.13M of it by `match_heading_a_binary_chain_parses_holds`, which is the dag grammar preparation; the rest is small grammar fills). Following the floor's shared-fill accounting, that fill is shown separately and not charged to any one claim.
- **Discovery mode is broken on main.** `claim_batch --roster-from-discovery` refuses on this tree with `no declaration named 'discover_floor_corpus_rows_from_host_facts' in this execution's loaded index` (invocation 79ba7512). This is a separate defect, reported here and not repaired.

The 72,300 eval-step budget applies per claim. The "> budget" column counts claims whose marginal `eval_steps` exceed it.

## Table

Class key:
- **(a)** guards a live grammar or route, so GATE it.
- **(b)** subject has no live consumer, so DELETE it.
- **(c)** another lane's subject.

"Live" for (a) was established by import: all (a) modules reach `v2.compiler.parse`, `tokenize`, `normalize` or `program_assembly` over `v2.extdeps.languages.dag`, and `dag_grammar_root` is consumed in production by `v2.compiler.program_assembly`, `namespace_graft` and `occurrence_role`.

| module (`src/v2/test/claim/parse/`) | claims | guards | class | passes on main | eval steps (Σ marginal) | cpu ms | > 72.3k |
|---|---|---|---|---|---|---|---|
| alias_decl_parse.dag | 3 | `alias` decl parse plus its `AliasBindingRow` / lexical lookup in `symbol_index_fill` | a | **2/3**: FAIL `alias_decl_fill_lexical_lookup_holds` | 163,863 | 564 | 1 |
| block_expr_as_binary_operand_parse_test.dag | 7 | dag `expr`: match/if/loop heading a `binary_expr` (#11911 revival) | a | 7/7 | 888,433 | 1,949 | 6 |
| block_headed_operand_class_parse_test.dag | 7 | per-operator-class block-headed left operand via `v2.compiler.parse_acceptance_census` | a | 7/7 | 549,086 | 1,185 | 7 |
| closure_parse_batch_two_test.dag | 23 | Pkg11b grammar rows: int generic arg, `module` as value, anonymous record type, admit-callers, early-return guard; parse→normalize fidelity | a | **22/23**: FAIL `other_keywords_are_not_values_holds` | 1,429,297 | 2,828 | 5 |
| coproduct_leading_pipe_and_positional_payload_parse_test.dag | 24 | coproduct leading `\|`, positional payload variants and patterns, through normalize | a | 24/24 | 2,005,647 | 3,644 | 8 |
| d1_declaration_grammar_parse_test.dag | 8 | type-alias `where` refinement (D1 cause A), parse plus normalize | a | 8/8 | 1,505,116 | 2,784 | 7 |
| d5_expression_grammar_parse_test.dag | 13 | D5 expr rows: keyword field-init, `as` cast, match-as-infix, unary minus, brace map, `return` | a | 13/13 | 777,991 | 1,786 | 1 |
| expression_bodied_fn_decl_parse_test.dag | 8 | `fn_decl` `= expr` body, plus empty `=` refusals | a | 8/8 | 667,102 | 1,391 | 3 |
| g0_service_decl_parse_probe_test.dag | 24 | G0 `service` decl family (exact-word terminals), the normalized-tree door | a | **21/24**: FAIL `type_field_default_still_refuses_holds`, `type_field_from_key_still_refuses_holds`, `config_only_service_is_refused_for_its_set_aside_realization_holds` | 1,553,511 | 3,053 | 8 |
| grammar_expect_test.dag | 3 | 02_parse `Expect` label on ordered choice | a | 3/3 | 21,767 | 51 | 0 |
| grammar_not_predicate_test.dag | 4 | 02_parse PEG not-predicate in `Repeat` | a | 4/4 | 16,904 | 36 | 0 |
| keyword_path_segment_parse_test.dag | 3 | keyword as module/import path segment refuses | a | 3/3 | 82,156 | 381 | 0 |
| layout_line_break_test.dag | 8 | 02_parse layout from stream source: `AfterLineBreak` / `RefuseOnMatch` | a | 8/8 | 174,311 | 566 | 1 |
| list_literal_parse_cost_test.dag | 2 | list literal on the prepared route (memo verification cost) | a | 2/2 | 101,993 | 446 | 1 |
| match_arm_statement_body_parse_test.dag | 1 | `match_arm_stmt_body` grammar row stamping and stop | a | **0/1**: FAIL `a_match_arm_statement_body_is_stamped_and_stops_at_the_next_arm_holds` | 115,291 | 438 | 1 |
| parse_acceptance_census_test.dag | 2 | `v2.compiler.parse_acceptance_census` accepted / refusal-by-reason reporting | a | 2/2 | 122,763 | 447 | 1 |
| parse_binding_fidelity_test.dag | 3 | cross-file import binding, unbound / ambiguous fail-closed through `resolve` | a | 3/3 | 54,692 | 152 | 0 |
| parse_table_claims_test.dag | 6 | 02_parse table morphisms: nullable, left-corner, undefined nonterminal, combinator | a | 6/6 | 5,525 | 14 | 0 |
| parse_table_content_key_test.dag | 9 | parse-table memo subject key and stream digest sensitivity | a | 9/9 | 6,029 | 4 | 0 |
| parse_token_first_empty_semantics_test.dag | 2 | `parse_token_first` agrees with `list_at_optional` | a | 2/2 | 48 | 0 | 0 |
| token_stream_cursor_test.dag | 6 | 02_parse `TokenStreamCursor` operations | a | 6/6 | 316 | 0 | 0 |
| type_decl_modifier_carrier_test.dag | 6 | type-decl modifier slot on `NormalizedTree` (#13033) | a | 6/6 | 251,036 | 644 | 0 |
| type_decl_modifier_g0_parse_probe_test.dag | 13 | G0 type-decl modifiers (`nominal_opaque`, `sole_constructor`), lexeme stamping | a | 13/13 | 511,904 | 1,267 | 0 |
| variant_field_lowering_test.dag | 34 | Pkg11c: variant and record fields to declared field identities on the native route | a | **33/34**: FAIL `vf_shadowing_binder_is_carried_on_the_arm` | 3,996,435 | 6,902 | 15 |
| weather_g0_prefix_parse_probe_test.dag | 13 | G0 float literals (weather.dag native-parse frontier) | a | 13/13 | 237,322 | 644 | 0 |
| where_refinement_clause_parse_test.dag | 5 | `where` refinement clause (brand/range), parse plus normalize graft | a | 5/5 | 474,587 | 1,049 | 4 |
| parse_binding_fidelity_support.dag | 0 | support: consumed by `test.claim.long` parse_binding_fidelity witness and `body_lowering` helpers | support (no gate decision) | — | — | — | — |
| supplied_token_stream_support.dag | 0 | support: consumed by 11 claim modules (normalize, namespace_graft, provenance, parse) | support (no gate decision) | — | — | — | — |
| go.dag | 1 data row | Go lex+grammar accepts `add` | b (see note) | not executable on floor route | — | — | — |
| kotlin.dag | 1 data row | Kotlin grammar | b (see note) | not executable on floor route | — | — | — |
| lean.dag | 1 data row | Lean grammar | b (see note) | not executable on floor route | — | — | — |
| python.dag | 1 data row | Python lex+grammar, extended fn | b (see note) | not executable on floor route | — | — | — |
| rust.dag | 3 data rows | Rust grammar: extended fn, `return;`, tight layout | b (see note) | not executable on floor route | — | — | — |
| swift.dag | 1 data row | Swift grammar | b (see note) | not executable on floor route | — | — | — |
| typescript.dag | 6 data rows | TS type alias, string-literal union | b (see note) | not executable on floor route | — | — | — |

**Note on the 7 language modules (b).** The subject is ingesting a foreign language through `X_grammar()` / `X_lex()`. A grep for callers outside `src/v2/extdeps/languages/` and the test trees (`git grep -nE "(python|go|kotlin|swift|lean|typescript|rust)_(grammar|lex)\b" -- '*.dag' ':!src/v2/extdeps/languages/*' ':!src/v2/test/*' ':!dag/test/*'`) returns **nothing**. Neither does a grep of `src/v2/compiler`, `src/v2/cli` and `dag/gunbc` for `X_grammar_root(`. So no part of the compile closure ingests these languages. Two further facts:

- python, go and kotlin already have `src/v2/test/claim/manual/*_grammar_claim_test.dag` over the same grammars.
- None of the 7 has a callable claim shape.

The proposed disposition is DELETE: the manual claims remain for the three languages that have them. **This contradicts the brief, which named `python_grammar_root` as a live root.** Grep shows no production consumer of `python_grammar_root`; its only references are `extdeps/languages/python.dag` and two manual test modules. Rust is an emission target, but `rust_grammar()` itself has no caller, and the brief's "one grammar both directions" route does not reach it today. The owner of `extdeps/languages` should confirm before anything is deleted.

**(c) other lane:** none assigned. Every in-scope module was added by `gunbai-bot[bot]` commits (PR titles in `git log --follow`), and the subjects are all dag-grammar / 02_parse, which is the grammar program's own territory. The G0 probes (`g0_service_decl`, `type_decl_modifier_g0`, `weather_g0_prefix`) came from the native-parse frontier work (#11575, #116xx). If that lane is still open, it is their natural owner. I could not establish an open owner from the dashboard.

## Summary

| class | modules | claims |
|---|---|---|
| (a) gate | 26 | 237 (7 FAIL on main) |
| (b) delete | 7 | 14 data rows (not floor-executable) |
| (c) other lane | 0 | — |
| support (no claims) | 2 | 0 |
| **total** | **35** | |

**Added floor cost if every (a) were gated** (measured by ea96a65d, before any split or supply rework):
- **15,713,125 marginal eval steps / 32,225 cpu ms** over 237 claims,
- plus the shared fill of **6,207,706 eval steps / 19,104 cpu ms** that the floor already serves for any dag-grammar consumer (it is not new cost if the floor already pays it).

**69 of 237 claims exceed the 72.3k budget.** The largest are `vf_distinct_payloads_normalize_distinct` (328,315), `vf_content_hash_declarations_and_positional_match_normalize` (316,893) and `cause_a_where_clause_survives_normalize_holds` (306,417). Nearly all of the over-budget claims run parse→normalize on a source string. Under DESIGN §3 "a witness discriminates at one interface" that is the reach-too-far shape: supply the token stream or tree, and keep one inhabitance claim per boundary. They cannot be enrolled as they stand.

## What this means for gating

1. Gating any (a) module turns the required floor red at once for the five modules carrying the 7 FALSE claims (`alias_decl_parse`, `closure_parse_batch_two`, `g0_service_decl_parse_probe`, `match_arm_statement_body_parse`, `variant_field_lowering`). Those reds are standing defects on main today, invisible because of exactly the gap this census is about. They need triage first. Each one is either a real regression or a stale claim.
2. The 69 over-budget claims need a split by producer or supplied inputs before enrolment. A debt row is not the remedy (§3, gunbc#11457).
3. Cheap and green now, so they can be gated as-is: `grammar_expect`, `grammar_not_predicate`, `keyword_path_segment`, `parse_binding_fidelity`, `parse_table_claims`, `parse_table_content_key`, `parse_token_first_empty_semantics`, `token_stream_cursor`, `type_decl_modifier_carrier`, `type_decl_modifier_g0_parse_probe`, `weather_g0_prefix_parse_probe`. That is 11 modules, 68 claims, 1,187,699 eval steps and 3,193 cpu ms.
