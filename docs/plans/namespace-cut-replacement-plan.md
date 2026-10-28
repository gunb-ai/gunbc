# Namespace cut: replace imports with containment resolution

**Goal:** delete the import grammar, parse surface and import-name universe. **Method:** `SymbolIndex` from `v2.compiler.symbol_index_fill`, resolved by `v2.compiler.resolve` containment walks; derive dependencies from references. **Validation:** typed parse refusal for `import`, containment-only binding, restored reference-derived emission/order, dispositioned census and applicable drift/regen receipts.

This is reasoning and history, not live standing. Current authorities:

- `gunbc.namespace_cut.namespace_cut_landing_order.current_landing_order` derives the supersession: the 2026-08-15 grammar-first ruling was replaced **on order only** by `namespace_cut_grammar_last_ruling` (2026-08-25). Grammar deletion is **last**; the root and concealment census remain. The rulings do not agree on order; the later ruling also permits a technically non-atomic cut.
- `required_ci_phase_roster.required_ci_phases` owns CI enrollment; `namespace_cut_stage.stage_prerequisites` owns sequence; `namespace_cut_subject_roster.namespace_cut_subjects` and `namespace_cut_subject_observation.subject_roster_report` own the denominator/observations; `compiler_frontend_program_status.where_are_we` derives standing. Predating claims of missing walls are historical.
- [Floor-cut census limits](floor-cut-replacement-plan.md) apply unchanged: deletion does not reveal its three silent populations.

Generation 3 executed steps 0–5 on frozen `integration/namespace-cut` / #8282, based on `64ebefa74`; that branch will not merge. Land its findings on main and perform the re-cut in a temporary worktree, not another maintained integration PR. #9201 carries the read-and-close postmortem; no execution requirement depends on it. Its findings’ rulings live in typed carriers. Full qualification versus shortest unique suffix remains an operator decision. Existing designs supply terminal-shape evidence, never authority to defer the root cut.

At each major step, stop and present for operator satisfaction; fix forward continuously within the step. The pre-strip resolver is an **offline differential oracle only**: occurrence→exact old binding→canonical qualification. Refuse old-ambiguous/unbound migrations; never install it as fallback or a production second opinion.

## Step 0 — historical strip and census

