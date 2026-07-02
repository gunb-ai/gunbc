# Compute envelope — one authority for the CI fleet's resource dimensions

> Plan doc. Resolves the §1 ROADMAP pointer **"CI on compute fabric"**. Co-owned: **warm-lark-306** (authoring + operator context) and **quick-ant-298** (§1 CI-floor lead; owns the spawn-width slice). CC **bright-stag-194** (ROADMAP owner + test-profile owner for the fan-out cap). DESIGN refs: §1 (time = cost/safety), §2 (Realization — the parallelization arm), §3 (single authority + the measured-peripheral split), §5 (fail-closed; *derive*, don't hand-set).
> A few live numbers below are marked **[confirm]** — quick-ant has the freshest values from the §1 investigation; the *model* is the point, not those scalars.

## 1. The symptom: a fleet with no operating point

Two 128-core hosts (srv1, srv2) that are **either oversubscribed/crashing or sitting at ~1% doing nothing — never the sane middle.** That bimodal swing *is* the diagnosis. A system that swings between starvation and thrashing, with no stable middle, is a system with **no modeled operating point**: nothing answers "how much work fits this host," so the fleet falls to one extreme or the other.

Live evidence (srv1, 2026-06-21):

- **Under side (~1%):** 128 cores, loadavg **1.27** (~1% util), **9 / 125 GB** mem used, 1 `Runner.Worker` executing but **0 rustc / 0 cargo**. The box is nearly empty during CI.
- **Over side (crash):** a single debug build's thread fan-out (`CARGO_BUILD_JOBS=15` × rustc codegen-units 256 × LLVM × sccache) bursts **one** runner cgroup past `TasksMax=4096` → `clone()` rejected by the pids controller → **EAGAIN** false build failure — a spike on an otherwise-idle host.

Neither extreme is a capacity problem. The host has enormous headroom on both axes; the failures are **coordination** failures.

## 2. The root: N hand-tuned dimensions, no single authority (§3)

Every resource/parallelism knob is set **independently, by a different hand, blind to the others** — the textbook §3 violation (one fact, many homes). There is no `ResourceEnvelope` the knobs derive from, so they cannot be mutually consistent:

| Dimension | Set today by | Blind to |
| --- | --- | --- |
| floor **spawn width** (RUN) | a **pinned constant 4** (`gunbc_ci_floor_spawn_width`, `ci_floor_plan.dag:292`) — the memory-aware derivation was *removed* as unwired (#5419); #5444 (not yet on main) re-adds the memory term | the true core count (envelope says 64c — **a lie**, it's 128c); the ~124 idle cores; the free memory |
| **host packing** (BUILD) | implicit — ~one PR's CI runs at a time across 50 runner units | the idle 120+ cores; the pids it jointly bursts with fan-out |
| per-build **fan-out** (BUILD) | `CARGO_BUILD_JOBS` × codegen-units (debug 256) × LLVM | the pids cap it bursts (× concurrent builds) |
| cgroup **pids cap** | `TasksMax=4096` hand-set drop-in (ctrl) | the fan-out that legitimately needs >4096 |
| **jobserver tokens** | `~120` magic constant (ctrl) | cores, the pids cap |
| cgroup **`MemoryMax`** | per-host hand edit — **drifted** (srv1 ≠ srv2) [confirm] | the other host |
| watchdog / isolation / runner count | hand-set systemd | the envelope |

The under-side and the over-side are the **same defect** seen twice: width bounded *below* the envelope (idle) and fan-out bursting *above* a cap that the envelope never informed (crash).

## 3. The move: one ResourceEnvelope, every knob derived (§3 single authority)

Collapse the N hand-tuned dimensions into **one authority** — the host `ResourceEnvelope` (cores, memory, pids capacity; already a type in `dag/product/compute_fabric.dag`) — and make **every knob a function of it**. Then the dimensions stop being N things to juggle: they are one measured fact with derivations, and the fleet gets **exactly one coherent operating point**. It *cannot* be both starved and thrashing, because both the width floor and the fan-out ceiling come from the same envelope.

| Knob | Derivation from the envelope | Tier (§4) | Owner | Status |
| --- | --- | --- | --- | --- |
| measured host facts | **the envelope itself** — cores, mem, pids capacity, *measured* | ctrl realization | ctrl | model is **wrong** (64c/256GB; real 128c/128GB) — **fix first** |
| spawn width (**RUN** phase) | `min(shard_demand, cores, mem_budget ÷ per-shard-peak)` — replace the **pinned 4** with a derivation that lifts **both** the pin **and** the `shard_demand` cap from the envelope (cores ∧ mem), building on #5444's mem term (+#5421's layer-at-once, done) so the corpus actually uses the 128c/115GB-free headroom. Discovery shards run witnesses against the **prebuilt** release binary — light on pids, no rustc — so **no pids term**; width-up is pids-safe on its own | public | quick-ant (slice) | §5 below |
| host packing (**BUILD** phase concurrency) | `concurrent_builds` — how many PRs' builds co-run on a host; the factor that multiplies the build-phase pids fan-out (see invariant) | public/ctrl | quick-ant + ctrl | §5 below |
| per-build fan-out (**BUILD** phase) | codegen-units / `CARGO_BUILD_JOBS` capped so `concurrent_builds × per_build_pids ≤ pids_cap` | public (test profile) | bright-stag (#5456 area) | §5 below |
| pids cap (`TasksMax`) | `≥ peak_per_build_pids × safety`, bounded by `kernel_threads_max ÷ runner_count` | ctrl realization | ctrl | tourniquet applied (4096→16384, 2026-06-21); should become **derived** |
| jobserver tokens | a function of cores (the cooperative compile limit), consistent with the pids cap | ctrl realization | ctrl | derive, don't magic-constant |
| `MemoryMax` | one value from the envelope, identical across same-spec hosts | ctrl realization | ctrl | de-drift |

**The invariant that forecloses both failures — but mind the two phases (they don't multiply each other):** a PR's CI is a **BUILD** phase (rustc fans out, once) followed by a **RUN** phase (the discovery corpus shards run witnesses against the *prebuilt* binary). They touch different resources:

- **RUN phase — spawn width** is cores ∧ mem bound. Discovery shards barely touch pids (no rustc), so width does **not** multiply the build fan-out. **Width-up is pids-safe on its own** — you can raise it toward the envelope without crash risk. (Premise: shards consume the prebuilt binary — exactly what #5450's Pop-B build-once enforces; a shard that still rebuilds is a *defect to fix*, not a reason to pids-bound width.)
- **BUILD phase — the pids crash** is `concurrent_builds × per_build_pids ≤ pids_cap`. This couples **host-packing** (how many PRs build at once) with **per-build fan-out** (codegen-units) — *not* spawn_width. That is where by-construction crash-prevention lives.

So the same-envelope coupling is **packing × fan-out** (build phase); width-up (run phase) is an *independent* cores∧mem lever. You can't tune either into a crash, because each is bounded by the envelope on its own axis — but they are **two invariants, not one product.**

## 4. The PUBLIC / CTRL split (§3 measured-peripheral — the doc's spine)

Per §3, the agnostic **shape** is central; the **measured realization** and the **dispatch** are peripheral. Applied here:

- **PUBLIC (central, topology):** the envelope *shape*, the width *derivation*, the codegen-units cap — all live in `dag/product/compute_fabric.dag` / `std/realization_width.dag` / the `[profile.test]` block. These are host-agnostic functions: "given an envelope, here is the width / the fan-out."
- **CTRL (peripheral, realization):** the *measured host facts* (128c/128GB), `TasksMax`, `MemoryMax`, runner count, the systemd-cgroup knobs, the unit generation. These are srv1/srv2 *instances* of the shape. **gunbc owns the shapes; ctrl owns the instances and *generates* the units** from the public derivation (the units are emitted, never hand-typed — that's what kills the drift).

So the doc maps both tiers, but routes implementation accordingly: width/fan-out → public PRs; host-fact grounding + unit generation → ctrl.

## 5. Sequencing (operator wants this ASAP)

1. **Ground the envelope in measured truth** — kill the 64c/256GB lie (real: 128c/128GB, both hosts). *Everything downstream inherits this; you cannot derive width from a lying envelope.* (ctrl + the `compute_fabric` host facts.)
2. **Spawn width up (RUN phase)** — derive width from the grounded envelope on **cores ∧ mem** (no pids term — discovery shards run the *prebuilt* binary), raising it toward the host's real capacity instead of the low bound that leaves ~124 cores idle. **Current width is a pinned constant 4** (`ci_floor_plan.dag:292`; the memory-aware model was removed as unwired in #5419) — on 128c that is ~3% of cores from width alone. The lever lifts **both** the pin **and** the `shard_count` cap. **This is pids-safe on its own** (run phase ≠ build phase — see the §3 invariant), so width can go up independently of the crash-side discipline. Builds on **#5444** (memory term, re-adds it; not yet on main) + **#5421** (even-width removal — *landed*); a **refinement**, not a re-derivation. **Owned by quick-ant** (single fresh slice — see §6; do **not** fork). *(Premise: shards consume the prebuilt binary — #5450 Pop-B build-once; a rebuilding shard is a defect to fix, not a reason to pids-bound width.)*
3. **Cap the per-build fan-out + bound host-packing (BUILD phase)** — codegen-units so `concurrent_builds × per_build_pids ≤ pids_cap`. This is the crash-side discipline: it couples host-packing (#2's BUILD-phase concurrency) with fan-out, **not** spawn_width. Codegen-units composes into **one** `[profile.test]` block with bright-stag's **#5456** opt-level work (not a fork).
4. **(2) and (3) each derive from the same envelope on their own axis** ⇒ the coherent operating point — *two* invariants (run-phase cores∧mem; build-phase packing×fan-out≤pids), not one product. Then generate the ctrl units (`TasksMax`/`MemoryMax`/jobserver) from the envelope too, retiring the hand-set drop-ins.

The top lever is **(2)**: it converts the idle 120 cores into real utilization — the biggest single CI-slow win. (1) is its precondition.

## 6. Ownership + the no-fork rule

- **spawn width** is **quick-ant's lane** — a single slice on top of #5444/#5421. `warm-crane-135` (an earlier candidate owner) is **archived** (closed 2026-06-20); there is no parallel item, and one must not be opened — a forked width fix would be the exact §3 violation this doc exists to fix. Note: `adhoc-240256ec-32b` is **done** (it = merged #5421) — *not* the open slice; no existing open item covers "derive width up (unpin + lift the `shard_count` cap) + host-packing across PRs", so quick-ant **reopen-scopes a fresh slice** (confirmed), sequenced after envelope-grounding (#1) and built on #5444 + #5421.
- **fan-out cap** → bright-stag (test-profile owner), composed with #5456.
- **host-fact grounding + unit generation** → ctrl.
- **this doc + the §1 ROADMAP bullets** → bright-stag applies (author→owner pattern); quick-ant co-signs the §1 substance.

## 7. Landed / in-flight context (so this builds on, not re-derives)

- **#5421** (merged) — per-node resource demand + stop the executor's even-width division (CI 24m→~10m).
- **#5375 → superseded by #5419** — #5375's memory-aware scheduling was removed as unwired complexity by #5419's pin (#5375 < #5419, so #5419 reverted it); **#5444 re-adds** the memory term. Current state = **pinned 4**, not memory-aware.
- **#5419** (merged) — pinned floor width (the bound (2) raises).
- **#5444** (merging) — width value from measured-RAM-budget ÷ measured-per-shard-peak (the **memory term** of the envelope derivation).
- **#5431** (merged) — the measured-peak source the memory term reads.
- **#5456** (merged) — `[profile.test] opt-level=3` for `v1-compiler` (the fan-out cap composes here).
- **TasksMax 4096→16384** applied on srv1+srv2 (2026-06-21) — the tourniquet; should become *derived*.

This is the §2 **parallelization-by-realization** arm (Schedule/Placement/Width of `std/realization.dag`) made coherent with its measured substrate — the structural twin of the §2 caching arm, both derived from one authority.

## Dissolution trigger (DESIGN §6)

Delete this doc when the host `ResourceEnvelope` is grounded in measured truth (128c/128GB, both hosts) and every knob — spawn width, host packing, per-build fan-out, `TasksMax`, jobserver tokens, `MemoryMax` — is *derived* from that single authority (the ctrl units generated, never hand-typed), at which point the coherent operating point is a witnessed by-construction property (§3/§5) and this planning tracker is redundant.
