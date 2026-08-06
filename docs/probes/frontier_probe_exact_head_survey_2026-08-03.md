# Frontier probe exact-head survey (Lane E interim census)

**Status:** INTERIM AUDIT — superseded vocabulary; not a roster closeout. Do not cite "17 surveyed / 10 open."

**Census at head `9f978aa8df` only:** execution_measurement=17, no_execution_measurement=10, self_emit_ready=0, emitter_produced=0. All 27 rows remain SeedRetained. Not combinable with a later head. Three NameResolutionGap rows in the 17 are suspect pre-#7762 manifest-elision artifacts (see #7762).

**Head:** `9f978aa8df` (main at survey start, 2026-08-02) on worktree `neat-cat-330`. **Point-in-time only:** at time of writing current main is `19cc776d4e` (44 commits later); five of those commits touched the v1 emission path (#7685, #7691, #7708, #7709, #7733). Verdicts may have moved — this receipt is a dated reading, not a claim about current main. Spot-check at merged main (`d1fe52102b`, 2026-08-03): `04_infer`, `06_translate`, `materialization_carriers` still `RealizationGap`/`^parse_grammar_choice_overlap_residue` @ assemble; `03_ingest` still `NameResolutionGap`/`^resolve_module_not_found` @ assemble.

**Harness:** `frontier_probe_survey` seed bin, per-module via `frontier_probe_emit_from_ingest` (fixture-free ingest overlay). Receipt TSV: `frontier_probe_exact_head_survey_2026-08-03.tsv`.

**Denominator:** 27 modules in `compiler_frontier_sweep_order` (`frontier.dag`). **execution_measurement at 9f978aa8df:** 17. **no_execution_measurement at 9f978aa8df:** 10 (listed below).

## Findings (execution-measured, assemble stage only)

| Blocker class | Count (of 17) | Located stage | Dominant located_reason |
|---------------|---------------|---------------|-------------------------|
| RealizationGap | 14 | ProbeStageAssemble | `^parse_grammar_choice_overlap_residue` |
| NameResolutionGap | 3 | ProbeStageAssemble | `^resolve_module_not_found` |

**Not observed in this slice:** EmitSurfaceGap, UpstreamSemanticRefusal, SelfEmitReady. The probe stops at assemble for the dominant class — emit and semantic-derivation stages were not reached for these 17 modules. This is evidence about which stage blocks the probe, not an inferred emit blocker.

### NameResolutionGap modules (3/17; suspect pre-#7762 manifest elision)

- `src/v2/compiler/source_authority.dag`
- `src/v2/compiler/00_compile.dag`
- `src/v2/compiler/03_ingest.dag`

### Reconciliation vs pre-7627 roster rows

PR #7627 landed vocabulary + totality wall without a second measurement pass. Before this PR, several roster rows carried `UpstreamSemanticRefusal` / `EmitSurfaceGap` at emit/semantic stages from prior knowledge attribution. This survey **contradicted** those rows for all 17 probed modules: each refuses at **ProbeStageAssemble** with grammar residue or module-not-found, not at emit.

**This document is an audit receipt.** In PR #7697 all **17 execution-measured** `frontier.dag` rows were hand-authored from this TSV (`measured_probe` aligned; `migration_trigger` kept as authored claims). Totality-wall witnesses and the remaining **10 no-execution-measurement** roster rows await a follow-up closeout PR (blocked on #7762 merge for post-fix 27/27 rerun).

## Declared vs survey oracle (2026-08-03)

Run: `docs/probes/diff_frontier_declared_vs_survey.sh` — refuses when `frontier.dag` measured_probe disagrees with this TSV (independent oracles; does not write rows).

**At survey time (pre-hand-edit):** 12 mismatches among the 17 execution-measured modules — declared rows still carried #7627 `UpstreamSemanticRefusal` / `EmitSurfaceGap` at emit/semantic stages while survey measured `RealizationGap` or `NameResolutionGap` at **ProbeStageAssemble** only.

| Declared (pre-PR #7697 edit) | Survey (17/17 measured) |
|------------------------------|---------------------------|
| UpstreamSemanticRefusal @ SemanticDerivation, `^gate_a_flip_probe_unresolved_compiler_error` | RealizationGap @ Assemble, `^parse_grammar_choice_overlap_residue` (9 modules) |
| UpstreamSemanticRefusal @ SemanticDerivation | NameResolutionGap @ Assemble, `^resolve_module_not_found` (3 modules) |
| EmitSurfaceGap @ Emit, type-ref / namespace reasons | RealizationGap @ Assemble, grammar residue (2 modules) |

**After PR #7697:** all 17 execution-measured modules have hand-authored rows in `frontier.dag` aligned to this TSV; `diff_frontier_declared_vs_survey.sh` greens for 17/17. The **10 no-execution-measurement** modules (`03_normalize` through `program_assembly` below) were not rewritten — their rows still reflect pre-survey knowledge attribution.

**Authoring rule:** update `frontier.dag` rows by hand using this TSV as evidence; migration triggers state what would let migration, not restate the measured located_reason.

## Remaining modules (no_execution_measurement at 9f978aa8df; roster rows not rewritten in PR #7697)

`03_normalize`, `03_resolve`, `03_name_resolve`, `emit_module`, `05_emit_orchestration`, `emit_semantic_decl`, `emit_host`, `emit_produced`, `03_body_producer`, `program_assembly`
