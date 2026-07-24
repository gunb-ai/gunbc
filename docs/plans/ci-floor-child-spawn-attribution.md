# CI floor time — child-spawn attribution (Pi-3 exaggeration bench × srv1 × CI logs)

**Status:** measurement receipt, 2026-07-23. **DESIGN.md + carriers remain authority** — this is
prose + TSV receipts over execution, **no floor behavior changes** in this PR. Follow-on to the
merged #7106 audit ([`ci-floor-time-45-72-band-attribution.md`](ci-floor-time-45-72-band-attribution.md)):
same mandate, re-run on a **Raspberry Pi 3 used as a slowdown-exaggeration bench** against
**srv1** (Ampere Altra, same class as the CI runners) with release binaries built from the
merged head. Dissolves when the child-spawn class is dissolved (see §8.1) — its counterfactuals
become that PR's before/after.

**Method.** Release bins built on srv1 from the #7106 head, run unmodified on (a) srv1 (125 GB) and
(b) a Pi 3 (aarch64, 905 MB + 12 GB swap). Every number below is **by execution**; CI figures are
re-derived from anchor run `29976989996` (ci job `89110963350`) and trivial-diff run `29970583893`.
The Pi is not a target platform — it is an instrument: a phase that stretches ~8× Pi/srv is
CPU-shaped, one that stretches 25–78× is memory-shaped, and that single signal separates real
mechanisms from plausible-but-wrong ones (§6).

**Carriers (this PR):**

- [`docs/probes/ci_floor_pi_srv_stretch_2026-07-23.tsv`](../probes/ci_floor_pi_srv_stretch_2026-07-23.tsv) — phase × CI-wall × srv × Pi × stretch × shape
- [`docs/probes/ci_floor_child_spawn_counterfactual_2026-07-23.tsv`](../probes/ci_floor_child_spawn_counterfactual_2026-07-23.tsv) — pooled-vs-spawned ledger

---

## 1. What #7106 got right (corroborated by execution)

The phase walls in #7106 §2 match the re-derived log exactly (batch 1 ≈ 10.1m, discovery ≈ 12.8m,
batch 6 ≈ 12.15m, batch 7 ≈ 3.3m); so do `resolves_total = 4` and the materialization receipt
counts (`keyed 2413946 / unkeyed 2193388`). The governor pins the cgroup at its 16 GiB
`memory.high` ceiling by t≈30m and stays there, width = 1 the whole run. And "the band is **not**
4–5× whole-tree cold re-ingests" holds — **for the executor process**. The correction below is
about the work that happens **outside** that process.

## 2. The dominant class #7106 missed: cold child processes (≈48% of the floor)

