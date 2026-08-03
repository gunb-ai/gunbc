# Frontier probe exact-head survey (Lane E interim census)

**Status:** INTERIM AUDIT — 17 of 27 compiler roster modules probed at HEAD; not a roster acceptance or closeout.

**Head:** `9f978aa8df` (main at survey start) on worktree `neat-cat-330`.

**Harness:** `frontier_probe_survey` seed bin, per-module via `frontier_probe_emit_from_ingest` (fixture-free ingest overlay). Receipt TSV: `frontier_probe_exact_head_survey_2026-08-03.tsv`.

**Denominator:** 27 modules in `compiler_frontier_sweep_order` (`frontier.dag`). **Surveyed:** 17. **Remaining:** 10 (probe in progress at commit time).

## Findings (execution-measured, assemble stage only)

| Blocker class | Count (of 17) | Located stage | Dominant located_reason |
|---------------|---------------|---------------|-------------------------|
| RealizationGap | 14 | ProbeStageAssemble | `^parse_grammar_choice_overlap_residue` |
| NameResolutionGap | 3 | ProbeStageAssemble | `^resolve_module_not_found` |

**Not observed in this slice:** EmitSurfaceGap, UpstreamSemanticRefusal, SelfEmitReady. The probe stops at assemble for the dominant class — emit and semantic-derivation stages were not reached for these 17 modules. This is evidence about which stage blocks the probe, not an inferred emit blocker.

### NameResolutionGap modules (3/17)

- `src/v2/compiler/source_authority.dag`
- `src/v2/compiler/00_compile.dag`
- `src/v2/compiler/03_ingest.dag`

### Reconciliation vs pre-7627 roster rows

PR #7627 landed vocabulary + totality wall without a second measurement pass. Several roster rows still carry `UpstreamSemanticRefusal` / `EmitSurfaceGap` at emit/semantic stages from prior knowledge attribution. This survey **contradicts** those rows for every module probed so far: all 17 refuse at **ProbeStageAssemble** with grammar residue or module-not-found, not at emit.

**This document is an audit receipt.** Roster row rewrite and totality-wall witnesses follow when all 27 modules are probed and `frontier.dag` rows bind the manifest.

## Remaining modules (not yet surveyed at commit)

`03_normalize`, `03_resolve`, `03_name_resolve`, `emit_module`, `05_emit_orchestration`, `emit_semantic_decl`, `emit_host`, `emit_produced`, `03_body_producer`, `program_assembly`
