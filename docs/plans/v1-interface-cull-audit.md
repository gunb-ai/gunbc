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
| `expected_red_roster_join.dag` | 10 | the `expected_red_roster_join_*` report functions are individually dead, but **the module is live**: `cli_run.rs` and `required_floor_runner.rs` use its types and `classify_verdict` (lane F) |
| `trace.dag` | 8 | `capture_repro`, `replay_trace`, …; **zero importers**, confirmed by lane F; deleted in F batch 1 |
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

#### P0.7 findings — the hot-frame split (Lane D, diagnosis only)

Method: `perf record -F 299 -g --call-graph fp -p <pid>` on srv1 (aarch64, kernel
`6.8.0-147-generic`) against a release `gunbc` built with
release build from `891198514b8` in a private `CARGO_TARGET_DIR` with
`RUSTFLAGS="-C force-frame-pointers=yes"` (confirmed from the crate fingerprints; frame pointers
are what makes `--call-graph fp` usable — `eh_frame` unwinding was not relied on), compiling the
`graphiti-seed-snapshot.json` seed (230 files emitted, 4,485 advisory diagnostics, 0 blocking
errors; the run's phase lines are frontend 55 s, normalize 807 ms, **reconcile 86 s**, analyses
835 ms, emit 3 min). The `perf record` cmdline is recorded in the capture's header, so this profile
is the same artifact §3a's reconcile table came from and is directly comparable to it.
Frames are aggregated **by symbol** — a symbol's lines are distinct return addresses under
`--call-graph fp`, so counting them apart splits one function across many rows and understates
it. Percentages below are of the chain-attributed reconcile subtree; seconds apply the 86 s
reconcile phase line. Chain attribution reproduces every §3a row above to ~0.1 pp
(58.10 / 24.15 / 22.39 / 15.32 / 13.71 / 12.26 / 7.12 / 3.72 / ~3), which is the check that the
aggregation is right; a naive phase-window split by sample timestamp does not reproduce them and
should not be used.

**Two definitions, used consistently below.** *inclusive* = a sample counts toward a frame if the
frame appears anywhere on its chain (this is how every §3a row above is measured). *exclusive
child* = a sample counts toward exactly one frame: the nearest named frame below a given parent on
the chain, so the numbers under a parent partition that parent's inclusive share.

Inside the pass (58.10% inclusive), its exclusive children:

| of reconcile | ~s | exclusive child of the pass | what it is |
|---|---|---|---|
| 7.32% | 6.3 | `v1.rt::chars_to_string` | char-code → string materialisation (a leaf; its own caller is `source_text_at` 6.34%) |
| 6.74% | 5.8 | `v1.std.core::authored_name_at` | name re-materialisation (15.47% of its 22.39% inclusive sits in-fold) |
| 3.54% | 3.0 | `infer_env::alias_rhs_of_declaration` | alias right-hand side pulled per formal |
| 3.35% | 2.9 | `infer_resolve::resolve_node_bounded` | bounded re-resolution demanded per formal |
| 3.24% | 2.8 | `v1.std.core::module_path_segments` | module path split on the hot path |
| 1.61% | 1.4 | `v1.std.core::kernel_span` | span fetch |
| 1.31% | 1.1 | `infer::infer_expr_body` | nested inference re-entered from the pass |
| 1.28% | 1.1 | `v1.std.core::qualified_last_segment` | last name segment |
| 1.09% | 0.9 | `infer_resolve::expand_nominal_alias_rhs` | alias expansion demanded per formal |
| 1.03% | 0.9 | `infer::substitute_generics_apply` | substitution |

