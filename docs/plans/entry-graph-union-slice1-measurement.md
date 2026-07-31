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

Repeating the partition (operator review point 1) was asked for in order to show the share
ordering holds. **It does not hold, and that is the finding.** Every row below is read from
a committed receipt in `docs/plans/receipts/entry-graph-union-slice1/`; an earlier revision
of this table quoted a run set that was never retained, so its figures were unverifiable and
are withdrawn.

| entry (closure modules) | run | parent | `load` | `typecheck_compute` |
|---|---|---:|---:|---:|
| `ci_floor_measurement_test` (159) | r1 | 75.2 s | **51.24 s / 68.11%** | 18.48 s / 24.57% |
| `ci_floor_measurement_test` | r2 | 66.4 s | **44.96 s / 67.68%** | 16.57 s / 24.95% |
| `generated_artifact_drift_test` (504) | r1 | 131.0 s | 50.98 s / 38.93% | **65.56 s / 50.06%** |
| `generated_artifact_drift_test` | r2 | 155.7 s | **77.11 s / 49.52%** | 64.55 s / 41.46% |

The ordering flips **between two runs of the same entry**: on `generated_artifact_drift_test`
r1 `typecheck_compute` leads, on r2 `load` leads. So no share in A.4 is a reproducible
quantity, and the earlier "the dominant row inverts across entries" reading was itself an
artifact — it happened to compare two runs that agreed.

The absolute columns say why, and they are the durable part:

- **`typecheck_compute` is stable within an entry and scales with closure size** —
  18.48 / 16.57 s on the 159-module closure against 65.56 / 64.55 s on the 504-module one.
  Roughly 3.6× the time for 3.2× the modules, and ±6% run-to-run.
- **`load` does not track closure size at all** — 51.24 s on the small closure and 50.98 s
  on the large one, then 77.11 s on a repeat of that same large one. It is a corpus-fixed
  cost (`load_bare_edge_index` is 99.9–100.0% of `load` in all four runs) whose *magnitude*
  is host-noise-dominated, spanning 44.96–77.11 s across runs that differ in nothing else.

A share is a ratio of a stable numerator to a noisy one, so the share moves with the noise.
**Confound, disclosed:** these runs shared the host with concurrent `cargo build` work, which
is the most likely source of the 1.7× spread in a cost that should be constant. That does not
rescue the shares — it explains why they cannot be quoted.

### A.8 The amortization is now measured, not assumed

§A.7 said `load`'s share on a floor run "must be smaller — by how much is unmeasured." It is
measured here. Two runs on the **explicit-entry** path (`--entry` + `--functions`), which
puts N entries against **one shared index**, holding everything else fixed:

| row | N=1 (2 spans) | N=6 (7 spans) | growth |
|---|---:|---:|---:|
| parent | 69.97 s | 114.12 s | 1.63× |
| **`load`** | **47.44 s** | **51.32 s** | **1.08×** |
| `load_bare_edge_index` | 47.40 s | 51.18 s | 1.08× |
| `typecheck_compute` | 17.34 s | 49.64 s | 2.86× |
| `parse` | 1.14 s | 3.66 s | 3.21× |
| `normalize` | 0.09 s | 0.35 s | 4.05× |
| `resolve_modules` | 0.02 s | 0.11 s | 4.40× |
| **`load` share of parent** | **67.79%** | **44.97%** | — |

Span count rises 3.5× and every per-entry row rises with it, while `load` rises 8%. That 8%
is within the run-to-run noise band already established for this row (44.96–77.11 s), so the
measurement is *consistent with `load` being flat in N* and does not resolve a small residual
slope. **`load` is paid once per index, not once per entry** — the existing per-index memo
already amortizes it, with no union graph involved.

The N=1 run reproduces the discovery-path receipts (67.79% against 68.11% / 67.68%),
confirming the explicit-entry path measures the same thing.

