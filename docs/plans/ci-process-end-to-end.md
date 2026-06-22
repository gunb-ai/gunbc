# CI, end to end — what runs, why it runs poorly, what to do now

> **The consolidating map for ROADMAP §1.** Every other CI plan doc is a *slice*; this one is the
> whole pipeline in one place, with each slice linked at the point it applies (no dual representation —
> the carriers and the linked docs stay the authority). Written from two independent audits that agree
> (neat-dove-397 Explore + quick-ant-298, the §1 CI lead, both verified on `origin/main`, 2026-06-22).
> DESIGN refs: §1 (time = cost/safety), §2 (Realization — the parallelization arm), §3 (single
> authority; the *measured-peripheral* split), §5 (fail-closed; **derive, don't hand-set**).
>
> Live scalars below are marked **[confirm]** — quick-ant-298 holds the freshest numbers; the *shape*
> is the point, not the digits.

---

## 0. Where this sits on the ROADMAP

CI is **ROADMAP §1 — "CI under control (the correctness floor)"**, and it is *upstream of every §0
claim*: a flaky or green-but-broken floor means no gate protects anything, so every `[x]` in §0–§4
rests on this. It is also the near-term face of the project thesis — **infra migrating onto the `.dag`
substrate** (DESIGN §7): the Rust/host layer shrinks toward being a *derived realization*. Faster CI is
the displaced-cost payoff; **the fleet-as-a-model is the moat.**

The §1 milestone spine: `opt-3 per-PR ✓ (#5456) → ▸ NOW: floor never OOMs (memory-aware width) →
floor runs the affected set → every host knob from one measured ResourceEnvelope`.

---

## 1. The pipeline as it actually runs today

One PR push → one CI run, in this order:

1. **Trigger.** `ci.yml` fires on `pull_request` **and** `workflow_dispatch`.
2. **The workflow is derived, not hand-written.** `ci.yml` is a byte-for-byte projection of
   `gunbc.ci_yaml_emit` (`dsl/gunbc/ci_workflow.dag` → `ci_yaml_emit.dag`), drift-gated by
   `dsl/tools/ci_yaml_gate.dag` (floor-enrolled, fail-closed). The workflow **structure is on fabric.**
3. **One job, one runner.** `jobs.ci` has **no `matrix`/`strategy`** — it is a *single* job. Its
   `runs-on: [self-hosted, linux, arm64]` is derived (`ci_fleet.gunbc_ci_runner_spec()` →
   `runner_spec_from_offer`, projecting our self-hosted `ComputeOffer`). GitHub's own scheduler then
   assigns that one job to the **first idle label-matching runner.**
4. **The job runs one composed floor pass:**
   `claim_executor --source-root src/v2 --source-root dsl --plan-entry
   src/v2/workflow/ci_floor_plan.dag --plan-function gunbc_ci_floor_batches`.
5. **Batches come from a single authority.** `gunbc.ci_spec` → `ci_floor_plan.gunbc_ci_floor_batches`:
   the only structural fact the plan adds is *compile-clean gates the rest* (batch-1
   `dsl_compile_clean_gate` → batch-2 everything else). Heavy-resolve gates are serialized by
   `ResourceDependsOn` edges so their memory peaks don't co-occur.
6. **Within-job parallelism is memory-aware (the one live, load-bearing piece).**
   `std.realization_width.memory_aware_spawn_width = min(shard_count, cores, ⌊0.8·budget ÷ peak⌋)`,
   consuming the **committed measured per-shard peak** (`gunbc.ci_floor_measurement.dag`, provenance-
   stamped VmHWM, ~8.4 GB **[confirm]**) and the **live cgroup `memory.max`** read at runtime by
   `claim_executor.eval_spawn_width` (`/proc/self/status`). Green-by-execution (#5444, #5524).
7. **The `.dag` floor already shrinks to the affected set.** `affected_set_floor_runner` does
   node-level `SkipAssumedGreen` for discovery rows whose node-closure misses the git-diff frontier,
   wired through `claim_executor` (`skip_unaffected_node_frontier`). Stateless; skipped rows are assumed
   green from the last full `main` run.
8. **Effectful gates ride batch-2:** rust fmt/clippy monolith + run-all (on `.rs` PRs), emit-host
   smokes, layering-imports scan, source-root-ingest, the `ci.yml` drift gate. sccache is wired
   (`extdeps/cache/sccache.dag`).

**So the model is largely on fabric** — workflow, spec, batch plan, within-run width, the per-run
demand envelope (`compute_fabric.parallel_run_demand_envelope`, #5524), the `.dag` affected-set, the
fleet *inventory* (`operator_fleet.dag`: srv1 + srv2 as real `ComputeOffer`s, Ampere Altra 128c/128 GiB),
even the runner-identity projection. That is more than either audit expected going in.

---

## 2. Why it runs poorly — one root, several faces

**The root (one sentence): the model is on fabric, but the decisions that *govern utilization* —
which host a run lands on, how many runners per host, each runner's memory cap — are NOT derived from
it. They are hand/host/ctrl-managed and demand-blind. That mismatch *is* the dumpster fire.**

The symptom is a fleet with **no modeled operating point** — two 128-core hosts that are either
oversubscribed/crashing or sitting at ~1% doing nothing, never the sane middle
([compute-envelope-model.md](compute-envelope-model.md) §1). The faces:

| # | Face | Mechanism | Cost |
|---|------|-----------|------|
| 1 | **Cross-host placement is demand-blind** | single job, no matrix → GitHub picks the first idle runner with no idea of the run's ~40 GB demand → heavy runs **pile onto one host while the other idles**; `operator_fleet` has **no CI-run placement consumer** (the ctrl `plans.capacity` consumer places *sessions/containers, not GHA jobs*) | ~29 min contention **[confirm]**; the underutilization root |
| 2 | **Runner deployment is hand-managed** | how many runners per host + each runner's cgroup cap are **not in the repo at all** (host/ctrl-set), so nothing prevents too many ~40 GB runners co-residing | the OOM/EAGAIN crashes |
| 3 | **Resolve cache dormant in CI** | `GUNBC_RESOLVED_GRAPH_CACHE_DIR` is never set in `ci.yml` (only sccache is wired), so the cold resolve re-runs every CI **and** keeps per-shard peak high → caps `spawn_width` low → a 128-core box runs at **width ~4** | ~191 s/run **[confirm]** + suppressed width |
| 4 | **`workflow_dispatch` dup** | `ci.yml` fires on `workflow_dispatch` *and* `pull_request`; the dashboard fires a dispatch on top of the natural PR run; the modeled concurrency key (`workflow-{pull_request.number ‖ run_id}`) falls back to `run_id`, so the two same-SHA runs **escape into different groups and co-reside** → OOM | dup ~40 GB runs |
| 5 | **Rust gate is all-or-nothing** | `rust_gates_ci.dag` has no affected-set reference — on a `.rs` PR it runs every rust test, execution-dominated | the ~50 min ceiling on `.rs` PRs |
| 6 | **Stale-green merge** *(distinct root, mostly settled)* | a PR validated against a base that pre-dates a newly-landed gate, merged without re-validation | reds `main` post-merge ([ci-merge-freshness.md](ci-merge-freshness.md)) |

Faces 1–4 are **one disease**: utilization decisions live outside the model. Face 5 is *selection*
(orthogonal — [ci-selection-vs-scheduling.md](ci-selection-vs-scheduling.md)). Face 6 is *freshness*.

---

## 3. The graveyard — what's live, half-built, and dead

The "scattered features that don't work" resolve to exactly this. **Almost all of it is one missing
consumer**, not many broken things.

| Component | Path | State | Evidence |
|-----------|------|-------|----------|
| within-run memory-aware `spawn_width` | `ci_floor_plan.dag` + `ci_floor_measurement.dag` + `realization_width.dag` | **LIVE** | `claim_executor` reads live budget, shards discovery batch (#5444) |
| topology scheduler + heavy-resolve serialization | `ci_floor_plan.dag`, `executor.dag` | **LIVE** | `claim_executor.run_walk` walks batches; single-host only |
| `ci.yml` derived + drift-gated | `ci_workflow.dag`, `ci_yaml_emit.dag` | **LIVE** | `ci_yaml_gate` fail-closed |
| `.dag` floor affected-set selection | `affected_set_floor_runner` | **LIVE** | wired via `claim_executor` |
| per-run demand `ResourceEnvelope` | `compute_fabric.parallel_run_demand_envelope` | **LIVE (producer)** | #5524; **no placement consumer reads it** |
| fleet inventory (srv1/srv2) | `operator_fleet.dag`, `operator_fleet_network.dag` | **HALF — input only** | feeds `ci.yml` labels; **not** runtime placement. *Keep* — it's the input to the fix below |
| placement predicate | `compute_fabric.satisfies()`, `placement_supply.dag` | **DEAD** | the only consumer (ctrl `plans.capacity`) places sessions, **not CI runs**; a CI-run placement consumer exists *nowhere* |
| cross-host matrix | — | **MISSING** | `ci.yml` emits no `matrix`; sharding across srv1+srv2 unbuilt |
| runner deployment config (count/host, caps) | — | **MISSING from repo** | host/ctrl-managed, demand-blind |
| resolve cache in CI | `resolved_graph.dag` modeled | **DORMANT** | env var never set in `ci.yml` |
| rust-gate affected-set (edge-b) | `rust_gates_ci.dag` | **MISSING** | no affected reference |

DESIGN §6 note: the DEAD/MISSING rows are scaffolds without a named dissolution trigger — that is
itself the §6 violation. Each needs to **become load-bearing or be deleted** (§5 below).

---

## 4. What to do now — leverage-ranked, none of it a tweak

The ordering principle: **finishes and enables that make utilization fall out of the model come before
any new scheduler.** Sharding/tiering knobs are explicitly *not* here — they lower a number once and
don't compound (the purity trap, DESIGN §6).

1. **Derive the runner *deployment* config from `operator_fleet` + `ResourceEnvelope`, generate-and-
   drift-gate it** (same pattern as `ci.yml`). Makes *"N runners × per-run peak < host RAM"* a **derived
   fact, not a hand guess** → ends over-commit (face 2) and gives the demand envelope its first real
   consumer. This is the cheapest *substrate-migration* win and the literal answer to "runner config
   should go through `.dag`." Not a placement scheduler. → [compute-envelope-model.md](compute-envelope-model.md)
2. **Enable the resolve cache in CI** (binary-keyed `GUNBC_RESOLVED_GRAPH_CACHE_DIR`). Cuts the cold
   resolve **and** lowers per-shard peak → width rises → the active host fills up (faces 3 + the
   width cap). Cheap, orthogonal, purity already proven 616/616.
   → [realization-measurement-loop.md](realization-measurement-loop.md)
3. **Emit a cross-host `matrix` from the fleet model** (shard the single job across srv1+srv2). Turns
   the serial single-host run into parallel shards that use the idle host (face 1). **This is the only
   step that needs a genuine decision** — it is the placement predicate's real consumer, so it forces
   *build-now vs fence-under-one-runway* on the DEAD rows in §3.
4. **edge-(b) rust affected-set** — selection for the rust gate, so a `.rs` PR runs only the affected
   rust tests, fail-closed (face 5). Scoped keystone. → [ci-selection-vs-scheduling.md](ci-selection-vs-scheduling.md)
5. **Collapse the `workflow_dispatch` dup** in the concurrency-key model + stop the redundant dispatch
   (face 4). Model edit. → [ci-merge-freshness.md](ci-merge-freshness.md) (adjacent)

**Expected shape of the win:** steps 1–2 use idle capacity and cut time *with no new scheduler* — the
non-tweak progress, robust regardless of the step-3 decision. Step 3 removes the ~29 min contention;
the matrix shards the rest. The 50 min is a *symptom of placement*; fix placement structurally and it
falls out.

---

## 5. The two open decisions (operator)

1. **Target ceiling.** "~50 is the unacceptable status quo; get it to **~___**." Decides whether
   steps 1–2 suffice or step 3 is load-bearing this week.
2. **Step 3 — build CI-run placement now, or fence it.** *Build now* → the DEAD placement predicate
   (§3) gets its consumer and becomes load-bearing. *Defer* → delete or fence it under **one** named
   runway so it stops reading as a broken feature. (Keep `operator_fleet` either way — it's the input
   to step 1.)

Steps 1, 2, 4, 5 are sound under either answer. Only step 3 waits on decision 2.
