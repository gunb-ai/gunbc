# CI, end to end — what's on `.dag` today, what isn't, and the lane that closes the gap

> **The charter for ROADMAP §1.** Every other CI plan doc is a *slice*; this is the whole pipeline in
> one place — the causal chain from `git push` to test execution, each link marked on-`.dag` or not, with
> each slice linked where it applies (no dual representation — the carriers and linked docs stay the
> authority). Built from two independent audits that agree (neat-dove-397 Explore + quick-ant-298, the §1
> CI lead), both verified on `origin/main`, 2026-06-22. DESIGN refs: §1 (time = cost/safety), §2
> (Realization — the parallelization arm), §3 (single authority; the *measured-peripheral* split), §5
> (fail-closed — **derive, don't hand-set**), §6 (price the lane in displaced cost, not elegance), §7
> (the substrate analyzing itself).
>
> Live scalars are marked **[confirm]** — quick-ant-298 holds the freshest numbers; the *shape* is the point.

---

## 0. Why CI is a lane, not a chore

CI is **ROADMAP §1**, and it is *upstream of every §0 claim*: a flaky or green-but-broken floor means no
gate protects anything, so every `[x]` in §0–§4 rests on it. But the deeper reason it deserves a lane:
**CI is the one workload that flexes *every* substrate layer at once** — execution-as-a-DAG, scheduling,
caching, secrets/effects, emission — so it is the **forcing function** that turns each *modeled-but-inert*
abstraction load-bearing (the direct antidote to DESIGN §6's "the machinery exists but nothing gates on
it").

**The deliverable is therefore not "faster CI."** Faster CI is the displaced-cost *symptom* you feel
(*"move with confidence"*). The deliverable is **shared abstractions, proven by CI consuming them**: one
Materialization kernel (collapsing the caches), one Placement authority (collapsing jobs/threads/sessions),
one secrets model. Speed falls *out* of that. Priced in elegance instead, "flex every layer" is unbounded
(the §6 purity trap) — so the lane is on-dial only insofar as CI-timeliness is the *cheapest path to the
pain*, and each shared abstraction is pulled in **as CI actually flexes it**, not by taxonomy.

---

## 1. The pipeline as it runs today

One PR push → one CI run: trigger (`ci.yml` on `pull_request`+`workflow_dispatch`) → GitHub starts the
run → concurrency-group eval → GitHub places the **single** `jobs.ci` (no `matrix`) on the first idle
`[self-hosted, linux, arm64]` runner → the runner builds `v1-compiler` and invokes
`claim_executor … --plan-function gunbc_ci_floor_batches` → the executor derives batches from
`ci_floor_plan.dag`, reads the live cgroup budget, shards by `spawn_width`, and runs the `.dag` witness
corpus + gates → exit code → GitHub marks the check → dashboard gates review/merge.

`ci.yml` itself is a byte-for-byte projection of `gunbc.ci_yaml_emit` (`ci_workflow.dag` → `ci_yaml_emit`,
authored steps incl. the build command and the `claim_executor` invocation), drift-gated by
`tools/ci_yaml_gate.dag`. So the **workflow description is on fabric.** The question §2 answers is whether
the *fabric* is.

---

## 2. The causal chain — what's on `.dag` today vs not

**Legend:** 🟢 **DERIVED** (`.dag` is the authority; artifact generated + drift-gated) · 🔵 **EXECUTING
`.dag`** (the `.dag` is literally interpreted at runtime — the strongest sense) · 🟡 **MODELED-INERT**
(`.dag` *describes* it but doesn't operate/enforce it) · 🔴 **OFF-FABRIC** (GitHub-native or hand/host;
no `.dag`).

| # | Link (push → execute) | Verdict | Authority today / gap |
|---|------|---------|----------------------|
| 1 | push branch / open-sync PR | 🔴 | git + GitHub native |
| 2 | GitHub reads committed `ci.yml` triggers, starts run | 🟢 *file* / 🔴 *act* | file derived+gated; dispatch engine native. **Bootstrap seam**: reads the *committed* file → stale-on-invocation-surface is uncaught |
| 3 | concurrency group + cancel-in-progress | 🟢 *key* / 🔴 *eval* | key modeled (`ci_workflow_expressions`), **carries the `run_id`-fallback dup bug**; evaluated by GitHub |
| 3′ | dashboard *also* fires `workflow_dispatch` same SHA | 🔴 | ctrl/dashboard — **the dup source** |
| 4 | GitHub matches labels & **places the job on a host** | 🟢 *labels* / 🔴 *placement* | labels derived (`runner_spec_from_offer`); **placement is native, demand-blind, first-idle** ← **underutilization root (G1)** |
| 5 | a runner daemon on srv1/srv2 picks it up | 🔴 (🟡 inventory) | runners/host, registration, on-box labels = **hand-run shell, no repo artifact (G2)**; `operator_fleet` only *describes* the hosts |
| 6 | the runner's cgroup cap bounds it | 🔴 | `TasksMax`/`MemoryMax` host-set by hand **(G3)**; `.dag` only *reads* it live — adaptive, not authoritative |
| 7 | steps: isolate toolchain, checkout, setup-rust, cache | 🟢 *list* / 🔴 *tools* | step list derived; bodies shell out to git/rustup/cache |
| 8 | `cargo build -p v1-compiler` + freshness/exists verify | 🟢 | command + §5 fail-closed guards derived; cargo/sccache external |
| 9 | `claim_executor … gunbc_ci_floor_batches` | 🔵 | **executor interprets `ci_floor_plan.dag`; batches from dependency edges** — strongest on-fabric link |
| 10 | read live budget → `memory_aware_spawn_width` → shard | 🔵 *decision* / 🔴 *input* | width logic executes `.dag`, reading off-fabric host state |
| 11 | affected-set skip over the git-diff frontier | 🔵 | the `.dag` floor already shrinks to affected |
| 12 | gates: ci.yml-drift, rust fmt/clippy, layering, witness corpus | 🔵 *corpus* / 🔴 *rust* | the witness corpus **is `.dag` executing**; fmt/clippy shell to cargo and have **no affected-set (G5)** |
| 13 | exit code → GitHub marks the check | 🔴 | GitHub-native |
| 14 | dashboard reads check + reviews → merge | 🔴 | ctrl/dashboard; merge manual |

---

## 3. The conflation, named: three layers of "on fabric"

"Largely on fabric" hid that **two different links are both genuinely on `.dag`, at opposite ends**, with
an off-fabric band between them:

- **A — the workflow *description*** (links 2–8): `ci.yml`. **🟢 DERIVED**, drift-gated.
- **B — the fabric *operation*** (links 4–6): which host, runners-per-host, cgroup caps, the shell you SSH
  in and run. **🔴 OFF-FABRIC** — imperative, hand-run, unversioned, *no repo artifact*. `operator_fleet`
  *describes* the hosts (🟡 inert) but does not *operate* them; nothing generates host config, nothing
  reconciles host supply against `ci.yml`'s demand. A mis-set host doesn't red — CI silently runs
  narrow/slow or OOMs.
- **C — the A↔B relationship**: **not enforced.** Coupled only by a label string + a one-way live read of
  `memory.max` (adaptive, not authoritative).

**The one-line gap:** the *description* (A) and the *work* (§4) are on `.dag`; the *operation of the
fabric* (B — placement + runner deployment + caps) is entirely off it. Close B as one unit and "on `.dag`"
becomes true of the whole chain except GitHub's irreducible engine.

---

## 4. Which substrate layers CI flexes — and do they work?

| Layer | State | Evidence |
|---|---|---|
| **Execution as a dependency graph** | **✅ WORKS — the realest layer** | the floor *is* a bounded forward DAG walk; `claim_executor` interprets `ci_floor_plan.dag`, one fold (`fold_node`), batches from dependency edges. The core thesis runs in prod every push |
| **Scheduling** | **◑ one axis live, rest inert** | **Width** consumes the measured envelope (#5444), single-host. `Placement`/`Materialization`/`RealizationObjective` are modeled + witness-passing but have **no non-test consumer** — the same band as the host-ops gap (§3-B), substrate side |
| **Caching** | **◑ five forks, converging** | sccache live (build) · resolve-cache **dormant** (modeled, pure-proven 616/616, env var unset — biggest dormant lever) · ParseTable memo live (content-addressed, amortization witness) · RecordedFixture (record/replay) · BuildBuddy opt-in. The one-door `realize(subject)` kernel (§2 P2) is staged |
| **Secrets / effects** | **○ ad hoc** | live cgroup reads, sccache servers, BMC, tokens — no single model yet |
| **Emission** | **✅ works** | `ci.yml` + testgen are emitted + drift-gated; stage0 seed `--verify` is the one missing gate (§5 keystone) |

So: execution and emission **work**; scheduling and caching **work in one slice each and are inert/forked
otherwise**; secrets are unmodeled. CI is the workload that makes finishing each of these *pay*.

---

## 5. Why it runs poorly — one root, several faces

**Root: the model is on fabric, but the decisions that govern utilization — which host a run lands on,
runners-per-host, per-runner caps — are NOT derived from it. They're demand-blind hand/host knobs.** The
symptom is a fleet with **no operating point** — two 128-core hosts either oversubscribed/crashing or at
~1%, never the middle ([compute-envelope-model.md](compute-envelope-model.md) §1). Headline: a 128-core
box runs CI at **width ~4 [confirm]** while the other host idles.

| Face | Mechanism | Link |
|------|-----------|------|
| placement demand-blind | single job, no matrix → first-idle → heavy runs co-reside, other host idles | G1 (link 4) |
| runner deployment hand-managed | runners/host + caps not in repo → over-commit | G2/G3 (links 5–6) |
| resolve cache dormant | `GUNBC_RESOLVED_GRAPH_CACHE_DIR` unset → ~191s [confirm] cold resolve + high peak caps width low | §4 caching |
| `workflow_dispatch` dup | dispatch + PR → two same-SHA runs escape the `run_id` key → OOM | G4 (links 3/3′) |
| rust gate all-or-nothing | no affected-set on `.rs` PRs | G5 (link 12) |
| stale-green merge | PR validated before a new gate landed | [ci-merge-freshness.md](ci-merge-freshness.md) |

---

## 6. What to do now — leverage-ranked, none a tweak

Ordering principle: **finishes/enables that make utilization fall out of the model come before any new
scheduler.** Sharding/tiering knobs are excluded — they lower a number once and don't compound (§6).

1. **G2/G3 — derive runner *deployment* (count + caps) from `operator_fleet` + `ResourceEnvelope`,
   generate + drift-gate it** (the `ci.yml` pattern, for the host). Makes *"N runners × peak < host RAM"* a
   computed fact, gives the envelope its first real consumer, and converts "I SSH in and run shell" into
   "regenerate + a gate reds on drift." The cheapest substrate-migration win; not a scheduler.
   → [compute-envelope-model.md](compute-envelope-model.md)
2. **Enable the resolve cache in CI** — cuts cold resolve *and* lowers per-shard peak → width rises → the
   host fills. Cheap, orthogonal, purity proven. → [realization-measurement-loop.md](realization-measurement-loop.md)
3. **G1 — emit a cross-host `matrix` from the fleet model** (shard across srv1+srv2). The placement
   predicate's real consumer. **The one step needing a decision** — build CI placement now vs fence the
   dead predicate.
4. **G5 — edge-(b) rust affected-set.** → [ci-selection-vs-scheduling.md](ci-selection-vs-scheduling.md)
5. **G4 — collapse the `workflow_dispatch` dup** (key model + stop redundant dispatch). → [ci-merge-freshness.md](ci-merge-freshness.md)

Steps 1–2 use idle capacity and cut time with no new scheduler. Step 3 removes the contention; the matrix
shards the rest. The 50 min is a *symptom of placement*.

---

## 7. The lane's real deliverable — shared abstractions

Beyond closing the gaps, the lane converges the forks CI exposes (pull each in **as CI flexes it**, §6):

- **One Materialization kernel** — collapse sccache / resolve / ParseTable-memo / RecordedFixture /
  BuildBuddy onto `realize(subject)` (§2 P2).
- **One Placement authority** — jobs (GitHub), threads (`spawn_width`), sessions (ctrl `plans.capacity`)
  are three forks of "put work on a host."
- **One secrets/effects model** — BMC, tokens, sccache-auth modeled once.
- **gunbhub** — owning the Git/CI engine eventually closes the irreducible GitHub boundary (links 2,
  13–14 / G6). *Not pressing*; the pain that denominates the lane is timeliness-for-confidence, not engine
  ownership.

---

## 8. Open decisions (operator + bright-stag)

1. **Target ceiling** — "~50 is the unacceptable status quo; get it to **~___**." Decides whether steps
   1–2 suffice or step 3 is load-bearing this window.
2. **Step 3 — build CI-run placement now, or fence it.** *Build* → the DEAD placement predicate (§4) gets
   its consumer. *Defer* → delete/fence it under one named runway. (Keep `operator_fleet` either way — it's
   the input to step 1.)
3. **Lane shape** — adopt §0's framing (CI as the substrate integration dogfood; deliverable = shared
   abstractions) as the §1 ROADMAP lane, with this doc as its charter.

Steps 1, 2, 4, 5 are sound under any answer; only step 3 waits on decision 2.