**Direction, with the extrapolation marked as such.** Non-`load` work is 22.53 s at N=1 and
62.80 s at N=6, i.e. ~8.05 s per added entry. Holding the fixed cost at 51.32 s, that model
puts `load` at ~11% of parent by N=50 and ~2% by N=316 (the typical subject's selected set).
**This is a projection from two points over six small entries, not a receipt** — per-entry
cost varies by closure size, and no floor run has been executed. What is *measured* is the
sign and the magnitude at N=6: the share falls, steeply, with no change to the code.

**Whole-corpus floor runs were attempted and could not complete on this host** — OOM-killed
twice (exit 137, at 1,513 and 1,046 entries), including under
`GUNBC_MEMORY_BUDGET_BYTES=44 GiB`, which governs realization admission rather than
resolve-pool retention. The `claim_executor` discovery path remains the surface that would
produce a floor receipt, and no share in §A.4 may be quoted as a floor fact.

### A.9 Where this leaves the union program's target

The consequence is a redirection, and it is the sharpest thing in this receipt.

`load` — the row that dominated every single-entry partition and that this receipt originally
built its whole case on — **cannot be what a union graph displaces**, because it is already
paid once per index and is flat in the number of entries sharing that index. There is no
repeated work there to remove.

The only measured row that grows with *both* entry count (2.86× over 6 entries) and closure
size (3.6× for 3.2× the modules) is **`typecheck_compute`**, which is also the row whose unit
of work is module membership — the quantity §B measures repeating 38× by count and 48× by
bytes. So if a union prize exists, that is where it is, and it is the lane's original
typecheck framing rather than the loading framing this receipt spent §A.6 chasing.

**Not established, and deliberately not claimed:** whether typecheck work is genuinely
*repeated* across entries, or whether each entry's typecheck is already specific to its own
environment such that shared membership implies no shared computation. That is a different
measurement — per-entry typecheck attribution against a shared env — and nothing here
answers it. §B's duplication is a bound on repeated *membership*; converting it to repeated
*computation* still requires the mechanism this receipt has twice failed to establish.

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

1. **The factor is stable across breadth *by module count*, and rises by bytes.** 35.9 → 38.4 → 38.3 while changed paths go 3 → 13 → 29 — so by count, overlap is a property of the corpus's shape rather than of the diff. Byte-weighted the same three subjects give 43.0 → 47.9 → 49.3, monotonically rising (§B.1a): a broader diff adds proportionally more *large* shared modules. Both readings come from the same probe runs; the count one alone would have understated how breadth interacts with overlap.
2. **The union is nearly saturated at the narrow subject.** N grows 37% (286 → 393) while |⋃Cᵢ| grows 15% (1,196 → 1,378). Most of what a broader diff adds is *already in the union*.
3. **Max fanout ≈ N** in every subject (281/286, 311/316, 388/393). A small `std` core sits in essentially every selected closure.
4. **The median module is rare and falls as N rises** (5 → 3 → 2). The distribution is a universal core plus a long private tail — not uniform sharing. Any union benefit is concentrated in the core.

### B.1a The weighting, stated — and redone byte-weighted

**Every figure in the table above is module-count weighted.** It counts a 200-byte module and
a 200-KB module as one membership each, which is a uniformity assumption the count itself
cannot expose (operator review point 3).

Redone with each membership weighted by its module's source bytes — all three subjects, same
selection as the table above (N = 286 / 316 / 393, matching entry-for-entry):

| | narrow | typical | broad |
|---|---:|---:|---:|
| `duplication_factor` (module count) | 35.89 | 38.42 | 38.27 |
| **`byte_duplication_factor`** | **43.04** | **47.92** | **49.25** |
| byte / count | 1.199 | 1.247 | 1.287 |
| repeats as share of Σ, by count | 97.21% | 97.40% | 97.39% |
| repeats as share of Σ, by bytes | 97.68% | 97.91% | 97.97% |

For the typical subject in full: Σ over memberships is 548,899,266 bytes against a union of
11,454,316 bytes, upper bound on repeats 537,444,950 bytes.

**The count-weighted figure was the conservative one.** Byte-weighted duplication is 20–29%
*higher* in every subject, which says the modules with high fanout are systematically
**larger** than average — consistent with B.1 read 3, since the universal core is
`std.algebra`, `std.types`, `std.error_primitives` and friends rather than small leaf modules.

**The weighting also changes a reading, not just a magnitude.** By module count the factor
plateaus and dips at the broad subject (35.89 → 38.42 → **38.27**); byte-weighted it rises
monotonically (43.04 → 47.92 → **49.25**), and the byte/count ratio itself climbs with
breadth. So B.1 read 1's "stable across breadth" holds by count but not by bytes: a broader
diff pulls in proportionally *more* large shared modules. The read is corrected in place
below.

This moves the weighting question in the direction that *favours* the union program, and it
is worth being explicit that it does not rescue it: both numbers are still counts of repeated
membership, and §A.7 is why no count here converts into a displaced cost. The byte figure is
a better-weighted upper bound, not a different kind of claim.

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
- **`load` is `build_both_closure_edge_index`** — 99.9–100.0% of `load` in all four
  partition runs — a corpus-wide index memoized per `MultiEntryIndex` and **independent of
  the entry's closure size** (51.24 s on a 159-module closure, 50.98 s on a 504-module one).
  Its *magnitude* is not a stable measurement on this host: across four runs at 2 spans each
  it spans 44.96–77.11 s, i.e. ~22–39 s per index. The size-independence is the durable
  claim; the per-index figure is not.

### Retracted

1. **"`load` is dominated by an unmemoized per-(entry, module) content scan."** Measured at
   ~0.0% of `load` (§A.6). Inferred from the call path, never measured.
2. **"`load` is the dominant cost."** Not established either way. It is 67.7–68.1% on the
   159-module closure but 38.93% and 49.52% on two runs of the *same* 504-module closure —
   `typecheck_compute` leads on the first, `load` on the second. A share whose ordering
   flips between repeats of one entry is not a measurement (§A.7).
   *A previous revision of this list retracted the claim in favour of "false at 38–40% on a
   504-module closure." That replacement was also wrong: it rested on the same unretained
   run set as the old §A.7 table, and the committed fourth receipt contradicts it.*
3. **"The duplication factor is the multiplier on the dominant cost," and the per-source
   content-hash memo that followed.** The dominant row is fixed per index — not per entry,
   not per membership — so neither A×B join direction holds, and the memo recommendation
   has no measured support. Withdrawn.

### Does the evidence strengthen, shrink, or close the union program?

**It shrinks the program by eliminating its apparent target, and relocates what remains onto
a claim this slice does not establish.** That is a narrower answer than "not answerable," and
§A.8 is what earned it.

The elimination is the firm part. The union program's implicit target was the row that
dominates every single-entry partition — `load`, at 67–68%. §A.8 measures that row growing
**1.08× while entry count grows 6×** on one shared index. It is a per-index fixed cost that
the existing memo already amortizes; its share falls to 44.97% at N=6 and, on a marked
projection, to a few percent at floor-scale N. **A union graph cannot displace a cost that is
already paid once.** Any case for the program built on §A.4's shares was built on an artifact
of a one-entry instrument, and that case is now closed rather than merely unproven.

What remains is narrower and is *not* established here. `typecheck_compute` is the only
measured row that scales with both entry count and closure size, and its unit of work is
module membership — the quantity §B measures repeating 38× by count and 48× by bytes. That
makes it the only surviving candidate. But the conversion from repeated *membership* to
repeated *computation* is exactly the mechanism this receipt has now failed to establish
twice: once via the refuted content-scan story (§A.6), and once by finding that the row it
would have applied to is fixed (§A.8). **No mechanism is offered here, and the duplication
factor must not be quoted as a multiplier on typecheck time without one.**

The pole that would have closed the program outright — disjoint closures, factor 1.0 — is
still decisively absent, so the program is not dead. It is smaller, pointed at a different
row, and gated on a measurement nobody has taken.

### The one measurement that would decide it

**Per-entry typecheck attribution against a shared environment** — does typechecking entry
B re-do work already done for entry A when their closures overlap, or is each entry's
typecheck specific to its own env such that shared membership implies no shared computation?
That is the question §A.8 promoted from "one of several" to "the only one left," and nothing
in this slice answers it.

A floor-scale partition from the **`claim_executor` discovery path** remains wanted for the
denominators, but it is no longer the deciding measurement — §A.8 obtained the amortization
direction without it. Whole-corpus runs OOM-killed twice on this host (exit 137), so the
floor receipt still does not exist and no share in §A.4 may be quoted as a floor fact.

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
