# Plan — cli_run.rs hollowing map (Chunk F)

**Status:** high-level hollowing map (operator-directed rework of closed #7074) · **DESIGN.md + the carriers remain the authority** (DESIGN §6). **`dag/gunbc/plans/cli_run_hollowing_plan.dag` is the authority**; `docs/plans/cli-run-hollowing-plan.md` is its generated projection (PlanArtifact). Linked from [seed-shrink-census.md](seed-shrink-census.md) Chunk **F** (dissolution sign-off unit for `cli_run.rs`, not a live whole-file LOC) and [v2-self-hosting.md](v2-self-hosting.md) Wave 4 kernel story — cite those for sequencing; **§0 below is the live scale receipt**; this doc answers *what the file does* and *how each chunk dies*.

> **Read this in 5 minutes.** `cli_run.rs` is the v1 **workflow orchestration kernel**: it walks the `.dag` corpus, drives resolve/typecheck/reconcile, runs the CI floor and witness corpus, bridges host facts into lenses, and pins a shrinking set of HAND-Rust path/grant/discovery scaffolds. It is **not** the compiler front-end (that is already `.dag` + GENERATED Rust). Terminal shape: emit the orchestration from workflow `.dag` + shrink to the pinned bootstrap kernel ([seed-shrink-census.md](seed-shrink-census.md) §6). Deep dive on the resolve/reconcile/discovery *cluster only*: [cli-run-reconcile-defork.md](cli-run-reconcile-defork.md) — link, do not duplicate here.

---

## 0. Scale receipt (live tree)

- **Live whole file (measured by execution, 2026-07-23):** `wc -l src/v1/stage0/src/cli_run.rs` → **26,773 lines**; `rg -c '^\s*(pub\s+)?(async\s+)?fn\s+' src/v1/stage0/src/cli_run.rs` → **976 fns** (22 `test_*` inline modules; remainder production-scope). The §7 seed is **not** stable — it grew ~6× since the census snapshot.
- **`seed-shrink-census.md` Chunk F (~4,165 LOC)** names the **dissolution sign-off unit** (CI orchestration → workflow `host_effect_apply`), not a live whole-file count. The other **~22.6k LOC** are the same file's interim seed body: classified by the **16 functional areas in §2** below, and tracked as managed seed growth in ROADMAP **§7 Seed interim** ([`ts-seed-interim`](../../ROADMAP.md) in `dag/gunbc/roadmap_authority.dag`) — host bridges, lens host feeds, floor/discovery machinery absorbed while v2 lanes land.
- **Sign-off order:** Chunk F runs **after** emitter chunks A–E and de-fork G per census §4 — orchestration dissolves once the compiler pipeline and host-effect spine can carry the floor without a hand-written driver.

## 1. Area map (summary)

| Area | v2 subsumption (authority) | Disposition | Named trigger |
| --- | --- | --- | --- |
| Workspace & repo paths | `gunbc.cli_run_workspace_root_scaffold` · `gunbc.cli_run_repo_grant` · [effect-namespace-grants.md](effect-namespace-grants.md) | emit-when | Chunk F → GENERATED `host_effect_apply`; grant presets land |
| CLI argv & bin dispatch | `gunbc` bins + `src/v2/workflow/*` orchestration rows | seed-kernel-retained | Wave 4 pinned bootstrap (~8–15k LOC) |
| Corpus discovery & file walk | `gunbc.ci_spec` discovery_scan_dirs · `gunbc.ci_floor_plan` | emit-when | CI native routing (#7069) + witness-realization |
| Resolve / typecheck entry orchestration | v2.compiler pipeline (GENERATED) · [cli-run-reconcile-defork.md](cli-run-reconcile-defork.md) | emit-when | Reconcile-defork Phase 1–3 + closure equivalence receipt |
| Reconcile & env merge | `v2.compiler` reconcile stage (GENERATED) | already-subsumed | Orchestration wrapper deletes with Chunk F |
| Typed module cache / memo | `std.materialize` / duplicate-work lattice ([duplicate-work-graph-lens-design.md](duplicate-work-graph-lens-design.md)) | emit-when | ComputationIdentity qualifier lands on materialize spine |
| Import & module-graph host facts | `v2.lens.module_graph` · `import_resolution_facts` host seam | emit-when | Reconcile-defork Phase 1 repoint + namespace SymbolIndex |
| Reference / cross-tree edges | `v2.std.cross_tree.resolution` · [namespace-resolution-design.md](namespace-resolution-design.md) | emit-when | Reference-deps B3 + import grammar deletion (B4) |
| Affected-set & diff provenance | `gunbc.ci_spec` selection · `v2.lens.effect_reach` · [module-identity-storage-binding-design.md](module-identity-storage-binding-design.md) | emit-when | Declared `SourceRef` boundary + provenance ingest |
| Floor / witness execution | `claim_executor` · `src/v2/workflow/ci_floor_plan.dag` · [witness-realization-plan.md](witness-realization-plan.md) | emit-when | Workflow `host_effect_apply` replaces HAND driver |
| Compile-clean shard scope | `tools.dag_compile_clean_scope` · `v2.compiler.compile` | partial | Import-closure authority unified on `module_graph` |
| Regen oracle & self-host scope | `regen_stage0` · `v2.compiler.self_host.frontier` | delete-with-v1 | Wave 4 real-fixpoint cutover (`5-real-fixpoint`) |
| Host builtin bridge (interpreter seam) | `v1_interpreter` builtins · per-lens `*_live` projections | seed-kernel-retained | Node-tree readers dissolve seams (#5364 class); kernel keeps physics |
| Lens census / hygiene host feeds | `v2.lens.*` tables · `non_fold_residue` · `fact_cardinality` | emit-when | Each lens lands node-tree reader or explicit host row retires |
| Floor observability & width | `gunbc.floor_materialization` · `dag/std/realization_width.dag` · [realization-measurement-loop.md](realization-measurement-loop.md) | partial | Scheduler emits receipts; HAND timing deletes with floor native routing |
| Test-migration debt builtins | `v2.lens.test_migration_debt` | delete-with-v1 | v1 `src/v1/tests` deleted at collapse (census §5) |

## 2. Functional areas (plain language)

### 2.1 Workspace & repo paths

**Today:** discovers git checkout root (`workspace_root`, `OnceLock` scaffolds) and enforces repo-relative path containment before filesystem effects (`repo_relative_path*` gates). **Subsumes:** `gunbc.cli_run_workspace_root_scaffold` + `gunbc.cli_run_repo_grant` (effect-grant model). **Disposition:** emit-when — HAND-Rust until Chunk F emits workflow host-effect apply with typed grants. **Trigger:** Chunk F + effect-namespace-grants P-B preset rows.

### 2.2 CLI argv & bin dispatch

**Today:** `claim_batch`, `claim_executor`, and `gunbc ci` entry routing — argv parse into which `.dag` entry/function runs. **Subsumes:** thin bootstrap bins ([seed-shrink-census.md](seed-shrink-census.md) §6 terminal harness). **Disposition:** seed-kernel-retained — some argv seam survives as pinned bootstrap. **Trigger:** Wave 4 kernel pin; not zero.

### 2.3 Corpus discovery & file walk

**Today:** walks `dag/` + `src/v2/` for `*_test.dag`, roster entries, and compile-clean shard inputs (`collect_dag_files`, discovery skip arms). **Subsumes:** `gunbc.ci_spec` + floor plan nodes. **Disposition:** emit-when. **Trigger:** CI onto v2 native routing (#7069) + witness realization scheduling. **In motion (2026-07-22+):** native bulk-machine arc (ROADMAP group 2a) and #7090 floor DECISIONS repoint (discovery producer + compile-clean scope) reduce HAND discovery/scope arms.

### 2.4 Resolve / typecheck entry orchestration

**Today:** `resolve_entry_graph*`, whole-tree resolve drivers, diagnostic gating — orchestrates GENERATED `03_resolve`/`04_infer` but the *closure selection and entry wiring* still live HAND. **Subsumes:** compiler pipeline + [cli-run-reconcile-defork.md](cli-run-reconcile-defork.md). **Disposition:** emit-when. **Trigger:** Phase 1 closure repoint onto `v2.lens.module_graph.import_closure_live` with equivalence receipt; Phases 2–3 cache/reconcile repoint.

### 2.5 Reconcile & env merge

**Today:** thin wrapper calling GENERATED reconcile — the algorithm is already `.dag`. **Subsumes:** `v2.compiler` reconcile stage. **Disposition:** already-subsumed (logic); HAND wrapper deletes with Chunk F. **Trigger:** Chunk F orchestration emit.

### 2.6 Typed module cache / memo

**Today:** `typed_module_cache_*`, `resolved_graph_cache` integration — memoizes resolve/typecheck products keyed partly by module path. **Subsumes:** `std.materialize` / duplicate-work identity lattice. **Disposition:** emit-when. **Trigger:** duplicate-work graph lens Half A/B lands on materialize spine.

### 2.7 Import & module-graph host facts

**Today:** host builtins `import_resolution_facts`, `module_declaration_facts`, `layer_import_facts` regex-scan sources and feed lenses. **Subsumes:** `v2.lens.module_graph`, `v2.lens.layering_imports`. **Disposition:** emit-when. **Trigger:** Reconcile-defork Phase 1 + layering reference-repoint terminal (import grammar deletion).

### 2.8 Reference / cross-tree edges

**Today:** `reference_resolution_facts` / `reference_edges_as_import_facts` bridge import-less qualified refs for layering and module graph. **Subsumes:** `v2.std.cross_tree.resolution` + namespace containment tree. **Disposition:** emit-when. **Trigger:** namespace-only resolution terminal ([namespace-resolution-design.md](namespace-resolution-design.md) B3→B4).

### 2.9 Affected-set & diff provenance

**Today:** git-diff observation, import-closure shard compile, `ReadsLiveTree` inference bridge, declared-source-ref selection — decides what the floor runs on a PR. **Subsumes:** `gunbc.ci_spec` selection + `v2.lens.effect_reach`. **Disposition:** emit-when. **Trigger:** module-identity-storage-binding Phase 0 enrollment + provenance ingest (`affected_set_reading_from_git_diff_provenance`).

### 2.10 Floor / witness execution

**Today:** `run_discovery_corpus`, parallel witness scheduling, governor width — the CI floor runtime. **Subsumes:** `claim_executor` workflow + witness-realization plan. **Disposition:** emit-when. **Trigger:** `host_effect_apply` workflow realization replaces HAND loop ([host-effect-orchestration.md](host-effect-orchestration.md)). **In motion (2026-07-22+):** witness-realization scheduling + host-effect transport lanes (shell→intent Phase 2) and #7090 returning work on floor execution paths.

### 2.11 Compile-clean shard scope

**Today:** decides whole-tree vs import-closure compile for `dag_compile_clean_gate` on PR diffs. **Subsumes:** `tools.dag_compile_clean_scope` + compile gate in `v2.compiler.compile`. **Disposition:** partial — policy modeled, HAND still executes scope walk. **Trigger:** single `module_graph` closure authority end-to-end.

### 2.12 Regen oracle & self-host scope

**Today:** `regen_input_sources`, regen skip on PR diffs, self-host staleness inputs. **Subsumes:** `regen_stage0` + frontier carrier. **Disposition:** delete-with-v1 — bulk GENERATED cutover, not piecemeal. **Trigger:** `5-real-fixpoint` gate ([v2-self-hosting.md](v2-self-hosting.md) Wave 4).

### 2.13 Host builtin bridge

**Today:** dispatches `filesystem_read`, fact builtins, measurement counters from `.dag` eval into Rust host physics. **Subsumes:** interpreter kernel D + witness realization. **Disposition:** seed-kernel-retained — irreducible host physics shrinks but does not hit zero. **Trigger:** per-builtin dissolution rows on frontier carrier.

### 2.14 Lens census / hygiene host feeds

**Today:** projects `non_fold_residue`, `fact_cardinality`, inert-layer reach, construction-justification census via host-fed rows. **Subsumes:** respective `v2.lens.*` modules. **Disposition:** emit-when — each host feed retires when lens reads node tree. **Trigger:** node-tree reader lands per lens (#5364 class).

### 2.15 Floor observability & adaptive width

**Today:** floor materialization receipts, slowest-witness attribution, cgroup/memory governor hooks for parallel width. **Subsumes:** `gunbc.floor_materialization`, `realization_width`, measurement-loop plan. **Disposition:** partial. **Trigger:** scheduler emits receipts natively; HAND stderr-only timing deletes.

### 2.16 Test-migration debt builtins

**Today:** `test_migration_debt_*` builtins back the v1-test→floor witness migration ratchet. **Subsumes:** `v2.lens.test_migration_debt`. **Disposition:** delete-with-v1. **Trigger:** census §5 — coverage migrated before `src/v1/tests` delete.

## 3. Gaps (no v2 home yet — named, not invented)

- **Union-resolve once-per-node receipt counter** — process-wide resolve counting for union-resolve MUB contract tests; no modeled carrier; stays HAND until resolve orchestration emits from workflow (.dag issue tracked in reconcile-defork fence).
- **Pre-resolve discovery skip before modeled `floor_kernel_precompute_would_skip`** — `cli_run_discovery_skip_before_resolve` scaffold; unblock named in ROADMAP `2-provenance-ingest` / affected-set precompute pruning.

## 4. What this doc is not

- Not a per-function ledger (#7074 closed) — area grain only.
- Not a substitute for [cli-run-reconcile-defork.md](cli-run-reconcile-defork.md) on the resolve/reconcile cluster.
- Not a LOC census — see [seed-shrink-census.md](seed-shrink-census.md) for file-level receipts.

## Dissolution trigger (DESIGN §6)

Delete when `cli_run.rs` Chunk F is GENERATED or deleted — orchestration runs from workflow `.dag` + pinned bootstrap kernel only, and this high-level map is redundant with the frontier carrier + seed-shrink census.
