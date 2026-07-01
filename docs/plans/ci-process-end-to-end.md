# CI, end to end — what's on `.dag` today, what isn't, and the lane that closes the gap

> **The charter for ROADMAP §1.** Every other CI plan doc is a *slice*; this is the whole pipeline in one place — the causal chain from `git push` to test execution, each link marked on-`.dag` or not, with each slice linked where it applies (no dual representation — the carriers and linked docs stay the authority). Built from two independent audits that agree (neat-dove-397 Explore + quick-ant-298, the §1 CI lead), both verified on `origin/main`, 2026-06-22. DESIGN refs: §1 (time = cost/safety), §2 (Realization — the parallelization arm), §3 (single authority; the *measured-peripheral* split), §5 (fail-closed — **derive, don't hand-set**), §6 (price the lane in displaced cost, not elegance), §7 (the substrate analyzing itself).
> Live scalars are marked **[confirm]** — quick-ant-298 holds the freshest numbers; the *shape* is the point.

---

## 0. Why CI is a lane, not a chore

CI is **ROADMAP §1**, and it is *upstream of every §0 claim*: a flaky or green-but-broken floor means no gate protects anything, so every `[x]` in §0–§4 rests on it. But the deeper reason it deserves a lane: **CI is the one workload that flexes *every* substrate layer at once** — execution-as-a-DAG, scheduling, caching, secrets/effects, emission — so it is the **forcing function** that turns each *modeled-but-inert* abstraction load-bearing (the direct antidote to DESIGN §6's "the machinery exists but nothing gates on it").

**The deliverable is therefore not "faster CI."** Faster CI is the displaced-cost *symptom* you feel (*"move with confidence"*). The deliverable is **shared abstractions, proven by CI consuming them**: one Materialization kernel (collapsing the caches), one Placement authority (collapsing jobs/threads/sessions), one secrets model. Speed falls *out* of that. Priced in elegance instead, "flex every layer" is unbounded (the §6 purity trap) — so the lane is on-dial only insofar as CI-timeliness is the *cheapest path to the pain*, and each shared abstraction is pulled in **as CI actually flexes it**, not by taxonomy.

---

## 1. The pipeline as it runs today

One PR push → one CI run: trigger (`ci.yml` on `pull_request`+`workflow_dispatch`) → GitHub starts the run → concurrency-group eval → GitHub places the **single** `jobs.ci` (no `matrix`) on the first idle `[self-hosted, linux, arm64]` runner → the runner builds `v1-compiler` and invokes `claim_executor … --plan-function gunbc_ci_floor_batches` → the executor derives batches from `ci_floor_plan.dag`, reads the live cgroup budget, shards by `spawn_width`, and runs the `.dag` witness corpus + gates → exit code → GitHub marks the check → dashboard gates review/merge.

`ci.yml` itself is a byte-for-byte projection of `gunbc.ci_yaml_emit` (`ci_workflow.dag` → `ci_yaml_emit`, authored steps incl. the build command and the `claim_executor` invocation), drift-gated by `tools/ci_yaml_gate.dag`. So the **workflow description is on fabric.** The question §2 answers is whether the *fabric* is.

---

## 2. The causal chain — what's on `.dag` today vs not

**Legend:** 🟢 **DERIVED** (`.dag` is the authority; artifact generated + drift-gated) · 🔵 **EXECUTING `.dag`** (the `.dag` is literally interpreted at runtime — the strongest sense) · 🟡 **MODELED-INERT** (`.dag` *describes* it but doesn't operate/enforce it) · 🔴 **OFF-FABRIC** (GitHub-native or hand/host; no `.dag`).

| # | Link (push → execute) | Verdict | Wall (#5427 run) | Authority today / gap |
| --- | --- | --- | --- | --- |
| 1 | push branch / open-sync PR | 🔴 | — | git + GitHub native |
| 2 | GitHub reads committed `ci.yml` triggers, starts run | 🟢 *file* / 🔴 *act* | ~0 | file derived+gated; dispatch engine native. **Bootstrap seam**: reads the *committed* file → stale-on-invocation-surface is uncaught |
| 3 | concurrency group + cancel-in-progress | 🟢 *key* / 🔴 *eval* | ~0 | key modeled (`ci_workflow_expressions`), **carries the `run_id`-fallback dup bug**; evaluated by GitHub |
| 3′ | dashboard *also* fires `workflow_dispatch` same SHA | 🔴 | — | ctrl/dashboard — **the dup source** |
| 4 | GitHub matches labels & **places the job on a host** | 🟢 *labels* / 🔴 *placement* | ~0 (queue) | labels derived (`runner_spec_from_offer`); **placement is native, demand-blind, first-idle** ← **underutilization root (G1)** |
| 5 | a runner daemon on srv1/srv2 picks it up | 🔴 (🟡 inventory) | — | runners/host, registration, on-box labels = **hand-run shell, no repo artifact (G2)**; `operator_fleet` only *describes* the hosts |
| 6 | the runner's cgroup cap bounds it | 🔴 | — | `TasksMax`/`MemoryMax` host-set by hand **(G3)**; `.dag` only *reads* it live — adaptive, not authoritative |
| 7 | steps: isolate toolchain, checkout, setup-rust, cache | 🟢 *list* / 🔴 *tools* | **~15s** | step list derived; bodies shell out to git/rustup/cache |
| 8 | `cargo build -p v1-compiler` + freshness/exists verify | 🟢 | **~1m45s** | command + §5 guards derived; sccache-warm (847/859 hits) at `BUILD_JOBS=1` |
| 9 | `claim_executor … gunbc_ci_floor_batches` | 🔵 | *(batches ↓)* | **executor interprets `ci_floor_plan.dag`; batches from dependency edges** — strongest on-fabric link |
| · | ⤷ batch 1 — compile-clean gate | 🔵 | **~22s** | `gunbc compile --target rust` over the whole tree + a RED control |
| · | ⤷ batch 2 — corpus + rust monolith + 5 gates (width 8) | 🔵/🔴 | **~43m20s** | **87% of the run; one atomic rust node holds it open — see §6** |
| · | ⤷ batch 3 — source-root-ingest + self-host closure | 🔵 | **~4m29s** | 2 serial (width-1) sub-shells of heavy `gunbc run`s |
| 10 | read live budget → `memory_aware_spawn_width` → shard | 🔵 *decision* / 🔴 *input* | *(in 9)* | width logic executes `.dag`, reading off-fabric host state; gave `width=8` from a 112 GB live budget |
| 11 | affected-set skip over the git-diff frontier | 🔵 | *(in 9)* | the `.dag` floor already shrinks to affected (0 skipped here) |
| 12 | gates: ci.yml-drift, rust fmt/clippy+**run-all**, layering, corpus | 🔵 *corpus* / 🔴 *rust* | *(in batch 2)* | corpus **is `.dag` executing**; the rust gate shells to cargo, **no affected-set (G5)**, and is the tentpole |
| 13 | exit code → GitHub marks the check | 🔴 | ~0 | GitHub-native |
| 14 | dashboard reads check + reviews → merge | 🔴 | — | ctrl/dashboard; merge manual |
|  | **total step** |  | **49m58s** | one `gunbc ci` step |

---

## 3. The conflation, named: three layers of "on fabric"

"Largely on fabric" hid that **two different links are both genuinely on `.dag`, at opposite ends**, with an off-fabric band between them:

- **A — the workflow *description*** (links 2–8): `ci.yml`. **🟢 DERIVED**, drift-gated.
- **B — the fabric *operation*** (links 4–6): which host, runners-per-host, cgroup caps, the shell you SSH in and run. **🔴 OFF-FABRIC** — imperative, hand-run, unversioned, *no repo artifact*. `operator_fleet` *describes* the hosts (🟡 inert) but does not *operate* them; nothing generates host config, nothing reconciles host supply against `ci.yml`'s demand. A mis-set host doesn't red — CI silently runs narrow/slow or OOMs.
- **C — the A↔B relationship**: **not enforced.** Coupled only by a label string + a one-way live read of `memory.max` (adaptive, not authoritative).

**The one-line gap:** the *description* (A) and the *work* (§4) are on `.dag`; the *operation of the fabric* (B — placement + runner deployment + caps) is entirely off it. Close B as one unit and "on `.dag`" becomes true of the whole chain except GitHub's irreducible engine.

---

## 4. Which substrate layers CI flexes — and do they work?

| Layer | State | Evidence |
| --- | --- | --- |
| **Execution as a dependency graph** | **✅ WORKS — the realest layer** | the floor *is* a bounded forward DAG walk; `claim_executor` interprets `ci_floor_plan.dag`, one fold (`fold_node`), batches from dependency edges. The core thesis runs in prod every push |
| **Scheduling** | **◑ one axis live, rest inert** | **Width** consumes the measured envelope (#5444), single-host. `Placement`/`Materialization`/`RealizationObjective` are modeled + witness-passing but have **no non-test consumer** — the same band as the host-ops gap (§3-B), substrate side |
| **Caching** | **◑ five forks, converging** | sccache live (build) · resolve-cache **dormant** (modeled, pure-proven 616/616, env var unset — biggest dormant lever) · ParseTable memo live (content-addressed, amortization witness) · RecordedFixture (record/replay) · BuildBuddy opt-in. The one-door `realize(subject)` kernel (§2 P2) is staged |
| **Secrets / effects** | **○ ad hoc** | live cgroup reads, sccache servers, BMC, tokens — no single model yet |
| **Emission** | **✅ works** | `ci.yml` + testgen are emitted + drift-gated; stage0 seed `--verify` is the one missing gate (§5 keystone) |

So: execution and emission **work**; scheduling and caching **work in one slice each and are inert/forked otherwise**; secrets are unmodeled. CI is the workload that makes finishing each of these *pay*.

---

## 5. Why it runs poorly — one root, several faces

**Root: the model is on fabric, but the decisions that govern utilization — which host a run lands on, runners-per-host, per-runner caps — are NOT derived from it. They're demand-blind hand/host knobs.** The symptom is a fleet with **no operating point** — two 128-core hosts either oversubscribed/crashing or at ~1%, never the middle ([compute-envelope-model.md](compute-envelope-model.md) §1). Headline: a 128-core box runs CI at **width ~4 [confirm]** while the other host idles.

| Face | Mechanism | Link |
| --- | --- | --- |
| placement demand-blind | single job, no matrix → first-idle → heavy runs co-reside, other host idles | G1 (link 4) |
| runner deployment hand-managed | runners/host + caps not in repo → over-commit | G2/G3 (links 5–6) |
| resolve cache dormant | `GUNBC_RESOLVED_GRAPH_CACHE_DIR` unset → ~191s [confirm] cold resolve + high peak caps width low | §4 caching |
| `workflow_dispatch` dup | dispatch + PR → two same-SHA runs escape the `run_id` key → OOM | G4 (links 3/3′) |
| rust gate all-or-nothing | no affected-set on `.rs` PRs | G5 (link 12) |
| stale-green merge | PR validated before a new gate landed | [ci-merge-freshness.md](ci-merge-freshness.md) |

---

## 6. Where the 50 minutes goes (measured — #5427 run 27924855136, srv2, width 8)

The whole 50 min is one step (`gunbc ci`, 49m58s). By phase, then one level deeper into what each runs:

| Phase | Wall | What it runs (one level down) |
| --- | --- | --- |
| Setup | ~15s | checkout (fetch-depth 0, ~1s) · setup-rust (rustfmt already current, ~10s) · cargo-cache restore |
| Build `v1-compiler --release` | ~1m45s | `cargo build … --bins` at **`CARGO_BUILD_JOBS=1`**; **sccache warm (847/859 hits)** so fast anyway. Produces the seed bins: `gunbc` · `claim_executor` · `discover_source_root_ingest` |
| **Batch 1** — compile-clean gate | ~22s | `gunbc compile --target rust` over `dsl`+`src/v2` (whole substrate must compile clean) **+ a RED control** (a deliberately-perturbed compile must fail) |
| **Batch 2** (width 8, 7 nodes) | **~43m20s** | one *atomic* node dominates — split below |
| **Batch 3** — ingest + self-host | ~4m29s | 2 **serial** (width-1) sub-shells: ① `discover_source_root_ingest` + 3 program-assembly ingest witnesses (~2m16s) · ② self-host closure + gap4-parse + closure module-count witnesses (~2m13s) |
| Teardown | ~19s | save cargo cache (tar+zstd, 233 MB) |

**Batch 2, one level down.** The 7 nodes run width-8 parallel but are wildly unbalanced — one *atomic* node holds the batch open for 43 min while the rest finish in the first ~7:

| Batch-2 node | Wall | What it runs |
| --- | --- | --- |
| discovery corpus (660 `.dag` witnesses) | ~6m (done 02:06) | **resolve 840s + eval 910s = 1750s CPU** ÷ width 8. resolve = re-resolving each witness's import graph **cold (resolve cache OFF)**; eval = interpreting each witness **un-memoized** |
| 5 lens/gate witnesses (emit-host · layering · resolved-imports · extdeps-authority · ci.yml-drift) | sec–min | each a `gunbc run --claim-run` over one `_test.dag`; finish early, masked |
| **`rust_monolith_gate`** | **~43m** | `cargo fmt` (~s) · `clippy --all-targets` (~35s, sccache-warm) · **`cargo test -p v1-compiler-tests` = 42m39s** ← the #5427 run-all |

**The tentpole, one more level — `cargo test -p v1-compiler-tests` (42m39s):** *compile* the test crate at **`CARGO_BUILD_JOBS=1`** + *run* the ~792 tests. The compile-vs-run split is **not instrumented** — the one measurement worth taking before committing a fix (`cargo test --no-run` timing). The *mechanism*, though, is already clear (§7).

---

## 7. Why batch 2 is slow — the coherent, whole-pipeline reading

Not "the rust gate is inherently 43 min." It's the same mechanism §1 keeps hitting: **on a 128-core box, every heavy phase runs at single-digit parallelism or recomputes cold — because nothing derives the safe parallelism from the host.** Seen across the whole pipeline:

- **The rust tentpole compiles at parallelism ~1.** `CARGO_BUILD_JOBS=1` is force-set whenever sccache is active — a **panic-clamp** to dodge the `sccache × codegen-units` pids-cap EAGAIN crash ([compute-envelope-model.md](compute-envelope-model.md) §1). To avoid the *crash* extreme the box is pinned to the *idle* extreme: it compiles the test crate single-threaded on 128 cores. That clamp **is** the crash-or-idle swing, landed on the rust gate.
- **It is also a single un-sharded node.** The width-8 scheduler shards the `.dag` corpus (1750s CPU → 6m wall) but treats `cargo test -p v1-compiler-tests` as **one opaque node** — no nextest partition, no per-test-group shard — so its cost is *serial wall*, not divided. The scheduler's parallelism never reaches inside it.
- **The `.dag` corpus is fast only because it's sharded — it still pays cold.** 1750s CPU for 660 witnesses is itself high: resolve runs **cold every run** (dormant resolve cache) and eval is **un-memoized**. Width 8 hides it as 6m wall; it is not cheap.
- **Batch 3 is serial** — two width-1 sub-shells of heavy `gunbc run`s for ~4.5m.

So the whole pipeline tells **one** story: the heaviest work is either clamped to low parallelism (rust `jobs=1`, batch-3 serial) or recomputed cold (resolve cache off) — a 128-core machine used at single-digit parallelism for its two most expensive phases. That is exactly the **scheduling-inert + caching-dormant** gap of §4, on the clock. The levers, none a tweak:

1. **Derive a safe `CARGO_BUILD_JOBS` (>1) from the envelope** (`jobs = f(cores, pids-cap, mem)`) instead of the panic-clamp to 1 — unclamps compile (G3 / compute-envelope).
2. **Shard the rust node** (nextest partition) so the width-8 scheduler reaches inside it (G5 / §9 one-Placement-authority — the rust gate is off the `.dag` scheduler today).
3. **Enable the resolve cache** — drops the 840s cold resolve under the corpus.
4. **edge-(b) selection** — run only the affected rust tests, so the run-all isn't paid per-PR at all.

(1)+(2) attack the tentpole's two mechanisms; (3) the corpus; (4) removes the run-all from most PRs. Each closes a derive-from-the-host gap — none is "trim a number."

---

## 8. What to do now — leverage-ranked, none a tweak

Ordering principle: **finishes/enables that make utilization fall out of the model come before any new scheduler.** Sharding/tiering knobs are excluded — they lower a number once and don't compound (§6).

1. **G2/G3 — derive runner *deployment* (count + caps) from `operator_fleet` + `ResourceEnvelope`, generate + drift-gate it** (the `ci.yml` pattern, for the host). Makes *"N runners × peak < host RAM"* a computed fact, gives the envelope its first real consumer, and converts "I SSH in and run shell" into "regenerate + a gate reds on drift." The cheapest substrate-migration win; not a scheduler. → [compute-envelope-model.md](compute-envelope-model.md)
2. **Enable the resolve cache in CI** — cuts cold resolve *and* lowers per-shard peak → width rises → the host fills. Cheap, orthogonal, purity proven. → [realization-measurement-loop.md](realization-measurement-loop.md)
3. **G1 — emit a cross-host `matrix` from the fleet model** (shard across srv1+srv2). The placement predicate's real consumer. **The one step needing a decision** — build CI placement now vs fence the dead predicate.
4. **G5 — edge-(b) rust affected-set.** → [ci-selection-vs-scheduling.md](ci-selection-vs-scheduling.md)
5. **G4 — collapse the `workflow_dispatch` dup** (key model + stop redundant dispatch). → [ci-merge-freshness.md](ci-merge-freshness.md)
6. **Reclassify contention OOMs before green-on-main can gate** — a cross-run clustering consumer that promotes a clustered `exit-137` back to `Infra` (two OOM-kill handlers as one fact + a clustering discriminator, fail-closed). Prerequisite (a) of the merge-freshness LIVE flip; model-only design today. → [ci-oom-reclassification.md](ci-oom-reclassification.md)

Steps 1–2 use idle capacity and cut time with no new scheduler. Step 3 removes the contention; the matrix shards the rest. The 50 min is a *symptom of placement*.

---

## 9. The lane's real deliverable — shared abstractions

Beyond closing the gaps, the lane converges the forks CI exposes (pull each in **as CI flexes it**, §6):

- **One Materialization kernel** — collapse sccache / resolve / ParseTable-memo / RecordedFixture / BuildBuddy onto `realize(subject)` (§2 P2).
- **One Placement authority** — jobs (GitHub), threads (`spawn_width`), sessions (ctrl `plans.capacity`) are three forks of "put work on a host."
- **One secrets/effects model** — BMC, tokens, sccache-auth modeled once.
- **gunbhub** — owning the Git/CI engine eventually closes the irreducible GitHub boundary (links 2, 13–14 / G6). *Not pressing*; the pain that denominates the lane is timeliness-for-confidence, not engine ownership.

---

## 10. Open decisions (operator + bright-stag)

1. **Target ceiling** — "~50 is the unacceptable status quo; get it to **~___**." Decides whether steps 1–2 suffice or step 3 is load-bearing this window.
2. **Step 3 — build CI-run placement now, or fence it.** *Build* → the DEAD placement predicate (§4) gets its consumer. *Defer* → delete/fence it under one named runway. (Keep `operator_fleet` either way — it's the input to step 1.)
3. **Lane shape** — adopt §0's framing (CI as the substrate integration dogfood; deliverable = shared abstractions) as the §1 ROADMAP lane, with this doc as its charter.

Steps 1, 2, 4, 5 are sound under any answer; only step 3 waits on decision 2.

## Dissolution trigger (DESIGN §6)

Delete this doc when §3-B (the fabric operation — placement, runner deployment, per-runner caps) is on `.dag` and drift-gated (steps 1 + 3 landed), the resolve cache is enabled in CI (step 2), and the rust tentpole no longer dominates a CI run — at which point the push→execute chain is on fabric except GitHub's irreducible engine, the §4 scheduling/caching gaps are closed by a real non-test consumer, and this charter's gap-map is a witnessed property rather than an audit.
