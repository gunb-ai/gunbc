# Bounded-input / cost-envelope scheduling — operator-signed design

> **Status: operator-signed design record (2026-07-02).** Model-before-implement: this doc is the single authority for how bounded input, cost envelopes, and scheduling compose. It declares carrier shapes, the pipeline order, and acceptance tiers — **not** scheduler-type implementations (those dispatch as separate work items). DESIGN refs: §1 (time = cost; safety — cost never drives selection), §2 (one concept every scale — width is a budget fit count), §3 (single authority — do not re-mint InputEnvelope / symbolic_cost / budget tree), §5 (fail-closed — unbounded input and zero predicted cost are the same smell; Predicted authority / Measured falsifier), §6 (this doc dissolves when the carriers state the policy).

## 1. The displaced cost (why one design)

Scheduling work was split across four plan docs — [`input-envelope-roadmap.md`](input-envelope-roadmap.md) (data ceiling), [`resource-aware-scheduler.md`](resource-aware-scheduler.md) (width from cost), [`ci-selection-vs-scheduling.md`](ci-selection-vs-scheduling.md) (selection vs scheduling axis), [`compute-envelope-model.md`](compute-envelope-model.md) (host resource envelope). Each was correct locally; together they lacked one signed answer to: **given a must-run set, how does declared input bound predicted cost, and how does predicted cost bound width — without fusing selection?** This doc is that answer. Companion docs remain implementation trackers; **this doc is the authority they must not contradict**.

## 2. Operator-signed invariants (2026-07-02)

