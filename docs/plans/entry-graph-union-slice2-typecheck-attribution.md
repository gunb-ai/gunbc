# Receipt — entry-graph-union slice 2: repeated-typecheck attribution

**Status:** measurement instrument landed 2026-08-01 (PR #7533). **DESIGN.md + slice 1 remain the authority** — this is a dated receipt scaffold, not a floor fact ledger. Dissolves with `cli_run_repeated_typecheck_attribution_probe` when the slice-2 union verdict is taken.

**ROADMAP:** `gunbc.roadmap_authority` id `entry-graph-union-construction` (lane `ci-cost`, slice 2 of the same row as slice 1).

**Lane:** `ci-cost` · **Subject:** `entry-graph-union-slice2-typecheck-attribution` · **Deliverable:** measurement only. No union implementation.

**Predecessor:** [entry-graph-union slice 1](entry-graph-union-slice1-measurement.md) (#7483).

---

## A — what slice 1 left open

Slice 1 established that production-selected closures repeat **module membership** at 35.9–38.4× by count (43–49× byte-weighted), while the dominant single-entry `load` row is `build_both_closure_edge_index` — already paid once per `MultiEntryIndex` and flat in entry count (1.08× over N=2→7). It explicitly **withdrew** quoting membership duplication as a typecheck-time multiplier.

The operator question slice 2 answers: **how often does a repeated module membership cause a real `typecheck_compute` miss, and how much time do those repeated misses consume?**

---

## B — the instrument

**Probe:** `measure_repeated_typecheck_attribution` (`src/v1/stage0/src/bin/measure_repeated_typecheck_attribution.rs`)

**Mechanism:** production-selected entries (same machinery as slice 1) resolved sequentially against **one** `build_multi_entry_index` shell. Attribution hooks in `reconcile_with_typed_cache` and `try_reconcile_all_cache_hits` are **flag-gated** (`arm_repeated_typecheck_attribution_probe`); default resolve flow is unchanged.

### Per (entry, typed module content key)

| field | meaning |
|---|---|
| `entry` | witness entry path |
| `module_key` | `typed_module_content_key` (never bare module name) |
| `module_path` | authored name (readability) |
| `in_closure` | always `true` when observed from reconcile |
| `cache_disposition` | `Hit` \| `Miss` \| `Refused` (three states, never two) |
| `typecheck_compute_ns` | wall on `Miss` only; `0` on `Hit` |
| `first_computing_entry` | entry that first computed this key in this run |
| `later_requester_count` | prior entries that already observed this key |

### Per entry

`entry_timings[]`: `reconcile_assembly_ns`, `typecheck_compute_ns`, `resolve_nanos` from `ResolveStageNanos`.

### Aggregates

- selected entry count, `sum_closure_memberships`, `union_modules`, membership duplication factor (slice-1 comparison)
- `total_cache_hits` / `total_cache_misses` / `total_cache_refusals`
- `repeated_typecheck_misses` — misses with `later_requester_count > 0`
- `first_computation_typecheck_ns` / `repeated_typecheck_compute_ns`
- `fanout_by_module` (from slice-1 overlap probe)
- `memory.process_vm_hwm_bytes` — process VmHWM at measurement end (includes selector/index setup, not an attribution-only peak)
- `memory.rss_before_measurement_bytes` / `memory.rss_after_measurement_bytes` — scoped VmRSS bracketing the resolve loop
- `memory.cgroup_memory_events_high` — leaf cgroup `memory.events` high counter when readable

### The decision quantity

```
decision_ratio = repeated_typecheck_compute_ns / total_typecheck_compute_ns
```

**NOT** closure duplication. Low → cache already eliminates repeated work (union program shrinks). High → repeated membership becomes repeated typecheck (union/shared-compute program justified).

Diagnostic only: `cache_hit_ratio = hits / (hits + misses)`.

---

## C — controls (proven by execution)

| control | witness |
|---|---|
| Reordering entries preserves distinct compute set | `repeated_typecheck_attribution_reorder_preserves_distinct_computes` (union_resolve_receipts_test) |
| Shared prefix observed as cache hit on second entry | `repeated_typecheck_attribution_records_shared_prefix_hits` |
| Hits carry zero `typecheck_compute_ns` | `cache_hits_carry_zero_typecheck_compute_ns` (cli_run arithmetic) |
| Refused is third state, not counted as hit | `refused_rows_are_not_counted_as_hits` |
| Decision ratio arithmetic | `perfect_sharing_yields_zero_decision_ratio`, `repeated_recompute_raises_decision_ratio` |

---

## D — host constraints (carried from slice 1)

- Whole-corpus discovery OOMs on dev hosts (exit 137 at 1,046/1,513 entries). Use `--max-entries N` explicit entries against one shared index.
- Do not rebuild the binary mid-probe (resolve cache content-addresses executable).
- M2 retention (#7129) on main — re-verify OOM binding before floor-scale runs.

---

## E — reproducing

```sh
# Fixture-scale (fast, no git diff required):
cargo test -p v1-compiler-tests repeated_typecheck_attribution

# Production-selected set (cap entries on dev hosts):
GUNBC_CI_DIFF_BASE=<base-sha> measure_repeated_typecheck_attribution \
  --source-root dag --source-root src/v2 \
  --scan-dir dag/test/claim --scan-dir src/v2/test/claim/manual \
  --max-entries 6
```

Emits `[typecheck-attribution-measurement] {json}`.

---

## F — execution matrix (2026-08-01, branch `78ccb693`, binary `target/slice2-measurement-bin/`)

**Verdict slot intentionally empty** — numbers below; three-way handback (union justified / attack per-entry assembly / close program) deferred to operator.

### Host envelope / governor

| field | value |
|---|---|
| cgroup `memory.max` | 33,578,549,248 bytes (~31.3 GiB) |
| `GUNBC_MEMORY_BUDGET_BYTES` | unset |
| probe scheduling | serial, one shared `MultiEntryIndex`; adaptive `MemoryGovernor` **not engaged** |
| `cgroup_memory_events_high` | 0 on every run |

**Retraction:** uncapped narrow production (`N=286`, `GUNBC_CI_DIFF_BASE=e30621111f37`) was **OOM-killed** at the cgroup ceiling after ~31 min (exit `Killed`, no receipt). Production subjects below use `--max-entries 50` (production-selection order preserved, first 50 of each subject's roster).

Binary snapshot: `sha256:8af9c72606aba8373c611c9608e4402bd6c7a4308862f80536af2a381a79174e`.

### Matrix handback

| run | N | cap | diff base | distinct module keys | distinct typecheck computes | repeated typecheck computes | repeated wall / total typecheck wall | decision_ratio | cache_hit_ratio | Σ assembly wall | VmHWM |
|---|---:|---|---|---:|---:|---:|---|---:|---:|---:|---:|
| explicit-order-a | 7 | — | — | 398 | 398 | **0** | 0 / 14.0s | **0** | 0.362 | 6.0s | 3.11 GiB |
| explicit-order-b | 7 | — | — | 398 | 398 | **0** | 0 / 13.3s | **0** | 0.362 | 5.6s | 3.05 GiB |
| narrow-production | 50 | 50 | `e30621111f37` | 622 | 622 | **0** | 0 / 36.6s | **0** | 0.884 | 38.2s | 8.30 GiB |
| typical-production | 50 | 50 | `b01cdf4d8914` | 622 | 622 | **0** | 0 / 32.4s | **0** | 0.884 | 33.9s | 8.29 GiB |
| broad-capped | 50 | 50 | `0d6ffc4db975` | 622 | 622 | **0** | 0 / 34.7s | **0** | 0.884 | 38.3s | 8.27 GiB |

Membership at `N=50`: `sum_closure_memberships=5339`, `union_modules=619`, duplication **8.63×** (vs slice-1 full-roster 35.9–38.4× — cap shrinks overlap denominator).

### Structural reads (execution, not verdict)

1. **Reorder invariance holds at execution scale:** explicit A vs B — identical distinct computes (398), identical repeated misses (0), identical membership; only per-entry assembly/typecheck walls reorder.
2. **Repeated membership does not become repeated typecheck** on any matrix cell: `repeated_typecheck_misses=0` and `decision_ratio=0` throughout, while `cache_hit_ratio` rises to **0.884** at `N=50` (membership duplication 8.6× still present).
3. **Per-entry assembly is non-trivial** relative to typecheck at production cap: Σ `reconcile_assembly_ns` ≈ 34–38s vs total typecheck ≈ 32–37s for `N=50` — the lane's alternate target if union is not justified by repeated compute.
4. **Cap artifact:** at `--max-entries 50` the three production subjects selected the same first-50 roster prefix (alphabetical production order), so narrow/typical/broad rows coincide on attribution — breadth contrast requires uncapped runs on a host that survives `N≈300`.

Full per-row receipts: `docs/plans/receipts/entry-graph-union-slice2/receipt-*.json`. Reproduce: `docs/plans/receipts/entry-graph-union-slice2/run_matrix.sh`.

### Verdict slot

*Empty — operator three-way handback pending.*
