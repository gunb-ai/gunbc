# first_slice measurement — demand-serve-program-image

Off-srv1 source: fleet-converge run **43637222454** / job **132581870410**
(srv1-09 runner), revision branch
`t6-cutover-819918c44c768c92045f54e4a6ee5632381f87e7-ec057ca81a88d012-plan`,
step `serve_entry_verdict` (gunbc.live_deploy.pre_swap_admission), 45-min bound per #16485.

## What the step runs
`gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/live_deploy/pre_swap_admission.dag
 --function record_plan_serve_entry_compile_verdict`
That entry's pre-entry resolves+types a **1436-module** closure (a proxy for a serve entry's
restart cost), then spawns a CHILD `gunbc compile` of roadmap_ingress (their node: the 23-min cold
compile). This step is the serve ENTRY-VERDICT harness, not a live serve process.

## [pre-entry] block (the restart-cost decomposition we DO have)
phase=workspace_discovery            ms=1        scale=closure
phase=pool_source_newline_index      ms=1414     scale=closure
phase=pool_source_tokenize           ms=26408    scale=closure
phase=demand_layout_acquire          ms=45497    scale=closure
phase=pool_heads_parse               ms=4292     scale=closure
phase=pool_census_parse              ms=5817     scale=closure
phase=closure_name_census_build      ms=2623     scale=closure
phase=reference_pool_names_from_census ms=60     scale=closure
phase=closure_load                   ms=0
phase=closure_parse                  ms=0
phase=closure_resolve                ms=0
phase=closure_normalize              ms=0
phase=closure_typecheck              ms=0
phase=closure_parent_envs            ms=0
phase=closure_reconcile_assembly     ms=0
phase=closure_ownership              ms=0
phase=reconcile_root_bare_underlay   ms=0
phase=reconcile_root_variant_base    ms=0
phase=resolve_entry_graph_inclusive  ms=175352   scale=closure   <-- dominates
population pool_modules=refused closure_files=1436 demand_acquired_modules=1436
process_cpu user_ms=172002 sys_ms=3287 max_rss_kib=5682716   (~5.68 GiB peak)

v2 compiler phase receipts on the same run: compile.frontend 55s, compile.normalize 3s,
compile.reconcile 41s, compile.analyses 276ms.
Step timing: step start 20:51:09Z -> [pre-entry] printed 20:54:05Z (~176s outer pre-entry),
then child gunbc.WitnessBin.Run 20:54:05Z -> 21:17:38Z = 23 min (1,413s, the cold compile).

## What this tells us (drives the plan)
- The pre-serve restart cost is overwhelmingly **resolve+type of the closure**:
  `resolve_entry_graph_inclusive` = 175.4s of a ~176s outer pre-entry. Persisting that result and
  loading it at run start (this node) is the right lever; the win ceiling is the whole pre-entry
  (~11 min for the real 1629-module roadmap_ingress serve closure per the boundary).
- `closure_typecheck ms=0` and resolve+type are BOTH inside `resolve_entry_graph_inclusive`:
  the CI logs **cannot** separate resolve vs type. The only tool that does is `[phase-profile]`
  (`GUNBC_FLOOR_PHASE_PROFILE=1` -> FloorPhase{Discovery,Resolve,Typecheck,Eval,HostEffect,Gate}),
  which was NEVER armed in CI -> those rows do not exist in any log.
- MEMORY: only ONE `max_rss_kib` per invocation exists (combined image + eval heap).
  `[phase-profile]` carries no RSS field. So the node's own question — "how much of the resting
  memory is the image vs the evaluation heap" — is **unmeasured**; the memory half of the node
  cannot be sized from existing data.

## Therefore the ONE measurement (instrument row, manager to approve) must:
- arm `GUNBC_FLOOR_PHASE_PROFILE=1` -> resolve/type/eval time split, and
- add a per-phase RSS probe (VmHWM/VmRSS at phase-enter, esp. end-of-Typecheck and end-of-Eval)
  -> image-vs-eval-heap split (end-of-Typecheck RSS = the image; the delta through Eval = heap).
- run on BOTH consumers, one row: (a) MY serve run `gunbc run --entry dag/gunbc/roadmap/roadmap_ingress.dag
  --function roadmap_ingress --arg handler=... --arg listener=unix:/nonexistent/sock ...` (bind-refusal
  exit after pre-entry+evaluate), and (b) smart-badger-726's `gunbc compile ... --target dag` child.

---

## CLASSIFICATION (v1_seed_standing) + v2-native route — for the operator fork