**Phase namespace:** §7 uses **P0–P5** (this design). Companion plans keep their own labels — always qualified (e.g. input-envelope-roadmap **P2** = this design's **P5**).

1. **Bounded input before cost before schedule.** A floor run whose corpus lacks a declared `InputEnvelope` is fail-open (§5): the scheduler has no principled admission or cost basis. `InputEnvelope = BoundedInput | EnvelopeUnknown` (`gunbc.ci_input_envelope`); `EnvelopeUnknown` is honest bottom — admission refuses (`RefusedUndeclared`), never silent green.
2. **Predicted cost is authority; Measured is falsifier.** `CostBasis = Predicted` — symbolic cost evaluated at the declared input envelope — is what width, shard balance, and budget consumers read. `CostBasis = Measured` compares against Predicted; a mismatch is a bug in the cost model, not a silent override. Measuring where you can derive is the §5 fail-open smell.
3. **Cost informs scheduling, never selection** (operator, 2026-06-21 — see [`ci-selection-vs-scheduling.md`](ci-selection-vs-scheduling.md)). Expensive-but-affected work runs per-PR; the answer to "the affected set is costly" is parallelize + cache, never "run it later".
4. **Schedule topology stays central; width/cost realization is peripheral.** `Schedule = List<List<Runnable>>` is hardware-free partial order (`std.realization_schedule`). Width, shard balance, and cache-layer choice are peripheral realizations folded against deployment facts — measured decisions must not break `schedule_eq` across hosts.
5. **More memory → more width → more throughput, by construction.** `spawn_width = floor(available_memory / max_runnable_predicted_space)` — derived, never hand-authored in plan data rows. Host memory increase reflected in budget → width rises automatically → no `.dag` ceiling edits.

## 3. The two envelopes

### 3a InputEnvelope — the data-side ceiling

**Authority:** `gunbc.ci_input_envelope` (`InputSizeAxis` · `InputBound` · `InputEnvelope` · `input_admitted` · `gunbc_ci_corpus_envelope`). **P0 landed (PARTIAL ✓):** types + admission witness (`dag/test/claim/input_envelope_admission_test.dag`). Scaffold ceilings in `gunbc_ci_corpus_envelope` dissolve when **P5** (input-envelope-roadmap **P2**) derives them from corpus discovery (witness roster count, source-node count, corpus-node count) — operator-set operating-point ceilings in P0 are explicitly **not** measurements.

### 3b CostEnvelope — symbolic cost at the envelope

**Definition:** `CostEnvelope = symbolic_cost(graph structure)` evaluated with size variables bound to the `BoundedInput` ceilings (reuse `v2.lens.cost` / `symbolic_cost_of_node` — do **not** mint a parallel cost fold). The envelope binds axes (`WitnessCount`, `SourceNodeCount`, `CorpusNodeCount`) to the symbolic cost's size parameters so predicted space/time is a **function of declared input**, not a static data row.

**On `Runnable`:** `cost: CostAccount` with `basis = Predicted`; `space` (and rolled-up `time` where modeled) come from the CostEnvelope evaluation for that runnable's graph slice. `cost_account_predicted_zero()` is the tell that Predicted authority is not yet wired — it must have no consumers on the floor path when implementation closes.

## 4. The pipeline — three stages, never fused

| Stage | Question | Authority | Fail-closed bottom |
| --- | --- | --- | --- |
| **SELECTION** (§4) | does this result depend on what changed? | `v2.lens.affected_set` — transitive node closure | provenance gap → run-all baseline (#5427) |
| **ADMISSION** (data) | is actual input within declared ceiling? | `input_admitted(envelope, actual)` | `EnvelopeUnknown` → `RefusedUndeclared` |
| **SCHEDULING** (§1/§2) | given admitted work, how wide / where / when? | CostEnvelope (Predicted) + `gunbc.fleet_container.ResourceEnvelope` / budget tree | over-commit → refuse or shrink width, never silent OOM |

**Load-bearing seam:** selection produces membership; admission gates input; scheduling consumes **Predicted** cost against **available** budget. None of the three may read cost to skip selection, and admission must run before width derivation uses cost (you cannot predict cost honestly on unbounded input).

## 5. Width derivation (construction invariant)

**Formula:** `spawn_width = min(cpu_bound, floor(memory_budget / max_runnable_predicted_space))` — the RUN-phase lever (`compute-envelope-model.md` §3: discovery shards use the prebuilt binary; width-up is pids-safe on its own). **Budget source:** `extdeps.accounting.budget` / `product.budget_tree` / `std.realization_width` — one capped-resource→claims concept (§3 convergence with complexity `EffortBudget` and `memory_aware_spawn_width`). **Side-channel deletion:** `gunbc_ci_floor_spawn_width_for_budget`, static `gunbc_ci_floor_measured_peak` rows, and a second `eval_spawn_width` pass in `claim_executor` retire when `WidthBoundedRealization` derives width from the plan.

## 6. Single authorities — do not re-mint (§3)

- **Input envelope** — `gunbc.ci_input_envelope` (not a scheduler-local ceiling type)
- **Symbolic cost** — `v2.lens.cost` (`symbolic_cost_of_node`, `SymbolicCost` lattice)
- **Schedule topology** — `std.realization_schedule` (`Schedule`, `Runnable`, `RealizationPlan`)
- **Width fold** — `std.realization_width` + peripheral `WidthBoundedRealization`
- **Host / run budget** — `product.budget_tree` + `gunbc.fleet_container.ResourceEnvelope` (BUILD vs RUN phases stay distinct — two invariants, not one product; `#5904` moved this type off dissolved `product.compute_fabric`)
- **Measured observation** — `gunbc.fleet_intent.PerformanceReceipt` → roll-up to `CostAccount` with `basis = Measured` (falsifier only; `#5904` live home — not dissolved `compute_fabric.PerformanceReceipt`)

## 7. Implementation phases (dispatch separately — not this capture)

Phases are ordered; each closes at a named consumer tier (mirrors §2 fleet acceptance: algebra alone is not done). **This capture lands phases 0–1 of the design record only.**

1. **P0 — InputEnvelope admission (PARTIAL ✓).** Types + `input_admitted` + CI corpus instance + witness. **Accept:** `input_envelope_admission_test.dag` green; RED on revert of `RefusedUndeclared` / over-envelope paths.
2. **P1 — CostEnvelope on `Runnable` (Predicted).** Populate `Runnable.cost.space` from symbolic evaluation at `gunbc_ci_corpus_envelope` bindings; delete `cost_account_predicted_zero()` on the floor rollup path. **Accept (T2):** floor plan reads cost from runnables; remove cost side-channel → witness fails.
3. **P2 — Scheduler consumes Predicted cost.** `WidthBoundedRealization` derives `spawn_width` from plan; delete `gunbc_ci_floor_spawn_width_for_budget` and static peak rows. **Accept (T2):** host memory increase → higher width without data-row edits (discriminating receipt).
4. **P3 — Measured falsifier + calibration loop.** `gunbc.fleet_intent.PerformanceReceipt` / per-shard RSS (`resource-aware-scheduler.md` Node A) feeds `CostBasis = Measured`; mismatch vs Predicted → typed diagnostic. Calibration updates symbolic formula inputs, not the Predicted row directly. **Accept (T3):** planted underestimate in Predicted → falsifier RED.
5. **P4 — Runtime admission gate.** Wire `input_admitted` into the floor path (`ci_floor_plan.dag` Runnable roster + `gunbc.fleet_container.demand_envelope_fits_budget` / `witness_run_demand_envelope_fits_budget`) so runtime refuses before schedule build — **not** dissolved `WorkDemand` (`#5904`). **Accept (T2):** corpus exceeding `gunbc_ci_corpus_envelope` → schedule refuses, not OOM.
6. **P5 — Ceiling derivation (input-envelope-roadmap P2).** Replace scaffold ceilings in `gunbc_ci_corpus_envelope` with values derived from discovery roster + measured node counts; dissolve `Scaffold` disposition on the envelope data.

## 8. Relationship to companion docs

- [`input-envelope-roadmap.md`](input-envelope-roadmap.md) — input-envelope-roadmap P1/P2 detail; maps to this design's P0/P5; defers to §3a here for authority
- [`resource-aware-scheduler.md`](resource-aware-scheduler.md) — Nodes A–D implementation tracker for this design's P2–P3; defers to §5–§7 here
- [`ci-selection-vs-scheduling.md`](ci-selection-vs-scheduling.md) — selection axis framing; §4 pipeline table is the composed view
- [`compute-envelope-model.md`](compute-envelope-model.md) — host envelope framing (live type: `gunbc.fleet_container.ResourceEnvelope`); BUILD-phase fan-out; RUN-phase width is §5 here
- [`realization-measurement-loop.md`](realization-measurement-loop.md) — broader cost→schedule loop coordination; this design is the scheduling-arm authority named by that doc's companion pointer
- [`budget-tree.md`](budget-tree.md) — admission/construction for memory budget; width fold is a consumer leaf

## 9. Explicit non-goals (this capture)

- No new scheduler types in `src/v2/workflow/scheduler.dag` or `std.realization_schedule` beyond what P1–P2 dispatch items specify
- No `claim_executor` / Rust seed edits for measurement plumbing (Node A remains its own PR)
- No fusion of selection and scheduling (no cost-based nightly routing)
- No replacement of GitHub/job placement (`RunShape → Allocation → Receipt` fabric is orthogonal host-level placement)

## 10. Red controls (design-level — implementation must wire each)

- `EnvelopeUnknown` demand → `RefusedUndeclared` (already witnessed in P0)
- Actual axis over declared ceiling → `RefusedOverEnvelope`
- Predicted vs Measured space mismatch → typed falsifier (not silent width change)
- Drop Predicted cost from runnable → width fold fails closed / refuses over-wide spawn
- Affected witness skipped on touched dep → selection soundness RED (independent of this design, but scheduling must not bypass it)

## Dissolution trigger (DESIGN §6)

Delete this doc when `gunbc_ci_corpus_envelope` carries derived `BoundedInput` ceilings (P5), every floor `Runnable` carries Predicted `CostEnvelope`-derived cost, `spawn_width` is derived from `WidthBoundedRealization` (no side-channel), `gunbc.fleet_intent.PerformanceReceipt` falsifies Predicted with a discriminating RED, runtime admission consumes `input_admitted` on the live floor path, and the selection/scheduling axis split is witnessed — at which point the carriers state the policy and companion trackers (`resource-aware-scheduler.md`, `input-envelope-roadmap.md`) may dissolve.
