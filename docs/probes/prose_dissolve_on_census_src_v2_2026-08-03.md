# Prose `dissolve-on` census — `src/v2` (Census B)

**Status:** classification complete, measured at `44126ca1de0`, 2026-08-03. No rows
migrated or deleted.

Sibling: Census A covers `dag/` (separate work item). This receipt is the `src/v2`
half of the dissolution-census program priced by
[`disposition_carrier.dag`](../../dag/gunbc/plans/disposition_carrier.dag) and
[`dag-note-prose-census.md`](../plans/dag-note-prose-census.md).

Row-level authority: [`prose_dissolve_on_census_src_v2_2026-08-03.tsv`](prose_dissolve_on_census_src_v2_2026-08-03.tsv).

---

## 0. Population

| tier | files | marker rows | prose bytes |
|---|---|---|---|
| files with ≥1 `dissolve-on` substring | **83** | — | — |
| files with an actual marker row/field | **67** | **128** | **94.9 KiB** |
| files with mention-only (no marker) | **16** | 0 | — |

**Mention-only** means the file contains `dissolve-on` in prose describing *another*
carrier's trigger (witness tests asserting marker presence, policy notes citing a
sibling row, etc.) — not an authored dissolution marker itself. Those 16 files are
listed in §4; they are out of scope for row migration but in scope for the file
population count the brief names.

**Marker** means a `data …: String` row whose name carries `dissolve_on` /
`dissolution_trigger` / `dissolution_note`, whose value begins with `dissolve-on:` /
`🟡 dissolve-on:` / `DISSOLVES WHEN`, or a struct `dissolve_on:` field.

Concentration: **50% of marker bytes** live in **12 files** (compiler/self_host
frontier + witness routing + ci_floor_plan). `src/v2/compiler/self_host/frontier.dag`
alone carries 4 markers / 6.8 KiB.

---

## 1. Closed-sum disposition migration class

Every marker row is assigned exactly one `disposition_class` — the construction lane
that retires the prose when it lands:

| class | rows | share | migrates to |
|---|---|---|---|
| **ScaffoldOther** | 64 | 50.0% | typed `Disposition` + case-by-case bind (residual) |
| **ScaffoldWitnessCadence** | 24 | 18.8% | witness admission / falsifier / live-read enrollment |
| **ScaffoldSubstrateWall** | 12 | 9.4% | body producer, grammar-owned reads, construction walls |
| **ScaffoldRealizationDispatch** | 8 | 6.3% | `host_effect_apply` / materialization / observation projection |
| **ScaffoldShellEmit** | 9 | 7.0% | bash-emit #5828 / shell→intent Phase 2 |
| **ScaffoldModelMigration** | 7 | 5.5% | single-authority de-fork / namespace resolution |
| **ScaffoldSelfHostRetire** | 3 | 2.3% | seed-retained module/bin retirement |
| **ProseObsoleteFired** | 1 | 0.8% | delete prose (trigger already fired) |

**Finding:** half the markers are `ScaffoldOther` — the lexical classifier's residual.
That is expected: `dissolve-on` prose was authored case-by-case and names heterogeneous
triggers. The actionable head is not deletion (1 fired row) but **typed migration** —
especially the 9 `DuplicateTypedRow` files that already carry a parallel
`Disposition` value beside the prose twin (§2).

### Layer distribution (marker files)

| layer | marker files | marker rows |
|---|---|---|
| compiler | 17 | 34 |
| workflow | 14 | 27 |
| test | 15 | 19 |
| std | 10 | 26 |
| extdeps | 6 | 11 |
| lens | 5 | 11 |

---

## 2. Migration readiness

| readiness | rows | meaning |
|---|---|---|
| **NeedsHumanBind** | 115 | trigger named; `DeclarationRef` bind requires human judgment |
| **DuplicateTypedRow** | 9 | file already has `data …: Disposition = …`; prose is parallel debt |
| **ReadyNamedTrigger** | 2 | `feature:` / `trigger:` spine parseable without bind guesswork |
| **ReadyShortTrigger** | 1 | short explicit `dissolve-on:` under 200 B |
| **FiredDeletable** | 1 | row opens with `DISSOLVED` — hand-verified deletable |

### DuplicateTypedRow files (first migration slice)

These nine files are the cheapest `src/v2` conversions: the typed carrier already
exists; deleting the prose row is a redundancy cleanup, not a modeling exercise.