These ten partition 30.5 of the pass's 58.10; the rest is a long tail of the same shape. Two
observations carry the weight. First, **the top four exclusive children are name/span/text
materialisation and alias re-resolution — not generic solving.** Second, the text classifier
`infer_env::text_representation_by_identity` is 13.71% inclusive (12.89% of it in-fold, so it is
already inside the pass's 58%) and is a recursive tree walk whose own self time is 0.66%: the cost
is the walk plus what it drives into `chars_to_string`.

Generic solving proper is **not** what the 58% buys. Inclusive: `unify_generics` 0.41% (0.4 s),
`unify_callable_generics` 0.10%, `split_sig_params` 0.65% (0.52% in-fold),
`param_node_type_expr` 0.07%, `select_formal_for_call_argument` 2.03% (1.74% in-fold),
`substitute_generics` 2.51% (2.29% in-fold). Summed, all of it is **under 6% of reconcile**, and
the pure-unification pair is under 0.6%. P0.7's premise is confirmed by measurement: the pass's
58% is name/text materialisation, alias re-resolution and nested inference.

**Repeated argument-to-parameter matching — answer: yes, structurally, and once measurably
quadratically.** Inside the pass, `select_formal_for_call_argument` is called once per argument
(`12147`) and its result is then destructured **five** separate times (`12154, 12169, 12177, 12186,
12205`, each `(*formal_selection.clone()).clone()`) to pull raw / declared-context / has-formal /
conformance-type / declaration-bound. The same selection is then **recomputed** by the duplicate
check: `direct_call_shape_diags` re-runs it per argument (`9578`) and, for each argument, re-runs
it again over every *earlier* argument to decide `earlier_same_formal` (`9581`) — quadratic in
argument count, with the self-recursive edge visible in the profile. Measured:
`select_formal_for_call_argument` 2.03% inclusive / 0.65% self; `direct_call_shape_diags` 3.04%
inclusive. `formal_code_point_view` is reached from the selection path at 8.20% inclusive
(see D.7). Small in absolute terms — a §2 defect to clean up, not a term to scope as a repair.

**Direct-call diagnostics split by call site.** `direct_call_arg_mismatch_diags` is 15.32%
inclusive, but it is not one producer: **11.19% is nested inside the generic pass and 4.14% (3.6 s)
runs directly from `infer_expr_body`** — a per-call-expression check outside the pass. Its own
exclusive children are the same shape (`resolve_node_bounded` 2.30%, `authored_name_at` 1.85%,
`chars_to_string` 1.77%, `module_path_segments` 1.22%).

**Round 2 is not a uniform tax.** The two calls in `infer_expr_body` (lines 13896 and 13909) are
separable by return address (`+0x12e84` = round 1, `+0x12f74` = round 2; both `bl` targets are the
pass, confirmed by disassembly). Round 1 is 37.67% (32.4 s), round 2 is 20.38% (17.5 s). The
round-2 predicate `call_needs_generic_rebinding_pass` measures 0.07% / 0.1 s, so round 2 is already
conditional and its 17.5 s is work the predicate decided is needed — consistent with §5b.2 and with
item 4's "do not attempt reuse".

**The open lead (§5b.2, ~29 s outside both argument folds) is attributed.** Chain method: inside
either fold 62.23% (53.5 s); **outside both 37.77% = 32.5 s**, so the ~29 s estimate is right in
kind and slightly low in amount. Every one of those samples sits under
`reconcile → compile_to_resolved_with_options → typecheck_module`, and the nearest named frame on
the root side under `typecheck_module` partitions the bucket:

| of reconcile | ~s | subtree owner | |
|---|---|---|---|
| 19.50% | 16.8 | `infer::infer_items` | per-item body inference outside the folds (81.81% inclusive overall; the 62.23% in-fold part is the pass) |
| 7.12% | 6.1 | `infer::build_type_env` | per-module environment build — **0.00% in-fold** |
| 4.61% | 4.0 | no `typecheck_module` frame on the chain | chains truncated at the frame-pointer walk depth (20–25 frames), so the owning child is unrecovered; leaves are `[unknown]` libc 1.55%, `bitmaps::Iter::next` 0.39%, `__aarch64_ldadd8_rel`/`_relax` 0.42% and `malloc` 0.21% — Rc/HAMT atomic and allocator traffic |
| 3.72% | 3.2 | `infer::build_module_context` | per-module context build — **0.00% in-fold**; §3b's top suspect, measured small |
| 1.44% | 1.2 | `infer::analyze_item` | |
| 0.70% | 0.6 | `infer::populate_output_provenance` | |
| 0.24% | 0.2 | `infer::ground_kernel_views` | |
| 0.28% | 0.2 | `build_module_interface` + `refresh_direct_service_name` + tail | |

By attributed **self** time, the same bucket is dominated by the leaf pair and the Rc-map
machinery: `chars_to_string` 3.70% (3.2 s), `authored_name_at` 2.99% (2.6 s),
`rt::rc_map_insert` 2.09% (1.8 s), `resolve_node_bounded` 1.96% (1.7 s), `rt::rc_map_merge` 1.91%
(1.6 s), `infer_env::guarded_union_str_bindings_into_acc` 1.12%, `infer_env::union_deps_map_into_acc`
1.10%, `sigs::topo_resolve_loop` 1.07%, then a tail in the 0.3–0.8% band.

**Answer for the lead, stated plainly:** it is **not** a second argument fold and not one hidden
defect. It splits three ways — (1) body inference reached outside the folds under `infer_items`
(~16.8 s), (2) **per-module setup that no fold explains**: `build_type_env` 6.1 s +
`build_module_context` 3.2 s, both with literally zero overlap with the folds (9.3 s), (3) name and
Rc materialisation at the leaves (`chars_to_string` + `authored_name_at` ≈ 5.8 s out-of-fold, and
the `rc_map_*` / `union_*_into_acc` family ≈ 5.3 s). The only large, cleanly-owned, removable term
in it is (2), and that is exactly D.8.

**Two corrections to §3a rows, both load-bearing for later lanes.**
`build_type_env` (7.12%) and `build_module_context` (3.72%) have **zero** overlap with the argument
folds (0.00% in-fold), so unlike every other row in that table their cost is per-module, not
per-call-site — they are pure setup. And `rc_map_insert` / `rc_map_merge` / `rc_empty_map`
(2.64% / 1.91% / 1.20% inclusive, all self) are mostly out-of-fold (2.09% / 1.91% / 0.74%), which
supports §3b's withdrawal of the emitter-clone reading of them.

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

> **Corrections (source review at `56b18b20cc7` plus the §3a profile).** The generated code and
> `v1_rt` use `im::{HashMap, Vector}`, which are persistent collections. A clone shares structure,
> and append/update copies only the affected paths. So three claims below are **withdrawn**:
> - the annotation concat does not copy the whole accumulated list;
> - `global_variant_base` is not deep-cloned per module;
> - the four `RealizeState` updates are not full map copies.
>
> The profile agrees: all `rc_map_*` together are ~3% of reconcile. The 315x/88% emit figures are
> historical (a different commit and a 108-file workload) and do not price today's phase.


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

## 4. Cleanup plan (P0 → P2)

This combines the §3a profile with the source review at `56b18b20cc7`. The review found the shape
behind the waste: **information produced at one boundary, then discarded or carried incompletely,
then reconstructed downstream.** Order rule: delete demand that has no consumer first, share a value
that is computed twice next, and only then repair contracts. A cache never conceals authored duplication.
A provider is admissible only for unavoidable recurrence, after demand is minimized and identity is
complete (§2, `std.materialization_ladder`).

Each item lands as its own PR. Each one must:
- name the multiplicity it collapses before the change;
- show that semantic outputs and refusals are unchanged on a small discriminating specimen;
- run the required self-host / native routes;
- re-profile, so the price comes from measurement, not from this table.

**~s** is a *baseline enclosing-frame estimate*: an inclusive sample share times the phase wall. It
is a clue, not a measured saving and not a strict upper bound. Items overlap: annotation deletion
competes with the annotation-sort fix, and name materialization overlaps record, alias and import
work. Re-profile targeted before/after when a PR claims a saving, and fully after an interacting
group lands.
**src** means the item comes from the source review and was not seen as a separate frame in the
profile.

### P0: high certainty, no contract change, or the biggest measured term

| # | Item | Phase | ~s | What disappears / what to check |
|---|---|---|---|---|
| P0.1 | **Print the existing `pre_entry_phase` receipt** from `compile_emission_over`, plus a total transaction duration so unattributed time stays visible. | pre | — | Hidden observation. Check whether each row is inclusive or exclusive before summing. |
| P0.2 | **Stop binding annotations on the compile path.** `front_end_sources` always runs `bind_annotations` (region derivation and sort), and `compile_to_resolved_with_options` never reads `frontend.annotations`. Dedicated annotation and round-trip consumers keep their demand and derive it themselves. | frontend | 35 | A production with no consumer on this path. Enumerate every reader of `frontend.annotations` first. |
| P0.3 | **Emit: delete the absorbed candidate predicate.** In `(authored ∨ anonymous_head ∨ emitted_token) ∧ ¬builtin ∧ emitted_token`, the disjunction is redundant. Remove it and its now-unused local producers (authored-source reconstruction, anonymous-record-head collection). | emit | src | Removes the authored-source reconstruction and anonymous-head collection that feed the filter. **The record-field scan survives this deletion**, because it is reached through a live value-reference path (P1.5). Keep candidate order and membership identical. |
| P0.4 | **Emit: compute `applied_type_argument_names` once** in `emit_module_full`, and pass it to both `emit_imports` and `reference_derived_use_line_plan`. | emit | src | One full traversal per module that has imports. |
| P0.5 | **Emit: carry `provider_module` from `CandidateSurvived`** instead of discarding it and looking it up again. Fold the per-provider binding rescan into one grouping pass with canonical order. | emit | src | A decision recomputed after it was already made. The qualified route already carries it. |
| P0.6 | **Loader: reuse `pool_path_lookup(index)`** in `extend_with_bare_reference_closure` instead of rebuilding `path_to_source_lookup(&index.source_files)` on every fixpoint round. | pre | src | A tree-sized lookup rebuilt per round. |
| P0.7 | **Split the reconcile hot frames.** `infer_call_arguments_generic_pass` (58%) runs for ordinary calls and recursively infers their arguments, so its share includes nested inference, not just generic solving. Give alias expansion (`expand_nominal_alias_rhs`, 24%), call checking (`direct_call_arg_mismatch_diags`) and text classification (`text_representation_by_identity`) their own coverage, and look for repeated argument-to-parameter matching. | reconcile | ≤50 | This item only produces a diagnosis. Read exclusive sub-frames before choosing a repair. |

### P1: clear defects with a bounded contract to preserve

| # | Item | Phase | ~s | Notes |
|---|---|---|---|---|
| P1.1 | **Emitter `sort_by` lowering: compute each key once.** The key is currently recomputed for both operands on every comparison (22 sites in 12 generated files). Also make `swept_region_key` a structural `(Int, Int, Int)`, not a zero-padded string, **but the runtime currently refuses tuple sort keys**, so that part needs the runtime's ordering contract first. | all | ≤35 | Root generalization (§6b). Still worth doing after P0.2, because the other 21 sites remain. |
| P1.2 | **Module index without tokenizing every file.** `build_module_path_index_uncached` fully tokenizes all ~8.8k files to read each one's `module` line. The source-layout rule (DESIGN §3) makes the name derivable from the path. | pre | ≤75 | **The index validates more than names** (for example layout mismatch). Each check needs a new owner before tokenization is dropped. Preserve the non-standard fixture roots. Later bare-name queries may still force heads (see P2.1). |
| P1.3 | **Parser allocator postcondition.** Every parse result carries an allocator advanced past every identity it constructed. Then delete the `parse_context_after_node` recovery walks in `parse_expr_loop(_no_brace)` and their sibling helpers. | frontend | ≤5 | Quadratic in left-associated chain length; §6 says a proven cost-shape defect is always fixed. Fix any producer that currently relies on the rescue before removing a scan. Give genuine adoption an explicit boundary. |
| P1.4 | **Closure fixpoint as one worklist** over the union of edges that `build_both_closure_edge_index` already produces. | pre | ≤25 | Deterministic output and the same refusal selection. Reuse the existing worklist machinery. |
| P1.5 | **`struct_candidates_by_field_names`**: look up by field-name set instead of scanning every struct per literal. | emit | ≤67 | **Independent of P0.3.** It survives that deletion, so it can start now. |
| P1.6 | **Post-emit reachability:** `std.graph` `reverse_adjacency` is rebuilt for each `reached_set` query in `gunbc_crate_partition` `emitted_target_reach`. Build it once per graph. | post | 19 | Generalize at `std.graph`, not at the call site. |
| P1.7 | **Exit teardown:** ~27 s of `free`/`Rc::drop_slow` after the work is done, plus ~10 ms per advisory line for 4,485 lines. | post | ~27+ | Decide what owns the process-exit boundary. Advisory output may want a buffered writer. |
| P1.8 | **One census service view** per effective census, built beside `symbol_index` and carried into `build_module_context`, instead of being rebuilt per module. | reconcile | ≤3 | Measured as small. It is still a §2 defect. The view belongs to the producer that owns that census, not to a process-global table. |

### P2: contract work, larger seams, and the cull itself

| # | Item | Notes |
|---|---|---|
| P2.1 | **One acquisition authority**, parameterized by subject and declared name-resolution rules. Entries acquire their justified dependencies, not the tree. | The `run` loader answers a narrower bare-name question. Account for the pool's bare-name guarantee, primary-root precedence, fixture roots, derived rosters and layout mismatches before removing any production. |
| P2.2 | **One complete parsed-source product** consumed by both reference discovery (`parsed_file_references`) and the frontend. | Carry the whole bundle: tree, intern state, occurrence transport, allocator, extents, captures and indices. Otherwise IDs collide. |
| P2.3 | **One owner of interning.** Today `front_end_sources` pre-interns everything and `parse_context_for_tokens` does it again. | Check that the ID assignment order does not change meaning. |
| P2.4 | **Per-target type-declaration-name set** beside the exported-name facts, instead of being derived per import in `03_resolve`. | Collision diagnostics stay per importer. |
| P2.5 | **Typed-module service export view**, consumed by `merge_scope_from_imports` instead of rebuilt per importer. | Preserve the layering census < local < imports, in resolved-import order. |
| P2.6 | **Effect propagation:** reuse `UniqueAccum { seen, result }` in `expand_transitive_services_once` (today membership is a linear `any` scan). Carry the `after` count into the next `before`. | Preserve convergence, exhaustion and `remaining_delta`. |
| P2.7 | **Reconcile assembly:** drop the parent list from `collect_parent_envs` that no caller reads, keeping its errors. Share the module-name index across the two final rewires. | Small. |
| P2.8 | **Typed-graph identity, aware of traversal role:** extend `OccurrenceProjected` to the inferred and rebuilt type nodes that are still `OccurrenceSynthetic`. The key covers immutable node state **and** traversal role. | The prerequisite for deduplicating `collect_type_node_import_surface_occurrences` (33 s). A visited set keyed on node ID alone would drop classifications. |
| P2.9 | **`authored_name_at`**: carry the name instead of re-materializing it from char codes. | ~19 s in reconcile and ~43 s in emit. The same identity domain as P2.8, so decide the model there. |
| P2.10 | **Cull pass 1:** subtract the names reached by string (rosters, `--function`, claims) from the 116 unreferenced v1 functions (§3), then delete in batches, each checked by a build. | The original goal of this doc. |
| P2.11 | **Dead whole modules:** `trace.dag`, `expected_red_roster_join.dag`. List their importers by name first (DESIGN §3, roots outside the gate). | |
| P2.12 | **Host Rust and `src/bin`:** trace the single-referrer modules (§1.3) and binaries to a live invocation, or delete them. | |
| P2.13 | **Promote** the censuses and the per-phase breakdown to `gunbc test` rows, then delete this file. | DESIGN §6: name the instrument. |

### Also from the second side-chat pass (to be scoped)
- Kernel setup repeated per module in reconcile.
- Provider grouping inside authored imports (emit).
- The CI contract: the two required products have different closures, and both must be emitted and built fresh. Sharing or skipping a final product changes what the checks establish, so it is out of scope for this plan.
- Dependencies, not priority labels, gate the order. P1.5 does not wait for P0.3, and P2.9 (carrying authored names) does not wait for P2.8.

### Specimens to build alongside
- A long left-associated expression chain (P1.3).
- One shared type reached in both roles (P2.8).
- Service propagation through a chain and a diamond (P2.6).
- Closure cases with bare names and primary-root precedence (P1.4, P2.1).

## 5. Dispatch briefs, wave 1

A worker **pins** its brief by citing this file at a fixed commit sha (`git show <sha>:docs/plans/v1-interface-cull-audit.md`) in its first message and its PR body. It works from that text, not from a later edit. If a brief changes, the new text gets a new sha and the worker is told which one.

**Common preamble (applies to every lane).** Read DESIGN.md §2, §5 and §6b first, then §3a and §4 above. Rules for every lane:
- Delete demand that has no consumer, or share a value computed twice. Never add a cache.
- Before the change, state which repeated work disappears and what must stay unchanged.
- Show that outputs and refusals are unchanged on a small discriminating test case.
- Run the required routes `//gunbc/instruments:self-host` and `//gunbc/instruments:v2-native-cli` through CI.
- srv1 and srv2 are available for profiling. Use `sudo /usr/lib/linux-tools/6.8.0-147-generic/perf record --call-graph fp -p <pid>` on a seed built with `-C force-frame-pointers=yes`, under a private `CARGO_TARGET_DIR`. Change nothing host-wide.
- One PR per lane. Edit only the files your lane names. Ask the operator to land it.

### A: Loader timing receipt and shared lookup (P0.1 + P0.6)
In `src/v1/stage0/src/cli_run.rs`:
1. Make `compile_emission_over` print the existing `cli_run/pre_entry_phase.rs` receipt (`take_lines()`), plus a total transaction duration. Say whether each row is inclusive or exclusive.
2. In `extend_with_bare_reference_closure`, reuse `pool_path_lookup(index)` instead of rebuilding `path_to_source_lookup(&index.source_files)` on every fixpoint round. Show the closure and its order are unchanged on the self-host entry and on a bare-name test case.

Files: `cli_run.rs` and `cli_run/pre_entry_phase.rs` only.

### B: Stop binding annotations on the compile path (P0.2)
`v1.compiler.compile` `front_end_sources` always runs `bind_annotations`, which derives and sorts regions. It costs ~35 s of frontend's 55 s, and `compile_to_resolved_with_options` never reads `frontend.annotations`.
1. Enumerate every reader of annotation binding (round-trip and annotation-gap claims).
2. Move the demand to those readers. Verify that binding is total and has no refusal to keep.
3. Keep those claims green.

Files: `src/v1/compile.dag`, `src/v1/annotation_bind.dag`, and their consumers. Leave the emitter's `sort_by` lowering alone.

### C: Emit import planning (P0.3 + P0.4 + P0.5)
In `src/v1/05_emit_rust.dag`:
1. Delete the absorbed disjunction in `(authored ∨ anonymous_head ∨ emitted_token) ∧ ¬builtin ∧ emitted_token`, plus any local producers that become unused. Check each helper's other consumers first.
2. Compute `applied_type_argument_names` once in `emit_module_full`, and pass it to both `emit_imports` and `reference_derived_use_line_plan`.
3. Carry `provider_module` from `CandidateSurvived` instead of looking it up again. Turn the per-provider binding rescan into one grouping pass in canonical order.
4. Emitted bytes must be identical for both required closures; diff the emitted trees.

`struct_candidates_by_field_names` is out of scope (wave 2). Files: `05_emit_rust.dag` and its regenerated mirrors only.

### D: Reconcile hot-frame diagnosis (P0.7), diagnosis only
Profile on srv1 and take the exclusive (self) time of each sub-frame under `v1.compiler.infer`. Cover:
- `infer_call_arguments_generic_pass` (58% inclusive, which includes recursive argument inference);
- `expand_nominal_alias_rhs` (24%);
- `direct_call_arg_mismatch_diags`;
- `text_representation_by_identity`;
- `authored_name_at` (22%);
- repeated per-module kernel setup;
- repeated argument-to-parameter matching.

For each real defect, name the earliest unjustified boundary (§6b) and propose it as a P-item. Commit findings to this file, §3a and §4 only, on this PR's branch.

### E: Post-emit reverse index (P1.6)
`gunbc_crate_partition` `emitted_target_reach` → `reached_set` → `std.graph` `graph_reverse_reachable_members` → `reverse_adjacency` rebuilds the reverse index for every query, ~19 s after emit.
1. Build the index once per graph, at `std.graph`.
2. Check the other callers (§6b breadth).
3. The emitted workspace must be identical.

Files: `dag/std/graph.dag` (and its mirror) and the partition call sites.

### F: Cull discovery and deletion (P2.10 + P2.11 + P2.12)
1. From the 116 single-occurrence v1 functions (§3), subtract the names that are string-reachable: rosters, `--function` entries, claim and witness discovery, instrument rows.
2. List the importers of `trace.dag` and `expected_red_roster_join.dag` by name. They are outside the required gate.
3. Trace each single-referrer host module (§1.3) and each `src/bin/*` to a live invocation.
4. Delete in batches, one PR each, each proved by a build. Avoid files owned by lanes B, C and E until they land.
5. Turn the census into a `gunbc test` row (P2.13).

### Wave 2 (after C lands)
- P1.5: record-field lookup.
- P1.1: `sort_by` lowering.
- Whatever D proposes.

### Wave 3
- P1.2: module index from paths.
- P1.3: the parser allocator postcondition.
- P1.4: one closure worklist.
- P1.8: the census service view.
- P1.7: exit teardown.

## 5a. Amendments to wave 1 (side-chat review of §4 at `35ca4ead3c9`)

A lane's pin is its §5 brief **plus** its amendment below, both read at this commit's sha.

- **A.** P0.1 needs an **outer** timing boundary. `CompileRun.wall_ms` is sampled before the locals finish dropping, and output materialization and diagnostic rendering happen in the caller. Time the whole CLI invocation as well, and keep unattributed time visible.
- **B.** Remove the demand from the compile route, but keep every annotation-producing interface truthful. Returning an empty graph from an interface that promises complete annotations is a contract change, not a deletion.
- **C.** P0.3's saving is **unmeasured**: the record-field scan survives through `collect_value_ref_names` → `record_lit_ref_names` and the body type-surface route. For P0.4, the overlapping signature-name and argument-classification walks have different child-slot coverage, so any later sharing needs explicit projections. Do not substitute one collector for another.
- **D.** Report aggregate counts, not per-argument logs, and treat these as bounded subitems:
  1. `unify_generics` with an **empty generic roster** still walks type trees. That is a proven no-op; return the unchanged substitution at that boundary.
  2. Argument/formal correspondence (labels, positional ranks, reserved formals, inverse searches) is rebuilt across rounds, application planning and duplicate checks. Keep full-signature and value-slot identities distinct, and keep the two duplicate checks distinct.
  3. `split_sig_params` classifies twice.
  4. Second-round inference: which demands actually changed. Diagnose this one only; reusing results needs a proof.
  5. `expand_nominal_alias_rhs_by_name` builds `structural` and then returns `resolved` on its ordinary arm.
  6. `resolve_nominal_alias_rhs` builds a second transformed value only to read its diagnostics.
  7. `formal_code_point_view` classifies text twice per call.
  8. `build_type_env` rebuilds kernel indices, interning, Optional/Bool and the variant index per module. Find one prepared kernel context per equivalent view. The ordered cycle-key list is also computed three times.

#### Lane D's findings on the §5a-D subitems (P0.7 diagnosis; no code changed)

All eight were read in source at `891198514b8` (both files byte-identical to this branch's
`src/v1/stage0/src/`) and measured on the profile described in §3a's P0.7 findings. Two of the
eight as written are **not** what the code does, and three that are real have measured cost below
one second, which changes what they are worth scoping as. None of these numbers justify a repair
on their own except D.5 and D.8. Aggregate counts only, per the brief.

| item | claim | verdict | measured (incl, of reconcile) |
|---|---|---|---|
| D.1 | empty roster still walks type trees | **real, but the framing is wrong** | 0.41% / 0.4 s |
| D.2 | correspondence rebuilt across rounds and planning | **real** | 2.03% / 1.7 s (1.74% in-fold) |
| D.3 | `split_sig_params` classifies twice | **real** | 0.65% / 0.6 s (0.52% in-fold) |
| D.4 | second-round demands | **diagnosed; no reuse** | round 1 37.67% (32.4 s) vs round 2 20.38% (17.5 s) |
| D.5 | `structural` built then discarded | **real, and large** | 24.07% / 20.7 s frame, 8.86% (7.6 s) reached out-of-fold |
| D.6 | second transformed value read only for diagnostics | **real in source, no measured cost** | 0.00% — inlined, cold path |
| D.7 | text classified twice per call | **real** | 8.20% / 7.1 s, all in-fold |
| D.8 | per-module kernel/variant rebuild; cycle keys thrice | **real** | `build_type_env` 7.12% / 6.1 s, 0% in-fold |

- **D.1 — confirmed real, and worth less than it looks.** `unify_generics`
  (`v1_compiler_infer.rs:24997`) has **no** guard on `generic_names` anywhere in its body: the
  only length tests are on `formal.children`/`actual.children` (25016, 25021, 25054). With an
  empty roster the loop at 25029 iterates nothing, so `__found` stays `false` and the function
  still descends — the Arrow arm recurses through `unify_callable_generics` (25046) and the
  single-child arm requeues both children (25060), returning `acc` unmodified on every exit path.
  So the walk is a genuine no-op and an early return at that boundary is correct. **But** its total
  measured cost is 0.41% of reconcile (0.4 s), all of it in-fold, from 105 samples. Do not scope
  it as a performance item; fold it into whatever already touches this function.
- **D.2 — confirmed, and the count is concrete.** Inside the pass, `select_formal_for_call_argument`
  is called once per argument (`12147`) and the result is then destructured **five** separate times
  (`12154, 12169, 12177, 12186, 12205`, each `(*formal_selection.clone()).clone()`) to pull out
  raw / declared-context / has-formal / conformance-type / declaration-bound. The same selection is
  **recomputed** by the duplicate check: `direct_call_shape_diags` re-runs it per argument
  (`9578`) and then, for each argument, re-runs it again over every *earlier* argument to decide
  `earlier_same_formal` (`9581`) — quadratic in argument count, with a self-recursive edge visible
  in the profile. Frame inclusive 2.03% / 1.7 s (1.74% in-fold); its measured immediate callers are
  `call_argument_selects_formal_index` 0.73%, `direct_call_shape_diags` 0.64%, the cloned-iterator
  fold of that quadratic loop 0.45%, and itself 0.21%. `formal_code_point_view` is reached from the
  selection path at 8.20% inclusive. Keep the two identity kinds and the two duplicate checks
  distinct as the amendment says — the measured 2.03% does not justify merging them.
- **D.3 — confirmed, as written (twice).** `split_sig_params` (`24921`) runs `param_is_generic_decl`
  over the same `params` list in two separate full loops — once negated to keep non-generic formals
  (24929) and once positively to build `generic_names` (24940) — so every parameter is classified
  twice, each time re-entering `param_node_type_expr` / `param_node_name_at` / `type_node_label` /
  `is_type_variable` (24977–24986). Cost is 0.65% / 0.6 s.
- **D.4 — diagnosed, no reuse attempted** (per §5b.2 and quiet-hawk-370's four negatives). The
  two rounds are separable by return address in `infer_expr_body`: `+0x12e84` (line 13896) = round
  1 = **37.67% / 32.4 s**, `+0x12f74` (line 13909) = round 2 = **20.38% / 17.5 s**, verified by
  disassembly (both `bl` targets are `infer_call_arguments_generic_pass`). Round 2 is gated by
  `call_needs_generic_rebinding_pass` (0.07% / 0.1 s, 18 samples), so it is **already conditional**
  and the 17.5 s is work the predicate decided is needed — the demand that changes between rounds
  is the substitution map (`st.subst`, threaded at 12286–12293). Nothing here supports cutting the
  round; **do not attempt reuse**, as narrowed.
- **D.5 — real, and the only one of the eight with a large measured frame.**
  `expand_nominal_alias_rhs_by_name` (`v1_compiler_infer_resolve.rs:480`) computes `structural`
  eagerly through up to two `resolve_node_bounded` walks (490–524) and then discards it on the
  ordinary arms, which return only `resolved` (551, 554) — the `structural` binding is read on no
  path there. Frame inclusive 24.07% / 20.7 s, of which **8.86% / 7.6 s is reached outside the
  argument folds** (recursive self-calls), so this is not folded into the pass's 58%. Highest
  value-to-risk ratio of the eight: the demand is provably unobservable on those arms.
- **D.6 — real in source, no measured cost.** `resolve_nominal_alias_rhs` (`1062`) builds a second
  transformed value and then reads **only** `validation.diagnostics` from it (1116–1123);
  `validation.resolved` is discarded at every use, so the value is thrown away. But the function has
  **zero** samples under its own name: it is inlined at both call sites and reached from
  `resolve_item_types` (`4446`) once per *parametric type-alias declaration*, not per argument or
  per formal. It is a clean §2 defect to remove, and must **not** be scoped as a performance item —
  there is nothing measurable to win.
- **D.7 — real, exactly as described.** `formal_code_point_view` (`12086`) classifies the same
  `declared_type` twice per call: once at 12087 (`declaration_reading_of_type_node`) and again at
  12110 inside the `.str` arm. 8.20% / 7.1 s inclusive, **all of it inside the pass** — so it is
  already counted in the 58%, and any pass-level repair subsumes it.
- **D.8 — real, and the answer to the open lead.** `build_type_env` (`28278`) is called once per
  module from `typecheck_module` (`32169`; `build_module_context` at `32238`), so everything it
  rebuilds is rebuilt 230 times, and it rebuilds structure that does not depend on the module:
  `seed_kernel_intern_table` (`28338`), the kernel Optional and Bool nodes
  (`kernel_optional_type_node` `28420`, `kernel_bool_type_node` `28440`), and
  `compiler_recursive_types()` twice (`28449`, `28461`) plus `compiler_recursive_name_set()` twice
  (`28691`, `28742`). The ordered cycle-key list is computed **three times** as written:
  `v1_rt::sorted_map_keys(&cycle_set_str)` at `28746`, `28758` and `28773`. The two
  `build_unit_variant_index` calls are **not** a duplicate as this item implied — `28492` builds
  over the kernel string bindings with no parents, and `28914` builds over the module's own
  bindings with its parents, which is different work; keep both. Measured: `build_type_env` 7.12% /
  6.1 s and `build_module_context` 3.72% / 3.2 s, both with **0.00% inside the argument folds** —
  they are pure per-module setup, which is why the folds' 62% neither contains nor explains them.
  A single prepared kernel context per equivalent view removes the kernel-independent part of the
  6.1 s, which is the largest cleanly-owned term in the open lead.

**What Lane D recommends wave 1 take from this.** D.5 and D.8 are the two with enough measured
mass to matter (7.6 s out-of-fold and 6.1 s out-of-fold respectively, both with zero overlap with
the argument folds). D.1, D.2, D.3, D.6 and D.7 are genuine §2 defects whose measured cost is
sub-second to ~2 s — carry them as cleanups inside a PR that already touches their file, not as
separate performance items. D.4 is closed: diagnosed, no reuse.
- **E.** Reuse the existing `std.graph` `CallGraphAdjacencyViews`, which builds forward and reverse together, across reach queries. The caller already deduplicates targets. Preserve direction, start-node exclusion, and cycle/refusal behavior.
- **F.** Single-occurrence names are not enough. Compute reachability from the declared roots (CLI, interpreter, test/claim, roster, script, host-dispatch, qualified indirect entries). Find dead **groups** (self- and mutually-recursive), dead types/data/constants, and dead interpreter and host-adapter code. Include the mirrors, exports, manifests, crate membership and registrations each deletion touches. Every candidate ends as *retained for a named consumer*, *deleted*, or *unresolved with a named question*. Don't add `#[test]`s to `module_path_index` expecting them to execute; its retained test module is not enrolled.

## 6. Backlog additions (not yet assigned)

- **P1.5 expanded (record emission):**
  1. Delete the shape-only fallback search that cannot succeed after a failed hint-aware search. Keep the anonymous helper's resolved-node fallback.
  2. Keep the complete zero/one/many selection result instead of re-searching to tell absence from ambiguity.
  3. Index the remaining field-shape queries per type-summary population, returning every match.
  4. Share the constructor decision across admission, import planning and rendering.
  - Steps 1–2 can start now.
- **P1.1 split.**
  - P1.1a: compute each admitted key once, preserving selector, canonical comparison, multiplicity and ties. The census counts lowering variants, not just generated sites.
  - P1.1b: define canonical structural-key ordering (`CanonicalOrdKey` admits only strings, ints and bools today) before the annotation key changes.
- **P1.2 precondition.** Inventory every output and refusal of `module_path_index` (identifier span, malformed heads, unterminated bodies, moduleless input). The header is not guaranteed to be on line 1. Assign each to locate, read-admission or a declared whole-corpus validation. Preserving every whole-pool validation is incompatible with eliminating every whole-pool read, so choose.
- **Authored-import planning.** Variant-parent decisions, a name's defining module, and a remote parent's representative are each recomputed. Carry the binding decisions and group once. Preserve scope, re-exports, marker/variant classification and ambiguity.
- **Rendering.**
  - `collect_type_node_realized_surface_names` renders a type to Rust and then re-tokenizes the string. A larger contract would have rendering carry its references; coordinate it with the constructor decision.
  - `dying_literal_plan` renders a field before choosing the branch that ignores it. That is a local deletion.
- **P2.9 does not wait for P2.8.** `authored_name_at` reads `ident_span` and the source map, not occurrence identity. The design question is authored versus structural name (`with_authored_identity`). Replacing it with `node.name` changes nominal-alias behavior. Start that contract brief now.
- **P2.8 refined.** Keep three things separate: immutable node identity, source provenance, and query context. The role belongs to the query's identity, not the node's. `OccurrenceProjected` requires a cause, so it cannot represent every synthetic node.
- **P2.2/P2.3 additions.**
  - Don't keep every module's prepared tokens resident at once; fuse or release each one when its consumer finishes.
  - Carry the source-index map forward, since `compile_to_resolved_with_options` rebuilds it.
  - Define the parsed product's identity and lifetime.
- **P1.7 split.**
  - Lifetime and terminal cleanup: `main.rs` `Verdict::apply` already flushes and calls `_exit` on another route. In-process callers still need normal release.
  - Diagnostic output: measure formatting, writes and blocking separately; 10 ms log gaps are not 10 ms of printing. `render_one_diagnostic` recomputes the location.
- **CI envelope.** The seed build, emitted cargo builds and controls each need a disposition. Both closures must stay fresh, so ordinary cargo reuse is fine but sharing a product is not.
- **P2.13.** Add one supported profiling recipe. When this doc is deleted, move its unfinished items to the maintained backlog.
- **Specimen families**, enrolled through a route that actually executes:
  - annotations and ordering;
  - loading and parsing;
  - records and imports;
  - calls and aliases;
  - names and graph identity;
  - effects, graph and finalization.
- **Generated output.** One integration owner regenerates mirrors from the merged authorities. Never hand-merge generated Rust.

## 7. Later waves: briefs and start triggers

Every lane below takes the §5 common preamble plus its own brief here, pinned at the sha this
section was committed in. **Start** says when a worker may begin. **Land** says what must merge
first. A lane may begin before the lanes that block its landing; it then merges main and regenerates
before landing. Generated mirrors are regenerated by lane Z, never hand-merged.

### Wave 2: start now

**Z. Integration owner** (standing, staff now)
- Owns regenerating the stage0 mirrors from merged `.dag` authorities for every lane touching `src/v1/*.dag` or `dag/std/*.dag`.
- Sequences landings on `05_emit_rust.dag` (lanes C, G, I, L, T) and `04_infer.dag` (lanes J, M1, R).
- Runs the full srv1 re-profile after each interacting group lands, and updates §3a.
- Owns no semantic change.

**G. Record emission, part 1** (§6 P1.5 steps 1–2). Start now. Land after C.
1. Delete the shape-only fallback search that cannot succeed after a failed hint-aware search. Keep the anonymous helper's resolved-node fallback.
2. Keep the complete zero/one/many selection result instead of re-searching to tell absence from ambiguity.

Specimens: zero, one and many matching structs; hint refinement; single- versus multi-field literals. Files: the record-selection and record-rendering functions in `05_emit_rust.dag`.

**H. Authored-name contract** (P2.9; design only). Start now.

Write the contract for authored versus structural names:
- source snapshot, name role, and propagation through constructors and rebuilds (`with_authored_identity`);
- kernel and missing-source behavior;
- non-ASCII offsets.

Decide whether `authored_name_at` becomes a carried value, and where. Output is a design section in this doc (or a model PR in `std`), not an implementation.

**I. Ordering, part 1** (§6 P1.1a). Start now. Land after C.

In the emitter's `sort_by` lowering, compute each admitted key once (decorate, sort, undecorate). Preserve:
- the supplied selector;
- `canonical_key_cmp`;
- multiplicity;
- stable ties.

Census the lowering *variants*, including non-lambda selectors and unknown element types, not just the 22 generated sites. Emitted output may differ only in sort mechanics; ordering results must be identical.

**J. Effects and assembly** (P2.6 + P2.7). Start now.
- `04_service.dag`:
  - reuse `UniqueAccum { seen, result }` in `expand_transitive_services_once`;
  - carry the `after` count into the next `before`.
  - Preserve convergence, exhaustion and `remaining_delta`.
- `04_infer.dag`:
  - drop the parent list that `collect_parent_envs` produces and no caller reads, keeping its errors;
  - share the module-name index across the two final rewires.

Specimens: chain and diamond service propagation, and exhaustion.

**K. Module-index admission inventory** (§6 P1.2 precondition; inventory only). Start now.

Enumerate every output and refusal of `module_path_index` and its consumers: identifier span, malformed heads, unterminated bodies, moduleless or unclassified input, headers after blanks or comments, fixture roots, precedence, derived sources. Classify each one as:
- locate,
- read-admission, or
- declared whole-corpus validation.

End with one operator decision: which whole-pool validations survive. Output is a section in this doc.

**N. Parser allocator postcondition** (P1.3). Start now.

Establish that every parse result carries an allocator advanced past every identity constructed in it. Fix each producer that relies on the rescue. Then delete the `parse_context_after_node` recovery walks in `parse_expr_loop(_no_brace)` and their siblings (statements, arguments, parameters). Give true adoption of a foreign tree an explicit boundary.

Specimens:
- a long left-associated chain;
- occurrence IDs and transport identical before and after.

Files: `src/v1/02_parse.dag`.

### Wave 3: triggered by a wave-1 landing

**M1. Call inference repairs.** Start when lane D's findings are committed. Land after J.

From D's §5a subitems 1–3, plus whatever D confirms:
- return the unchanged substitution from `unify_generics` when the generic roster is empty;
- prepare the argument/formal correspondence once per call;
- carry the `split_sig_params` result.

Preserve full-signature and value-slot identities and both duplicate checks. Files: `04_infer.dag` (call path).

**M2. Alias and text views.** Start when lane D's findings are committed.
- `expand_nominal_alias_rhs_by_name`: decide the head/brand arm before building `structural` where that is possible.
- `resolve_nominal_alias_rhs`: obtain the diagnostics without building a second transformed value.
- `formal_code_point_view`: classify text once per formal view, scope and substitution stage. Preserve host-text precedence and the difference between not-text and unidentified.

Files: `04_resolve.dag`, `04_env.dag`.

**L. Import planning, part 2, and the local rendering deletion.** Start after C lands.
- Carry authored-import binding decisions (variant parent, defining module, remote representative) and group them once. Preserve scope, re-exports, marker/variant classification and ambiguity.
- Delete the unused field render in `dying_literal_plan`.

Files: `05_emit_rust.dag` import and literal-plan regions.

**O. Closure worklist** (P1.4). Start after A lands.

Run one worklist over the union of the edges `build_both_closure_edge_index` produces. Specimens:
- deterministic closure order;
- identical refusal selection;
- bare and qualified chains;
- precedence.

Files: `cli_run.rs` closure functions.

**Q. Exit and diagnostics** (§6 P1.7 split). Start after A lands, because A's outer timer touches the same boundary.
1. Find what is dropped inside compilation, on return, and at process exit. Evaluate sharing `Verdict::apply`'s flush-and-`_exit` boundary for the CLI route only; in-process callers keep normal release.
2. Measure diagnostic formatting, writes and blocking separately.
3. Compute the location once in `render_one_diagnostic`.

Files: `main.rs` and the compile CLI route.

### Wave 4: gated on a contract or an operator decision

**P. Module index from paths** (P1.2). Start after K's decision is made and A lands.

Implement the chosen admission split. Remove the whole-pool tokenization that only locating needed.

**R. Shared module context** (P1.8 + P2.4 + P2.5 + kernel context). Start after M1 lands, since it is the same file.
- One census service view per effective census.
- One prepared kernel context per equivalent source/intern view.
- A typed-module service export view consumed by importers.
- A per-target set of type-declaration names.
- Preserve census < local < imports layering and the module-scoped refusals.

**S. Ordering, part 2** (§6 P1.1b). Start after I lands.

Define canonical structural-key ordering in the runtime's `CanonicalOrdKey` contract, agreeing with every realization. Then replace the annotation string key if a consumer still needs it.

**T. Record emission, part 2** (P1.5 steps 3–4, plus rendering references). Start after G and L land.
- Build a field-shape index per type-summary population.
- Make one constructor decision, shared by admission, import planning and rendering.
- Have rendering carry its references, instead of `collect_type_node_realized_surface_names` re-tokenizing.

**U. Parsed-source product** (P2.2 + P2.3 + the §6 additions). Start after N and B land.
- One owner of interning.
- One complete parsed product shared by reference discovery and frontend.
- Release each prepared token stream once its consumer is done; carry the source-index map forward.
- Define the product's identity and lifetime.

**V. Unified acquisition** (P2.1). Start after O and P land.

One acquisition authority, parameterized by the subject and the name-resolution rules. Account explicitly for the bare-name guarantee the pool currently provides.

**W. Computed-node identity** (P2.8). Start after H.

Keep node identity, provenance and query context separate, with the role in the query key. Then share the role-aware type-walk results (`collect_type_node_import_surface_occurrences`).

**X. CI envelope** (diagnosis only). Start any time; lowest priority.

Give a disposition to the seed build, the emitted cargo builds and the controls, keeping both closures fresh.

**Y. Instruments and retiring this doc** (P2.13). Start last, after wave 3 lands.
- Turn the censuses and the phase breakdown into `gunbc test` rows.
- Write one profiling recipe.
- Move unfinished items to the maintained backlog.
- Delete this file.

## 5b. Constraints from the parent lane (sleek-owl-397, v2 native work). Applies to every lane.

1. **`src/v1/04_infer.dag` and `v1_compiler_infer.rs` are being edited concurrently.** The concurrent edits come from:
   - Rule A (crisp-otter-44, for #14087);
   - the operator's Int carrier (`n3/int-carrier`);
   - #14185.

   Any lane touching `04_infer` or its mirrors (J's `collect_parent_envs`/rewire part, M1, R) must **sequence through tidy-ant-251** before editing, and must not race. Note that the parent tree also has an "M1" lane, unrelated to this doc's M1.
2. **Measured negatives, do not retry.** The second generic-argument round in `infer_call_arguments_generic_pass` cannot be cut under frozen verdicts: quiet-hawk-370's closed v1-reconcile lane recorded 4 negatives. Its open lead is **~29 s of reconcile outside both argument folds, unattributed**. Lane D should attribute that, and read quiet-hawk-370's findings first. §5a D item 4 is narrowed to "do not attempt reuse".
3. **PURPOSE test** (`gunbc.v1.v1_maintenance_standing v1_seed_standing`). v1 is semantics-frozen. Every lane's PR must state how it serves the v2 self-host program. For this plan the statement is: seed self-host time and memory on the required job, plus less seed surface for v2 to carry. A lane that cannot say this does not land.

## 9. Authored-name contract (lane H, P2.9; design only)

Symbols, not positions, are the receipts. Re-check the cited declarations before acting.

### 9.1 What exists
- `v1.std.core` `Node` carries `name` (the STRUCTURAL name, set by the `make_*_node` constructor) and `ident_span: SourceSpan?` (the identifier token span; `default_ident_span` is `none` for an empty name).
- `v1.std.core` `authored_name_at` is a projection of `ident_span`, not a read of `name`. `Absent` gives the empty string. `Present` with the file in `source_indices` slices `source_text_at` (`chars_to_string` over `NewlineIndex.char_codes`). An absent file of the form `<kernel:N>` (`kernel_span`) yields N. Any other absent file (`no_span`, `<synthetic>`) falls back to `node.name`.
- `v1.compiler.resolve` `with_authored_identity` builds a node whose structural fields come from `structural` and whose `ident_span` comes from `identity`. It is the one sanctioned place the two names differ: the node is structurally the alias target and spelled as the alias. `preserve_nominal_brand_on_resolve` and the transparent-alias relation read that difference (`gunbc.recurring_failure_mode` `transparent_alias_relation_read_the_brand_keeping_peel`).
- Cost is §3a "Cross-phase". Re-derive the call-site count with `git grep -c authored_name_at`; no figure is transcribed here.

### 9.2 The contract
- **C1 one fact, two roles.** Authored is the spelling at the identity position; structural is what the node denotes. A carried value replaces `authored_name_at` only, never `.name`.
- **C2 snapshot.** The authored name is the slice of the tokenized source at the identifier span. Spans index Unicode scalars (`chars`), not bytes or UTF-16 units, so non-ASCII is correct provided the carried string is cut from the same `char_codes`.
- **C3 fixed at construction.** Computed where `ident_span` is minted (parser, `kernel_span` and `no_span` constructors), never at use. No consumer needs `source_indices` to ask a node its name.
- **C4 propagation.** Constructors take it from the name they already receive. Rebuilds copy it with `ident_span` as one field group. `with_authored_identity` takes it from `identity` beside `ident_span`. Synthetic nodes make the `name` fallback explicit.
- **C5 missing source and kernel.** A construction with a span but no obtainable text is a typed construction error, not the empty string or a silent fallback. Kernel and synthetic nodes construct their variants explicitly.
- **C6 unchanged outputs.** For every reachable node the carried value equals today's `authored_name_at`, including the empty string for absent `ident_span` and `node.name` for unknown files. Checked by a corpus differential before the projection is deleted.

### 9.3 Decision: carry it
Per DESIGN §2 step 1 the repetition is authored duplication whose least common ancestor is the node's construction, so carry the first value. A cache may discharge unavoidable recurrence; it may never excuse authored duplication. A memo keyed by file and span also cannot express the `with_authored_identity` divergence without keying on the node, which is the carried design with extra steps. `ident_span` stays (diagnostics, `node_name_span`); only its text-projection role moves.

### 9.4 Relation to P2.8
The authored name is immutable node state at the identity position, so it belongs on the state side of P2.8's key, not the traversal-role side. The carried field changes no occurrence identity, so P2.9 does not wait on P2.8.

### 9.5 Open operator decisions
- **Node shape.** A new `Node` field touches every record literal and the stage0 mirror (regenerated by lane Z). Recommended alternative with the same contract: change the `ident_span` payload to a type holding span plus authored text, so the two cannot be updated apart. Either form reshapes `Node`, a load-bearing substrate type, so a model change lands before any pipeline lane migrates.
- The saving is not re-measured here; the contract does not depend on its size.

### 9.6 Verification for the implementer
- Corpus differential, carried equals `authored_name_at`, with RED controls: a rebuild that drops the field, and a `with_authored_identity` that takes it from `structural`.
- Specimens: a non-BMP scalar before the identifier, kernel and synthetic nodes, an alias brand-keeping peel.
- Required routes `//gunbc/instruments:self-host` and `//gunbc/instruments:v2-native-cli` unchanged.

### 9.7 PURPOSE test (§5b.3)
Serves the v2 self-host program by removing the per-call name re-slicing demand on the required job once implemented, and by deleting the `source_indices` threading through every call site, so less seed surface exists for v2 to carry. This section changes no behavior.

### 9.8 Operator decision (2026-10-19), and lane H2

The operator chose **payload**: `ident_span` carries span plus authored text. Lane **H2** implements it in two PRs.

1. **Model PR first.** Reshape the `Node` `ident_span` payload in `v1.std.core` and its constructors (parser, `kernel_span`, `no_span`, synthetic), per §9.2 C2–C5. `authored_name_at` stays as-is in this PR.
2. **Migration PR second.** Callers read the carried text. Rebuilds and `with_authored_identity` propagate it (C4). Delete `authored_name_at` and the `source_indices` threading it alone required.

Obligations:
- Behavior is unchanged, proven by the §9.6 corpus differential with its RED controls, run before the projection is deleted.
- Measure the reconcile and emit saving on srv1 against the baseline 19 s / 43 s estimates, and state what share of the unattributed ~29 s reconcile lead (§5b.2) it accounts for.
- Every edit to `04_infer` or its mirror goes through tidy-ant-251 (Rule A, Rule B and #14490 are in flight).
- Mirrors are regenerated by lane Z.
- Each PR states the PURPOSE test (§5b.3).

## 5c. Prior reconcile work (quiet-hawk-370, closed 2026-10-19). Required reading for M1, M2 and R

The lane's handoff was a dashboard message, not a doc. Its PRs:
- #13932 (B0)
- #13926 and #13972 (census fill deleted; the 337 s parse)
- #13925, with regression fix #14085
- #13971 (-14.6k lines)
- #14060 (pass 3)
- #14250
- #14075 (witness only; its PR comments hold the abk5 re-trigger variant, 1.82x worse)

Instruments and the arm recipe are in silent-lark-766's handoff, dashboard message `msg_4e1e0800`. Its logs are gone.

**Four measured negatives on cutting round 2 of `infer_call_arguments_generic_pass`. Do not retry them.**
1. Deleting round 2 loses O4/L5 and 8 `empty list literal` errors on `compile.dag`.
2. A judge-only phase cannot clear round-1 diagnostics.
3. A generic-free carry-forward reaches only 25 of 30,528 visits.
4. Per-call setup is 2.9 ms, because params are already `Rc`.

Lane D's 9.3 s per-module setup (`build_type_env` + `build_module_context`) is new and not covered by these negatives.

## 5d. Profiling recipe and trap (lane D handoff). Applies to every lane that measures

- **Aggregate by symbol, never by line.** Under `--call-graph fp`, one function's lines are distinct return addresses, so per-line aggregation splits a function and understates it. Split by return address only on purpose. For example, round 1 is `infer_expr_body+0x12e84` (37.67%) and round 2 is `+0x12f74` (20.38%); both are calls to `infer_call_arguments_generic_pass`.
- **Attribute by call chain, not by time window.** The sample clock and `run.log` timestamps drift: the sample span is 532.9 s against 423.8 s of wall time.
- **Validity gate:** a new script must reproduce the §3a reconcile rows (58.10 / 24.15 / 22.39 / 15.32 / 13.71 / 12.26 / 7.12 / 3.72) to about 0.1 pp. If it doesn't, the script is wrong.
- **Reusable on srv1:**
  - `~/samples.pkl` (the pickled samples);
  - `~/one.py`;
  - `~/perf-still-fox/out/VERIFIED/`. Never quote from `out/DO_NOT_TRUST/`.
  - The binary at `891198514b8` in `~/perf-still-fox`; don't rebuild it.
- About 4 s of chains truncate at fp walk depth (20–25 frames). Recovering them needs `--max-stack` or dwarf.
- Reconcile's mass is name/text materialization, alias re-resolution and per-module setup. **Do not scope reconcile work at `unify_*`**, which is under 0.6%.

## 5e. Preamble correction: branch base

"Branch from origin/main" applies to **code** lanes. A lane whose deliverable is a section of this doc commits to `v1-interface-cull-audit`, because a main-based PR would delete the doc. Lanes D, H and K already did this.
