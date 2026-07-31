# Receipt — entry-graph-union slice 1: exclusive attribution and selected-entry overlap

**Status:** measurement receipt, timestamped 2026-07-31. **DESIGN.md + the carriers remain the authority** — this is a dated receipt, not a fact ledger. Dissolves with the two scaffolds it reports (`cli_run_exclusive_cost_partition_probe`, `cli_run_selected_closure_overlap_probe`).

**Lane:** `ci-cost` · **Subject:** `entry-graph-union-construction` · **Deliverable:** measurement only. No union implementation, no eviction/retention redesign, no fork of `walk_memo` or the #6999 entry-closure memo, no local timing claim offered as implementation acceptance.

**The measurement is allowed to conclude the program is worth less than assumed.** Three claims an earlier draft of this receipt asserted were RETRACTED by further execution (§A.6, §A.7, § Verdict); they are kept visible rather than edited out, because the way they failed is the reusable part. See § Verdict for what survives.

---

## A — the exclusive cost partition

### A.1 Why no share was quotable before

The premise holds — but the mechanism is **not** nesting. `ResolveStageNanos` accumulates over *every* resolve the thread runs, witness entries and machinery entries alike (`resolve_entry_graph_shared` → `resolve_entry_with_index` → `resolve_entry_with_parse_cache`, used by the discovery producer, `floor_diff_observe`, and the module-graph facts build). The receipts printing those rows quote a parent covering only **one** of those universes. Two denominators on one line.

Reproduced (`claim_batch`, single entry, release):

```
[resolve]         dag/test/claim/ci_floor_measurement_test.dag: 45568ms
[resolve-summary] 1 resolve(s) in 45568ms
[resolve-split]   load=49878.3ms ... typecheck=17840.7ms
```

`load` alone exceeds the stated parent. The span account names the missing term: **two** top-level resolves ran, and the second (`dag/gunbc/output_policy.dag`, 27.6s) is machinery no receipt's parent counted.

This also explains why the old `other=` row never surfaced it: it is computed with `saturating_sub`, so an over-attribution reports a healthy-looking `0` remainder.

### A.2 The basis

`summed_top_level_resolve_span_nanos` — each top-level `resolve_entry_with_parse_cache` span's own duration.

Additive **by construction**: spans are thread-sequential, and nested spans are counted but never re-added. Elapsed wall is *not* additive over concurrent floor-worker spans, so no additive partition of it exists to manufacture; wall figures are carried under `observations` and never partitioned.

### A.3 The law

```
parent_span_nanos == sum_exclusive_nanos + remainder_nanos
```

Holds **by construction** — `remainder` is *derived* as `parent − Σ exclusive`, every row an integer nanosecond count in one basis, declared tolerance **0 ns**. What makes it non-vacuous is that the derivation can fail, and then it **refuses** rather than clamps:

| refusal | meaning |
|---|---|
| `OverAttributed` | rows sum past the parent — they are not a partition of it |
| `NestedSpanAttribution` | a nested span corrupted the per-entry slot (`resolve_stage_slot_reset` runs at span entry) |
| `NoSpans` | the measured work opened no resolve span, so it carries no stage attribution |

`share_of_parent()` returns `None` in every refused state, and the JSON marks them `"shares_quotable": false`. **A component receives no percentage until the accounting is exclusive.**

### A.4 The exclusive partition (verdict `Reconciled`, remainder 55.1 ms of 73158.3 ms)

| exclusive row | ms | share of parent |
|---|---:|---:|
| **load** | 49878.3 | **68.18%** |
| **typecheck_compute** | 17840.7 | **24.39%** |
| reconcile_assembly | 3602.4 | 4.92% |
| parse | 1238.2 | 1.69% |
| assembly_rewire | 250.6 | 0.34% |
| normalize | 90.9 | 0.12% |
| assembly_emit_info | 77.9 | 0.11% |
| ownership | 68.6 | 0.09% |
| assembly_services | 27.3 | 0.04% |
| resolve_modules | 24.8 | 0.03% |
| assembly_registry / parent_envs / assembly_schedule / assembly_probe | 3.5 | 0.00% |
| **(unattributed remainder)** | **55.1** | **0.08%** |

