# Postmortem — "CI is extremely flakey": the fleet-overcommit root cause

**Date:** 2026-06-20
**Author:** warm-crane-135 (with eager-boar-790, vivid-bee-801)
**Status:** root cause identified; one structural fix merged (#5375), the load-bearing fix still open (host-level admission)
**Trigger:** operator report — *"CI is extremely flakey"* and, hours later, *"srv2 is crashing every ~hour … my concern is that we are treading water."*

---

## TL;DR

CI flakiness was **not one bug**. It was four compounding failures stacked on a single
under-modeled fact, and we spent real effort chasing the *wrong layer* of it more than once. The
deepest root cause is:

> **There is no authority bounding the aggregate memory of concurrent CI runs on a host.**
> Per-run resource use is now modeled (#5375); the *sum over runs sharing a host* is not. srv2 ran
> 7–11 floor runs at once, their combined peak exceeded 125 GiB physical RAM, and the host
> livelocked before any single per-run cap could fire.

Everything else — the memory-blind floor width, the WIP-merge breakage, sccache corruption, the
approval-reset livelock — was either a contributing flake or a *symptom of the same missing model
seen at a different scale*. The "treading water" feeling was real and diagnostic: we kept fixing the
**per-run** scale while the failure lived at the **per-host** scale.

---

## The one-sentence root cause, stated as a model

It is the **same inequality at two scales** (DESIGN §2 horizontal — one concept, every scale), and
only one scale was ever modeled:

| scale | inequality | status |
|---|---|---|
| **per-run** | `width × per_unit_peak ≤ per_run_budget` | **modeled** — `placement_spawn_width`, merged in #5375 |
| **per-host** | `Σ_concurrent_runs(per_run_peak) ≤ host_physical_RAM` | **UNMODELED** — this is the crash |

A second, orthogonal §3 (single-authority) violation hides inside the per-run term: `per_unit_peak`
is a **hand-set constant** (`14 GiB/unit` in `dsl/gunbc/ci_fleet.dag`), calibrated once against
*main's* corpus. A constant is a second authority for "what the corpus costs" that silently forks
from the actual corpus, so any corpus growth outruns it. It must become **probe-derived**:
`WorkDemand.memory = f(corpus)`.

---

## Timeline & the chase (where we burned time, and why)

### 1. Symptom: "CI is extremely flakey"
PR CI ran 30–45 min and failed intermittently with unrelated errors each retry. Initial surface
read found **four** distinct contributors, which is why no single fix "stuck":

- **WIP/partial merges breaking main.** A `bash_lex_rule` dangling ref from a WIP commit (#5347)
  made main red tree-wide; PR branches inherited it via main-merge and looked like *their* bug.
  → *Lesson already in memory: a floor resolve-fail in a file your PR didn't touch usually means
  main is already red.*
- **sccache corruption.** `"failed to fill whole buffer"` → spurious build failures naming a
  different dep each retry. Worse: `ctrl-build` returned **exit 0 with no binary** (false green).
  → *Always `ls` the artifact; `--stop-server` + `RUSTC_WRAPPER="" SCCACHE_DISABLE=1` to bypass.*
- **Memory-blind floor width.** The CI floor fanned out batch-2 at `width = min(breadth, threads) = 7`
  with **no memory term** → deterministic exit-137 OOM as the corpus grew.
- **srv2 crashing every ~hour**, cancelling in-flight CI (the `cancelled`-not-`failed` runs that the
  dashboard surfaced as "1 failing").

### 2. First root-cause attempt: the per-run width (#5375) — correct, but not the crash
We modeled the missing memory term: `placement_spawn_width(supply, demand) = min(cpu_width,
mem_budget / per_unit_peak)`, dropping batch-2 from width-7 to width-4, with a discriminating §5
witness (`witness_floor_width_fits_memory_budget`) that goes red if the memory term is removed.
**This was a real, correct structural fix** — but it bounds *one run's* fan-out, not the host.

### 3. srv2 crash diagnosis: hardware ruled out, OS memory-exhaustion confirmed
Via BMC Redfish (OpenBMC on ASRock ALTRAD8UD): `LastResetTime` stale, `FaultLog` empty, Core/DIMM
temps 60/58 °C, Core power 72 W — **not** thermal, not a chassis power event, not a BMC hard reset.
The OS rebooted itself without tripping any chassis event ⇒ **OS-level memory-exhaustion livelock**.
The smoking gun was OS-side: wall-to-wall `systemd-journald: Under memory pressure, flushing caches`
until an abrupt stop with no clean OOM-kill line — so memory-starved it livelocked before it could
log its own death. `journalctl --list-boots`: stable ~2 days/boot until 2026-06-20 ~00:15, then
crashing every ~3 h — i.e. a **load surge today** (many concurrent agent sessions + CI floors on one
host), not a hardware regression.

### 4. The head-fake (and the §5 recovery that found the real cause)
eager-boar-790 tested Arc1+#5375 and saw it **still OOM at width-4**. The inversion was striking:
main passed at width-7, but Arc1 OOM'd at width-4 — *fewer* concurrent resolves, yet OOM. The
natural hypothesis: Arc 1's eager materialization ballooned per-unit memory, so `4 × (big unit) >
7 × (small unit)`. We (warm-crane) endorsed this and eager-boar agreed to slim Arc 1.

**Then eager-boar checked the actual diff (DESIGN §5 — green/red by execution, not by plausibility)
and refuted their own hypothesis:**
- Arc 1's whole diff is +39 net lines in `typescript.dag` — a fn-refactor extracting a shared
  dispatcher, **no new eager decls**; the heavy whole-corpus self-emit witness was *already dropped*.
  It adds ≪1 % to the 2968-item shared resolve closure. It **cannot** account for GBs of per-unit.
- The arithmetic refutes per-unit too: at 12.5 GiB/unit, width-4 ≈ **50 GiB** — well **under** srv2's
  ~87 GiB cgroup cap. An OOM at ~50 GiB cgroup usage means the kill came from **host physical-RAM
  exhaustion** (multiple concurrent runs), not the run exceeding its own budget.

That is the pivot. The per-run model was never going to explain a host-level crash.

---

## What was actually fixed vs. what remains

### Fixed / in motion
- ✅ **#5375 merged** — memory-aware per-run width (7→4). Bounds the CI floor's *own* per-run
  footprint. Correct and load-bearing for the per-run scale; **not** the crash fix.
- ✅ **WIP-merge breakage** — dangling `bash_lex_rule` resolved; main green.
- 🔁 **sccache** — server stopped on both hosts. **Still TODO:** purge the on-disk cache
  (`rm -rf "${SCCACHE_DIR:-$HOME/.cache/sccache}"`) — `--stop-server` kills the process but leaves
  corrupt objects on disk for the next server to re-serve.
- ✅ **Reactive host backstop** — operator set `MemoryMax` on `system-actions-runner.slice` (kills
  one cgroup instead of livelocking the box). Mitigates, does not prevent.

### Open — the load-bearing fixes (both ours/ctrl, not corpus)
1. **Per-host admission control (THE crash fix, unmodeled).**
   `Σ_concurrent_runs(per_run_peak) ≤ host_physical_RAM`. Concretely: bound `CTRL_JOBSERVER_TOKENS`
   / runner slots so the host *cannot* admit more concurrent runs than fit in physical RAM. Today
   nothing enforces this; srv2 admitted 7–11 runs and overcommitted.
2. **Probe-derived per-unit memory (§3 single-authority).**
   Replace the hand-set `14 GiB/unit` constant in `ci_fleet.dag` with a peak-RSS-probe-derived
   `WorkDemand.memory = f(corpus)`. A constant silently forks from the corpus it's meant to describe;
   any growth outruns it (it nearly bit Arc 1 for the wrong reason). eager-boar offered to be the
   first consumer of the probe.
3. **srv1 idle (orthogonal 2× capacity).** srv1 sits at ~5 % while srv2 takes all the load. If srv1
   picked up half the floors, srv2's aggregate would stay under the wall even at today's per-run
   footprint. Why its runners are idle is not yet diagnosed (registration/labels/host-assignment).

---

## Root-cause analysis: why this *felt* like treading water

The "error after error" experience was structurally honest feedback, not bad luck:

1. **Four independent flakes shared one symptom** ("CI fails"), so each fix appeared not to work —
   another contributor was still firing. (Fixing main's red doesn't stop sccache; fixing sccache
   doesn't stop the crash.)
2. **We fixed the wrong scale twice.** The memory-blind *width* was a real bug, so fixing it felt
   like progress — but the crash lived at the *host* scale, which #5375 doesn't touch. Then the
   *per-unit* head-fake re-aimed at the corpus, still the wrong scale. Only checking the artifact
   (the diff + the arithmetic) broke the loop. **This is the §6 trap: a local-subsystem patch when
   the root is one layer up (or, here, one scale up).**
3. **An operational livelock on top of the technical one.** #5375 itself couldn't land: the
   auto-committer kept merging main into its branch (resetting earned approvals to 0), and the srv2
   crash loop kept cancelling its CI. A PR that *fixes a crash contributor* was blocked by the exact
   instability it fixes. Broken only by the operator merging on strength.

The meta-lesson, in DESIGN terms: **we kept modeling the per-run inequality (§2 horizontal at one
scale) and never lifted it to the host scale.** The same concept — "demand must fit supply" — applies
at run, host, and fleet; modeling it at only the innermost scale guarantees the outer scale fails
silently. The fix is not more patches; it's **one admission model expressed at every scale from one
authority** (the §2 master move), with the per-unit term **grounded by probe** rather than guessed.

---

## Concrete next actions

- [ ] **Model per-host admission** as the outer instance of the same inequality, derived from the
      same fleet authority as `placement_spawn_width`. (warm-crane — Task #2/#4.)
- [ ] **Cap `CTRL_JOBSERVER_TOKENS` / runner slots** on both hosts so admitted concurrency × per-run
      peak ≤ physical RAM. Generate the cap from the model, don't hand-set it (closes the drift that
      let srv2's slice run looser than srv1's).
- [ ] **Probe-derive `WorkDemand.memory`** (peak-RSS per resolve) → retire the 14 GiB constant.
- [ ] **Purge sccache on-disk cache** on both hosts.
- [ ] **Diagnose srv1 idle runners** (2× capacity, orthogonal but high-value).
- [ ] **Watch main's own post-#5375 floor (`637e4a9b2c`)** at width-4: if it also OOMs with no Arc 1
      present, that is the definitive witness that the failing axis is host-concurrency, not corpus.

---

## Appendix — evidence pointers

- BMC: `https://192.168.1.184` (srv2) / `.183` (srv1), OpenBMC on ASRock ALTRAD8UD; chassis id
  `ALTRAD8UD_1L2T`. Use `curl --netrc-file` (0600), never `-u` on argv. **BMC credentials never go in
  any repo.**
- Per-run model: `dsl/product/compute_fabric.dag` (`placement_spawn_width`,
  `placement_memory_width_bound`), `dsl/gunbc/ci_fleet.dag` (the `14 GiB/unit` constant +
  `ci_fleet_floor_spawn_fits_memory_budget` oracle), `src/v2/workflow/ci_floor_plan.dag`.
- §5 witnesses: `src/v2/test/claim/ci_floor_plan_witness_test.dag`
  (`witness_floor_width_fits_memory_budget`, `witness_floor_width_memory_bounded`).
- The §5 recovery that found the root cause: eager-boar-790 refuting its own per-unit hypothesis by
  reading the Arc 1 diff and the cgroup-usage-vs-cap arithmetic.