Every wet gate discharges its witnesses through `run_gunbc_claims`
(`dag/tools/host_prelude.dag:177`, `run_gunbc_claims_ready`), whose fold spawns **one cold
`gunbc` process per `ClaimRun`**. Each child rebuilds the module index and closure resolve from
scratch; each costs ~35–73 s on srv **regardless of which claim it runs** (the cost is
startup + resolve, not gate semantics). None of it is visible to `resolves_total = 4`, which counts
executor-grain resolves by design (the receipt note says as much: "the duplication this lane
still owes lives OUTSIDE this receipt").

| batch | #7106's story | actual mechanism (same log) |
|---|---|---|
| 1 cheap gates (10.1m) | "shared resolve + host-effect scan; wall = **max** of parallel gates" | **12 serial cold children Σ=479.6s** (layering 7/289.8s, extdeps 5/189.8s, drift in-process) + ~2.1m executor gate-entry resolve. Children are **serial** — wall is the **sum** |
| 6 source_root_ingest (12.15m) | "separate bin re-walks the tree; module-identity lane" | the `discover_source_root_ingest` **bin costs 0.008–20s** (it reads 2 fixture files, not the tree; <1s even on a Pi 3). The 12.15m is **12 cold children across 3 sub-gates Σ≈702s** |
| 7 reads_real_bytes (3.3m) | "heavy resolve" | **2 cold children (119.6s + 78.3s) = 100% of the wall** |

**26 cold children across batches 1/6/7 ≈ 1379.6s ≈ 23.0 min ≈ 47.5% of the 48.4-min floor step.**
Identical on trivial diffs (run `29970583893` batch 5 = 12 children Σ629s with compile-clean
skipped) — a **diff-independent floor tax**.

This is the **third instance of one root — "cold index per execution unit"**:

- *per-thread* — fixed by #7030 (the `heavy_whole_tree_resolve` profile flips)
- *per-repeated-entry* — being fixed on `session/double-resolve-rewire`
- *per-child-process* — **unaddressed, and the largest**

Fixing them one instance at a time is the forked-logic trap (DESIGN §6). The root fix is the one
`process_shared_index` authority every execution unit references — the `ci_floor_resolve_receipt_note`
terminal, "resolve once, share by reference."

## 3. The measured counterfactual — and the Pi's load-bearing correction

| workload | CI (spawned) | srv1 pooled (1 process) | Pi spawned (est) | Pi pooled (measured) |
|---|---:|---:|---:|---:|
| batch-1's 12 claims | 479.6s | **93.4s** (5.1×) | ~66–78m | **72.9m** |
| ingest gate's 3 real-ingest claims | 190.5s | **106.5s** (1.8×) | — | 44.0m |
| single child (clean_tree) | 49.8s | 47.2s | — | 6.5m (8.2×) |

**srv verdict:** pooling each gate's `ClaimRun`s into one process displaces **~13–17 min/run**
(marginal warm resolve ≈ 0 ms/claim); running them against the executor's already-warm index
displaces more.

**Pi verdict (the exaggeration bench earning its keep):** *pooling's 5.1× win is a warm-cache
win, not a work-reduction win, and it vanishes at the memory wall.* Pi pooled (72.9m) ≈ Pi
spawn-sum (66–78m): the pooled process's unioned closure (VmSize 2.7 GB) swap-thrashes what 12
small sequential children avoid. The 16 GiB-capped fleet runners sit on the same cliff (the
executor already pins the cgroup). **So the child-spawn fix pays on the CI wall — but only because
the CI runner has 125 GB; on a capped runner it helps only when paired with per-worker index
footprint reduction** (the eviction / M2 lane, ROADMAP ①). This is a §5 caution: a warm-cache
speedup measured on the build box must not be allowed to mask the footprint deficit on the capped
runner.

## 4. Corrections to #7106's ranked levers

1. **Lever #1 (per-entry bare-reference fixpoint, "8–12 min") is stale.** Both dissolution PRs
   (#7030, #7056) are **ancestors of the measured run's commit** (`6c01a58`), yet discovery is
   still ~971 ms/group (5× the pre-#6848 ~200). The fixpoint is already dissolved and recovered
   nothing (the #6999 pattern repeating). The 643s discovery resolve — eval is only **18.4s** of
   it — needs **fresh** attribution; the candidate is the 468-row affected-closure resolve, not
   the fixpoint. (Also: 1738/2206 rows *are* skipped without resolve, so #7106's "971 ms/group"
   divides by the wrong denominator — per **affected** row it's ~1.37s.)
2. **Lever #2 (source_root_ingest re-walk) targets the wrong mechanism.** The ingest bin reads two
   fixture files in 0.008s (sub-second on a Pi 3). The 12 min is the child-spawn class of §2. The
   module-identity lane would displace ≈ nothing here.
3. **Lever #3 (cheap-gate scan) is 12 child spawns**, not a scan — pooling removes most of it.
4. **§3.1's per-gate split needs no GANTT replay** — it is already in every run's log (`env -C`
   child walls, in order): layering 4.8m / extdeps 3.2m / drift ≈ 0.

## 5. Two floor costs with no ledger row

- **Post-floor selection control — 4m51s/PR.** `floor_skip_discovery_witness` runs 04:17:21→04:22:12
  every PR. Real wall, no redundancy-ledger row.
- **Teardown of the retained store — ~2.5m.** After the last gate PASS (04:14:44) the process
  spends ~2.5 min in `Drop` while swap grows **5.5→9.4 GB** and `high_events` 2792→4785 — the
  16 GB store paid for a second time to free it (ROADMAP ①, "paid twice"; the srv-scale twin of
  the 40-min Pi teardown thrash documented on this bench). Together these close most of #7106 §2's
  2.7-min unaccounted gap.

## 6. The bench as an instrument (validated)

The stretch factors are diagnostic, not noise: CPU-bound work stretches ~8× (child resolve
389s/47.2s), swap-bound work stretches 25–78× (pooled closures, witness eval over whole-tree
scans). The clearest case is the compiler's **emit** stage: on srv it is ~20–29 s (invisible next
to reconcile); on the Pi it is **2.7 hours — ~337×** (dag-serialize alone 2.5 h), against ~8× for
frontend and ~19× for reconcile. Emit is the most memory-shaped compiler stage and a latent wall
as the emitted corpus grows or worker width rises — worth a footprint probe before it becomes
one. (Whole-tree compile-clean is otherwise **84% reconcile**: 175.7s of a 209.5s 3-root compile,
5.9 GB RSS.)

## 7. Side finding (needs root-cause; **not** a floor-time claim)

Standalone `gunbc compile --target dag` over the three witness-layer roots (`dag`, `src/v2`,
`src/v1` — the exact `compile_clean_source_roots()` set) exits **rc=1 with 2652 `UnlistedImportUse`
errors** on the tree CI greens (identical count at 2 and 3 roots). CI's compile-clean gate does
**not** exercise this path: it consumes `claim_executor`'s internal shared-index compile receipt
(`run_clean_tree_compile` → `consume_floor_compile_clean_gate_verdict`), and the CLI transport
(`run_clean_tree_compile_typed` / `run_dag_compile_clean_gate_shell`) has no in-tree caller.
`UnlistedImportUse` is precisely the resolver class the namespace-resolution lane is mid-promotion
on (`namespace_import_closure_behavioral_transport.dag:15`: "DISSOLVES WHEN … UnlistedImportUse
promoted to an error"). So two "compile-clean" realizations apply **opposite policy to the same
not-yet-promoted class**. Whether this is a benign staging artifact (the CLI ahead of the resolver
promotion) or a fail-open fork (the floor's green depending on **not** enforcing what the CLI
enforces, §5) is undetermined and worth a dedicated look. It does not affect §2–§5.

## 8. Recommendations

### 8.1 Dissolve the child-spawn class — the single largest lever

Convert `run_gunbc_claims_ready` from one-cold-process-per-`ClaimRun` toward the shared-index
terminal, in two honest shapes:

- **Interim — pool per gate.** One `claim_batch` process per gate carrying all its `ClaimRun`s
  (measured 5.1× on srv; ~13–17 min/run). Landable now; a strict improvement on the 125 GB CI
  runner.
- **Terminal — run gate claims against the executor's warm `process_shared_index`.** The
  `ci_floor_resolve_receipt_note` "resolve once, share by reference" end-state; folds the
  per-child, per-thread (#7030), and per-repeat (`session/double-resolve-rewire`) instances onto
  **one** authority instead of a third point-fix.
- **Gate it on footprint (§5, load-bearing).** Pair either shape with per-worker index-footprint
  reduction, or it will not help the capped fleet runners — the Pi proves the pooled closure
  swap-thrashes exactly where the win is needed. Land the counterfactual as the before/after and
  this receipt dissolves.

### 8.2 Re-open the discovery 643s as an unknown

Retire the stale "per-entry fixpoint" lever; the fix landed with no recovery. Attribute the 643s
fresh (eval is 18.4s — it is all resolve) against the 468-row affected closure, not the corpus
size.

### 8.3 Add the two unledgered costs to the redundancy ledger

The 4m51s selection-control step and the ~2.5m teardown are real, recurring, and currently
invisible to the ledger — give them rows so they can be priced and prioritized (DESIGN §6).

### 8.4 Keep a footprint probe on the emit stage

The 337× Pi stretch flags emit as the compiler stage most likely to become the next memory wall.
A cheap standing footprint probe (emit RSS vs emitted-module count) would catch it before the
fleet does.

---

## 9. Provenance

- 2026-07-23. Release bins built on srv1 from the #7106 merged head; run on srv1 and a Pi 3.
- CI figures re-derived by execution from runs `29976989996` (anchor) and `29970583893` (trivial).
- Related: [`ci-floor-time-45-72-band-attribution.md`](ci-floor-time-45-72-band-attribution.md) (#7106, the audit this corroborates and corrects),
  [`floor-time-namespace-walk-regression-diagnosis.md`](floor-time-namespace-walk-regression-diagnosis.md),
  [`floor-shared-compute-memoization.md`](floor-shared-compute-memoization.md),
  [`v1-run-stability-throughline.md`](v1-run-stability-throughline.md).