Inclusive rows, carried and never summed: `assembly_rewire_type_env`, `assembly_rewire_import_str`, `assembly_rewire_func_env` ⊂ `assembly_rewire`.

Per entry — machinery is **37.7%** of all resolve-span time even in a single-entry run:

| entry | span ms | load share |
|---|---:|---:|
| `dag/test/claim/ci_floor_measurement_test.dag` (witness) | 45568.2 | 55.0% |
| `dag/gunbc/output_policy.dag` (machinery) | 27590.0 | 89.9% |

### A.5 Correction to the dispatch's starting point

`measure_whole_tree_resolve` and `measure_dependency_view_build` — named as the starting point — carry **no stage attribution at all**. They run the monolithic `compile_to_resolved` path, which fills no `ResolveStageNanos` row and opens no resolve span. `measure_whole_tree_resolve` now emits the partition and **refuses with `NoSpans`**, making that an executed receipt rather than a prose claim. The counters the dispatch describes live on the per-entry path (`claim_batch`, `claim_executor`).

### A.6 What `load` actually is — RETRACTED and replaced

**An earlier version of this section claimed** `load` is dominated by
`referenced_module_paths_in_text`, an unmemoized full-content scan run
once per (entry, module) pair, and concluded that B's duplication factor is "directly the
multiplier on the dominant exclusive cost."

**That is refuted by execution.** Instrumenting exactly that call:

| row (inclusive, inside `load`) | measured |
|---|---:|
| `load_reference_scan` | 3.9–23.0 ms (**~0.0% of load**) |
| `load_import_closure` | 5.3–6.3 ms (~0.0%) |
| `load_pool_reference_closure` | 7.9–16.1 ms (~0.0%) |
| **`load_bare_reference_closure`** | **47.6–53.3 s (~100% of load)** |
| └ `load_bare_edge_index` | **51.2 s (~100% of the above)** |
| └ `load_bare_path_lookup` | 6.7 ms |
| └ `load_bare_edge_walk` | 0.2 ms |

The mechanism was inferred from reading the call path, never measured — the §5
"specification without execution" trap, committed while writing about it. The missed step:
`load_sources_for_entry_with_pool` calls `load_sources_for_entry_with_index` *and then*
`extend_sources_to_both_closure_fixpoint`, which the first instrumentation never timed.

**What `load` actually is:** `build_both_closure_edge_index` — a **corpus-wide** edge index,
memoized per `MultiEntryIndex`, costing ~25.6 s per index. It is **independent of the
entry's closure size** (159-module and 504-module closures pay the same), so it is fixed per
index — *not* per entry, and *not* per membership. The single-entry `claim_batch` harness
pays it twice only because two indices exist on that path: `claim_batch`'s own
`build_multi_entry_index`, plus `process_shared_index` for the machinery entry.

### A.7 The bound this puts on ALL of section A

Because the dominant row is a fixed per-index construction, the shares in A.4 describe a
**fixed-cost-dominated harness, not the floor**. On a floor run one index amortizes across
hundreds of entries, so `load`'s share there must be smaller — by how much is unmeasured.

Repeating the partition (operator review point 1) shows why this matters. Run-to-run within
an entry is stable; **across entries the dominant row inverts**:

| entry (closure modules) | run | parent | load | typecheck_compute |
|---|---|---:|---:|---:|
| `ci_floor_measurement_test` (159) | r1 | 70.0 s | **67.02%** | 25.62% |
| `ci_floor_measurement_test` | r2 | 75.3 s | **67.96%** | 24.64% |
| `generated_artifact_drift_test` (504) | r1 | 125.4 s | 37.89% | **51.48%** |
| `generated_artifact_drift_test` | r2 | 133.2 s | 39.52% | **49.73%** |

Typecheck scales with closure size; the index build does not. So "load is the dominant
cost" was an artifact of the entry chosen, and the lane's original typecheck framing is
correct on the larger closure.

