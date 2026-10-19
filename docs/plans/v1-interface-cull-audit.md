# v1 interface cull audit (working notes)

A working document for going through every interface in the v1 compiler and interpreter
(`src/v1/*.dag` and its seed realization `src/v1/stage0/`) and deleting what is clearly unused.
It is meant to be reread and amended over several passes. Each confirmed deletion lands as its own
change. This file records the survey and the open questions, not the authority. Once a candidate
class is confirmed, the census that found it should become an instrument (`gunbc test <label>`)
instead of staying a transcribed list here (DESIGN.md §6: name the instrument, never transcribe its
output). This file is then deleted.

Standing (DESIGN.md §3, `gunbc.v1.v1_maintenance_standing v1_seed_standing`): v1 is
semantics-frozen and maintenance-active. Deleting code that nothing consumes reduces seed surface
without changing semantics. That serves the self-host program and is admissible. Every row below is
a **candidate** until a build plus the required job shows it has no consumer.

Surveyed at `891198514b8`.

## 1. Top level

### 1.1 Two layers

| layer | where | size | notes |
|---|---|---|---|
| v1 compiler as `.dag` | `src/v1/*.dag` (49 files) | 92,548 lines | the authority for the stages |
| seed mirrors (generated) | `src/v1/stage0/src/*.rs` marked `merge=generated-artifact` | 181 files, ~237k lines | emitted from `.dag`, so delete at the `.dag`, never the mirror |
| seed host Rust (hand-authored) | `src/v1/stage0/src/*.rs`, the rest, plus `cli_run/`, `bin/` | ~85 top-level files, ~106k lines (top-level only) | the interpreter, CLI and host effects |

Every generated mirror's `// Source module:` resolves to a `module` declared somewhere in the
corpus. No mirror is orphaned.

### 1.2 Stages (`src/v1`, the largest first)

| file | lines | role |
|---|---|---|
| `05_emit_rust.dag` | 22,478 | Rust target emission |
| `04_infer.dag` | 18,457 | type inference / check driver |
| `02_parse.dag` | 7,110 | parser |
| `compiler_tests_rust.dag` | 5,960 | compiler tests |
| `complexity.dag` | 4,876 | complexity analysis |
| `05_emit.dag` | 4,251 | shared emit core |
| `00_core.dag` | 2,975 | core node model |
| `trait_derive_emit.dag` | 2,834 | trait derive emission |
| `04_env.dag` | 2,403 | scopes / environment |
| `runtime_rust.dag` | 1,948 | Rust runtime support |
| `compile.dag` | 1,747 | pipeline orchestration |
| `04_types`, `04_resolve`, `04_emit_info`, `04_lookup` | 1.1k–1.7k each | |

### 1.3 Hand-authored host Rust (the largest first)

| module | lines | files referencing it |
|---|---|---|
| `cli_run` (+ `cli_run/`) | 35,222 + subdir | 49 |
| `v1_interpreter` | 30,566 | 49 |
| `required_regen_host` | 7,923 | 1 |
| `target_invocation_host` | 5,617 | 2 |
| `emitted_closure_compile_host` | 3,876 | 6 |
| `declaration_index` | 3,717 | 8 |
| `behavioral_receipt_host` | 3,205 | 1 |
| `memory_governor` | 3,192 | 12 |
| `coproduct_reflection` | 2,553 | 25 |
| `namespace_baseline` | 2,350 | 5 |
| `evaluation_budget_consequence_falsifier_host` | 2,208 | 1 |

The "files referencing it" column counts files with a `module::` path or a `use` naming the module.
A `pub mod` declaration alone is not counted. Single-referrer modules are where the deep dives start.

## 2. Performance (where the time goes)

Taken from the required job `v2-bootstrap` on main, run 39287508336 (three consecutive green main
runs agree within ~15%). The instrument is the job log's `compile.*` phase lines, so re-read it
there rather than trusting these figures.

| step | wall |
|---|---|
| build the seed (`cargo build --release -p v1-compiler --bin gunbc`) | 242–282 s |
| `//gunbc/instruments:self-host` | 784–914 s |
| `//gunbc/instruments:v2-native-cli` | 799–835 s |

The phases inside `self-host` (run 39287508336):

| phase | wall |
|---|---|
| before `compile.frontend` (load / dispatch) | ~107 s |
| `compile.frontend` | 56 s |
| `compile.normalize` | 0.8 s |
| `compile.reconcile` | ~90 s |
| `compile.analyses` | 0.9 s |
| `compile.emit` | ~303 s |
| cargo build of the 247 emitted files | ~240 s |
| discriminating red + driver controls | ~112 s |

