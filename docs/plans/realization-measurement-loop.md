# Plan — Realization & Measurement: closing the cost → schedule loop (+ infra onto `.dag`)

**Status:** stable coordination plan · dispatch workers per phase · **DESIGN.md + the carriers remain the
authority** — this doc is a worker-dispatch tracker, not a fact ledger, and each phase dissolves into a
mark on its carrier when it lands (DESIGN §6 "no parallel-ledger docs"). A phase's real state is its
branch/PR, not this file (mirrors `ROADMAP.md`).

**Owner:** `zesty-deer-479` (CI profiling) coordinates; phases dispatch to children.

---

## 0. Thesis — the substrate is shape-complete but input-starved

The realization layer is *fully modeled and almost entirely inert*: every carrier that should drive
"minimal work, as parallel as possible, guaranteed" exists with the right type and is read by **no
scheduler, fed by **no measurement, consulted by **no router. This is DESIGN §6's failure mode — *"the
tier where the machinery exists but nothing gates on it"* — landed squarely on §1's **cost axis**.

The fix is **one control loop**, not a pile of local patches (this is why §4 "evaluate the whole
pipeline" is load-bearing, not a footnote):

```
  measure (Phase 0)                    ← the keystone; makes cost a real fact
    → CostAccount.time = Measured       (today: cost_account_predicted_zero())
      → RealizationObjective Pareto over {time, space, power, $}
         bounded by a deployment HardwareBudget (Phase 1 / Phase 3)
        → Schedule width + shard balance + cache-layer choice
          → total rolled up along the CRITICAL PATH, not the node sum (Phase 1.5)
```

The three notes from the operator are **not peers — they are a dependency chain rooted at measurement**:
`minimal work` = §2 (cache planner) and `as parallel as possible` = §1 (width) can only be *optimized
against a measured cost*, and `guaranteed` = §5 (measured, not predicted). Cost must become a measured
fact before anything can plan against it. The independent CI audit reached the same conclusion ("add the
two-clock timing first"); the dependency structure above is *why*.

### The invariant that governs every phase (DESIGN §3 / §4)

> **Pure schedule TOPOLOGY stays central and deterministic; every measurement-fed decision is a
> PERIPHERAL realization.**

The schedule (`Schedule = List<List<Runnable>>`) is a deterministic partial order — it expresses what
*can* run in parallel, hardware-free, and is pinned by the `schedule_eq` /
`schedule_generates_identical_schedule` witnesses (`realization_schedule.dag:222-236`;
`src/v2/test/claim/realization_schedule_witness.dag:35-48`). A schedule whose *width or membership*
depended on a measured per-machine `HardwareThreadCount` would break that equality across hosts. So:
**measured ⇒ peripheral, by construction.** Width, shard balance, cache-layer choice, and the timing tap
all live in the periphery (`extdeps/realization`, the host realizer), checked by *"given these
DeploymentFacts + these measured costs, the realizer picks X"* witnesses — deterministic *given their
inputs*. This is exactly what the `CostAccount` dissolve-on comment anticipates
(`realization_schedule.dag:25-26`). Most `.dag` programs are stateless/short-lived: they declare
dependency structure and never mention hardware; the width-fold degenerates to "just run it."

### Realization has THREE dimensions (schedule · cache · compute-fabric)

"Realization" is one concept with three orthogonal-but-composed dimensions, **all keyed on the same
content-addressed `ExecutionReceipt`**, all **peripheral** (§3), all governed by the **same measured cost
+ Pareto objective**:

| Dimension | Carrier | Asks | §1 axis |
|---|---|---|---|
| **schedule** | `std.realization_schedule` | *when / how parallel* | parallel |
| **cache** | `std.cache_interface` | *whether it needs to run at all* | §2 minimal work |
| **compute fabric** | `product.compute_fabric` | *where / on what it runs* (provision) | placement / $ |

The proof they are dimensions of *one* concept rather than three separate models is **structural and
already in the tree**: cache `MUST NOT import compute_fabric` (`cache_interface.dag:4`) and compute_fabric
`MUST NOT import cache_interface` (`compute_fabric.dag:9`); **both compose only over `ExecutionReceipt`**.
They meet *only* at the content-addressed receipt — never by direct coupling (§3/§4).

**The `compute_fabric()` zero-arg interface is the same §3 move as the whole plan.** `compute_fabric` is a
**demand → placement → supply** market: an execution emits a `WorkDemand` (isolation, memory, GPU) and
`satisfies(offer, demand)` (`compute_fabric.dag:377`) places it on a satisfying `ComputeOffer`. "Provision
compute for this execution" with **empty args** = the `WorkDemand` is *inferred from the .dag-modeled
execution*, and placement is *resolved from the deployment's offers* — **neither is a caller argument**.
Identical to the cache planner auto-wiring and the scheduler's hardware-budget fold: *caller declares
intent; the realization is resolved from modeled deployment facts.* All three dimensions share this.

**Layering discipline (§3):** `compute_fabric` is **product**; schedule/cache are **std** (imports point
toward std). Split: std owns the *budget shape* (`std.placement_supply` ✓) and the `{time,space,power}`
cost axis; product `compute_fabric` owns the **$/billing axis** (`CostClass`, `MoneyMicros`) and *projects*
fleet rows into the std shape (`placement_supply_row`, `compute_fabric.dag:220`); the **peripheral host
realizer** bridges product→std and runs the Pareto pick. std stays product-free.

---

## Verified ground truth (file:line — do not re-derive)

| Fact | Evidence |
|---|---|
| Schedule width = full dependency frontier, **no capacity bound** | `ReadinessLayer.ready: List<Node>` `src/v2/workflow/scheduler.dag:29`; `runnable_frontier_from_dependencies` `scheduler.dag:147-154` |
| Thread-spawn **unbounded by hardware** (one OS thread per runnable-in-batch) | `thread::spawn(... run_one_runnable ...)` `src/v1/stage0/src/bin/claim_executor.rs:403-410` |
| Corpus = **one** `RunnableDiscoveryBatch` node → serial `for` loop (293 witnesses invisible) | `ci_floor_plan.dag:107`; `run_discovery_corpus` serial loop `cli_run.rs:2148-2171` |
| `CostAccount.total` is hardcoded zero | `cost_account_predicted_zero()` `realization_schedule.dag:33-39`; `CostBasis/Measured` exist **only** in the `:25-26` dissolve-on comment |
| Single eval seam exists; per-variant self-time tap already there, **not** content-hash-keyed, **not** fed to CostAccount | `eval_expr` `v1_interpreter.rs:1451`; profiler `:1466-1476` (keys by `expr_variant_index`) |
| Per-witness/per-resolve wall-clock timing **exists but only goes to stderr** | `claim_batch.rs:372,404`; the floor runner `claim_executor.rs` does no timing; `DiscoverySummary` keeps only `{total,passed,failures}` `cli_run.rs ~1903` |
| Cache catalog has **zero runtime consumers**; `CacheLayerPlan` unused | `cache_interface.dag:91`; importers are comments only (`realization.dag:16`, `02_parse.dag:162-168`) |
| 3 live caches hand-wired, **3 different key derivations** | `resolved_graph_cache.rs:76-199` (content-hash); `ParseTableMemo` `v1_interpreter.rs:768,1116` (in-proc map); `pure_call_memo` keys by **address** `(usize,Vec<usize>)` not hash `v1_interpreter.rs:748-752`; sccache external |
| Carrier for hardware budget **already exists** | `HardwareThreadCount = Measure<Count,One,Nat>` `measure.dag:254`; `PlacementSupplyRow.hardware_threads` `placement_supply.dag:29-31`; `CpuDeploymentFacts` `cpu/types.dag:42` |
| Pareto kernel + objective vocabulary already shared | `std.pareto` dominance + `AxisGoal`; `RealizationObjective.goals: List<AxisGoal>` `realization_schedule.dag:41` |
| Runner model exists; ci.yml **is generated + byte-drift-gated** | `RunnerSpec = HostedRunner \| SelfHosted{labels}` `extdeps/github/actions.dag:233` (emission consumer is a future trigger, `:226-232`); generator = `expected_ci_yml()` `dsl/gunbc/ci_yaml_emit.dag:9`, drift gate `dsl/tools/ci_yaml_gate.dag` (byte-for-byte), parse-validate `dsl/gunbc/ci_yaml_validate.dag`. **NOT** `dsl/tools/gunbc_ci.dag` (a 732B wrapper). ⚠️ `extdeps/github/ci.dag:7` has a **stale** comment "ci.yml is hand-edited, not generated" — contradicts the live gate; ignore it (track for deletion). `runs-on: [self-hosted,linux,arm64]` is emitted as a **literal** today — the seam Phase 3a routes through `RunnerSpec` |
| A 3rd (richest) measured-time carrier already exists | `compute_fabric.PerformanceReceipt{wall_duration:Duration, sample_count, confidence: MeasurementConfidence = SingleSample\|Range\|DistributionSummary}` `:414-423`; on `ExecutionReceipt.performance :453` + `ComputeSupplyFacts.observed_performance :166` |
| `ExecutionReceipt` digest is **itself a stub** (not yet a content hash) | `execution_receipt_digest` returns `receipt.work.id` (a hand-authored brand) `:588-594` — "full canonical encoding supplied by the harness when Outcome/ArtifactRef inhabit" |
| Corpus sharding already has a demand-model home | `ParallelismShape = ... \| IndependentShards{shard_count} \| PartitionedReduce{...}` `compute_fabric.dag:280-289`; `WorkDemand.parallelism :273` |
| srv1/srv2 concrete facts are **ctrl-tier (private), not public gunbc** | `hardware_selection.dag:8`; `compute_fabric.dag:5` ("ctrl `plans.fabric.operator_fleet`") |
| compute fabric = demand→placement→supply market; provisioning is `satisfies(offer, demand)`; composes over `ExecutionReceipt` | `product.compute_fabric` — `WorkDemand` `:268`, `ComputeOffer`/`satisfies` `:377`, `ExecutionReceipt<T>` `:449`, projects `PlacementSupplyRow` `:220`; `satisfies` checks **isolation only** today (no thread-count/Pareto dim) |
| map→reduce execution decomposition is forward-stubbed (`= Node`) — anticipates sharding | `WorkGraph`/`Partitioner`/`Reducer`/`SymbolicCost`/`EffectBoundary` `compute_fabric.dag:48-58` |

---

## Phase 0 — Measured cost (THE KEYSTONE)

**Goal:** flip `CostAccount.time` from `predicted_zero` to `Measured`. Everything quantitative depends on
this. NB: the timing *primitives already exist* — this is **"wire the existing tap to a content-hash key
+ CostAccount,"** not "build timing from scratch."

- **Grain decision (settled this session): key on the content-hash of the cache SUBJECT, not per-AST-node,
  not per-variant.** Rationale: the `Node` is an `Rc<Node>` with **no ambient content-hash** (today the
  profiler keys by `expr_variant_index`), so per-node hashing at `eval_expr` is *not free* and would
  perturb the measurement (§5 observer effect). The cache-subject frame is exactly where a content-hash
  *is already computed*, and it is the grain the scheduler (per-Runnable) and the cache router
  (per-subject recompute cost) both consume — so per-subject is the §2 "model once, derive every use"
  point: per-Runnable is its aggregate, per-variant its `group-by`, full per-node an **opt-in deep
  projection** over the same seam. Per-variant counters are a *dead end* (a variant is not a cacheable
  identity → can't feed Phase 2).
- **Instrumentation = the dual of the cache.** Both wrap the same fold boundary keyed by the same
  content-hash: the cache stores node→value, the instrument stores node→duration. The *analysis* is a pure
  lens; the measurement itself is a §1 host-effect. Use the existing gross−children self-time decomposition
  (`v1_interpreter.rs:1466-1476`).
- **The measurement output is a `PerformanceReceipt`, NOT a new struct (§3 — do not coin a 4th).** The
  observation carrier already exists: `compute_fabric.PerformanceReceipt{ wall_duration, sample_count,
  confidence }` (`compute_fabric.dag:414-423`), and `ExecutionReceipt.performance` (`:453`) already hangs it
  on the receipt. So Phase 0 emits a `PerformanceReceipt` keyed by the cache-subject hash → it rolls up into
  `CostAccount.time` (the aggregate). This is what *fuses the keystone to the compute-fabric deliverable*:
  the thing Phase 0 measures **is** the fabric's execution receipt, and `MeasurementConfidence`
  (SingleSample/Range/DistributionSummary) is exactly the per-witness-distribution honesty the audit's
  observability gap needs.
- **Feed `CostAccount`.** Add `CostBasis = Measured | Predicted` (the `:25-26` dissolve-on); aggregate
  PerformanceReceipts → `RealizationPlan.total`.
- **Converge the time authorities — there are THREE, not two (§3).** `NanosecondDuration` (`measure.dag:258`)
  is the authority. `TestNodeCostDimension.measured_ms` (Milliseconds, `verification.dag`) **dissolves**
  into a budget-*predicate* over the measured `CostAccount.time` (a ns threshold), not a parallel measured
  field (ms truncates a 5 ns step to 0 — DESIGN §6). And `PerformanceReceipt.wall_duration` is the
  *observation* feeding the roll-up — observation vs roll-up, not a fork. Net: one observation carrier
  (`PerformanceReceipt`), one roll-up (`CostAccount.time`), `measured_ms` deleted.
- **Persist the dropped timing.** Carry per-witness/per-entry timing through `DiscoverySummary` and the
  floor path (`claim_executor`), closing the audit's observability gap (the "two-clock" per-entry-resolve
  vs aggregate-per-witness split) — which is *itself* the profiling the operator was about to do by hand,
  made first-class.
- **§5 gate:** purity oracle — witness verdicts byte-identical instrumented vs not.
- **Dissolution trigger:** `cost_account_predicted_zero()` has no callers on the floor path; `measured_ms`
  deleted.

---

## Phase 1 — Make parallelism visible + hardware-bounded width  (note 1)

**Depends on:** Phase 0 (shard balance needs measured per-shard cost).

- **Explode the corpus node** (`ci_floor_plan.dag:107`) into per-entry-group `Runnable`s — **rows, not
  Rust**. Express it as a `WorkDemand` whose `parallelism = IndependentShards{shard_count}`
  (`compute_fabric.dag:282`) — the corpus is today a *degenerate (unsharded) projection* of exactly that,
  so this reuses the carrier rather than minting a scheduler-local shard list. Shard *by whole entry-group*
  to preserve the warm `typed_module_cache` (splitting an entry's witnesses re-incurs its cold resolve).
  This is the keystone both background audits independently named.
- **The hardware budget is what `compute_fabric` PROVISIONS — do not coin a fresh carrier (§2/§3).** The
  width-fold reads the `PlacementSupplyRow` that `compute_fabric` already projects (`placement_supply_row`,
  `compute_fabric.dag:220`; `PlacementSupplyRow.hardware_threads` `placement_supply.dag:29-31`). **The
  width decision *is* a placement decision:** today `satisfies(offer, demand)` (`compute_fabric.dag:377`)
  checks *isolation only* — Phase 1 is "add the thread-count / Pareto dimension to placement." So this
  phase is co-designed with Phase 3 (compute_fabric is the provisioning interface; the runner *declares*
  the offer, the scheduler *consumes* the placement — one market, two readers). DFS `PlacementSupplyRow` +
  `CpuDeploymentFacts` (`cpu/types.dag:42`) before extending; do **not** invent `DeploymentFacts`.
- **The width-fold.** Schedule stays the pure partial order; a peripheral realizer folds it against the
  `HardwareBudget` → actual width, a Pareto pick over `{wall-time↓, peak-memory↓}` bounded by `width ≤
  cores` — reusing `std.pareto` + `RealizationObjective.goals` (already `List<AxisGoal>` — wiring carriers
  that already speak the same vocabulary, not inventing one).
- **Bound the spawner** (`claim_executor.rs:403-410`) by `available_parallelism`/budget — fixes the *dual*
  hazard: under-parallel (serial corpus) **and** over-parallel (unbounded spawn on a wide batch).
- **Determinism:** schedule topology stays witness-stable; width/shard are peripheral, checked by
  given-these-facts witnesses.
- **Dissolution trigger:** the corpus is ≥2 runnables in the schedule; the spawner reads a budget.

---

## Phase 1.5 — Holistic cleanups (note 4)  ·  mostly parallel to 1

- **Wire `affected_set` into the corpus + gate roster** (audit lever #1). The compile-time, fail-closed
  `v2.lens.affected_set` + runner + superset proof exist and are green but unimported. Skip only on a
  content-addressed match to a verified-green baseline; **fail closed** on any miss. Biggest wall-clock
  lever on PR branches. (Independent of Phase 0.)
- **`total` = measured critical-path roll-up** (note 4): weights a short phase *on the critical path*
  (the perturb gate) at full cost and a short phase *hidden under a parallel branch* at ~0. (Needs Phase 0.)
- **Collapse the perturb cold-recompile** (~14 s, fail-fast root, critical path): resolve the planted
  (fresh-name, unimported) module against the green compile's index instead of `cp -r dsl` + cold rebuild;
  preserves the discriminating-red (§5). (Independent of Phase 0.)

---

## Phase 2 — Cache-layer planner: the catalog's first consumer  (note 2)

**Depends on:** Phase 0 (cost-aware reach needs measured recompute cost).

- **Land the planner the catalog header already promises** (`cache_interface.dag:21` "downstream cache
  lookup planner"; dissolve-on-arrival "when first consumer lands"). Given the `CacheInterfaceFacts` rows +
  a deployment's *available* backends, emit a `CacheLayerPlan` (type exists, `:91`): L1 in-process → L2
  sccache / cargo target / resolved-graph → L3 CAS.
- **One content-address lookup kernel, N backends bound by `transport_encoding`** (§4 one-kernel/N-handlers).
  The in-process memo is just the `InProcess` row of the *same* kernel. **Converge the key derivations onto
  ONE content-addressed subject identity.** Today there are *four* keyings: `parse_cache` by path,
  `typed_module_cache` by module-name, `resolved_graph` by content-hash, `pure_call_memo` by address — **and
  the `ExecutionReceipt` digest the whole three-dimension unification rests on is itself a 4th stub**:
  `execution_receipt_digest` returns `receipt.work.id`, a hand-authored brand, not a content hash
  (`compute_fabric.dag:588-594`). The §0 thesis ("all three dimensions keyed on one `ExecutionReceipt`")
  is only *true once this digest becomes a real content hash aligned with the cache-subject identity* — so
  this convergence is shared work between Phase 2 (cache) and the compute-fabric track, not cache-local.
  The `key_derivation` axis exists for exactly this.
- **Cost-aware reach (§1).** Reach for a layer only when its predicted delta favors it:
  `min(recompute, lookup + p(miss)·recompute)` — `read_latency` (InProcessNs…WanTensMs) is already a
  modeled axis; `recompute` comes from Phase 0. A WAN CAS hit can lose to recomputing a cheap node.
- **Auto-wire (the operator's "user shouldn't wire it themselves").** The deployment declares which
  backends it offers; the planner emits the wiring — dissolving the bespoke `if sccache --show-stats`
  shell in `ci.yml` and the `GUNBC_RESOLVED_GRAPH_CACHE_DIR` env read.
- **Fail-closed (§5).** A backend probe that is absent/unauthorized falls through to the next layer →
  ultimately recompute, **never** serves a stale/unverified hit (`CacheRejectReason` already enumerates
  `BackendUnavailable`/`BackendUnauthorized`). Verdict-identity, not byte-identity, until the deferred
  canonical-byte compare lands — so do NOT route emit/lower consumers through it yet.
- **First free measurement:** un-dormant `resolved_graph_cache` in CI (read by the seed, unset in
  `ci.yml`) as a warm A/B.
- **Dissolution trigger:** `ci.yml` carries no hand-wired cache shell; one lookup kernel; one subject key.

---

## Phase 3 — Infra / deployment onto `.dag` (srv1 / srv2)  ·  the second ask

**Goal:** model the actual infra/deployment — runner config, access, host config — as **data**, dissolving
bespoke `ci.yml` shell + manual runner setup. **"Get CI onto `.dag`" = instantiate `compute_fabric` for
srv1/srv2:** a self-hosted runner is one `ComputeOffer`; GHA-hosted, `Ubicloud`, `GcloudRun` are others
(already enumerated, `compute_fabric.dag:71`). This meets note 1 at the placement carrier — the offer the
runner *declares* is the budget Phase 1's scheduler *consumes*.

**§3 boundary (decisive — and already declared by `compute_fabric`'s own header):**

- **3a — public gunbc substrate (this repo).** The *provisioning schema* is `product.compute_fabric`
  (already exists: `ComputeOffer`/`WorkDemand`/`satisfies`/`ExecutionReceipt`). Extend `RunnerSpec`
  (`actions.dag:233` — `SelfHosted{labels}` is stringly today; its dissolution note calls for a "typed
  runner-label substrate … where the compiler owns runner topology") so a runner spec is the GHA
  *realization* of a `compute_fabric` offer/demand (runner labels ≈ the demand's isolation/locality). ci.yml
  **is already generated and byte-drift-gated** — generator `expected_ci_yml()`
  (`dsl/gunbc/ci_yaml_emit.dag:9`), gate `dsl/tools/ci_yaml_gate.dag`, validate
  `dsl/gunbc/ci_yaml_validate.dag` — so this is **extend existing generation, not build new**. The concrete
  seam (start-now, measurement-free): `ci_yaml_emit.dag` emits `runs-on: [self-hosted,linux,arm64]` as a
  **literal** today; route it through `RunnerSpec` derived from a `compute_fabric` `ComputeOffer` (the
  carrier's own dissolution trigger, `actions.dag:226-232`, names "gunbc/ci_emission.dag projection for
  emitted workflow YAML"). DFS before minting: `compute_fabric` first, then `extdeps/github/actions.dag`,
  `extdeps/os/{ubuntu,…}`, `extdeps/container`, `extdeps/docker`, `extdeps/cloud`, `dsl/std/os.dag`.
  ⚠️ Ignore the stale `extdeps/github/ci.dag:7` "hand-edited, not generated" comment (track for deletion).
- **3b — private ctrl instantiation (separate, NOT this PR).** The *concrete* srv1/srv2 facts — access
  creds, runner registration tokens, the exact SKUs each host runs, host provisioning — are **ctrl-tier
  private**, in **ctrl `plans.fabric.operator_fleet`** (named by `compute_fabric.dag:5`;
  `hardware_selection.dag:8`; memory `idea-pr-compiler-ctrl-boundary`). They are concrete `ComputeOffer`
  rows instantiated *against* the 3a public fabric schema. Out of scope for the public plan except to fix
  the seam 3a must expose.

**Dissolution trigger:** `.github/workflows/ci.yml` is emitted from a `.dag` runner+floor spec (no
hand-authored runner block); the `HardwareBudget` the scheduler reads is the one the runner spec declares.

---

## Sequencing & dispatch

```
        ┌─────────────── Phase 0 (keystone: measured cost) ───────────────┐
        │                                                                  │
   Phase 1.5a (affected_set)   Phase 1 (visible parallelism      Phase 2 (cache planner)
   Phase 1.5c (perturb)        + hardware-bounded width)          │
   — independent of 0          │                                  │
        │                  Phase 1.5b (critical-path total) ←──────┘ (needs 0)
        │
   Phase 3a (runner schema + ci.yml gen)  — HardwareBudget carrier co-designed with Phase 1
        │
   Phase 3b (ctrl: srv1/srv2 concrete) — private, separate repo
```

| Phase | Depends on | Intricacy / Volume | Notes |
|---|---|---|---|
| 0 keystone | — | high / medium | root of everything quantitative; load-bearing seam (`v1_interpreter.rs`, escalate per DESIGN) |
| 1 visible parallelism + width | 0 | high / medium | shares `HardwareBudget` carrier with 3a |
| 1.5a affected_set | — | medium / small | fail-closed; can start now |
| 1.5b critical-path total | 0 | medium / small | |
| 1.5c perturb collapse | — | medium / small | can start now |
| 2 cache planner | 0 | high / large | converges 3 key derivations; first catalog consumer |
| 3a runner schema + ci.yml gen | (1 for budget carrier) | high / large | public; extends `CiFloorSpec` emission |
| 3b ctrl srv1/srv2 | 3a | — | **private `~/ctrl`**, separate PR |

**Start-now lanes (no measurement dependency):** 1.5a (affected_set), 1.5c (perturb), and 3a schema
groundwork. **Critical path:** Phase 0 → 1 / 2.

---

## Guardrails for every worker (DESIGN)

- **Pure central spec, peripheral measured realization** (the §3 invariant above). Anything measured is
  peripheral — never in the schedule topology the `schedule_eq` witnesses pin.
- **DFS existing carriers before minting** (§2/§3): `PlacementSupplyRow`, `CpuDeploymentFacts`,
  `RunnerSpec`, the cache catalog, `std.pareto`, `RealizationObjective.goals`. The vocabulary mostly
  exists — wire it, don't re-coin it. (Background verification specifically flagged: do not invent a fresh
  `DeploymentFacts`.)
- **Rows, not Rust** — express new schedule/cache/runner facts as data; do not cement the seed to satisfy
  a ratchet (the ratchet is downstream of substrate migration).
- **Fail-closed (§5)** + **purity oracle** on every memo/measurement (byte-identical warm-vs-cold; verdict
  identical instrumented-vs-not). A bounded "forever" ≠ an "unknown" error.
- **Green by execution, not spec-without-execution.** A real consumer runs green + a discriminating input
  goes red. Typecheck + grep are not consumers.
- **Each phase lands with a named dissolution trigger; the mark is on the carrier.** This doc is
  coordination only; delete its entry when the carrier records the fact.
- **Load-bearing files** (DESIGN-named pipeline stages, substrate types, gates: `v1_interpreter.rs`,
  `cli_run.rs`, the scheduler, `realization_schedule.dag`) carry a higher bar — escalate before touching
  under a brief that pre-dates the relevant model PR.

## Open decisions to confirm before scoping the affected phase

1. **Placement/budget carrier** — extend `compute_fabric`'s `satisfies` + `PlacementSupplyRow` with the
   thread-count/Pareto dimension (the width decision = a placement decision), vs a sibling carrier? (DFS
   task in Phase 1 / 3a.) Do not invent `DeploymentFacts`.
2. **The $/billing cost axis crosses the std↔product line.** std `CostAccount` carries `{time,space,power}`;
   the $ axis (`CostClass`, `MoneyMicros`) lives in product `compute_fabric`. Confirm: the std schedule's
   objective stays `{time,space,power}`; the **peripheral host realizer** combines it with the product $
   axis for the full Pareto pick. (Keeps std product-free.)
3. **Cache planner timing** — the *plan* (which layers, what order) can be compile-time data; only the
   *availability probe* is runtime/peripheral. Confirm the split when scoping Phase 2.
4. **Runner-label vocabulary** — how far to close `SelfHosted{labels: List<String>}` toward a typed
   topology / a `compute_fabric` offer (3a) without over-fitting to srv1/srv2 (which live in ctrl).
5. **Sharding lives in the demand model — Phase 1 and 3 meet at the parallelism shape, not just the
   budget.** `ParallelismShape = ... | IndependentShards{shard_count} | PartitionedReduce{...}`
   (`compute_fabric.dag:280-289`) already exists, and the corpus `RunnableDiscoveryBatch` is a **degenerate
   (unsharded) projection of a `WorkDemand` whose `parallelism = IndependentShards`** (independent
   witnesses). So the easy case needs no new stubs; the `Partitioner`/`Reducer`/`SymbolicCost = Node`
   forwards (`:48-58`) are only for the harder `PartitionedReduce` (whole-tree compile → reduce verdicts).
   Decide: does Phase 1 express the corpus as an `IndependentShards` `WorkDemand` (preferred — reuses the
   carrier) vs a scheduler-local shard list?
