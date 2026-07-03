# Plan — Rust seed-shrink census toward `5-collapse-v1` (DRAFT)

**Status:** DRAFT · **design-for-sign** · **do-not-merge** until operator review (parallel to determinism #5937). Carrier-grounded audit + per-chunk sign-off queue · **DESIGN.md + the carriers remain the authority** — this doc is an audit/tracker, not a fact ledger (DESIGN §6). A chunk's real state is its branch/PR + operator sign, not this file. Entry point: [v2-self-hosting.md](v2-self-hosting.md) Track Z (milestone `5-collapse-v1` in `dag/gunbc/roadmap_authority.dag`; `ROADMAP.md` is emitted realization and is not linked here). **Endorsed** by jolly-cat-29 (Section 5 self-host manager, 2026-06-29).

**Re-verified against the live tree on 2026-06-29 by execution** (session bright-lark-472, receipt `main @ 5acbff0f8b`). Re-run §1 receipt before acting on LOC figures — the seed drifts with every merge.

> **SCOPE.** READ-ONLY inventory for `5-collapse-v1`. **No deletions or seed authoring** until per-chunk operator sign.

---

## 0. Core decomposition (read this first)

**Registry authority:** `src/v1/stage0/src/bin/regen_stage0.rs` — `GENERATED_STAGE0_FILES`, `HAND_MAINTAINED_STAGE0_FILES`, `patch_*` hooks. This census is grounded in that registry, not an invented taxonomy.

Reframe "delete ~170k LOC of Rust" as **ground 24 files + one cutover**:

| Class | Files | LOC | What happens at collapse |
| --- | ---: | ---: | --- |
| **GENERATED** | 89 | **~102,450** | **CUTOVER-DELETE** — replaced wholesale by `regen_stage0` from a green whole-tree emit. **Not piecemeal. Not `rm -rf`.** Gated on **real fixpoint** (`5-real-fixpoint`: `content_hash` stage1==stage2). |
| **HAND_MAINTAINED** | 24 | **~32,528** | **DISSOLUTION QUEUE** — each file needs a `.dag` authority (or v2-emitted equivalent) before it flips GENERATED and joins the bulk cutover. Per-chunk operator sign. |
| **Integration tests** | 88 | **~29,934** | Delete **with** v1 — but coverage must **migrate to floor `*_test.dag` witnesses first** (§5). Silent drop = §5 fail-open. |
| **Terminal harness** | ~5 bins + runtime | **~16k+** (shrinks; see §6) | **Non-zero irreducible kernel** — pinned v2-emitted bootstrap that runs the floor and performs the cutover. "Zero" applies to hand-written *compiler logic*, not host physics. |

The honest work is smaller than the headline: **24 grounding PRs + test-migration sub-lane + one regen cutover**, not 170 independent deletes.

---

## 1. Receipt — scale snapshot (`main @ 5acbff0f8b`)

| Surface | Files | LOC | Notes |
| --- | ---: | ---: | --- |
| `src/v1` (all Rust) | 215 | **170,330** | Entire bootstrap seed |
| `GENERATED_STAGE0_FILES` | 89 | **102,450** | v2-emitted compiler + std mirrors |
| `HAND_MAINTAINED_STAGE0_FILES` | 24 | **32,528** | Dissolution queue |
| CLI bins (`stage0/src/bin`) | 10 | 4,843 | Bootstrap kernel (§6) |
| Integration tests (`src/v1/tests`) | 88 | 29,934 | `pipeline.rs` alone 11,842 |
| `src/v2` | 795 `.dag` | **0 `.rs`** | Authority |
| `dag/` | 632 `.dag` | — | Std + extdeps + CI spec |
| Floor witnesses (`*_test.dag`) | 342 | 822 `test fn` | Auto-discovered by `claim_executor` |

---

## 2. Sequencing — parallel lanes vs cutover gate

Coordinate with the **real-fixpoint scoping work-item** (sibling under jolly-cat-29) so both censuses agree on the gate.

```
HAND_MAINTAINED dissolution (Chunks A–H)     real-fixpoint lane (5-real-fixpoint)
  parse cursor #5864                           content_hash stage1==stage2
  main.rs dep-pool #5894                       self_host.dag Stage C
  lens projections, dag_collect, runtime…
         │                                              │
         │  GROUNDING work — design-first, PARALLEL      │
         │  (not deletion; moves files → GENERATED)     │
         └──────────────────┬───────────────────────────┘
                            ▼
              regen cutover (GENERATED bulk ~102k)
              gated ONLY when fixpoint lands
                            ▼
              test-migration sub-lane (§5) must be green
              for each module BEFORE its v1 test deletes
                            ▼
              terminal harness pins (§6) survive as
              v2-emitted bootstrap, not hand-written seed
```

**Rules:**

- **HAND dissolution** = grounding work. Proceeds **in parallel** with fixpoint scoping — emitter lane (jolly-cat) owns Chunks A–B.
- **Bulk cutover-delete** (89 GENERATED files) = **gated on real fixpoint only** — not on finishing every test migration, but test migration is a **hard gate per module** at delete time (§5).
- **NO deletions** until per-chunk operator sign.

---

## 3. GENERATED bulk (~102k LOC) — cutover-delete

v2-emitted compiler pipeline + std mirrors + extdeps language rows. At collapse: **one** `regen_stage0` from green whole-tree emit replaces all 89 files atomically.

**Largest generated modules:**

| File | LOC | `.dag` authority |
| --- | ---: | --- |
| `v1_compiler_emit_rust.rs` | 24,825 | `05_emit` / `06_translate` |
| `v1_compiler_infer.rs` | 14,264 | `04_infer` |
| `v1_compiler_complexity.rs` | 9,895 | infer complexity |
| `v1_compiler_emit.rs` | 6,214 | `05_emit` |
| `v1_test_non_ascii_perf_fixture.rs` | 6,029 | perf fixture |
| `std_*` mirrors (28 files) | ~10,401 | `dag/std/*` (de-fork target) |

**Pipeline stage LOC:**

| Stage | `.dag` | LOC |
| --- | --- | ---: |
| `05_emit` | `05_emit.dag` + orchestration | 38,558 |
| `04_infer` | `04_infer.dag` | 36,463 |
| `02_parse` | `02_parse.dag` | 13,911* |
| std mirrors | `dag/std/*` | 10,401 |
| `00_compile` | `00_compile.dag` | 2,500 |
| `01_tokenize` | `01_tokenize.dag` | 1,031 |
| `03_resolve` | `03_resolve.dag` | 982 |

\*parse is HAND until Chunk A (#5864).

---

## 4. HAND_MAINTAINED dissolution queue (24 files, ~33k LOC)

Copied through regen, excluded from `regen --verify`. Each row = **operator sign-off unit**.

| Chunk | File(s) | LOC | Dissolution trigger | Owner lane |
| --- | --- | ---: | --- | --- |
| **A** | `v1_compiler_parse.rs` | 13,911 | Ground #5864 cursor in `02_parse.dag` → GENERATED | jolly-cat emitter |
| **B** | `main.rs` | 513 | Emit dep-pool from `gunbc.compile_source_model` (#5894 model ✓) | jolly-cat emitter |
| **C** | `v1_compiler_dag_collect*.rs` | 521 | Emitter delegation; delete `patch_bootstrap_dag_collect` | emitter |
| **D** | Lens projections (10 files) | ~3,908 | Interpreter consumes `.dag` lens tables | lens + interpreter |
| **E** | `v1_interpreter.rs` | 6,373 | v2-emitted evaluator + host-effects | runtime |
| **F** | `cli_run.rs` | 4,165 | CI floor → workflow host-effect `apply()`; resolve/reconcile/discovery engine scoped in [docs/plans/cli-run-reconcile-defork.md](cli-run-reconcile-defork.md) | workflow |
| **G** | `coproduct_reflection.rs` | 1,297 | Model↔realization fork grounded | de-fork |
| **H** | Runtime support | ~1,500 | `recorded_fixture`, `resolved_graph_cache`, `wire_value_serialize`, … | runtime |

**Active `patch_*` (3):** `patch_bootstrap_dag_collect`, `patch_languages_consumer_census_mod`, `patch_cargo_toml_for_generated_crate`.

**Sign-off order:** A → B → C → D ∥ → G → E → F → H → GENERATED bulk cutover.

**HAND_MAINTAINED file list (LOC receipt):**

| LOC | File |
| ---: | --- |
| 13,911 | `v1_compiler_parse.rs` |
| 6,373 | `v1_interpreter.rs` |
| 4,165 | `cli_run.rs` |
| 1,297 | `coproduct_reflection.rs` |
| 1,182 | `extdeps_shape_transport_policy_project.rs` |
| 622 | `non_fold_residue_project.rs` |
| 531 | `recorded_fixture.rs` |
| 513 | `main.rs` |
| 481 | `resolved_graph_cache.rs` |
| 447 | `inert_carrier_project.rs` |
| 359 | `doc_reachability_project.rs` |
| 322 | `medium_structure_project.rs` |
| 297 | `v1_compiler_dag_collect_support.rs` |
| 286 | `fact_cardinality_census.rs` |
| 281 | `languages_consumer_census.rs` |
| 256 | `transport_script_position_project.rs` |
| 224 | `v1_compiler_dag_collect.rs` |
| 220 | `module_path_index.rs` |
| 210 | `wire_value_serialize.rs` |
| 179 | `rest_transport_facts.rs` |
| 149 | `corpus_lex.rs` |
| 99 | `import_resolution_project.rs` |
| 70 | `cache_purity_oracle.rs` |
| 54 | `layering_imports_project.rs` |

---

## 5. Test-migration sub-lane (§5 trap — coverage before delete)

The 88 v1 integration-test modules (~30k LOC, **939 `#[test]` fns**, `pipeline.rs` alone **418 tests / 11,842 LOC**) exercise the v1 compiler and interpreter. They delete **with** v1 at collapse — but their **coverage must migrate to `.dag` floor witnesses** (`*_test.dag` under `dag/test/claim` + `src/v2/test`, auto-enrolled by `claim_executor`) **before** each module can go, or collapse silently drops coverage (§5 fail-open).

**Floor discovery authority:** `gunbc.ci_layer_roots` — `witness_layer_roots = [dag, src/v2]`, `witness_discovery_scan_dirs = [dag/test/claim, src/v2/test/claim/manual]`; plus tree-wide `*_test.dag` walk under both witness layer roots.

### 5A. Migration status summary

| Tier | Modules | LOC | v1 `#[test]` fns | Meaning |
| --- | ---: | ---: | ---: | --- |
| **T0 — floor stem match** | 4 | ~1,400 | ~16 | Same-name `*_test.dag` already on floor |
| **T1 — topic witness exists** | ~12 | ~17,000 | ~500+ | Floor has witnesses in the concern area; not 1:1 with v1 module |
| **T2 — coverage debt** | ~71 | ~13,000 | ~400+ | **No floor equivalent** — must author `*_test.dag` before delete |

**T0 exact matches (safe to retire v1 test once floor green confirms equivalence):**

| v1 module | Floor witness |
| --- | --- |
| `coproduct_reflection_conformance_test` | `src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag` |
| `dag_comment_wall_test` | floor corpus (same stem) |
| `parse_table_memo_amortization_test` | floor corpus (same stem) |
| `typescript_program_emit_run_test` | floor corpus (same stem) |

**T1 — partial floor coverage by concern (migrate module-by-module; do not bulk-delete):**

| Concern | v1 LOC | v1 tests | Example floor witnesses (not exhaustive) |
| --- | ---: | ---: | --- |
| **pipeline bulk** | 11,842 | 418 | `dag_compile_clean_witness_test.dag`, `rust_gates_ci_witness_test.dag`, `generated_conformance_floor_test.dag` — covers compile-clean + gate roster, **not** each of 418 pipeline cases |
| **infer / resolve** | 4,716 | ~120 | `branch_infer_test.dag`, `match_infer_fail_open_audit_test.dag`, `name_resolve_cross_tree_resolution_test.dag` |
| **parse / tokenize** | 1,312 | ~60 | `parse_table_claims_test.dag`, `gap4_parse_tokens_remain_test.dag` |
| **emit / TypeScript** | 2,150 | ~30 | `typescript_add_emit_translate_test.dag`, `typescript_enum_union_emit_by_execution_test.dag` |
| **interpreter / eval** | 2,173 | ~45 | `bind_eval_by_execution_test.dag`, `hermetic_fixture_realization_test.dag` |
| **bootstrap / self-host** | 2,808 | ~45 | `compile_source_model_witness_test.dag`, `rust_stage0_gates_witness_test.dag` |
| **lens / hygiene** | 1,333 | ~25 | `lens_verdict_witness_test.dag`, `construction_justification_hygiene_tests` (in floor discovery) |
| **cache / fixture** | 916 | ~15 | `cache_purity_verdict_test.dag`, `cache_key_completeness_test.dag` |
| **cross-cutting** | 917 | ~15 | `generated_conformance_floor_test.dag` (cross_rep), `generic_alias_coproduct_instantiation_test.dag` |

### 5B. T2 coverage debt — derived live, not hand-maintained

**This is no longer a hand-authored table** (that was a parallel ledger — DESIGN §6 forbids it; the old static snapshot drifted from the tree the moment either side changed). The debt roster is now **computed from the live tree** by `v2.lens.test_migration_debt` (`src/v2/lens/test_migration_debt.dag`), which diffs `src/v1/tests/src/*.rs` modules containing a line-anchored `#[test]` against the floor `*_test.dag` witness roster (the same `witness_layer_roots` corpus the CI floor discovers from) by **exact stem equality** — a module is debt iff no floor witness has the identical stem. (A fuzzy substring match was tried and rejected in review: it let the single largest debt module, `pipeline` — 418 test fns — silently match the unrelated floor stem `typescript_import_pipeline`, hiding rather than counting debt. Exact equality can only ever *overstate* debt by missing a topically-covered-but-differently-named witness, never understate it.)

<<<<<<< HEAD
- **Live counts:** `test_migration_debt_module_count()` / `test_migration_debt_total_loc()` / `test_migration_debt_total_test_fns()` (builtins in `cli_run.rs`, dispatched via `v1_interpreter.rs`).
- **Live roster:** `test_migration_debt_module_names()` returns the current debt module list — run this instead of reading a table to see which v1 modules still need a floor witness.
- **Floor gate:** `src/v2/test/claim/manual/test_migration_debt_test.dag` enrolls three shrink-only ratchet witnesses (module count / total LOC / total test-fn count each `<=` their baseline) — floor-discovered and green-by-execution like every other witness. Debt may shrink freely; it must never silently grow.
- **Baselines** (set at this table's dissolution, re-verify by running the lens): 77 modules / 28,546 LOC / 912 `#[test]` fns.

**Migration rule (unchanged):** for each v1 test module, the floor `*_test.dag` witness must be **green-by-execution** in `claim_executor` before the v1 `#[test]` module deletes. Bulk-delete of `src/v1/tests` is forbidden. When `test_migration_debt_module_count()` reaches 0, §5 test-migration is done — no table to keep in sync.
=======
| LOC | v1 module | v1 tests | Required floor witness (to author) |
| ---: | --- | ---: | --- |
| 1,530 | `infer_semantics` | 51 | Infer semantics oracle suite → `src/v2/test/claim/infer_semantics_*_test.dag` |
| 1,300 | `interp_recorded_fixture_test` | 24 | Recorded-fixture replay witnesses |
| 1,238 | `source_audit` | 41 | Source-audit / dep-graph witnesses |
| 733 | `effects` | 36 | Effect-shape / idempotency witnesses |
| 458 | `resolve_cross_process_cache_test` | 10 | Resolve-cache cross-process witnesses |
| 364 | `extdeps_shape_transport_policy_lens_test` | 7 | Lens already in `.dag`; needs floor `*_test.dag` consumer |
| 330 | `floor_skip_discovery_host_test` | 9 | Floor-skip discovery witnesses |
| 274 | `auth_declared_but_unwired_witness_test` | 7 | Auth wiring witnesses |
| 242 | `measure_field_access_test` | 9 | Measure emit/access witnesses |
| 234 | `coverage_completeness_lens_test` | 6 | Coverage-completeness lens floor witness |
| 221 | `list_free_monoid_chokepoint_test` | 8 | FreeMonoid generic-inference witnesses |
| 208 | `consumed_input_closure_drift_test` | 2 | Wiring-liveness witnesses |
| 206 | `ir_fixture_seam_soundness_test` | 3 | IR-fixture seam witnesses |
| 187 | `sub_value_lattice_factor_test` | 8 | Sub-value lattice witnesses |
| 186 | `pd3_adversarial` | 8 | Adversarial parse witnesses |
| 181 | `route_a_final_six_test` | 6 | Route-A emit regression witnesses |
| 179 | `variant_owner_disambiguation_test` | 3 | Variant-owner resolve witnesses |
| 96 | `fn_as_value_test` | 2 | Fn-as-value infer witnesses — shrunk from 296/10 by #6140, residual only |
| 79 | `type_alias_phantom_param_test` | 2 | PhantomData type-alias emit witnesses |
| 81 | `languages_consumer_census_lens_test` | 1 | Lens has `.dag`; floor witness missing |
| 85 | `fact_cardinality_lens_test` | 1 | Lens has `.dag`; floor witness missing |
| 65 | `coproduct_reflection_conformance_test` | 1 | Pinned-harness residual after #6142 |
| 146 | `witness_option_bridge_test` | 3 | Optional/`Value::Null` bridge witnesses — shrunk from 223/6 by #6153, residual only |
| 92 | `map_lookup_dual_dispatch_test` | 2 | Map-lookup dispatch witnesses — shrunk from 175/6 by #6150, residual only |
| … | *(≈43 more modules < 200 LOC each)* | … | Per-module `*_test.dag` or fold into concern-suite above |

**Migrated off this table (2026-07-02, confirmed gone from `src/v1/tests/src/lib.rs`):** `render_repeat` (#6147), `cron_tag` (#6145), `money_carrier_cost_witness` (#6143) — fully deleted, no residual.

**Debt total:** ~66 modules / ~12.5k LOC / ~380+ `#[test]` fns without floor equivalent (revised down from the stale 71/13k/400+ count above — six rows corrected 2026-07-02 against current `main`).

**Migration rule:** for each v1 test module, the floor `*_test.dag` witness must be **green-by-execution** in `claim_executor` before the v1 `#[test]` module deletes. Bulk-delete of `src/v1/tests` is forbidden.
>>>>>>> origin/main

---

## 6. Terminal irreducible harness (honest end-state)

§7 ideal: "Rust shrinks to zero." §5 trap-word check: **decidable claim?** Only for **hand-written compiler logic** — NOT for the host that executes effects. The substrate is data; something must run the first `.dag`.

### 6A. What the harness does (cannot delete without replacing)

| Role | Current carrier | LOC | Irreducible? |
| --- | --- | ---: | --- |
| **Floor runner** | `claim_executor` bin | 1,711 | Yes — executes `ci_floor_plan.dag` witnesses |
| **Cutover tool** | `regen_stage0` bin | 1,681 | Yes until cutover complete; then pins as reproducibility oracle |
| **Compile CLI** | `main.rs` (HAND) | 513 | Yes — `gunbc compile` entry; dissolves to GENERATED after Chunk B |
| **CI orchestration** | `cli_run.rs` (HAND) | 4,165 | Yes until workflow host-effect `apply()` replaces it |
| **`.dag` evaluator** | `v1_interpreter.rs` (HAND) | 6,373 | Yes — runs `.dag` programs + host-effect dispatch (shell/file/REST) |
| **Host transports** | `rest_transport_facts`, `wire_value_serialize`, … | ~600 | Yes — physics boundary (OS/shell/network) |
| **Caches / fixtures** | `resolved_graph_cache`, `recorded_fixture` | ~1,000 | Shrinks with Materialization kernel (§4 infra); not zero at first cutover |

### 6B. Verdict — zero vs irreducible kernel

| Claim | Verdict |
| --- | --- |
| Hand-written **compiler logic** → zero | **Decidable YES** — the 89 GENERATED files (~102k) + dissolved HAND compiler files are the target; `.dag` is authority. |
| Total Rust bytes → zero | **Decidable NO** — host must execute effects; violates §5 if claimed. |
| Terminal end-state | **Pinned, content-addressed, v2-emitted bootstrap binary** ([v2-self-hosting.md](v2-self-hosting.md), `bootstrap.dag` `SeedHonestyDischarge` / DDC). Compiler logic is *emitted*; the harness is *pinned*, not hand-maintained. |
| Estimated irreducible kernel (post-collapse) | **~8–15k LOC** v2-emitted Rust (evaluator host-effects + `claim_executor` + pinned `regen` oracle) — order-of-magnitude, not a ratchet. Re-measure at cutover. |

**Name the kernel:** `gunbc` bootstrap = `{ claim_executor, gunbc compile, v1_interpreter host-effect runtime }` as one pinned, reproducible-from-`.dag` artifact. `regen_stage0` survives as the cutover/receipt tool, not as permanent compiler source.

---

## 7. Roadmap gates

| Gate | Status | Relation to collapse |
| --- | --- | --- |
| `5-cargo-green` | **done** (#5777/#5873) | Parent of `5-collapse-v1` |
| `5-regen-verify` | **done** | GENERATED byte-identical |
| `5-real-fixpoint` | open | **Gates bulk cutover** |
| `5-dissolve-patches` | open | Empty HAND + no `patch_*` |
| `5-defork` | open | Grounding cluster |
| `5-seed-honesty` | open | DDC |
| `5-collapse-v1` | open | Terminal |

---

## 8. Shrink ratchets

| Ratchet | State | Resolution |
| --- | --- | --- |
| Clone census | RED — 21,540 vs 20,200 (+1,138); `#[ignore]` | Substrate migration; no cap bump |
| Inert carrier roster | 12 entries | Shrink as consumers wire |
| Languages consumer census | 71 + 64 rows | Repoint to `extdeps/languages/*` |
| Fact cardinality | Cross-tree coexistence | De-fork consolidation |

---

## Dissolution trigger (DESIGN §6)

Delete this doc when `5-collapse-v1` lands. Until then: re-run §1 receipt; keep §5 T2 debt at zero before each v1 test delete.
