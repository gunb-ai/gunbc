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

## 4. Next passes

- [ ] Rerun the pass 1 census with string-reachable names (rosters, `--function` entries) subtracted, and confirm with a build that deletes the batch.
- [ ] Whole-module candidates: `trace.dag`, `expected_red_roster_join.dag`. Check what imports each, then delete it.
- [ ] Host Rust: single-referrer modules (§1.3). Trace each referrer up to a CLI subcommand or bin and check whether anything still invokes it.
- [ ] `src/bin/*`: which binaries does any workflow, instrument row or script still invoke?
- [ ] Deep dive on emit (`05_emit_rust.dag`, ~5 min): where the time goes inside the phase.
- [ ] Deep dive on reconcile (~90 s) and the ~107 s before frontend.
- [ ] Promote the confirmed census to an instrument row, then delete this file.
