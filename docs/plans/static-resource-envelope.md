# Static resource envelope — N-dimensional compile-time width for CI floor

> **Status: DESIGN — operator sign-off required before carrier or load-bearing edits land.**
> Dispatched from `vivid-carp-243` (OOM kill on CI) → `merry-koi-140` (PR #6062, closed without its §7 decisions ever being answered).
> **Revived 2026-07-02 (crisp-wren-74, work-item `floor-width-budget-source-a`).** A sibling brief drafted a parallel `bounded-input-cost-envelope-scheduling.md` framing with invented types (`InputProcess` / `FloorResourceEnvelope` / `RunnableDemand` / `CostEnvelope`) that fork this doc's `InputEnvelope` / `ResourceEnvelope` / `RunnableResourceProfile` / `CostAccount` (DESIGN §2/§3 nickname violation). That framing is retired; **this doc is the grounded original and stays the single authority** — §9 folds its live deltas in as *additions*, not new types. Census in §4 re-verified against current `origin/main` post `dsl→dag` rename (§8).
> Composes with `merry-crane-716` (affected-set → live scheduler): affected-set **prunes** the graph this envelope fold runs over; smaller closure → smaller retained base → width can safely rise.
> DESIGN refs: §2 (one Cost record, every axis — horizontal reuse), §3 (single authority — memory bytes ≠ disk bytes; no nickname), §5 (construction-not-validation; kill the tautology witness), §6 (dissolution trigger on every scaffold).

---

## 1. Symptom and root cause (why width=1 is a stop-gap, not the fix)

CI floor OOM-kills `claim_executor` (exit 137) under the 8 GiB runner cgroup cap (`gunbc_ci_runner_cgroup_memory_cap`).

**Mechanism chain (landed analysis, PR #6059 stop-gap):**

1. `claim_executor` is **one long-lived process** holding the whole `dag` + `src/v2` resolved corpus resident across all batches.
2. Batch 4 host-effect gates (`EmitHostGate`, `RegenVerifyGate`, `EmitDeterminismGate`) spawn cargo/rustc children (~1.5 GiB each, `gunbc_ci_floor_host_compiler_spawn_peak`).
3. PR #5975 used `int_max(floor_width, exec_width)` → width 3 when floor=1 and exec=3.
4. Three concurrent compilers stacked on the retained corpus → breached 8 GiB → cgroup OOM.

**Stop-gap (merged/underway):** `int_max` → `int_min` in `gunbc_ci_plan_spawn_width` (PR #6059) forces width 1. Correct as emergency tourniquet; leaves ~124 idle cores and does not make width *derivable*.

**Root cause (model):** width and fit are decided from **stale measured byte scalars** and a **§5 tautology witness** — not from a compile-time fold over the dependency graph with per-Runnable demand vectors reconciled per axis against independent caps.

---

## 2. What's wrong with today's model

### 2.1 Single-axis memory scalar as authority

| Fact today | Home | Problem |
| --- | --- | --- |
| `gunbc_ci_floor_per_shard_peak_rss_bytes()` | `dag/gunbc/ci_floor_measurement.dag` | Measured scalar (max of samples); not derived from closure size or declared input envelope |
| `gunbc_ci_floor_executor_base_overhead_bytes()` | same | Single measured constant (~1.5 GiB); assumes invariant retained corpus across batches |
| `gunbc_ci_floor_execution_corpus_per_witness_rss` | same | Separate scalar for execution corpus; hand-set 2 GiB |
| `gunbc_ci_floor_host_compiler_spawn_peak` | same | Host-compiler child peak; not distinguished as a *different binding axis* from resolve-heavy shard |

Width formula today (`gunbc_ci_floor_memory_derived_width`):

```
usable = cap * 17/20 - executor_base_overhead
width  = min(cpu_bound, floor(usable / per_shard_max))
```

Only **memory** participates. Compute, storage, and energy caps are absent from the fold.

### 2.2 §5 tautology witness

`witness_corpus_effective_widths_fit_runner_cap` (`src/v2/workflow/ci_floor_plan.dag`) checks:

```
base + floor_cap_w * per_shard_scalar <= cap
base + exec_cap_w * exec_per_witness_scalar <= cap
```

Failures:

- **Re-states the guess:** `per_shard_scalar` and `base` are editable data rows independent of the realizer; satisfying the witness does not prove the *scheduled* width is safe.
- **Wrong width:** uses `gunbc_ci_floor_spawn_width_from_tree()` and `execution_corpus_spawn_width()` — not the width each batch actually runs at (host-effect gates in batch 4 are ignored).
- **Single axis:** no compute/storage/energy reconciliation.

### 2.3 Runtime decides width; no fail-closed fork gate

`claim_executor.rs` calls `eval_spawn_width()` → evaluates `gunbc_ci_plan_spawn_width(memory_budget_bytes)` from live cgroup read. Runtime **decides** width; there is no enforcement that `memory.current + declared_peak` would breach before `fork()`.

### 2.4 Runnable demand is memory-only booleans + scalar

`RunnableResourceProfile` (`std.realization_schedule.dag`) carries:

- `heavy_whole_tree_resolve: Bool`
- `spawns_host_compiler: Bool`
- `memory: RunnableMemoryNegligible | RunnableMemorySubstantial { predicted_peak: ByteSize }`

Host-compiler gate and resolve-heavy corpus shard are not modeled as **distinct demand vectors** on separate axes (memory-bound vs compute-bound).

---

## 3. The move — N-dimensional static envelope (correctness by construction)

### 3.1 Grounding unit: Cost-style record, every axis present

DESIGN §2: `Cost = Time|Space|Energy` → a record (every cost has all three).

For **scheduling admission** (distinct from `CostAccount` roll-up for Pareto/measurement loop), introduce a **static peak demand record** — one field per constraint axis, each a grounded `Measure<Quantity, Scale>`:

| Axis | Quantity / carrier | Cap source (runner) | Notes |
| --- | --- | --- | --- |
| **memory** | `ByteSize` = `Measure<Memory, One, Nat>` | cgroup `memory.max` (`gunbc_ci_runner_cgroup_memory_cap`) | Resident RSS peak per concurrent shard |
| **compute** | `HardwareThreadCount` or `Measure<Time, Nano, Nat>` (thread-seconds) | CPU quota / core count | Resolve-heavy shard is compute-bound; host-compiler gate is memory-bound |
| **storage** | `DiskSize` = `Measure<Storage, One, Nat>` | disk quota / tmpfs cap | **Not** a nickname for memory — separate `Quantity` (§8.3: `Storage` added to `std.measure.Quantity`, was fused onto `ByteSize`/`Memory` before) |
| **energy** | `Watt` = `Measure<Power, One, Nat>` (future joules) | fleet power budget (future) | Scaffold `Watt(0)` until instrumented |

**Reconciliation is per-axis, independent:**

```
width_memory  = floor((cap.memory  - base.memory)  / per_shard.memory)
width_compute = floor((cap.compute - base.compute) / per_shard.compute)
width_storage = floor((cap.storage - base.storage) / per_shard.storage)
width_energy  = floor((cap.energy   - base.energy)   / per_shard.energy)   // when live
final_width   = min(width_memory, width_compute, width_storage, width_energy, shard_demand)
```

Do **not** fuse axes even where both are byte-shaped (§3 — memory bytes ≠ disk bytes).

**Relationship to existing carriers (no nickname fork):**

| Existing | Role after migration |
| --- | --- |
| `CostAccount { time, space, power, basis }` | Measurement-loop roll-up; `space`/`power` project from envelope axes for falsifier comparison |
| `ResourceEnvelope` (`gunbc.fleet_container`) | Host **supply** shape (deployment); caps feed the reconciliation |
| `RunnableResourceProfile` | **Dissolves into** per-Runnable `StaticPeakDemand` vector (or extends it) |
| `InputEnvelope` (`gunbc.ci_input_envelope`) | Declared input sizes → symbolic derivation input for compile-time lens |

### 3.2 Per-Runnable demand vector

Each `Runnable` leaf declares `StaticPeakDemand` — per-shard peak **per axis**:

- **Discovery corpus shard (resolve-heavy):** high `memory`, moderate `compute`
- **Execution corpus shard:** lower `memory` per witness (`gunbc_ci_floor_execution_corpus_per_witness_rss` today), same compute
- **Host-compiler gate (`EmitHostGate`, etc.):** high `memory` (spawn peak), low `compute` (single child)
- **Negligible gates (rust fmt, layering):** near-zero on all axes

Demand must be **derived** from modeled input size + graph structure, not hand-editable independently of the realizer (§5).

### 3.3 Compile-time lens: graph fold → peak envelope

A **compile-time lens** (pure fold over `Node` / `DependencyView`, same substrate as `v2.lens.affected_set`) computes:

```
peak_envelope[axis] = base_vector[axis](retained_closure) + Σ concurrent_shard_vectors[axis](batch)
```

Where:

- **`base_vector`** = modeled retained closure size from the **resolved graph** (module count, node count, mock-key closure — not `gunbc_ci_floor_executor_base_overhead_bytes` constant). Composes with `merry-crane-716`: smaller affected-set closure → smaller `base_vector` → higher safe width.
- **`concurrent_shard_vectors`** = per-Runnable `StaticPeakDemand` × scheduled width for that batch.
- Output is a **verdict** per batch: `{ batch, scheduled_width, peak_envelope, fits: Bool, binding_axis: ResourceAxis }`.

Runtime **reads** this verdict only. Runtime **never decides** width.

### 3.4 Kill the tautology — new fit witness shape

Replace `witness_corpus_effective_widths_fit_runner_cap` with:

1. Binds **actually-scheduled width** per batch (including batch 4 host-effect gates).
2. Uses **derived** `base_vector` + **derived** per-shard demand (from input envelope + graph fold).
3. Checks **every axis** independently; witness goes RED if any axis over cap.
4. Discriminating input: shrink cap → witness RED; inflate stale scalar while scheduled width unchanged → witness RED (today it stays green).

### 3.5 Runtime fail-closed (residue only)

`claim_executor` enforces: refuse to `fork()` when `cgroup memory.current + runnable.declared_peak.memory > cap.memory` (and analogues for other live axes when instrumented). This is a **safety backstop**, not the width authority.

---

## 4. Census — current resource-scalar sites

### 4.1 Load-bearing (DO NOT EDIT until operator sign-off)

| Site | Scalar / pattern | Axis modeled | Consumer |
| --- | --- | --- | --- |
| `dag/gunbc/ci_floor_measurement.dag` | `gunbc_ci_floor_per_shard_peak_*`, `executor_base_overhead`, `execution_corpus_per_witness_rss`, `host_compiler_spawn_peak`, `memory_derived_width`, `conservative_fallback_width` | memory only | width derivation |
| `src/v2/workflow/ci_floor_plan.dag` | `gate_runnable_profile`, `gunbc_ci_plan_spawn_width`, `witness_corpus_effective_widths_fit_runner_cap`, all `witness_floor_*` | memory only | schedule + witnesses |
| `src/v1/stage0/src/bin/claim_executor.rs` | `eval_spawn_width`, `read_host_memory_budget_bytes`, `run_walk(..., spawn_width)`, per-shard calibration emit | memory only | runtime width + falsifier |
| `src/v1/stage0/src/cli_run.rs` | `spawn_width_cap`, discovery corpus width cap | memory only | per-corpus cap |

### 4.2 Supporting / parallel (migrate or project, not primary authority)

| Site | Pattern | Notes |
| --- | --- | --- |
| `dag/std/realization_schedule.dag` | `RunnableResourceProfile`, `RunnableMemoryClass` | Extend → `StaticPeakDemand`; dissolve memory-only class |
| `dag/std/realization_width.dag` | `memory_aware_spawn_width`, `process_memory_aware_spawn_width` | Formula correct; reads external scalar — becomes projection from envelope lens |
| `dag/gunbc/fleet_container.dag` | `ResourceEnvelope`, `parallel_run_demand_envelope` | Host supply; caps feed reconciliation. Already multi-field (`cpu`/`gpu`/`memory`/`storage`/`network`) but `storage.min_bytes` is typed `ByteSize` (`Measure<Memory,...>`) today — the exact memory/disk fusion §3.1 calls out; §8.3 grounds `DiskSize` separately, migration of this field is a P1-consumer follow-up, not part of this draft |
| `dag/gunbc/ci_input_envelope.dag` | `InputEnvelope`, `input_admitted` | Declared input sizes → derivation input (P2); §9.1 batch/streaming projection is a live addition to fold in here |
| `dag/gunbc/ci_budget_tree.dag` | INTER-run co-residence | Orthogonal granularity (§3 split already documented) |
| `dag/gunbc/ci_compile_jobs.dag` | BUILD-phase `process_memory_aware_spawn_width` | BUILD vs RUN phase split (compute-envelope-model.md) |
| `dag/test/claim/ci_floor_measurement_per_shard_test.dag` | Pins scalar values | Dissolves when derivation lands |
| `src/v2/test/claim/ci_floor_plan_witness_test.dag` | Imports tautology witness | Update when new witness lands |

### 4.3 Axes needed vs modeled today

| Axis | Modeled today? | Cap source today? | Per-shard demand today? | Gap |
| --- | --- | --- | --- | --- |
| memory | partial (scalar) | `gunbc_ci_runner_cgroup_memory_cap` | measured scalar | base not from closure; demand not derived |
| compute | no | `altra_max_m12830_catalog.threads` (cpu_bound only) | no | no thread-seconds or quota reconciliation |
| storage | no | none | no | not in model |
| energy | no | none | no | future |

---

## 5. Sequencing (after operator sign-off)

**P0 — this design doc + census** (this PR). Escalate for carrier shape approval.

**P1 — carrier in `std/` (model-only, no scheduler edits):**

- `StaticPeakDemand` record (4 axes, grounded Measures)
- `ResourceCap` record (supply, same shape)
- `envelope_reconciled`, `derive_spawn_width` (min over axes)
- Witnesses on synthetic fixtures; no load-bearing consumers yet

**P2 — compile-time lens (`v2.lens` or `std.realization_schedule`):**

- Fold `ci_floor_plan` dependency graph → `base_vector` from closure metadata
- Per-batch scheduled width + peak envelope verdict
- New fit witness; **delete** `witness_corpus_effective_widths_fit_runner_cap`

**P3 — wire scheduler (with `merry-crane-716`):**

- `ci_floor_plan.dag`: Runnable demand from derived envelope, not scalars
- `claim_executor.rs`: read compile-time verdict; fail-closed fork gate
- Retire `gunbc_ci_floor_executor_base_overhead` scaffold; measurement → falsifier only

**P4 — calibration loop** (resource-aware-scheduler Node D): `Measured` vs `Predicted` per axis.

---

## 6. Sibling coordination (`merry-crane-716`)

Shared surface: **`Runnable` roster shape** and which graph the envelope fold runs over.

| This work | Sibling |
| --- | --- |
| Envelope fold computes `base_vector(retained_closure)` | Affected-set prunes closure → smaller base |
| Per-Runnable `StaticPeakDemand` on schedule rows | Same `Runnable` variants; affected-set may skip rows |
| Compile-time width verdict per batch | Executor reads verdict instead of `eval_spawn_width` side-channel |

**Contract:** envelope lens runs on the **post-affected-set** dependency graph. Width rises safely when pruning shrinks the retained corpus.

---

## 7. Operator decision requested

Before any carrier or load-bearing edit:

1. **Approve axis set:** memory / compute / storage / energy — each `Measure<Q,S>`, reconciled independently?
2. **Approve carrier name/home:** `StaticPeakDemand` in `std.realization_schedule` vs new `std.static_resource_envelope`?
3. **Approve lens home:** extend `std.realization_schedule` schedule lens vs new `v2.lens.resource_envelope`?
4. **Approve P1 landing without P2** (model + witnesses only, scalars untouched) vs atomic P1+P2?
5. (§9 addition) **Approve provenance field placement:** on `InputBound` only, or also on `StaticPeakDemand` (this draft puts it on both — see §9.2)?
6. (§9 addition) **Approve the cost-model-single-authority direction:** `StaticPeakDemand` as the one magnitude carrier read by both the scheduler width-fold (P2/P3) and the complexity lens's asymptotic-class analysis (silent-ferret), rather than a second scheduler-local taxonomy?

Reply on dashboard or PR review.

---

## 8. Census re-verification (2026-07-02, post `dsl→dag` rename, `origin/main` @ `1abef0e9fd`)

All §4.1/§4.2 sites re-checked directly against the tree (not re-derived from the closed PR's stale `dsl/` paths):

### 8.1 §4.1 load-bearing — all present, paths moved `dsl/` → `dag/` (rename #6165), no other drift

- `dag/gunbc/ci_floor_measurement.dag`: `gunbc_ci_floor_per_shard_peak_rss_bytes`/`_max`/`_samples`, `gunbc_ci_floor_executor_base_overhead_bytes`, `gunbc_ci_floor_execution_corpus_per_witness_rss`, `gunbc_ci_floor_host_compiler_spawn_peak`, `gunbc_ci_floor_memory_derived_width`, `gunbc_ci_floor_conservative_fallback_width` — all present, memory-only.
- `src/v2/workflow/ci_floor_plan.dag`: `gate_runnable_profile`, `gunbc_ci_plan_spawn_width`, `witness_corpus_effective_widths_fit_runner_cap` — all present and unchanged in shape (still the tautology described in §2.2). Additional witnesses landed since (`witness_floor_width_governed_by_budget_tree`, `witness_floor_width_derived_from_tree_is_positive`) read `gunbc_ci_floor_spawn_width_from_tree()`/`srv1_floor_memory_budget()` (the budget-tree Track A, `docs/plans/budget-tree.md` — a *sibling*, INTER-run-pool-sizing concern, not the within-run N-axis reconciliation this doc targets; do not conflate).
- `src/v1/stage0/src/bin/claim_executor.rs`: `read_host_memory_budget_bytes` (:894), `eval_spawn_width` (:937), `run_walk` (:1311) — present, unchanged shape.
- `src/v1/stage0/src/cli_run.rs`: `spawn_width_cap` (:3476) — present.

### 8.2 §4.2 supporting — all present

- `dag/std/realization_schedule.dag`: `RunnableResourceProfile { heavy_whole_tree_resolve, spawns_host_compiler, memory: RunnableMemoryClass }`, `RunnableMemoryClass = RunnableMemoryNegligible | RunnableMemorySubstantial { peak }` — present, still memory-only, still Bool-flag (not vector) for the other two dimensions.
- `dag/std/realization_width.dag`: `memory_aware_spawn_width`, `process_memory_aware_spawn_width` — present.
- `dag/gunbc/fleet_container.dag`: `ResourceEnvelope { cpu, gpu, memory, storage, network }` — present, **already multi-field** (more structured than this doc's §3.1 table implied): `CpuRequirement{min_threads,architecture}`, `GpuRequirement{min_vram,runtimes}`, `MemoryRequirement{min_bytes}`, `StorageRequirement{min_bytes,persistence}`, `NetworkRequirement{...}`. **Drift note:** `StorageRequirement.min_bytes: ByteSize` — literally the memory/disk fusion §3.1 flags (`ByteSize = Measure<Memory,...>` used for disk bytes). This is *existing* debt in a landed carrier, not introduced by this design; §8.3 grounds a separate `Storage` `Quantity`/`DiskSize` in `std.measure` so `StaticPeakDemand.storage` does not repeat the fusion, and migrating `fleet_container.StorageRequirement` onto it is noted as a P1-consumer follow-up (not this draft's scope — it is a live consumer of `ResourceEnvelope`).
- `dag/gunbc/ci_input_envelope.dag`: `InputSizeAxis`, `InputBound`, `InputEnvelope`, `AdmissionVerdict`, `input_admitted`, `gunbc_ci_corpus_envelope` — present, P1 shape landed (#5801/#5902), P2 ceiling-derivation still open (`docs/plans/input-envelope-roadmap.md`).
- `dag/gunbc/ci_budget_tree.dag`, `dag/gunbc/ci_compile_jobs.dag` — present, unchanged role.
- `dag/test/claim/ci_floor_measurement_per_shard_test.dag`, `src/v2/test/claim/ci_floor_plan_witness_test.dag` — present.

### 8.3 New grounding needed (identified during re-verification, not in the original PR #6062 census)

`std.measure.Quantity` has no `Storage` variant — `type Quantity = Time | Length | Mass | Memory | Information | DataRate | Frequency | Count | Currency | Power | Temperature | RotationalSpeed | <electrical> | Dimensionless`. `Quantity` is a **phantom type-level tag only** (never pattern-matched as a value anywhere in the corpus — `measure_add`/`measure_le`/`measure_fit_count_floor` are all generic over `Q`), so adding `Storage` is additive and exhaustiveness-safe. §9's P1 draft (§10) adds it.

---

## 9. Live deltas folded in (2026-07-02) — additions, not new types

These three points were raised against a (retired) parallel framing but are genuine gaps in *this* doc's model. Each extends an existing carrier; none forks one.

### 9.1 Batch/streaming unification — `InputEnvelope` gets a second projection, not a sibling system

Today `InputEnvelope = BoundedInput { bounds: List<InputBound> } | EnvelopeUnknown` assumes a **finite, known-at-admission-time** corpus (the CI batch case). A streaming/incremental consumer (a long-lived watch process, an interactive session) has no single "actual count" to admit against — it has a *rate*. Rather than a second envelope type (the §2/§3 fork this doc exists to prevent), `InputEnvelope`'s `BoundedInput` variant should carry **two projections of the same declared-ceiling concept**:

- `FiniteKnown { bounds: List<InputBound> }` — today's shape, renamed as a variant of the bound-kind rather than the whole envelope.
- `RateBounded { rate: Measure<Count, S, Nat>, burst: Nat, window: Measure<Time, S, Nat>, backlog: Nat }` — same admission *question* (`input_admitted`), answered against a rate ceiling instead of a count ceiling.

`input_admitted` gains a case per projection; `RefusedOverEnvelope`/`RefusedUndeclared` are unchanged (the verdict vocabulary is projection-agnostic — it already only names *why* admission failed, not *how* the bound was shaped). **This is P1-adjacent, not P1 itself**: `InputEnvelope` has live consumers (`gunbc_ci_corpus_envelope`, `input_envelope_admission_test.dag`), so extending it is a P1-consumer follow-up alongside the `fleet_container.StorageRequirement` migration (§8.2), not part of the model-only synthetic-fixture draft in §10.

### 9.2 Provenance-as-metric — how much of a bound is known at compile time

Add `DemandProvenance = StaticDerived | Measured | ConvergedState | Fixture` as a field on each declared bound. `StaticDerived` = computed from modeled input size + graph structure at compile time (the §3.2/§3.3 ideal); `Measured` = observed at runtime and fed back (the P4 calibration loop); `ConvergedState` = derived from a fixpoint/steady-state model (e.g. a long-running service's settled RSS); `Fixture` = a hand-set placeholder (today's entire model — `gunbc_ci_corpus_envelope`'s ceilings are explicitly `Scaffold`-marked `Fixture` values, §4.3's "gap" column is exactly the `Fixture` fraction).

Two consequences, both fail-closed (§5):

- **Production admission refuses `Fixture`.** A demand/bound whose provenance is `Fixture` is honest scaffolding, not a real ceiling — `input_admitted`/`envelope_reconciled` over a `Fixture`-provenance bound should not gate real traffic (mirrors `EnvelopeUnknown => RefusedUndeclared`; a `Fixture` bound is a *declared-but-not-yet-grounded* ceiling, a distinct failure mode from *undeclared*).
- **The `StaticDerived` fraction is the metric.** `static_derived_fraction(bounds) -> Percent` measures how much of the model is compile-time-known vs measured/converged/placeholder — this is literally the "input-starved" thesis from `realization-measurement-loop.md` §0 made quantitative, applied to this doc's axes.

This is §10's `StaticPeakDemand.provenance` field below (open decision §7.5 on whether it also lands on `InputBound` — that edit is deferred to the §9.1 follow-up since it touches a live consumer).

### 9.3 Cost-model single authority — one magnitude carrier, two readers

`StaticPeakDemand` should not become a scheduler-private taxonomy. The same per-axis magnitude record is the natural input to the complexity lens's asymptotic-class analysis (`gunbc.plans.*complexity*`, silent-ferret's lane) — the scheduler asks "does this fit the cap," the complexity lens asks "what order does this axis grow in as input scales." Both questions are over the *same* declared/derived magnitude; a second scheduler-only cost taxonomy would re-fork what `realization-measurement-loop.md`'s `CostAccount`/`PerformanceReceipt` convergence already fights to keep singular (§3). **This is a direction, not a P1 deliverable** — no complexity-lens code changes are in scope here; §7.6 asks the operator to confirm the direction before P2 wires a lens that assumes it.

**Scheduling axis conventions (from today's session, folded in as guidance for P2/P3, not a P1 code change):** space-like axes (memory, storage) schedule on the **upper bound** — OOM is a hard, asymmetric failure, so admission must never let a probabilistic average through. Time-like axes (compute, when expressed as thread-seconds rather than a hard thread-count cap) may schedule on **average/p95** — a slow run is a degraded outcome, not a crash, so the same asymmetry does not apply. This is why `StaticPeakDemand.compute` in §10 stays a hard `HardwareThreadCount` (upper-bound-shaped, matching memory/storage) rather than a distribution — a p95 compute model is a P4 calibration-loop concern, not P1.

---

## Dissolution trigger (DESIGN §6)

Delete this doc when:

1. Compile-time lens produces per-batch peak envelope from derived `base_vector` + per-Runnable demand (not measured scalars),
2. `witness_corpus_effective_widths_fit_runner_cap` is deleted and replaced by a non-tautological multi-axis witness binding scheduled width,
3. `claim_executor` reads compile-time width verdict and enforces fail-closed fork gate,
4. A host memory increase provably raises derived width without `.dag` data-row edits (construction invariant from resource-aware-scheduler.md),
5. §9's three additions have each either landed on their target carrier or been explicitly declined by the operator with a recorded reason.
