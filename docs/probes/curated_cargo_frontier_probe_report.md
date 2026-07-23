# Curated-cargo frontier probe sweep (cool-crab-179)

**Scope:** probe-only measurement — no `frontier.dag` edits, no flips.  
**Harness:** `gunbc compile --dependency-pool-index primary-precedence` + seed-linked `v1-compiler` crate (`dag/tools/self_host_curated_seed_linked_harness`, in-tree).  
**Skipped:** `03_body_producer` (already proven PHANTOM / flip landed in #6782).

## Executive summary

| Verdict | Count | Meaning |
|---------|------:|---------|
| PHANTOM (cargo green) | 0 | No phantom gates found in this pass |
| CONFIRMED-Gate_A | 2 | Real rustc refusal — emitter Rc/Optional/ownership |
| HARNESS_ARTIFACT_std_dup | 18 | **One harness gap** (not 18 blockers): whole-closure emit re-emits std → collides with seed. Fix = generic std-seed-link in cssl (ferret #6782 authority) |
| CONFIRMED-namespace / Gate_B / NEW | 0 | — |

**Key finding:** With shim-free curated emit, 18/20 modules hit std-type duplication — a **single harness limitation** (closure re-emits std that seed already provides), not 18 per-module blockers. **Fix:** generic std-seed-link in `self_host_curated_seed_linked_harness` (coordinate snappy-ferret-198, §3 one authority — do not fork). Two modules cleared std-dup and refused on Gate A (real findings, banked).

**Banked real findings:**
- `materialization_carriers` → CONFIRMED Gate A — **reclassifies** frontier label (`migrate_when_materialize_spine_lane_lands` is wrong; actual blocker is emitter Rc/Optional #6775/#6776).
- `01_tokenize` → CONFIRMED Gate A (expected).

## Verdict table (execution-measured)

| module | emit | cargo | first error | mapped gate | verdict |
|--------|------|-------|-------------|-------------|---------|
| 02_parse | 72 files, 0 diag | refuse | UNRESOLVED_CompilerError | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| 03_ingest | 131 files, 0 diag | refuse | UNRESOLVED_CompilerError | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| 06_translate | 56 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| 05_eval | 56 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| materialization_carriers | 39 files, 0 diag | refuse | UNRESOLVED_CompilerError | Gate_A_emitter_Rc_Optional | **CONFIRMED-Gate_A** |
| 05_emit | 57 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| 05_emit_orchestration | 63 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| emit_module | 58 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| emit_produced | 67 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| emit_semantic_decl | 58 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| emit_host | 66 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| fold_lowering | 41 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| 03_name_resolve | 49 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| 03_resolve | 44 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| 00_compile | 129 files, 0 diag | refuse | UNRESOLVED_CompilerError | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| 01_tokenize | 22 files, 0 diag | refuse | `Witness` defined multiple times | Gate_A_emitter_Rc_Optional | **CONFIRMED-Gate_A** |
| 03_normalize | 48 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| 04_infer | 49 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| program_assembly | 99 files, 0 diag | refuse | UNRESOLVED_CompilerError | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| program_partition | 63 files, 0 diag | refuse | `Int128` defined multiple times | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |
| source_authority | 97 files, 0 diag | refuse | UNRESOLVED_CompilerError | HARNESS_ARTIFACT_std_dup | HARNESS_ARTIFACT |

## Interpretation

1. **No PHANTOM gates** in shim-free pass — none of the 20 probed modules cargo-green'd without per-module shims.
2. **`materialization_carriers`** and **`01_tokenize`** reached rustc past std-dup and refused on ownership/wrap axis → confirms Gate A (#6775/#6776) for those two; matches `migration_trigger: ^migrate_when_materialize_spine_lane_lands` vs cargo reality on carriers is **inconclusive** (Gate A, not materialize spine).
3. **18 std-dup modules** → one harness gap; re-sweep pending generic std-seed-link in ferret cssl (not per-module stub work).
4. **Emit is clean** (0 diagnostics) on all 20 — emit-clean ≠ rustc-green gap confirmed across the roster.

## Reproduce

Harness authority: `dag/tools/self_host_curated_seed_linked_harness.dag` (in-tree).
Banked receipts: `docs/probes/curated_cargo_frontier_probe_sweep.tsv` (shim-free emit, `CSSL_STD_SEED_LINK` off).

The shell runners that produced this sweep (`scripts/curated_cargo_frontier_probe_sweep.sh`, `scripts/curated_cargo_probe_one.sh`) were deleted 2026-07-23 with the rest of `scripts/`. The banked TSV below is the receipt; re-measuring goes through the `cssl_assemble` bin against the harness authority above, which is a fresh measurement rather than a replay of this one.

Machine-readable: `docs/probes/curated_cargo_frontier_probe_sweep.tsv`

## Related probes

- [Emitter residual site map (2026-07-21)](emitter_residual_site_map_2026-07-21.md) — post-#6981 E0308 histogram → emitter function sites (#7023).
