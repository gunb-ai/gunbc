# Static resource envelope — N-dimensional compile-time width for CI floor

> **Status: DESIGN — operator sign-off required before carrier or load-bearing edits land.**
> Dispatched from `vivid-carp-243` (OOM kill on CI) → `merry-koi-140`.
> Composes with `merry-crane-716` (affected-set → live scheduler): affected-set **prunes** the graph this envelope fold runs over; smaller closure → smaller retained base → width can safely rise.
> DESIGN refs: §2 (one Cost record, every axis — horizontal reuse), §3 (single authority — memory bytes ≠ disk bytes; no nickname), §5 (construction-not-validation; kill the tautology witness), §6 (dissolution trigger on every scaffold).

---

## 1. Symptom and root cause (why width=1 is a stop-gap, not the fix)

CI floor OOM-kills `claim_executor` (exit 137) under the 8 GiB runner cgroup cap (`gunbc_ci_runner_cgroup_memory_cap`).

**Mechanism chain (landed analysis, PR #6059 stop-gap):**

1. `claim_executor` is **one long-lived process** holding the whole `dsl` + `src/v2` resolved corpus resident across all batches.
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
| `gunbc_ci_floor_per_shard_peak_rss_bytes()` | `gunbc_ci_floor_measurement.dag` | Measured scalar (max of samples); not derived from closure size or declared input envelope |
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

`witness_corpus_effective_widths_fit_runner_cap` (`ci_floor_plan.dag:588-596`) checks:

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
| **storage** | `ByteSize` (disk) | disk quota / tmpfs cap | **Not** a nickname for memory — separate Quantity |
| **energy** | `Watt` = `Measure<Power, One, Nat>` (future joules) | fleet power budget (future) | Scaffold `Watt(0)` until instrumented |

**Reconciliation is per-axis, independent:**

```
width_memory  = floor((cap.memory  - base.memory)  / per_shard.memory)
width_compute = floor((cap.compute - base.compute) / per_shard.compute)
width_storage = floor((cap.storage - base.storage) / per_shard.storage)
width_energy  = floor((cap.energy   - base.energy)   / per_shard.energy)   // when live
final_width   = min(width_memory, width_compute, width_storage, width_energy, shard_demand)
```

Do **not** fuse axes even where both are `ByteSize` (§3 — memory bytes ≠ disk bytes).

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
| `dsl/gunbc/ci_floor_measurement.dag` | `gunbc_ci_floor_per_shard_peak_*`, `executor_base_overhead`, `execution_corpus_per_witness_rss`, `host_compiler_spawn_peak`, `memory_derived_width`, `conservative_fallback_width` | memory only | width derivation |
| `src/v2/workflow/ci_floor_plan.dag` | `gate_runnable_profile`, `gunbc_ci_plan_spawn_width`, `witness_corpus_effective_widths_fit_runner_cap`, all `witness_floor_*` | memory only | schedule + witnesses |
| `src/v1/stage0/src/bin/claim_executor.rs` | `eval_spawn_width`, `read_host_memory_budget_bytes`, `run_walk(..., spawn_width)`, per-shard calibration emit | memory only | runtime width + falsifier |
| `src/v1/stage0/src/cli_run.rs` | `spawn_width_cap`, discovery corpus width cap | memory only | per-corpus cap |

### 4.2 Supporting / parallel (migrate or project, not primary authority)

| Site | Pattern | Notes |
| --- | --- | --- |
| `dsl/std/realization_schedule.dag` | `RunnableResourceProfile`, `RunnableMemoryClass` | Extend → `StaticPeakDemand`; dissolve memory-only class |
| `dsl/std/realization_width.dag` | `memory_aware_spawn_width`, `process_memory_aware_spawn_width` | Formula correct; reads external scalar — becomes projection from envelope lens |
| `dsl/gunbc/fleet_container.dag` | `ResourceEnvelope`, `parallel_run_demand_envelope` | Host supply; caps feed reconciliation |
| `dsl/gunbc/ci_input_envelope.dag` | `InputEnvelope`, `input_admitted` | Declared input sizes → derivation input (P2) |
| `dsl/gunbc/ci_budget_tree.dag` | INTER-run co-residence | Orthogonal granularity (§3 split already documented) |
| `dsl/gunbc/ci_compile_jobs.dag` | BUILD-phase `process_memory_aware_spawn_width` | BUILD vs RUN phase split (compute-envelope-model.md) |
| `dsl/test/claim/ci_floor_measurement_per_shard_test.dag` | Pins scalar values | Dissolves when derivation lands |
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

Reply on dashboard or PR review.

---

## Dissolution trigger (DESIGN §6)

Delete this doc when:

1. Compile-time lens produces per-batch peak envelope from derived `base_vector` + per-Runnable demand (not measured scalars),
2. `witness_corpus_effective_widths_fit_runner_cap` is deleted and replaced by a non-tautological multi-axis witness binding scheduled width,
3. `claim_executor` reads compile-time width verdict and enforces fail-closed fork gate,
4. A host memory increase provably raises derived width without `.dag` data-row edits (construction invariant from resource-aware-scheduler.md).