**A floor-representative partition requires the `claim_executor` discovery path** (wired,
emitting `[cost-partition]` against per-entry receipts from one universe) and **has not been
run**. Until it is, no share in A.4 should be quoted as a floor fact.

---

## B — closure overlap over the production-selected set

Inputs all taken from live production machinery — `discover_floor_witness_roster`, the same `floor_diff_observe` unified + name-status observations the executor uses, `entry_eligible_for_discovery_skip_before_resolve`, `collect_both_closure_module_names_for_entry`, and the floor's own `witness_exclusion_substrings()` authority. **No parallel hand-written selection model.** Nothing is resolved or typechecked.

Subjects are already-known main commits, reproducible via the production `GUNBC_CI_DIFF_BASE` override. Source roots `dag`, `src/v2`; scan dirs `dag/test/claim`, `src/v2/test/claim/manual`, `src/v2/test/claim/emit`.

| | narrow | typical | broad |
|---|---:|---:|---:|
| head | `a07d1b73f8e4` | `aaa3e8afb530` | `b01cdf4d8914` |
| base | `e30621111f37` | `b01cdf4d8914` | `0d6ffc4db975` |
| changed paths | 3 | 13 | 29 |
| roster entries | 807 | 809 | 808 |
| **selected (N)** | **286** | **316** | **393** |
| skipped | 521 | 493 | 415 |
| `sum_closure_memberships` Σ\|Cᵢ\| | 42,928 | 47,759 | 52,740 |
| `union_modules` \|⋃Cᵢ\| | 1,196 | 1,243 | 1,378 |
| **`duplication_factor`** | **35.89** | **38.42** | **38.27** |
| `membership_upper_bound` | 41,732 | 46,516 | 51,362 |
| repeats as share of Σ\|Cᵢ\| | 97.2% | 97.4% | 97.4% |
| fanout p50 / p90 / p99 / max | 5 / 104 / 218 / 281 | 3 / 117 / 219 / 311 | 2 / 108 / 295 / 388 |

Highest-fanout modules (narrow): `std.algebra` 281, `std.error_primitives` 281, `std.types` 281, `std.occurrence_identity` 256, `std.content_hash` 255, `v2.std.node` 255 — i.e. **~98% of every selected closure**.

Largest closures (narrow): `generated_artifact_drift_test.dag` 504, `srv3_runner_memory_converge_witness_test.dag` 483, `fleet_converge_cli_witness_test.dag` 475.

### B.1 Four structural reads

1. **The factor is stable across breadth.** 35.9 → 38.4 → 38.3 while changed paths go 3 → 13 → 29. Overlap is a property of the corpus's shape, not of the diff.
2. **The union is nearly saturated at the narrow subject.** N grows 37% (286 → 393) while |⋃Cᵢ| grows 15% (1,196 → 1,378). Most of what a broader diff adds is *already in the union*.
3. **Max fanout ≈ N** in every subject (281/286, 311/316, 388/393). A small `std` core sits in essentially every selected closure.
4. **The median module is rare and falls as N rises** (5 → 3 → 2). The distribution is a universal core plus a long private tail — not uniform sharing. Any union benefit is concentrated in the core.

### B.2 A selection observation, incidental but worth recording

A **3-file** diff still selects **286 of 807** roster entries (35%); the 29-file diff selects 49%. Selection shrinks the corpus by roughly 2×, not 10×. That is a fact about selection breadth, not about the union, and is not part of this lane's claim.

### B.3 Fidelity note — a defect caught and corrected mid-measurement

The probe first defaulted its exclusions to `whole_tree_probe_exclusion_substrings()`, which is the floor's list **unioned** with the whole-tree strict-resolve exclusions. That produced a **45-entry roster against 579 `*_test.dag` files** under the same scan dirs — roughly 8% of the corpus — and reported `N=6, Σ=335, ⋃=214, factor 1.565`.

Every derived quantity in that receipt was internally consistent and the accounting law held. It simply described a different, much smaller population. **It was invisible from inside the receipt** and surfaced only by checking the reported roster size against the corpus on disk. Those numbers are withdrawn; the table above is on `witness_exclusion_substrings()`, the floor authority.

