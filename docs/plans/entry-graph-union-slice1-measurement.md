# Receipt — entry-graph-union slice 1: exclusive attribution and selected-entry overlap

**Status:** measurement receipt, timestamped 2026-07-31. **DESIGN.md + the carriers remain the authority** — this is a dated receipt, not a fact ledger. Dissolves with the two scaffolds it reports (`cli_run_exclusive_cost_partition_probe`, `cli_run_selected_closure_overlap_probe`).

**Lane:** `ci-cost` · **Subject:** `entry-graph-union-construction` · **Deliverable:** measurement only. No union implementation, no eviction/retention redesign, no fork of `walk_memo` or the #6999 entry-closure memo, no local timing claim offered as implementation acceptance.

**The measurement is allowed to conclude the program is worth less than assumed.** It does not; but the axis it strengthens is not the one the program was framed around. See § Verdict.

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

### A.6 What `load` actually is — the load-bearing finding

`load` = `load_sources_for_entry_with_pool` → `resolve_transitively` + `extend_with_reference_closure`. The inner scan is `referenced_module_paths_in_text` (`cli_run.rs:2062`, called at `:2008`) — a **full-content byte scan, unmemoized, run once per (entry, module) pair**.

So the duplication factor measured in B is not an abstract membership ratio: **it is directly the multiplier on the dominant exclusive cost.**

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

## Verdict — joining A and B

**How much parent work is in repeatable graph construction?**
`load` is **68.2%** of resolve-span time, and it *is* the closure walk. `typecheck_compute` is 24.4%; everything else together is 7.4%.

**How much of that construction has overlapping module membership?**
**97.2–97.4%** of closure memberships are repeats, at a duplication factor of **35.9–38.4×**, stable across a 10× range of diff breadth.

**What upper bound could a union displace?**
On the membership axis, collapsing Σ|Cᵢ| → |⋃Cᵢ| removes up to **41.7k–51.4k** repeated memberships. Since the repeated unit is exactly the per-(entry, module) work that dominates `load`, the *ceiling* is ≈ 97% of 68% ≈ **66% of resolve-span time**. This is an upper bound on repeated membership, **not a promise of equivalent wall savings** — the `entry_closure_sources` memo already caches per-entry results and `index.source_files` already shares file reads, so only the scan-and-assemble portion is exposed.

**Does the evidence strengthen, shrink, or close the union program?**

It **strengthens the premise and relocates the prize.**

- Strengthened: the overlap is real, large, and stable — not an artifact of one subject. The pole that would have closed the program (disjoint closures → factor 1.0) is decisively absent.
- Relocated: the prize is in **`load`**, not typecheck. `typecheck_compute` is already content-key memoized through `typed_module_cache`, so its cross-entry duplication is largely collapsed already. A union graph justified as "typecheck once" would be buying something mostly already owned.
- **Cheaper rival that must be priced first.** Because the repeated unit is a *pure function of source content* (`referenced_module_paths_in_text(content)`), a per-source memo keyed on content hash would collapse the same 35.9–38.4× on the scan portion **without any union graph**. That is a far smaller intervention than an entry-graph union, and slice 2 should not commit to the union before pricing it. Recorded as a finding, deliberately **not implemented here** — it is an implementation decision, and this deliverable is measurement.

**Kept separate, as instructed:** `resolution_divergence_census` runs as its own **binary** (`dag/tools/resolution_divergence_silent_pick_gate_witness_transport.dag`, `bin_name: "resolution_divergence_census"`, `--closure-scoped`). Its closure-scoped resolve is a **different process**, so an in-process union cannot displace it. It is not part of the union prize above and must not be laundered into it.

**Not established here, and required before any implementation:** identical verdicts plus arm64 fleet receipts showing no material increase in cgroup peak, hard backoff, or throttle wall at the real slot budget. No local timing figure in this receipt is offered as implementation acceptance.

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