Manager's clarified premise: **serve runs on `gunbc run`, i.e. the v1 seed interpreter.** So
"persist the resolved and typed program and load it at run start" is a change to the v1 seed, and
`gunbc.v1.v1_maintenance_standing::v1_seed_standing` governs it. Standing: admitted iff it
**serves the v2 self-host program** (purpose test, open shapes); five refused classes **dominate**,
incl. **SeedFeatureCompletion** ("the seed FAILING TO DO something it never did"; "capability added
to the seed for its own sake supports v1 rather than v2"). Discriminating receipt on point: a lane
to author missing OperatorSpec/TypeCheckpoint rows so the **v1 emitter** could render constructs it
had no row for was refused as **SeedFeatureCompletion** — "makes the seed more capable without
advancing the v2 frontier."

### Route A — seed-interpreter persist/load (what the node words literally imply)
Add an on-disk program-image format + a load path in the v1 seed's `gunbc run` so the interpreted
serve reloads a stored resolved_graph+typed_module and skips resolve+type.
**CLASSIFICATION: REFUSED as SeedFeatureCompletion.** The seed has never persisted/reloaded a
compiled program; giving it that capability is the seed doing something it never did, and it makes
the v1 *interpreter* serve faster — i.e. it strengthens v1, not v2. It matches the TypeCheckpoint
refusal's shape. This is the manager's reading; I concur.

### Route B — v2-native: native emitted CLI serving the entry, image = provider realization
**A v2-native route EXISTS and is the sanctioned one.** The served entry is a `NativeServeDriver`
(`gunbc.native_serve`, `std.compiler_entry NativeServeDriver`): the seed emits a v2 closure, cargo
builds the emitted crate, and the **compiled binary** serves requests — rung_drop
`native_serve_request_budget_unrealized` is explicit: the native handler "runs its handler as
compiled code: there is no evaluator to interrupt," and its restart is "the RESTARTED process
(systemd Restart=on-failure)." Running an already-built binary **skips resolve+type by construction**
— they happened at build time, not start time. The "stored program image" is the **emitted-crate
product**, which the ladder already makes a materialization realization (rule 1: `emitted_crate`
artifact kind), and the provider already declares `emitted_crate_artifact_kind` /
`output_emitted_crate` keyed by identity incl. `compiler_digest` + closure digests.

**Precedent + near-implementation already in-tree:**
- `src/v1/stage0/src/cli_run/native_product_cache.rs` IS this node's mechanism, already built:
  it keys the product (emitted executable + build receipt) by the six causal axes (producer
  compiler, source closure, target model, toolchain, build configuration, lens contract) =
  entry/closure/compiler digest; on a read it re-hashes and **REFUSES a disagreement and rebuilds —
  "it never serves the entry"** (= red_control #3); a miss costs one rebuild (= red_control #2).
  Its own doc says it is "NOT `std.materialization_provider` `request_key`" **for a stated
  divergence reason**, and is **"dissolved into `request_key` ... when the native ancestry lane
  makes `provider_serve` the only producer (`native_product_cache_seed_growth`)"**. So the sanctioned
  end-state is exactly the brief's "key on complete identity via the provider, not a second cache."
- `dag/gunbc/native_serve_program_seed_growth.dag` = a `SeedGrowthJustification` admitting
  hand-authored seed declarations in `v1_compiler.cli_run.native_lane_runner` /
  `target_invocation_host` (`run_native_serve_program`, `native_serve_start`, …) because they host /
  launch the **v2-native serve program** → admitted under the purpose test (SupportsV2SelfHostProgram),
  not refused.

**CLASSIFICATION for Route B: ADMITTED as SupportsV2SelfHostProgram.** The served program is the v2
self-host program, the image is a provider realization, and the work advances the v2-native-cli /
native-ancestry frontier (the provider_serve dissolution the native_product_cache row already
anticipates). Any seed glue needed would ride a `SeedGrowthJustification`, as native_serve already did.

### Recommendation
Do NOT build Route A (refused class). Build **Route B**: make the served entry a `NativeServeDriver`
whose compiled emitted product is the program image, reused across restarts via the provider
(realize the `native_product_cache` -> `provider_serve`/`request_key` dissolution so it is not a
second cache), restart runs the built binary, and the provider's verified-read gives the
corruption-refusal + miss-recompute semantics. Red_controls #2/#3 are already embodied in
native_product_cache; #1 (same responses as a cold start) is shown by the native-serve instrument.

### Dependencies / caveats the fork must weigh
1. srv1's serve is currently the v1 interpreted `gunbc run` (manager). Moving it onto the native
   route is an **srv1 unit move — bright-ibex-807 owns srv1 lab/prod units and the prod handler
   fix adhoc-557579da-286**; it must go through that owner, not me.
2. Native-serve maturity for the roadmap_ingress serve entry specifically (budget wall is RETIRED,
   but per-entry readiness for the daily-workspace handler is open).
3. The provider not yet serving the native artifact (the divergence native_product_cache records)
   is the precise seam this node would discharge.
