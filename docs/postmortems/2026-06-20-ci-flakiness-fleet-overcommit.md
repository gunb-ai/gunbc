# Postmortem — "CI is extremely flakey": root-causing by execution

**Date:** 2026-06-20
**Author:** warm-crane-135 (with eager-boar-790, vivid-bee-801)
**Status:** root cause identified by execution; one structural fix merged (#5375); the proven fix
(Arc 1 per-unit) is in progress; a real latent gap (per-host admission) is modeled but was NOT this bug.
**Trigger:** operator report — *"CI is extremely flakey"* and, hours later, *"srv2 is crashing every
~hour … my concern is that we are treading water."*

> **Note on this document (it is the lesson):** the root cause below was reached only after *three*
> hypotheses were proposed and two were refuted — and every correction came from **discriminating
> execution overruling plausible reasoning** (DESIGN §5), including refuting a hypothesis this very
> doc asserted in its first draft. The thrash is not noise to be cleaned up; it *is* the finding. A
> postmortem that hid it would repeat the mistake it documents.

---

## TL;DR

CI flakiness was **not one bug**. It was several independent flakes sharing one symptom ("CI fails"),
plus one OOM whose cause took three tries to pin. The decisive evidence is a three-run table (below):
at the same fan-out width, in the same time window, the **main corpus passed on both hosts** while
**one branch's corpus was OOM-killed**. That isolates the cause to *that branch's content*, and rules
out every host-level and width-level explanation.

- **The OOM (proven):** a specific branch (Arc 1) inflated **per-unit resolve memory** past the
  hand-set `14 GiB/unit` estimate, so even at the reduced width-4 it exceeded the runner cgroup cap
  and was killed (exit 137). This is the **per-run / per-unit** scale.
- **The flakes (separate):** WIP-merge breakage, sccache corruption, and the crash-cancel loop each
  produced "CI fails" independently, which is why no single fix made the symptom disappear.
- **srv2's hourly host reboots (separate, under-diagnosed):** OS-level memory-exhaustion livelock
  (hardware ruled out via BMC), but **not** execution-pinned to floor concurrency — a strong
  candidate is the uncapped agent containers, not the CI floor.

Two hypotheses were proposed and **refuted by execution**, and naming them is the point:
1. *(refuted)* "Arc 1's diff is too small to matter; it's not per-unit." → refuted: the +39-line diff
   reasoning lost to the run that OOM'd.
2. *(refuted, and it was THIS doc's first headline)* "It's per-host overcommit — sum of concurrent
   runs exceeds RAM." → refuted: main-corpus runs at the same width in the same window passed on both
   hosts; overcommit would have taken them too.

---

## The decisive evidence (why we know)

| run | corpus | width | host | result |
|---|---|---|---|---|
| `637e4a9b2c` (main post-#5375) | main | 4 | srv1-32 | **PASS** |
| `27859412294` (#5375 branch) | main | 4 | srv2-17 | **PASS** (533 witnesses) |
| `27859296640` (Arc1+#5375) | **Arc 1** | 4 | srv2-15 | **Killed (exit 137)** |

All three are width-4, overlapping ~04:00–04:14. The two **main-corpus** runs passed on **both** hosts.
Therefore:
- It is **not width** — width-4 passes for main.
- It is **not the host** (overcommit/crash-load) — same window, same hosts, main passed.
- The **only** remaining variable is **Arc 1's corpus content**.

This is a textbook discriminating experiment: hold width and host fixed, vary only the corpus, observe
the flip. It overruled two rounds of plausible static reasoning.

---

## Root cause, stated as a model

The OOM lives at the **per-run** scale, and the failure was a **stale single-authority constant**, not
a missing model:

`per_run_peak = width × per_unit_peak`, and the run is killed when `per_run_peak > cgroup_cap`.

The width term is now modeled and correct (#5375). The defect is `per_unit_peak`: it is a **hand-set
constant** (`14 GiB/unit` in `dsl/gunbc/ci_fleet.dag`), calibrated once against *main's* corpus
(measured ~12.5 GiB/unit, padded to 14). A constant is a **second authority** for "what the corpus
costs" (DESIGN §3) that silently forks from the actual corpus — so when Arc 1's content pushed the
*real* per-unit past 14, the budget held a line the corpus had already crossed, and width-4 OOM'd. The
fix is to make per-unit **probe-derived**: `WorkDemand.memory = f(corpus)` via a peak-RSS probe, so the
estimate cannot fork from the corpus it describes. (eager-boar-790 offered to be the first consumer.)

### The latent gap that is NOT this bug (kept, demoted)

There genuinely is no authority bounding `Σ_concurrent_runs(per_run_peak) ≤ host_physical_RAM` — the
**per-host admission** scale. It is the same `demand ≤ supply` inequality one scale up (DESIGN §2
horizontal), and worth modeling so a future load surge can't oversubscribe a host. But the execution
table shows it did **not** cause this OOM, so it is recorded here as a **modeled latent risk**, not the
root cause. Modeling a plausible gap and *asserting it caused an observed failure* are different claims;
only the second needs a discriminating witness, and here that witness refuted it.

---

## Timeline & the chase (where time went, and why)

### 1. Symptom: "CI is extremely flakey"
Four independent contributors shared one symptom, which is why no single fix "stuck":
- **WIP/partial merges breaking main.** A dangling `bash_lex_rule` ref (#5347) reddened main
  tree-wide; PR branches inherited it via main-merge and looked like *their* bug.
- **sccache corruption.** `"failed to fill whole buffer"` → spurious build failures; worse,
  `ctrl-build` returned **exit 0 with no binary** (false green). *Always `ls` the artifact.*
- **Memory-blind floor width.** batch-2 fanned out at `min(breadth, threads) = 7` with **no memory
  term** → OOM risk as the corpus grows.
- **srv2 crashing ~hourly**, cancelling in-flight CI (the `cancelled`-not-`failed` runs the dashboard
  surfaced as "1 failing").

### 2. #5375 — memory-aware per-run width (correct, merged)
`placement_spawn_width = min(cpu_width, mem_budget / per_unit_peak)`, dropping batch-2 to width-4, with
a discriminating §5 witness. A real, correct fix at the per-run scale — it just isn't what failed for
Arc 1 (Arc 1 OOM'd *at* width-4).

### 3. srv2 crash diagnosis: hardware ruled out
Via BMC Redfish (OpenBMC on ASRock ALTRAD8UD): `LastResetTime` stale, `FaultLog` empty, Core/DIMM temps
60/58 °C, Core power 72 W — not thermal, not a chassis power event. OS rebooted itself without tripping
any chassis event ⇒ **OS-level memory-exhaustion livelock**. Smoking gun OS-side: wall-to-wall
`systemd-journald: Under memory pressure` until an abrupt stop with no clean OOM line. Stable ~2
days/boot until 2026-06-20 ~00:15, then crashing every ~3 h — a **load surge today**. (Mechanism not
pinned to a specific workload by execution; the uncapped agent containers are a strong candidate.)

### 4. Three hypotheses for the Arc 1 OOM — execution settled it
- **H1 (per-unit):** "Arc 1's eager materialization balloons per-unit." Proposed by warm-crane,
  endorsed by eager-boar.
- **H2 (refutes H1, by diff):** eager-boar reads the Arc 1 diff — +39 lines, a fn-refactor, *no new
  eager decls* — and argues it can't be GBs of per-unit; reframes as **per-host overcommit** (OOM at
  ~50 GiB under an ~87 GiB cap ⇒ host RAM exhaustion). **warm-crane wrote this into the first draft of
  this doc as the root cause.**
- **H3 (refutes H2, by execution):** the three-run table. Two main-corpus runs pass at width-4 on both
  hosts in the same window; only Arc 1 dies. Host-overcommit would have killed the main runs too.
  **Back to H1** — Arc 1's content is the delta; the diff-based refutation of H1 lost to the run.

The lesson is in the *direction* of each correction: static reasoning (diff size, back-of-envelope
arithmetic) twice produced a confident wrong answer, and **a discriminating run twice overruled it** —
including overruling this document. That is DESIGN §5 operating on the postmortem itself.

---

## What was fixed vs. what remains

### Fixed / in motion
- ✅ **#5375 merged** — memory-aware per-run width. Correct for the per-run scale; bounds the floor's
  own fan-out. (Not the Arc 1 fix — Arc 1 OOM'd at the reduced width.)
- ✅ **WIP-merge breakage** — dangling `bash_lex_rule` resolved; main green.
- 🔁 **sccache** — server stopped both hosts. **TODO:** purge the on-disk cache
  (`rm -rf "${SCCACHE_DIR:-$HOME/.cache/sccache}"`) — `--stop-server` leaves corrupt objects on disk.
- ✅ **Reactive host backstop** — operator set `MemoryMax` on `system-actions-runner.slice`.

### Open
1. **Arc 1 per-unit (THE proven OOM fix).** eager-boar is measuring Arc 1's resolve RSS vs a baseline
   main test file to pin the mechanism (resolve-time eager rows inflating the `typescript.dag` closure
   that every batch-2 node loads, vs witness-exec-time), then slimming the actual cost. **Until the
   RSS delta is measured, the mechanism is not yet pinned** — only the *attribution to Arc 1's content*
   is proven.
2. **Probe-derived per-unit (§3 single-authority).** Replace the `14 GiB/unit` constant with
   `WorkDemand.memory = f(corpus)`. Would have caught Arc 1 automatically; closes the forking constant.
3. **Per-host admission (latent gap, model it).** `Σ_concurrent_runs(per_run_peak) ≤ host_RAM_budget`
   as the outer instance of the same inequality, generating the `CTRL_JOBSERVER_TOKENS`/slot cap
   instead of the hand-set value that drifted between srv1 (65.6G) and srv2 (87.6G). Real, but not this
   bug — prioritize behind 1–2.
4. **srv1 idle (orthogonal 2× capacity).** srv1 at ~5 % while srv2 takes the load; runner
   registration/labels/host-assignment not yet diagnosed.

---

## Why it *felt* like treading water

The "error after error" was honest feedback, not bad luck:
1. **Several independent flakes shared one symptom** — fixing main's red doesn't stop sccache; fixing
   sccache doesn't stop the crash. Each fix looked ineffective because another contributor still fired.
2. **Confident static reasoning was wrong twice.** Diff-size and back-of-envelope arithmetic each
   produced a plausible root cause (per-host overcommit) that a discriminating run refuted. **The cost
   of the loop was paid entirely in reasoning we trusted before we ran the experiment.** The escape was
   not more analysis — it was the three-run table.
3. **An operational livelock on top of the technical one.** #5375 itself couldn't land cleanly: the
   auto-committer kept merging main into its branch (resetting earned approvals) and the crash loop
   kept cancelling its CI. A PR that fixes a crash contributor was blocked by the instability it fixes.

The meta-lesson, in DESIGN terms: **prefer the discriminating experiment over the plausible model, and
do it early** (§5). Hold the variables fixed, change one thing, watch the flip — it is cheaper than the
rounds of confident static reasoning it replaces. And when a postmortem asserts a cause, that assertion
needs a witness too; this one's first draft did not have one, and was wrong.

---

## Concrete next actions

- [ ] **eager-boar:** measure Arc 1 resolve RSS vs baseline → pin mechanism → slim → re-test at width-4.
- [ ] **Probe-derive `WorkDemand.memory`** (peak-RSS per resolve) → retire the 14 GiB constant.
- [ ] **Purge sccache on-disk cache** on both hosts.
- [ ] **Model per-host admission** (latent gap) and generate the slot cap from it (closes srv1/srv2
      drift). Prioritize behind the per-unit fixes.
- [ ] **Diagnose srv1 idle runners** (2× capacity).
- [ ] **Pin the srv2 host-reboot mechanism by execution** before attributing it to any workload
      (candidate: uncapped agent containers, not the floor).

---

## Appendix — evidence pointers

- Decisive runs: `637e4a9b2c`, `27859412294` (PASS, main, width-4, both hosts); `27859296640`
  (Killed, Arc 1, width-4, srv2-15).
- BMC: `https://192.168.1.184` (srv2) / `.183` (srv1), OpenBMC on ASRock ALTRAD8UD; chassis id
  `ALTRAD8UD_1L2T`. Use `curl --netrc-file` (0600), never `-u` on argv. **BMC credentials never go in
  any repo.**
- Model: `dsl/product/compute_fabric.dag` (`placement_spawn_width`, `placement_memory_width_bound`),
  `dsl/gunbc/ci_fleet.dag` (the `14 GiB/unit` constant + `ci_fleet_floor_spawn_fits_memory_budget`),
  `src/v2/workflow/ci_floor_plan.dag`.
- §5 witnesses: `src/v2/test/claim/ci_floor_plan_witness_test.dag`.
- The §5 recovery that found ground truth: eager-boar-790 letting the three-run execution table overrule
  two rounds of static reasoning (the diff and the arithmetic), including the per-host-overcommit cause
  this doc first asserted.
