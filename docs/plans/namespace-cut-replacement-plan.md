# Namespace cut (NAMESPACE-Y): delete the import concept, resolve by containment

> **CENSUS RECEIPT (2026-08-25).** Generation 3 executed steps 0-5 on
> `integration/namespace-cut` (gunbc#8282), now frozen. **This plan is not superseded as to
> its ROOT or its CENSUS** — its 2026-08-15 ruling called for exactly what happened on that
> branch, and the defects it surfaced are the concealment census running. (It IS superseded
> on ORDER; see the next paragraph, which an executor reads for sequencing.) What the census
> returned, with a terminal shape per class, is the namespace-cut postmortem — **deliberately
> NOT a file in this tree.** It is carried as an open PR (gunbc#9201) under the 2026-08-26
> operator rule that document-only changes are read as a PR and closed, never merged; its
> measurement rows are snapshots that rot. **Nothing an executor needs is sourced from it:
> every ruling it records lives in a typed carrier, cited by symbol below.**
> **THE 2026-08-15 ORDERING IS SUPERSEDED ON ORDER (operator, 2026-08-25): the grammar
> deletion is the LAST step, not the first.** The ROOT is unchanged — grammar, parse surface
> and import-name universe are all still inside it, and delete-first as a doctrine is
> untouched — but the order of cutting that root was reversed by a later ruling from the same
> authority, which also ruled the cut "doesn't have to be atomic technically". Every sentence
> below that states or implies grammar-first is read subject to this paragraph. Neither this
> plan nor any other artifact may claim the two rulings agree.
>
> **THE RULING'S AUTHORITY IS THE CARRIER, NOT THIS PARAGRAPH AND NOT A DOCUMENT.**
> `gunbc.namespace_cut_landing_order` holds both rulings verbatim with the supersession typed
> (`namespace_cut_grammar_last_ruling`, `SupersedesOnOrderOnly`); `current_landing_order`
> derives the answer rather than storing a copy. Cite that module and symbol, never a file
> path — DESIGN §4c names *ruling* among the facts that belong in a typed carrier; prose here
> is the reasoning, the carrier is the fact.
>
> Two further amendments it argues for: the census output lands on **main** rather than being
> fixed forward on the branch (the branch will not merge, so fixes on it are throwaway work
> per DESIGN section 6), and the re-cut runs in a temporary worktree rather than a maintained
> PR (222 merges of main into the branch, against roughly 380 commits of its own work). The
> **target qualification spelling** — this plan's full qualification vs
> namespace-resolution-design's shortest unique suffix, whose *sequencing* this plan quarries
> but whose *terminal shapes* it keeps as evidence — is reopened there as an operator
> decision.

> **CURRENT-STATE AUTHORITY (2026-08-29): this document carries REASONING and HISTORY, never
> live standing.** Enrolment, standing and sequencing facts are typed carriers, cited by symbol:
> the required-CI phase roster (and with it the wave-admission wall's enrolment) is
> `gunbc.required_ci_phase_roster` `required_ci_phases`; the NAMESPACE-XL stage chain and its
> prerequisites are `gunbc.namespace_cut_stage` `stage_prerequisites`; the deletion-subject
> denominator is `gunbc.namespace_cut_subject_roster` `namespace_cut_subjects`, observed by
> `gunbc.namespace_cut_subject_observation` `subject_roster_report`; and the derived standing
> report is `gunbc.compiler_frontend_program_status` `where_are_we`. Where a sentence below
> disagrees with those carriers, the carrier is correct and the sentence is a dated record —
> in particular, sentences predating #9365 that say no wall or CI mechanism exists.

Doctrine: DESIGN §3 *replacement migrations cut over at the root* (delete-first; the deletion is the census — bounded by the three silent populations measured across the five parallel cuts, recorded once in [floor-cut-replacement-plan.md](floor-cut-replacement-plan.md) under *What the census does not do*; they apply to this cut identically). Vehicle: integration branch `integration/namespace-cut`, forked from main `64ebefa74`; standing cutover PR gunbc#8282 (draft — the one merge main receives). Executing session: crisp-crab-430; doctrine/coordination: tidy-pike-117. Operator ruling (2026-08-15): delete all the grammar/import up front, then solve each problem as it is revealed — expecting the deletion to also reveal problems import was standing up (the concealment census). **SUPERSEDED ON ORDER 2026-08-25 — `gunbc.namespace_cut_landing_order` `namespace_cut_grammar_last_ruling`: grammar deletion is last.** The concealment-census expectation survives; what moved is when the grammar is cut, not what cutting it reveals. **Step gating:** each major step closes only when the operator is satisfied with its performance — the executing session stops at the boundary and presents, never rolls into the next step on its own judgment; within a step, the fix-forward loop runs continuously. **Existing designs** covering this region (namespace-resolution-design, the layering repoint design, and kin) are quarry — terminal shapes and mechanisms are evidence; their sequencing never defers the deletion (operator ruling, 2026-08-15; the 2026-08-25 reordering is the same authority moving the grammar step, not a quarried design deferring it — the distinction this clause enforces).

## The cut, stated in .dag terms

- **Old root:** the `import` concept — the grammar production, the parse surface, and the import-name universe (visibility = own declarations ∪ direct import lists).
- **New root:** containment-tree resolution — `v2.std.symbol_index` `SymbolIndex` filled by `src/v2/compiler/symbol_index_fill.dag`, walked by `src/v2/compiler/03_resolve.dag` (`lookup_symbol_index_atom_identity`, chain lookup, qualified projection) — with dependency edges **derived from references**, exactly as `src/v2/lens/module_graph.dag`'s own `dependency_edge_source_migration_note` already specifies.
- **X's residual roles:** the pre-strip resolver (from quarry or a pinned ref) as OFFLINE differential oracle for the qualification sweep — occurrence → exact old target → canonical qualified spelling; only exact old bindings rewrite mechanically; old-ambiguous and old-unbound refuse migration. Never a fallback, never a production second opinion.

## Step 0 — DONE (dated receipts, 2026-08-15): the strip

- `d4916cacf3` — Delete every import statement from `dag/` and `src/v2/` (2,975 files, −62,850 lines)
- `59db42ffc9` — Delete every import statement from `src/v1/` (the seed; 60 files, −1,837 lines)

**RE-DERIVED 2026-09-02 at main `de531c35496`; the 2026-08-15 figures below are a dated record, not standing.**

> **THIS SECTION IS A DATED RECEIPT OF TWO NAMED RUNS, NOT A STANDING CENSUS, AND IT IS NOT THE MEASUREMENT AUTHORITY FOR ANYTHING.** DESIGN §6 asks that a measurement be cited by naming the producer that re-derives it. Half of this producer is owned — the compile is `gunbc compile` over `gunbc.ci_layer_roots` `compile_clean_source_roots`, an entry point that exists. **The other half is not: no owned entry point performs the strip or derives the classification.** Brace-balanced deletion of an import statement is a parse-level rewrite and the corpus has no typed source-rewrite op for it, so a real producer is a new instruments-layer transport plus the op it needs. Recorded here as the obligation it is rather than left implicit (§4b(2), no untracked stall): **these figures rot the way the 2026-08-15 ones did until that producer exists, and the trigger that retires this admission is a modeled import-strip census instrument whose output is these counts — not a re-run of the procedure below by hand.** Raised by `review 58770`; the shortcut was checked and is not available, since the compiler's namespace-only resolution policy is test-only and would not remove the import universe in any case.

 Two instruments, both stated so they can be re-run rather than believed. **Current:** build `gunbc` from the measured commit (`cargo build --release -p v1-compiler --bin gunbc`), delete every import statement brace-balanced from `dag/`, `src/v2/` and `src/v1/` in a detached scratch worktree (a line-based delete strands the member lists), then run one compile over the compile-clean root set — `gunbc compile --source-root dag --source-root src/v2 --source-root src/v1 --dependency-pool-index primary-precedence --target dag --output-dir <tmp>` (`gunbc.ci_layer_roots` `compile_clean_source_roots`) — and read its terminal `N blocking error(s)` line and `error[file:line:col]: msg` rows. The unstripped tree under the same invocation is the control. **Pinned:** the same strip and the same invocation, with `gunbc` built from `64ebefa7416` instead — the tree `d4916cacf3` stripped — run against BOTH trees, which is what separates a corpus delta from an instrument delta. The current compiler cannot serve as the pinned instrument in the other direction: it refuses the August tree at module-index time (`module index refused: 22 unparseable .dag source(s)`, 26 unstripped) over walls that landed since, and patching those files would mutate the subject.

- **Strip subject:** 25,121 import statements across 3,886 files (single-line 17,980 · multi-line 6,765 · bare 376), zero residue — against 19,347 / 2,975 in 2026-08-15's `dag/`+`src/v2/` pass plus 381 / 60 in the seed.
- **Control (unstripped main, same invocation):** 78 blocking errors across 16 files. Main is not compile-clean under the v1-inclusive root set, so the stripped figure is a total, not a delta.
- **Stripped census:** **10,436 hard diagnostics · 272 distinct names · 923 files**  (a name is the subject the diagnostic names in its own class — the unresolved type, the missing function, the ambiguous reference; counting the first quoted token in every message instead gives 305); 3,427 sources resolved, 1,106 modules indexed in the name census.
- **By class, summing to 10,436:** unresolved type 3,247 · no-field 2,468 · ambiguous reference 1,701 · undefined variable 1,054 · method-unresolved 839 · function-not-found 777 · variant-not-found 186 · other 164. Six diagnostics carry a `<kernel:String>`/`<kernel:V>` span rather than a file; they are not a class of their own — all six are variant-not-found (`Accepted` ×3, `Rejected` ×3) and are counted in that class's 186. They are excluded from the 923-file spread, which counts files.
- **Top names:** `children` 704 · `Nat` 675 · `FilePath` 640 · `decl` 623 · `row` 598 · `trim` 528 · `Empty` 525 · `Node` 464.

**The 5,531/280/935 census does NOT still hold, and the cause is the CORPUS, not the instrument.** Establishing that needed a second run, because the first comparison changed the compiler and the tree at the same time: a delta across a changed instrument is not a delta in the subject. So the instrument was pinned at August — `gunbc` built from `64ebefa7416`, the tree `d4916cacf3` stripped — and pointed at both trees under the identical strip and invocation:

| instrument · subject | diagnostics | files | names |
|---|---|---|---|
| **August compiler · August tree, stripped** | **5,531** | 934 | 655 |
| **August compiler · current tree, stripped** | **12,548** | 1,312 | 870 |
| August compiler · August tree, unstripped (baseline) | 41 | 2 | 3 |
| current compiler · current tree, stripped | 10,436 | 923 | 272 |
| current compiler · current tree, unstripped (baseline) | 78 | 16 | 8 |

**The first row reproduces the 2026-08-15 census exactly** — 5,531 diagnostics, and its class breakdown to the digit: function-not-found 2,065 · unresolved type 1,742 · undefined variable 1,067. The strip subject reproduces too (19,728 statements / 3,035 files, matching the two strip commits' own 19,347+381 / 2,975+60). So the pipeline is validated against the original measurement before it is used to compare anything, and the second row is a like-for-like reading of the same instrument on a corpus 3,226 changed `.dag` files later: **5,531 → 12,548, files 934 → 1,312.**

**The compiler-refinement reading is refuted by its own prediction.** Had the doubling been finer diagnosis of a stable corpus, the August compiler on the August tree would have had to report roughly today's volume; it reports 5,531. Nor is the growth an artifact of one new class: 3,796 of the 12,548 are `source annotation names no subject`, §4c annotations the August compiler cannot attach, and excluding that class entirely still leaves 8,752 — **+58% on the August compiler's own vocabulary.**

**Two corrections to the first reading of these figures, both from comparing across instruments.** The spread did **not** stay flat: held at one compiler it grows 934 → 1,312 files, so the strip reaches materially MORE of the corpus than it did in August, not the same sites twice over. And the name column should not be compared at all — 280 is not re-derivable from its own instrument's output under any obvious rule (the reproduction gives 655 distinct quoted subjects, 555 restricted to the three enumerated classes), so 280 → 272 was two different counting rules, not a measurement.

**The four classes with no August number are UNREPORTED, not zero.** The 2026-08-15 census enumerated its top three classes only — 4,874 of 5,531, leaving 657 in classes it never listed — and all seven of today's class spellings are present in the August compiler's sources at `64ebefa7416`. No part of today's volume belongs to a diagnostic that did not exist.

**Consequence for step 4:** the census is genuinely bigger, so step 4 is re-sized as well as re-sorted. Its class-grain fix-forward should be prioritized off the current mix — and note that the current compiler reports FEWER diagnostics on the current tree than the August one does (10,436 vs 12,548), so the class counts to work from are the current compiler's.

Standing census on the branch (2026-08-15, dated record): **5,531 hard diagnostics · 280 distinct names · 935 files · 2,355 of 3,709 modules in closure**; by class: function-not-found 2,065 · unresolved type 1,742 · undefined variable 1,067; top names `LiveTreeDisposition` 579 · `SubstrateInputsOnly` 506 · `OccurrenceId` 347. Two earlier root-cause hypotheses (declaration-registration gap; sibling-homonym eligibility) were **retracted** — the probes ran under conditions differing from the corpus (their diagnostic appears zero times in the census). That census stood as of that date and is superseded as a measurement by the 2026-09-02 re-derivation above; the hypotheses remain open questions for step 4's instrument.

## Step 1 — import-name-universe deletion (before deep census diagnosis)

- `src/v1/03_resolve.dag`: `resolve_module_imports`, `resolve_import`, `ResolvedImport` + `resolved_imports`, `imports_by_name`, the `imported_names` term in `get_exported_names`, module-graph adjacency from `module_imports` (+ twin `v1_compiler_resolve.rs`)
- `src/v1/04_env.dag`: the `source_visible_names` field on `TypeEnv` — **this field is the import universe** — plus `bare_free_call_requires_listed_import`, `import_visible_name`, `global_bare_blocked_by_listed_import_requirement`, `listed_import_required_bare_call_blocked` (+ twin `v1_compiler_infer_env.rs`)
- `src/v1/04_infer.dag`: the `source_visible_names` fold, `merge_scope_from_imports`, `type_env_for_import`, `interface_env_for_import`, `overlay_direct_import_exports`, `bind_imported_name_from_surface`, `build_imported_variants`, `import_module_path_at`, `direct_import_export_name_sets`, `direct_import_exporter_counts`/`_count_of`, `rewire_type_env_import_str_binding_identity` (+ twin `v1_compiler_infer.rs`)
- `src/v1/04_resolve.dag`: the mask arm in `resolve_node` emitting `UnlistedImportUse`
- `src/v2/compiler/03_name_resolve.dag`: `collect_import_decl_nodes`, `import_rows_from_parsed_module`, `admit_import_root_find`, `admit_import_visible_entry`, `admit_import_entry`, `admit_imports`, the `Import` type + `imports` field, the `resolve_import_not_visible` / cross-tree-import refusal reasons
- `src/v2/compiler/06_translate.dag`: `is_dag_import_production_node`, `translate_import_production_qn_node`/`_block_node`/`_to_carrier`, `is_dag_import_carrier`, `translate_import_shortcut_after_subtree_gate` + both dispatch sites
- `src/v2/compiler/body_lowering_fold.dag`: the `dag_surface_import_decl`/`_block` passthrough arms
- `src/v2/lens/reference_deps.dag`: only the import-reading arms (`reference_qn_from_import_decl_strict` + dispatch, `reference_paths_from_import_edges`, `import_edge_to_reference_fact`) — the module survives as the replacement producer

**Sequencing consequence:** land this before deep-diagnosing the census. Until X's resolution arms are gone, part of the 5,531 may be X's machinery judging over now-empty import lists (e.g. `get_exported_names` folding `imported_names` into export sets) — measuring the corpse, not Y. Re-derive the census after; the numbers moving is expected.

## Step 2 — host and emit machinery

- `src/v1/stage0/src/cli_run.rs` (**frozen file — surgical trims, never deletion**): `extract_import_paths` (the starts-with-import text scanner every host import fact builds on), `resolve_virtual_source_with_imports`, `source_declares_import_lines`, `import_module_paths_for_typed_module`, `emit_import_admission_list`; the closure-walk family (`build_import_adjacency`, `import_closure_from_adjacency`/`_from_facts`/`_live_paths`/`_live_paths_with_facts`, `reference_only_direct_import_paths`, `entry_file_touched_via_import_closure`, `import_closure_module_reaches_carrier_home`, `collect_import_closure_module_names_from_facts`, `roster_import_closure_nodes_pre_resolve`, `import_closure_files_from_graph`, `touched_file_in_import_closure`, `import_closure_repo_paths_for_entry`, `augment_closure_modules_from_import_facts`, `import_closure_dag_files`); the `declared_import_closure_*` compile family incl. `compile_entry_on_declared_import_closure_only` and `observe_declared_import_closure_symbol_binding`; the unlisted-import census family (`classify_unlisted_import_binding_source`, `compile_clean_unlisted_import_census`, `compile_clean_unlisted_import_use_blocks_from_policy`/`_cached`); the class-b import-closure gates
- `src/v1/05_emit_rust.dag` (+ twin `v1_compiler_emit_rust.rs`): `emit_imports`, `emit_extract_import_paths_fn` — **this one prints the import scanner into generated stage0; cut it in the same pass or the seed regrows it on regen** — `augment_scoped_data_item_index_with_imports`, `module_imports_std_serialization_coproduct_wire_contract`, `record_lit_resolved_ctor_import_names`, `emit_specific_import_block`, `explicit_import_source_module_for_name`, `import_variant_parent_for_name`, `wildcard_import_pool_surface_names`, `import_module_enum_scope`
- Whole-file deletions: `dag/gunbc/instruments/namespace_import_closure_behavioral_transport.dag` (its own dissolution trigger names this cut) · `dag/gunbc/class_b_import_closure_overlay.dag` · `dag/gunbc/declared_import_closure_binding.dag` · `dag/gunbc/instruments/dag_compile_clean_cli_floor_agreement.dag` · `dag/std/import.dag` (empty stub) · `src/v2/test/claim/long/dag_import_block_lexeme_stamp_test.dag` · `dag/test/claim/module_graph_edge_source_witness_test.dag`
- Partial deletions: `dag/gunbc/compile_clean_diagnostic_policy.dag` (only the `UnlistedImportUse` enforcement declarations + dissolve rows) · `dag/gunbc/instruments/diagnostics_witness_transport.dag` (`diagnostics_import_resolution_suite`, `run_diagnostics_import_resolution_witness`)

**Trap (regen fixed point):** every v1 `.dag` deletion has a generated `.rs` twin under `src/v1/stage0/src/` — cut both sides in the same pass or `regen_verify` reds.

**Trap (attribute orphans, measured 2026-08-15 — three defects):** deleting a generated Rust struct while leaving its preceding `#[derive(...)]` lines attaches them to the next item (E0774 / conflicting impls). Rule: a struct deletion takes its preceding attribute lines with it, and the repair sweep is scoped to the files the deletion touched, never repo-wide (a repo-wide sweep removed three legitimate derives in untouched files, including the CLI's `#[derive(Parser)]`; caught only by pre-commit diff).

## Step 3 — repoints (contested: surviving consumers change edge supply)

- `src/v2/lens/module_graph.dag`: repoint `dependency_resolution_facts_live` to the reference-derived producer per the file's own `dependency_edge_source_migration_note`; `dependency_closure`, `import_closure`, `touched_path_in_closure`, `entry_affected_by_touched_paths` are edge-source-agnostic and survive under the new supply. Consumers riding through unchanged: `dag/gunbc/instruments/dag_compile_clean_scope.dag`, `dag/gunbc/instruments/module_impact_query_front_door.dag`, `dag/gunbc/instruments/rust_stage0_gates.dag`, `dag/gunbc/doc_graph_roots.dag`, `gunbc.stage0_partition_closure`, `gunbc.repo_atlas_projection`, affected-set entry selection
- `cli_run.rs` `import_resolution_facts` / `dependency_resolution_facts` / `union_dedup_import_facts_reference_first`: the reference-first union collapses to reference-only; the dedup authority survives with one arm
- `layer_import_facts` family + its interpreter builtin arm: `v2.std.layer` consumes it; the transitional reference arm already exists (`reference_edges_as_import_facts` strict=true, per `docs/plans/layering-imports-reference-repoint-design.md` §3.1) — becomes reference-only; repoint + rename
- `src/v1/05_emit_rust.dag` `reference_derived_use_lines`: re-derive its type-ref channel from the containment resolver; the pass becomes the sole use-line producer once `emit_imports` goes. **This is restoration, not cleanup** (crisp-crab finding, 2026-08-15): the channel was fed by `UnlistedImportUse` diagnostics whose mask guards on a non-empty `source_visible_names` — empty for every module post-strip — so the strip already silently emptied it. Rust emission is degraded on the branch until this lands; an emit-green result before it is the suspicious one. Related: `topological_sort` derives build order from `module_imports`, so ordering is flat (every module in-degree 0) until reference-derived edges supply it — ordering-sensitive results are not meaningful before then.
- `src/v1/05_emit_python.dag` `emit_py_imports` / `05_emit_go.dag` `emit_go_imports` etc.: target-language import emission survives; only the input (dag import nodes) is re-derived from references
- `dag/gunbc/instruments/dag_compile_clean_scope.dag`: re-word the import-closure prose; keep the tool
- `src/v1/04_emit_info.dag` `collect_type_node_import_surface_names`: re-derive from references

## Step 4 — fix-forward on the census, class grain

Method (post-retraction): instrument **real specimens** from the census, never synthetic fixtures — a probe answers questions about the probe. First specimen: `LiveTreeDisposition` (579 failures) at `dag/test/claim/lifecycle_survivor_corpus_census_test.dag`. Per specimen, pin first **which resolver path judged it** (mask / single-candidate-from-anywhere / NamespaceOnlyY chain filter); the instrumented binary is rebuilt from the branch head. Classes are decided deliberately in sequence and applied mechanically within a class; the function-not-found majority carries an open question — were function references ever candidates in the reference-derived path, or only visible through the import universe? The qualification sweep uses the old resolver as offline oracle only.

## Step 5 — grammar/parse deletion, LAST (`import` becomes a parse error)

> **THIS STEP MOVED FROM FIRST TO LAST (operator, 2026-08-25 — `gunbc.namespace_cut_landing_order`
> `namespace_cut_grammar_last_ruling`, derived by `current_landing_order`).** It was numbered Step 1
> until 2026-08-26, when `review 56029` found the operative sequence still encoded the superseded order
> while the header paragraph declared the new one — an executor reading the steps would have run
> grammar-first. Step CONTENT is unchanged; only its position moved. Steps 1-4 were renumbered down by
> one and every "step N" cross-reference re-pointed with them.


- `src/v2/extdeps/languages/dag.dag`: `dag_grammar_import_decl_expr`, `dag_grammar_import_block_expr`, `emit_import_decl_emitted_node`, `emit_row_import_decl`, `parse_import_block_idents`, the `dag_token_kw_import` lex rule, the import grammar formal rows, the production registrations, and the top-level-item alternative for import (retires the `dag_production_import_decl`/`_block` and `dag_surface_import_decl`/`_block`/`_qualified_name` identities). The v2 parser is production-table-driven, so deleting these rows is the cut.
- `dag/extdeps/languages/dag/syntax.dag`: the `import` entry in `dag_keyword_set` — plus its generated twin `src/v1/stage0/src/extdeps_languages_dag_syntax.rs`.
- v1 parse: `src/v1/02_parse.dag` `parse_imports`, `parse_imports_acc`, `parse_import`, `parse_import_names`, `parse_import_names_acc`, types `ImportResult`/`ImportsResult`, the call site in `parse_module` (+ twin `v1_compiler_parse.rs`). `src/v1/00_core.dag` `import_node`, `import_is_all`, `import_specific_names_at`, `module_imports`, the `imports` param on `module_node`, diagnostics `UnresolvedImport`/`MissingExport`/`UnlistedImportUse` (+ twin `v1_std_core.rs`). `src/v1/dag_collect.dag` `is_import_slot_node`. `src/v1/compile.dag` `serialize_import_node`, `is_import_statement_node`, `serialize_module_imports_json`.
- **The arity-zero import refusal dies here too, and it is listed rather than
  noted (operator ruling, warm-hawk-909, 2026-08-26).** `src/v1/02_parse.dag`'s
  arity-zero member-list refusal inside `parse_import` (+ twin
  `v1_compiler_parse.rs`) is a wall on **import syntax**, so by DESIGN §6's
  survival test — *will this artifact survive the terminal architecture
  substantially unchanged, and be consumed by it?* — the answer is **no: it dies
  with the grammar it parses**, making it presumed scaffold. It landed anyway as
  an explicit operator override on **cost, not principle**: it was already built,
  approved, regenerated and executing, and swapping it for an equal-strength
  fixture over a population of one is the churn §6 warns about as loudly as
  scaffolds. **The condition of that override is this entry.** It sits in Step
  5's deletion list rather than carrying a "temporary" note, so the grammar
  deletion removes it *by the census* — a row someone must execute — instead of
  leaving residue that survives because nobody remembers to look. A dissolution
  trigger that depends on memory is the self-authorized dissolution DESIGN names
  as a failure mode.

**Trap (typed refusal):** removing the keyword alone makes `import` lex as a bare identifier that fails much later with a confusing message. Keep a token class and add a refusing production so the refusal is typed, located, and names the cut.

## Do-not-delete (the replacement, and grep false-positives)

`src/v2/std/symbol_index.dag` · `src/v2/compiler/symbol_index_fill.dag` · `src/v2/compiler/03_resolve.dag` (verified zero import machinery) · `src/v2/compiler/namespace_graft.dag` · `src/v2/std/qualified_name.dag` · `src/v2/std/decl_ref_resolution.dag` · the rest of `src/v2/extdeps/languages/dag.dag` beyond the import production · the v2 stage files `01_tokenize`/`02_parse`/`03_normalize`/`03_ingest`/`program_assembly`/`parse_engine_hooks`/`normalized_tree` (verified import-free) · `src/v1/stage0/src/gunbc_namespace_reference_derived_closure_admission.rs` + `_contract.rs`. **CORRECTED (2026-08-15, lane divergence report):** the raw-text scanner family (`referenced_module_paths_in_text`, `extend_with_reference_closure` and kin) is **not** the terminal closure authority this plan originally named — it is a pre-parse byte scanner that longest-matches dotted identifiers against a name index and cannot distinguish a reference from prose (in-tree receipt: the English word "edge" in a source annotation bound to `fn edge` and pulled its module into an unrelated entry's pool). DESIGN §4 rules the heuristic never necessary in a closed system, so the terminal mechanism is **parse-then-derive**: parse the entry standalone (per-file, needs no closure), read real references from the `Node` tree, load those modules, fixpoint — terminating by construction. The scanner family dies, with its five `extract_imports` forks (tests/helpers.rs plus four witness bins). The new closure assembly is homed **outside `cli_run.rs`**, which `integration/cli-run-cut` deletes wholesale. Grep false-positives: the TypeScript/Go/Swift extdeps files and the two typescript import pipeline tests — other languages own their `import` keywords.

**Declared mid-loop gap (2026-08-15):** `regen_stage0` cannot run while the corpus does not resolve, so every `.dag` → `.rs` mirror edit is hand-written and verified by a real build for the lane's duration — a declared rung-drop carried openly.

**Restoration trigger is ORDER-DEPENDENT (corrected 2026-08-15).** `integration/v1-cut` (gunbc#8293) deletes `regen_stage0` and its 142-entry roster outright, so "regen green on the branch" can fire **only if this cut reaches main first**. If the v1 cut lands first, this obligation **retires with its mechanism, carrying a receipt** — not restored, and not left standing as a trigger whose subject no longer exists. That is §3 step 6 (evidence whose only subject is retired is retired with it), not a §4b stall: a class below its ceiling must name a trigger that *can* fire, and an unfireable one is worse than none because it reads as tracked. The same order-dependency governs step 2's two regen mentions (the `emit_imports` seed-regrowth warning and the regen fixed-point trap) and the green-bar item below; it is stated once here. The banked import-semantics properties (`docs/plans/import-deletion-recovered-behaviors.md`, branch-local) need a reachability edge before cutover.

## Green bar / cutover

`import` refuses at parse with a typed, located refusal (step 5, the LAST step — the branch receipt for it predates the 2026-08-25 reordering and is evidence the deletion works, not that this cut's step 5 is done); the corpus resolves through the containment rule **by construction — never pool coincidence**; Rust-emit use-lines restored from the containment resolver (the strip silently emptied the old channel — see step 3); the census worked to zero or every remaining row dispositioned; the regen fixed point and drift gates re-established on the branch — **or, if `integration/v1-cut` reached main first, the regen half retired with `regen_stage0` under the order-dependency above, the drift gates standing on their own** (they are independent of regen — **observed 2026-08-15** on `integration/v1-cut`: `heal_generated_artifacts` passed while `regen` was red; that branch has since retired the regen job outright, so the observation is a dated receipt for the independence, not a live claim about either branch's current jobs). Then #8282 flips ready and the operator merges — one atomic cutover.

## Registration

`gunbc.replacement_cut` row `NAMESPACE-Y` — the carrier merged to main in gunbc#8276 (2026-08-15), so the row is now authorable and is an open follow-up for the executing session rather than a blocked one.