Recorded because it is the failure mode this kind of measurement is most exposed to: a self-consistent receipt over the wrong population.

---

## Verdict — what survives, and what was retracted

### Survives (measured, repeated)

- **The partition mechanism.** The `[resolve-split]` rows were never quotable as shares
  because children were summed over every resolve a thread runs while the parent covered
  only witness-entry resolves — two denominators, not a nesting. The exclusive partition
  reconciles at 0 ns tolerance and refuses rather than clamps.
- **B's membership numbers.** Duplication factor 35.9 / 38.4 / 38.3 across narrow / typical
  / broad, 97.2–97.4% of memberships repeated, stable across a 10× range of diff breadth.
- **`load` is `build_both_closure_edge_index`**, a corpus-wide index memoized per
  `MultiEntryIndex` at ~25.6 s per index, independent of the entry's closure size.

### Retracted

1. **"`load` is dominated by an unmemoized per-(entry, module) content scan."** Measured at
   ~0.0% of `load` (§A.6). Inferred from the call path, never measured.
2. **"`load` is the dominant cost."** True at 67–68% on a 159-module closure, false at
   38–40% on a 504-module one where typecheck_compute is ~50% (§A.7).
3. **"The duplication factor is the multiplier on the dominant cost," and the per-source
   content-hash memo that followed.** The dominant row is fixed per index — not per entry,
   not per membership — so neither A×B join direction holds, and the memo recommendation
   has no measured support. Withdrawn.

### Does the evidence strengthen, shrink, or close the union program?

**Neither, yet — the question is not answerable from this data, and saying otherwise would
repeat the error above.** The reason is §A.7: every share in section A comes from a
one-or-two-entry `claim_batch` harness whose dominant row is a *fixed per-index
construction*. On a floor run that cost amortizes across hundreds of entries, so the
harness systematically overstates it. A union program justified by these shares would be
justified by an artifact of the instrument.

What B establishes independently still stands and is not harness-bound: the selected
closures genuinely do repeat module membership 36–38×, and the pole that would have closed
the program outright (disjoint closures → factor 1.0) is decisively absent. But repeated
*membership* only becomes a repeated *cost* through a mechanism, and the mechanism this
receipt proposed has been refuted. No replacement mechanism is offered here.

### The one measurement that would decide it

A partition from the **`claim_executor` discovery path** — wired in this PR, emitting
`[cost-partition]` over per-entry receipts drawn from a single universe, and never run.
That is the only surface where the parent is the floor's own work and per-index fixed costs
sit in their true proportion. Until it exists, no share in §A.4 may be quoted as a floor
fact.

### Kept separate, as instructed

`resolution_divergence_census` runs as its own **binary**
(`dag/tools/resolution_divergence_silent_pick_gate_witness_transport.dag`,
`bin_name: "resolution_divergence_census"`, `--closure-scoped`). Its closure-scoped resolve
is a **different process**, so an in-process union cannot displace it, and it must not be
folded into any union prize.

### Not established, and required before any implementation

Identical verdicts plus arm64 fleet receipts showing no material increase in cgroup peak,
hard backoff, or throttle wall at the real slot budget. No local timing figure in this
receipt is offered as implementation acceptance.

---

## Reproducing

```sh
# A — exclusive partition (emits [cost-partition] JSON)
claim_batch --source-root dag --source-root src/v2 \
  --entry dag/test/claim/ci_floor_measurement_test.dag \
  --function ci_floor_measurement_witnesses

# B — closure overlap over the production-selected set
GUNBC_CI_DIFF_BASE=<base-sha> measure_selected_closure_overlap \
  --source-root dag --source-root src/v2 \
  --scan-dir dag/test/claim --scan-dir src/v2/test/claim/manual --scan-dir src/v2/test/claim/emit
```

Run B from a worktree checked out at the subject's head. Do not rebuild while a probe runs — the resolve cache content-addresses its own executable and will refuse on a replaced binary.