Peak memory across the step: 13.2 GB.

The seed's own pipeline (frontend + reconcile + emit) is roughly 7.5 of the 15 minutes, and **emit
alone is ~5 minutes**. Emit and reconcile are the first problem sections. The ~107 s before
frontend is unexplained and is worth a look. Per §6b these numbers locate the slice. They do not
choose the repair.

## 3. Cull candidates, pass 1: v1 `.dag` functions named nowhere else

Method: take every `fn` declared in `src/v1/*.dag`, then count that name as a token across every
`.dag` file and every hand-authored `.rs` file (generated mirrors are excluded because they are
downstream). **116 functions** occur exactly once, which is their own declaration.

Known blind spots before anything is deleted:
- the name may be reached by string: `--function`, roster `.txt`/`.tsv` files, claim entries;
- the name may be a test or witness entry discovered by a harness;
- token counting cannot see that a same-named function elsewhere shadows it.

| file | count | names |
|---|---|---|
| `02_parse.dag` | 32 | `first_child_or_self`, a family of `is_*_shape` predicates (`is_and_shape`, `is_plus_shape`, …) |
| `05_emit_rust.dag` | 18 | `apply_clone`, `apply_deref_clone`, `apply_field_clone`, `emit_mock_prop_setup`, `emit_unrealizable_shell_channel`, `has_complex_variants`, `is_simple_disj`, … |
| `expected_red_roster_join.dag` | 10 | the whole `expected_red_roster_join_*` report surface; the module looks dead |
| `trace.dag` | 8 | `capture_repro`, `replay_trace`, `format_trace`, `trace_push_*`, …; the module looks dead |
| `05_emit.dag` | 7 | `classify_tco_expr`, `derive_module_imports`, `unwrap_single_field_product`, … |
| `04_env.dag` | 6 | `scope_push`, `scope_lookup`, `empty_scope`, … |
| `00_core.dag` | 6 | `has_service_config`, `service_config_rate_limit`, `transport_has_argv`, … |
| `04_infer.dag` | 5 | `typecheck_module_isolated`, `build_global_bare_census`, `corpus_*_counts`, … |
| `effect_derivation.dag` | 4 | `re_export_*` |
| `compile.dag` | 4 | `serialize_field`, `serialize_match_arm`, `serialize_named_arg`, `serialize_string_part` |
| `probe_emit_interp.dag` | 3 | the probes themselves (check whether they are enrolled witnesses) |
| `04_types.dag`, `04_resolve.dag` | 3 + 3 | |
| others | 1–2 each | `workspace_members`, `trait_derive_emit`, `stage0_crates`, `languages`, `dag_collect_support`, `04_sigs` |

## 3a. Problem sections, measured (perf, srv1)

`perf record -F 299 --call-graph fp` attached to one `gunbc compile --entry
src/v2/compiler/source_root_eval.dag --source-root dag --source-root src/v2
--dependency-pool-index primary-precedence --target rust` at `891198514b8`. The seed was built with
`-C force-frame-pointers=yes` on srv1 (aarch64, a shared host under load). Samples are split by
phase using the run's own `compile.*` lines. Phase walls: pre 101 s, frontend 55 s, reconcile 87 s,
emit 225 s, post-emit ~60 s. These agree with CI. The percentages are **inclusive within the phase**.
The seconds are those shares applied to the phase wall, so they are approximate.

This **supersedes** §3b where the two disagree. In particular, §3b's reconcile suspects were measured
and are small.

### Pre-frontend (101 s)
| inclusive | ~s | producer |
|---|---|---|
| 74% | 75 | `entry_resolve` `build_module_path_index_uncached` → `pool_acquire::heads_reading_for` → `module_path_index::index::parse_module_binding` → **`v1_compiler_tokenize::tokenize_artifact` (54%)** |
| 25% | 25 | `extend_sources_to_both_closure_fixpoint` → `build_both_closure_edge_index` → `closure_name_census` → `build_symbol_index_census_raw_nodes` |

To learn each file's `module` binding, the index **fully tokenizes every `.dag` in the pool**
(8,847 files). The binding is on line 1, and DESIGN §3 already says paths follow names. So the name
is derivable from the path, and the walk's demand is the defect (§2), not the tokenizer's speed.

### Frontend (55 s)
| inclusive | ~s | producer |
|---|---|---|
| 63% | 35 | `v1.compiler.annotation_bind` `bind_annotations` → `swept_region_order` → `swept_region_key` → `zero_padded` / `zero_padded_with` |
| 16% | 9 | tokenize |
| 8% | 5 | parse |
| 4% | 2 | resolve |