| file | decl |
|---|---|
| `src/v2/compiler/source_authority.dag` | `module_storage_supply_dissolution_trigger` |
| `src/v2/workflow/ci_floor_peak_emit.dag` | `ci_floor_phase_journal_shell_emit_dissolution_trigger` |
| `src/v2/workflow/ci_regen_rustfmt_path_emit.dag` | `ci_regen_ensure_rustfmt_path_shell_emit_dissolution_trigger` |
| `src/v2/workflow/ci_release_build_emit.dag` | `ci_release_build_shell_emit_dissolution_trigger` |
| `src/v2/workflow/ci_v1_compiler_tests_compile_gate_emit.dag` | `ci_v1_compiler_tests_compile_gate_shell_emit_dissolution_trigger` |
| `src/v2/workflow/ci_workflow_run_emit.dag` | `ci_native_cache_root_shell_emit_dissolution_trigger` |
| `src/v2/workflow/floor_discovery_producer.dag` | `floor_discovery_directory_candidate_dissolve_on` |
| `src/v2/workflow/floor_discovery_producer.dag` | `floor_discovery_live_tree_disposition_dissolve_on` |
| `src/v2/workflow/floor_discovery_producer.dag` | `floor_discovery_surface_text_scan_dissolve_on` |

---

## 3. Sequencing recommendation

1. **Do not plan a deletion pass.** One hand-verified `FiredDeletable` row
   (`target_semantic_decl_shape_predicate_dissolution_note`) — same order of magnitude
   as the corpus-wide prose census (§3 of `dag-note-prose-census.md`).
2. **First conversion slice:** the 9 `DuplicateTypedRow` files — prose twins beside
   existing `Disposition` values; redundancy lens target once presence is whole-tree.
3. **Second slice:** the 9 `ScaffoldShellEmit` markers — one shared trigger
   (`#5828` / shell→intent Phase 2); batchable across `v2.workflow.*_emit` modules.
4. **Third slice:** `ScaffoldWitnessCadence` (24 rows) — couples to exact witness
   admission and live-read classification lanes already active on main.
5. **Residual:** `ScaffoldOther` (64 rows) — requires per-row bind authoring; do not
   batch.

---

## 4. Mention-only files (16)

No marker row; `dissolve-on` appears only as cross-reference prose:

- `src/v2/compiler/build_workspace_grant.dag`
- `src/v2/compiler/host_run_boundary_admission.dag`
- `src/v2/compiler/self_host/frontier_probe_types.dag`
- `src/v2/compiler/self_host/seed_emitter_behavioral_wet_module_bindings.dag`
- `src/v2/compiler/self_host/wet_receipt_enrollment.dag`
- `src/v2/std/effects.dag`
- `src/v2/std/execution_envelope.dag`
- `src/v2/std/witness_evaluation.dag`
- `src/v2/test/claim/c_compilation_unit_witness_test.dag`
- `src/v2/test/claim/ci_spec_witness_test.dag`
- `src/v2/test/claim/long/orchestration_while_emit_test.dag`
- `src/v2/test/claim/shell_emission_authorization_test.dag`
- `src/v2/workflow/ci_heal_skew_guard_emit_test.dag`
- `src/v2/workflow/ci_regen_rustfmt_path_emit_test.dag`
- `src/v2/workflow/ci_v1_compiler_tests_compile_gate_emit_test.dag`
- `src/v2/workflow/witness_admission.dag`

---

## 5. Honesty bound

Classification is **lexical** over unstructured English dissolve-on strings — the
same limitation priced in `dag-note-prose-census.md` §6. Method: extract marker rows
per §0; assign `disposition_class` by highest-scoring keyword family (shell emit,
substrate/body-producer, witness cadence, model migration, realization, self-host,
fired); assign `readiness` from carrier shape.

Stratified hand-check (10 rows, one per class plus all `FiredDeletable` /
`DuplicateTypedRow`):

| disposition_class | correct | notes |
|---|---|---|
| ScaffoldShellEmit | 3/3 | |
| ScaffoldWitnessCadence | 2/3 | one row is live-read + effect-reach fused |
| ScaffoldSubstrateWall | 2/2 | |
| ScaffoldModelMigration | 2/2 | |
| ScaffoldRealizationDispatch | 2/2 | |
| ScaffoldSelfHostRetire | 1/1 | |
| ProseObsoleteFired | 1/1 | |
| ScaffoldOther | 2/3 | residual absorbs mixed witness+substrate notes |

**≈85% primary-class precision**, with known **understatement** of
`ScaffoldWitnessCadence` (some rows classified `ScaffoldOther` carry enrollment facts).
Treat shares as ±10pp.

**Instrument:** one-off Python extractor (not committed — same policy question as
`dag-note-prose-census.md` § instrument). Re-derive from the TSV + class definitions
above; long-term home is a `.dag` lens over marker rows once `Disposition` migration
begins.

---

## 6. Dissolution trigger for this receipt

Deletes when `std.disposition` marker rows replace the prose fleet and an annotation
budget lens counts typed rows — same trigger as `dag-note-prose-census.md` and
`doc_graph_roots.dag` annotation-budget entry.
