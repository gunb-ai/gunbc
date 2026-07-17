# Curated-cargo frontier probe sweep (cool-crab-179)

**Scope:** probe-only measurement — no `frontier.dag` edits, no flips.  
**Harness:** `gunbc compile --dependency-pool-index primary-precedence` + seed-linked `v1-compiler` crate (mirrors `tools.self_host_curated_seed_linked_harness` spine). Pull harness authority from `origin/session/snappy-ferret-198` before running; do not fork.  
**Skipped:** `03_body_producer` (already proven PHANTOM / flip in flight on ferret #6782).

## Executive summary

| Verdict | Count | Meaning |
|---------|------:|---------|
| PHANTOM (cargo green) | 0 | No phantom gates found in this pass |
| CONFIRMED-Gate_A | 2 | Real rustc refusal — emitter Rc/Optional/ownership |
| HARNESS_ARTIFACT_std_dup | 18 | Orthogonal closure std-dup (`Int128`/`Witness` vs seed) — **not** module verdict; needs per-module cssl shims |
| CONFIRMED-namespace / Gate_B / NEW | 0 | — |

**Key finding:** With shim-free curated emit (empty `shim_lib_rel`), 18/20 modules hit std-type duplication between emitted import-closure and `v1-compiler` seed — the harness rule's orthogonal artifact class. **Module-level gate truth for those 18 requires ferret harness extensions** (per-module `shim_lib` + dep shim writes, as body_producer demonstrates). Two modules cleared the std-dup layer and refused on Gate A.

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
3. **18 modules** need cssl shim extensions before module-level gate can be distinguished from harness artifact. Coordinate with snappy-ferret-198 — do not fork harness.
4. **Emit is clean** (0 diagnostics) on all 20 — emit-clean ≠ rustc-green gap confirmed across the roster.

## Reproduce

```bash
# one-time: pull harness authority (not committed here — ferret #6782)
git fetch origin session/snappy-ferret-198
git checkout origin/session/snappy-ferret-198 -- dag/tools/self_host_curated_seed_linked_harness.dag

# full sweep (~45min sequential)
./scripts/curated_cargo_frontier_probe_sweep.sh

# single module
./scripts/curated_cargo_probe_one.sh src/v2/compiler/02_parse.dag
```

Machine-readable: `docs/probes/curated_cargo_frontier_probe_sweep.tsv`