`swept_region_order` is `sort_by(r => swept_region_key(r))`. The key is a 36-char string built by
prepending `"0"` one character at a time, with `string_length` re-measured each step. The emitter
lowers `sort_by` to `slice::sort_by` with **the key recomputed for both operands on every
comparison** (`v1_compiler_annotation_bind.rs` `swept_region_order`, the `__ka`/`__kb` lines). The
emitter repeats this lowering in **22 sites across 12 generated files**, so per §6b the root is the
emitter's `sort_by` lowering: compute each key once, then sort. A secondary issue: the key should be
a structural `(Int, Int, Int)`, not a zero-padded string encoding of one.

### Reconcile (87 s)
| inclusive | ~s | producer |
|---|---|---|
| 58% | 50 | `v1.compiler.infer` `infer_call_arguments_generic_pass` |
| 24% | 21 | `v1.compiler.infer_resolve` `expand_nominal_alias_rhs(_by_name)` → `resolve_node_bounded` |
| 22% | 19 | `v1.std.core` `authored_name_at` (see the cross-phase entry below) |
| 15% | 13 | `direct_call_arg_mismatch_diags` |
| 14% | 12 | `v1.compiler.infer_env` `text_representation_by_identity_bounded` |
| 12% | 11 | `declared_type_obligation_diags` / `declared_type_inhabitance` |
| 7% | 6 | `build_type_env` |
| 3.7% | 3 | `build_module_context` (§3b's top suspect) |
| ~3% | 3 | all `rc_map_insert`/`rc_map_merge` (§3b's emitter clone suspect) |

These rows overlap, because a frame is inclusive of its callees. The next dive is inside
`infer_call_arguments_generic_pass`.

### Emit (225 s)
| inclusive | ~s | producer |
|---|---|---|
| 58% | 131 | `v1.compiler.emit_rust` `reference_derived_use_line_plan` |
| 30% | 67 | ↳ `record_lit_ref_names` → `record_lit_resolved_ctor_import_names` → **`struct_candidates_by_field_names` (12% self)** / `find_struct_name_by_fields` / `anonymous_record_struct_candidates` |
| 14.5% | 33 | ↳ `v1.compiler.infer_emit_info` `collect_type_node_import_surface_names` (the prior art's walk; it is no longer the top term) |
| 21% | 47 | `emit_imports` / `emit_specific_import_use_lines` |
| 19% | 43 | `authored_name_at` |
| 18% | 40 | `emit_typed_item` (rendering) |

The new top term is a structural-record lookup: for each record literal, find the struct whose
field names match. With 12% self time it looks like a scan of every struct per literal. Read it next.

### Post-emit (~60 s, not in any CI phase line)
| inclusive | ~s | producer |
|---|---|---|
| ~46% | ~27 | freeing the compile's data structures at exit (`cfree`, `Rc::drop_slow`) |
| 32% | 19 | `gunbc_crate_partition` `emitted_target_reach` → `reached_set` → `std.graph` `graph_reverse_reachable_members` → **`reverse_adjacency`**, rebuilt per reach query |
| — | — | printing 4,485 advisory lines (~10 ms apart in the log) |

### Cross-phase: `authored_name_at`
`v1.std.core` `authored_name_at` re-materializes a name from the file's char codes on every call
(`map_get` by file path, then `chars_to_string`). It is ~19 s in reconcile and ~43 s in emit. The
fact is computed once per occurrence and re-demanded many times. Before anyone reaches for a cache,
ask whether the name should be carried on the node (the occurrence-identity domain).

## 3b. Problem sections, dive 1 (code reading, not profiles)

Four read-only investigations, one per phase. **None of them got a profile.** The remote image has no
`perf`, and two of the scripts had bugs. So every attribution below comes from reading the code and
is not measured. One remote timing of the same transaction on a 32-core runner: pre-frontend 67 s,
frontend 36 s, peak 8.4 GB. That is about 1.5x faster than CI in both phases, so the difference is
the host, not a different code path.

### Pre-frontend (~107 s)
- All of it falls after `emitting … (seed, in-process)`. The freshness and dep-info checks take ~30 ms.
- The path is `cli_run.rs` `compile_emission_over` → `cli_run/entry_resolve.rs` `try_build_module_index_primary_precedence` (walks and reads **every** `.dag` under `dag/` and `src/v2/`: 8,847 files, ~115 MB) → `load_sources_for_entry_with_pool` → `extend_sources_to_both_closure_fixpoint`. That builds pool-wide name tables through `pool_parse` / `reference_pool_names`, which lex and heads-parse the whole pool to find one entry's closure.
- Its cost scales with the tree, not the closure: `gunbc.recurring_failure_mode` `gunbc_run_pays_a_large_fixed_pre_entry_cost_per_invocation`. `gunbc run` already avoids it by acquiring only what the entry demands. Each required step pays it again in a separate process.
- **An instrument already exists and is never printed on this path:** `cli_run/pre_entry_phase.rs` records `[pre-entry] phase=… ms=… scale=tree|closure`, but `compile_emission_over` never calls `take_lines()`. Printing it gives the split. It is the cheapest next measurement.

### Frontend (56 s)
- `v1.compiler.compile` `front_end_sources` covers tokenize and newline index per module, `pre_intern_tokens`, a parse fold of `parse_with_table_in_occurrence_scope` and `bind_annotations`, `merge_occurrence_transports`, then **resolve** (`resolve_modules_with_occurrence_transport`). So frontend is not only parsing.
- Likely redundant demand: the pre-frontend index already parsed every file, and frontend re-tokenizes and re-parses the 247-module closure from text. That is one parse fact produced twice (§2). Medium confidence.
- A proven cost-shape defect: the parse fold accumulates with `std.source_annotation` `source_annotation_graph_concat(acc.annotations, bound)`, which copies the accumulated list every step, O(modules × annotations). Its size is unknown.

### Reconcile (~90 s)
- `v1.compiler.infer` `reconcile` → `typecheck`: build the census once, then fold `realize_module` in dependency order. This CI path runs the pure generated `reconcile`, not the host's cached `reconcile_with_typed_cache`.
- Cost-shape defects, each per-module work over corpus-wide data:
  1. `build_module_context` `census_scope` rebuilds an `OpEntry` (`qualify_borrowed_inferred`, `inferred_to_outputs`) for **every service in the corpus, per module**. None of it depends on the module, so it should be built once beside `symbol_index`.
  2. `merge_global_bare_variant_locals` calls `map_merge(global_variant_base, locals)`. `v1_rt::map_merge` takes owned maps, so the whole corpus variant base is cloned per module.
  3. `realize_module` passes `dep_state.module_index.clone()` (and `variant_surfaces`, `item_registry`, `diags_by_name`) to `rc_map_insert`/`rc_map_merge` while `dep_state` still holds a reference, so `Rc::make_mut` copies four whole maps per module. **This is in the emitter's ownership lowering**, so the same pattern probably occurs across the generated compiler. Repair it at the emitter (§6b root generalization).

### Emit (~303 s)
- Prior art: `docs/plans/emit-type-surface-traversal-cost.md` (2026-08-29, 108 files). There, `v1.compiler.emit_rust` `reference_derived_use_line_plan` was 88% of emit, almost all of it `type_surface_names`.
- The cause is still in place: `v1.compiler.infer_emit_info` `collect_type_node_import_surface_occurrences` walks a shared type graph as a tree (into `children` and into `inferred`) with no visited set. The specimen counted 7.46M visits over 23,647 distinct nodes, a 315x amplification, half of it on `OccurrenceSynthetic` nodes.
- **A memo here is refused:** `gunbc.recurring_failure_mode` `identity_absent_graph_traversal`. Synthetic occurrences have no identity, so a visited-set key could silently skip a distinct node. The repair is upstream: give synthesised occurrences an identity in the occurrence-identity algebra.
- Already ruled out: list-concat string building (215 sites, no change), the scrutinee clone, per-module candidate rescans (#9666).
- Not yet known: whether its share still holds at 247 files.

### The measurement gap shared by all four
No instrument in the tree breaks these phases down, and `perf` is not on the remote image. Before any
repair, choose one of:
- a sub-phase `trace_mark` layer as a `gunbc test` row,
- printing `pre_entry_phase`,
- a `perf`-capable profiling run.

Each repair must also state beforehand which multiplicity collapses (§6b).

## 4. Next passes

- [ ] Rerun the pass 1 census with string-reachable names (rosters, `--function` entries) subtracted, and confirm with a build that deletes the batch.
- [ ] Whole-module candidates: `trace.dag`, `expected_red_roster_join.dag`. Check what imports each, then delete it.
- [ ] Host Rust: single-referrer modules (§1.3). Trace each referrer up to a CLI subcommand or bin and check whether anything still invokes it.
- [ ] `src/bin/*`: which binaries does any workflow, instrument row or script still invoke?
- [ ] Deep dive on emit (`05_emit_rust.dag`, ~5 min): where the time goes inside the phase.
- [ ] Deep dive on reconcile (~90 s) and the ~107 s before frontend.
- [ ] Promote the confirmed census to an instrument row, then delete this file.