Strip receipts: `d4916cacf3` (`dag`/`src/v2`) and `59db42ffc9` (`src/v1`). The [full dated readings](https://github.com/gunb-ai/gunbc/blob/3e5a0f1f0dbc62e0d3589df44146858d3147c9e9/docs/plans/namespace-cut-replacement-plan.md) retain raw counts, class breakdowns and reproduction notes. They are one-off outputs of an **unowned** procedure, not a live instrument.

Reproduction used brace-balanced import removal in detached worktrees, not line deletion or test-only `name_resolution_policy_set_namespace_only`. Build each compiler from its named tree with `cargo build --release -p v1-compiler --bin gunbc`; run `gunbc compile --source-root dag --source-root src/v2 --source-root src/v1 --dependency-pool-index primary-precedence --target dag --output-dir <tmp>`, reading hard diagnostics. `compile_clean_source_roots` owns the compile entry, not stripping/classification. A typed source-rewrite capability plus instruments-layer census producer retires these one-off readings; do not build a throwaway count reproducer.

Controls held the August compiler (`64ebefa7416`, not September’s `f84c91b774a`) fixed across August/current (`de531c35496`) stripped and unstripped subjects. The original strip/class breakdown reproduced before comparison. The current compiler refuses the old tree at module indexing; patching it would change the subject. Unstripped controls were not compile-clean, so stripped counts are totals, not deltas. Growth persists under the pinned compiler and after removing its source-annotation class: the corpus grew, not merely diagnostic precision. Historical name counts used incompatible rules; omitted classes mean unreported, not zero. Prior declaration-registration/sibling-homonym hypotheses were retracted because probes did not match corpus conditions. Re-size step 4 using current classes and real specimens.

## Step 1 — import-name-universe deletion (before deep census diagnosis)

- `src/v1/03_resolve.dag`: `resolve_module_imports`, `resolve_import`, `ResolvedImport` + `resolved_imports`, `imports_by_name`, the `imported_names` term in `get_exported_names`, module-graph adjacency from `module_imports` (+ twin `v1_compiler_resolve.rs`)
- `src/v1/04_env.dag`: the `source_visible_names` field on `TypeEnv` — **this field is the import universe** — plus `bare_free_call_requires_listed_import`, `import_visible_name`, `global_bare_blocked_by_listed_import_requirement`, `listed_import_required_bare_call_blocked` (+ twin `v1_compiler_infer_env.rs`)
- `src/v1/04_infer.dag`: the `source_visible_names` fold, `merge_scope_from_imports`, `type_env_for_import`, `interface_env_for_import`, `overlay_direct_import_exports`, `bind_imported_name_from_surface`, `build_imported_variants`, `import_module_path_at`, `direct_import_export_name_sets`, `direct_import_exporter_counts`/`_count_of`, `rewire_type_env_import_str_binding_identity` (+ twin `v1_compiler_infer.rs`)
- `src/v1/04_resolve.dag`: the mask arm in `resolve_node` emitting `UnlistedImportUse`
- `src/v2/compiler/name_resolve.dag`: `collect_import_decl_nodes`, `import_rows_from_parsed_module`, `admit_import_root_find`, `admit_import_visible_entry`, `admit_import_entry`, `admit_imports`, the `Import` type + `imports` field, the `resolve_import_not_visible` / cross-tree-import refusal reasons
- `src/v2/compiler/translate.dag`: `is_dag_import_production_node`, `translate_import_production_qn_node`/`_block_node`/`_to_carrier`, `is_dag_import_carrier`, `translate_import_shortcut_after_subtree_gate` + both dispatch sites
- `src/v2/compiler/body_lowering_fold.dag`: the `dag_surface_import_decl`/`_block` passthrough arms
- `src/v2/lens/reference_deps.dag`: only the import-reading arms (`reference_qn_from_import_decl_strict` + dispatch, `reference_paths_from_import_edges`, `import_edge_to_reference_fact`) — the module survives as the replacement producer

**Sequencing consequence:** land this before deep-diagnosing the census. Until X's resolution arms are gone, part of the 5,531 may be X's machinery judging over now-empty import lists (e.g. `get_exported_names` folding `imported_names` into export sets) — measuring the corpse, not Y. Re-derive the census after; the numbers moving is expected.

## Step 2 — host and emit machinery

- `src/v1/stage0/src/cli_run.rs` (**frozen file — surgical trims, never deletion**): `extract_import_paths` (the starts-with-import text scanner every host import fact builds on), `resolve_virtual_source_with_imports`, `source_declares_import_lines`, `import_module_paths_for_typed_module`, `emit_import_admission_list`; the closure-walk family (`build_import_adjacency`, `import_closure_from_adjacency`/`_from_facts`/`_live_paths`/`_live_paths_with_facts`, `reference_only_direct_import_paths`, `entry_file_touched_via_import_closure`, `import_closure_module_reaches_carrier_home`, `collect_import_closure_module_names_from_facts`, `roster_import_closure_nodes_pre_resolve`, `import_closure_files_from_graph`, `touched_file_in_import_closure`, `import_closure_repo_paths_for_entry`, `augment_closure_modules_from_import_facts`, `import_closure_dag_files`); the `declared_import_closure_*` compile family incl. `compile_entry_on_declared_import_closure_only` and `observe_declared_import_closure_symbol_binding`; the unlisted-import census family (`classify_unlisted_import_binding_source`, `compile_clean_unlisted_import_census`, `compile_clean_unlisted_import_use_blocks_from_policy`/`_cached`); the class-b import-closure gates
- `src/v1/05_emit_rust.dag` (+ twin `v1_compiler_emit_rust.rs`): `emit_imports` — `emit_extract_import_paths_fn` was deleted (generated main already routes compile through `cli_run::compile_emission`; the stamp had no caller) — `augment_scoped_data_item_index_with_imports`, `module_imports_std_serialization_coproduct_wire_contract`, `record_lit_resolved_ctor_import_names`, `emit_specific_import_block`, `explicit_import_source_module_for_name`, `import_variant_parent_for_name`, `wildcard_import_pool_surface_names`, `import_module_enum_scope`
- Whole-file deletions: `dag/gunbc/instruments/namespace_import_closure_behavioral_transport.dag` (its own dissolution trigger names this cut) · `dag/gunbc/class_b_import_closure_overlay.dag` · `dag/gunbc/declared_import_closure_binding.dag` · `dag/gunbc/instruments/dag_compile_clean_cli_floor_agreement.dag` · `dag/std/import_statement.dag` (empty stub) · `src/v2/test/claim/long/dag_import_block_lexeme_stamp.dag` · `dag/test/claim/module_graph_edge_source_witness.dag`
- Partial deletions: `dag/gunbc/compile_clean_diagnostic_policy.dag` (only the `UnlistedImportUse` enforcement declarations + dissolve rows) · `dag/gunbc/instruments/diagnostics_witness_transport.dag` (`diagnostics_import_resolution_suite`, `run_diagnostics_import_resolution_witness`)

**Trap (regen fixed point):** every v1 `.dag` deletion has a generated `.rs` twin under `src/v1/stage0/src/` — cut both sides in the same pass or `regen_verify` reds.

**Trap (attribute orphans, measured 2026-08-15 — three defects):** deleting a generated Rust struct while leaving its preceding `#[derive(...)]` lines attaches them to the next item (E0774 / conflicting impls). Rule: a struct deletion takes its preceding attribute lines with it, and the repair sweep is scoped to the files the deletion touched, never repo-wide (a repo-wide sweep removed three legitimate derives in untouched files, including the CLI's `#[derive(Parser)]`; caught only by pre-commit diff).

## Step 3 — repoints (contested: surviving consumers change edge supply)

- `src/v2/lens/module_graph.dag`: repoint `dependency_resolution_facts_live` to the reference-derived producer per the file's own `dependency_edge_source_migration_note`; `dependency_closure`, `import_closure`, `touched_path_in_closure`, `entry_affected_by_touched_paths` are edge-source-agnostic and survive under the new supply. Consumers riding through unchanged: `dag/gunbc/instruments/dag_compile_clean_scope.dag`, `dag/gunbc/instruments/module_impact_query_front_door.dag`, `dag/gunbc/instruments/rust_stage0_gates.dag`, `dag/gunbc/doc_graph_roots.dag`, `gunbc.stage0_partition_closure`, `gunbc.repo.repo_atlas_projection`, affected-set entry selection
- `cli_run.rs` `import_resolution_facts` / `dependency_resolution_facts` / `union_dedup_import_facts_reference_first`: the reference-first union collapses to reference-only; the dedup authority survives with one arm
- `layer_import_facts` family + its interpreter builtin arm: `v2.std.layer_import_scan` supplies `v2.std.layer.LayerImportFact` to surviving containment consumers. Its `reference_edges_as_import_facts(strict=true)` bridge covers import-less files; import syntax currently wins duplicate (path, module) pairs. Replace that mixed supply with reference-only facts. The retired directional layering gate is not a consumer to restore
- `src/v1/05_emit_rust.dag` `reference_derived_use_lines`: re-derive its type-ref channel from the containment resolver; the pass becomes the sole use-line producer once `emit_imports` goes. **This is restoration, not cleanup** (crisp-crab finding, 2026-08-15): the channel was fed by `UnlistedImportUse` diagnostics whose mask guards on a non-empty `source_visible_names` — empty for every module post-strip — so the strip already silently emptied it. Rust emission is degraded on the branch until this lands; an emit-green result before it is the suspicious one. Related: `topological_sort` derives build order from `module_imports`, so ordering is flat (every module in-degree 0) until reference-derived edges supply it — ordering-sensitive results are not meaningful before then.
- `src/v1/05_emit_python.dag` `emit_py_imports` / `05_emit_go.dag` `emit_go_imports` etc.: target-language import emission survives; only the input (dag import nodes) is re-derived from references
- `dag/gunbc/instruments/dag_compile_clean_scope.dag`: re-word the import-closure prose; keep the tool
- `src/v1/04_emit_info.dag` `collect_type_node_import_surface_names`: re-derive from references

## Step 4 — fix-forward on the census, class grain

Method (post-retraction): instrument **real specimens** from the census, never synthetic fixtures — a probe answers questions about the probe. First specimen: `LiveTreeDisposition` (579 failures) at `dag/test/claim/lifecycle_survivor_corpus_census.dag`. Per specimen, pin first **which resolver path judged it** (mask / single-candidate-from-anywhere / NamespaceOnlyY chain filter); the instrumented binary is rebuilt from the branch head. Classes are decided deliberately in sequence and applied mechanically within a class; the function-not-found majority carries an open question — were function references ever candidates in the reference-derived path, or only visible through the import universe? The qualification sweep uses the old resolver as offline oracle only.

## Step 5 — grammar/parse deletion, LAST (`import` becomes a parse error)

> **THIS STEP MOVED FROM FIRST TO LAST (operator, 2026-08-25 — `gunbc.namespace_cut.namespace_cut_landing_order`
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

`src/v2/std/symbol_index.dag` · `src/v2/compiler/symbol_index_fill.dag` · `src/v2/compiler/resolve.dag` (verified zero import machinery) · `src/v2/compiler/namespace_graft.dag` · `src/v2/std/qualified_name.dag` · `src/v2/std/decl_ref_resolution.dag` · the rest of `src/v2/extdeps/languages/dag.dag` beyond the import production · the v2 stage files `01_tokenize`/`02_parse`/`03_normalize`/`03_ingest`/`program_assembly`/`parse_engine_hooks`/`normalized_tree` (verified import-free) · `src/v1/stage0/src/gunbc_namespace_cut_namespace_reference_derived_closure_admission.rs` + `_contract.rs`. **CORRECTED (2026-08-15, lane divergence report):** the raw-text scanner family (`referenced_module_paths_in_text`, `extend_with_reference_closure` and kin) is **not** the terminal closure authority this plan originally named — it is a pre-parse byte scanner that longest-matches dotted identifiers against a name index and cannot distinguish a reference from prose (in-tree receipt: the English word "edge" in a source annotation bound to `fn edge` and pulled its module into an unrelated entry's pool). DESIGN §4 rules the heuristic never necessary in a closed system, so the terminal mechanism is **parse-then-derive**: parse the entry standalone (per-file, needs no closure), read real references from the `Node` tree, load those modules, fixpoint — terminating by construction. The scanner family dies, with its five `extract_imports` forks (tests/helpers.rs plus four witness bins). The new closure assembly is homed **outside `cli_run.rs`**, which `integration/cli-run-cut` deletes wholesale. Grep false-positives: the TypeScript/Go/Swift extdeps files and the two typescript import pipeline tests — other languages own their `import` keywords.

**Declared mid-loop gap (2026-08-15):** `regen_stage0` cannot run while the corpus does not resolve, so every `.dag` → `.rs` mirror edit is hand-written and verified by a real build for the lane's duration — a declared rung-drop carried openly.

**Restoration trigger is ORDER-DEPENDENT (corrected 2026-08-15).** `integration/v1-cut` (gunbc#8293) deletes `regen_stage0` and its 142-entry roster outright, so "regen green on the branch" can fire **only if this cut reaches main first**. If the v1 cut lands first, this obligation **retires with its mechanism, carrying a receipt** — not restored, and not left standing as a trigger whose subject no longer exists. That is §3 step 6 (evidence whose only subject is retired is retired with it), not a §4b stall: a class below its ceiling must name a trigger that *can* fire, and an unfireable one is worse than none because it reads as tracked. The same order-dependency governs step 2's two regen mentions (the `emit_imports` seed-regrowth warning and the regen fixed-point trap) and the green-bar item below; it is stated once here. The banked import-semantics properties (`docs/plans/import-deletion-recovered-behaviors.md`, branch-local) need a reachability edge before cutover.

## Green bar / cutover

`import` refuses at parse with a typed, located refusal (step 5, the LAST step — the branch receipt for it predates the 2026-08-25 reordering and is evidence the deletion works, not that this cut's step 5 is done); the corpus resolves through the containment rule **by construction — never pool coincidence**; Rust-emit use-lines restored from the containment resolver (the strip silently emptied the old channel — see step 3); the census worked to zero or every remaining row dispositioned; the regen fixed point and drift gates re-established on the branch — **or, if `integration/v1-cut` reached main first, the regen half retired with `regen_stage0` under the order-dependency above, the drift gates standing on their own** (they are independent of regen — **observed 2026-08-15** on `integration/v1-cut`: `heal_generated_artifacts` passed while `regen` was red; that branch has since retired the regen job outright, so the observation is a dated receipt for the independence, not a live claim about either branch's current jobs). Then #8282 flips ready and the operator merges — one atomic cutover.

## Registration

`gunbc.replacement_cut` row `NAMESPACE-Y` — the carrier merged to main in gunbc#8276 (2026-08-15), so the row is now authorable and is an open follow-up for the executing session rather than a blocked one.
