# CI humming — throughput, wall-clock, reliability (the single CI-operations authority)

Home: ROADMAP §1 (CI as the substrate integration dogfood). This is the **▸ NOW — host-operation on `.dag`** milestone made concrete: placement, runner deployment, and caps are hand-managed and off-fabric today; this plan migrates them onto the modeled budget + carrier and uses that to un-throttle CI.

**GOAL.** CI fast (low wall-clock per PR) + high throughput (many PRs concurrent) + reliable (no reds / OOM / flakes) — and all three falling *out* of a single modeled budget consumed by a verified-effective apply, never a hand-picked number (§2/§5).

## Root cause (live-verified 2026-06-23)

srv1 + srv2 (128c / 125GiB each) sit at ~10-34% CPU, ~24-32% MEM, with only **~3 runner slots per host** — so PRs **queue while the cores idle**. The throttle is the runner-slot count, which is set by the per-host **memory budget allocation**, not by saturation, not by fleet capacity, not by OOM.

The slot count is starved because the budget model (`dsl/gunbc/fleet_host_budget.dag`) derives `runner_slice_cap ≈ 0`: build memory is subtracted as a `build_pool` **and** re-charged inside the per-job whole-tree-peak divisor. A GHA-CI runner job *is* a build — its `rustc` lives **inside** `system-actions-runner.slice` — so the build memory must be counted **once**, not twice.

## oomd demoted (operator §5 insight)

Proper allocation (slice cap + per-slot caps ≤ physical − overhead, **verified-effective**) makes OOM impossible *by construction*. oomd is defense-in-depth for the **residue that is not yet capped-by-construction**, not a hard precondition. The srv2 reboots were construction *failing to realize* (the authored cap sat at `MemoryMax=infinity` on the live cgroup), not a missing daemon.

→ Relax `fleet_host_plan`'s hard `OomdUnverified` gate; the primary gate is **caps verified-effective**. oomd stays as a non-blocking backstop, and its honest residual job is the one co-resident slice still uncapped — see SessionSliceEnforcement below.

## 1. Throughput (the capacity critical path)

- **T0 — build-pool vs runner-slice double-count** [CRITICAL, design call signed]: `runner_slice` must NOT subtract the GHA-CI build memory it already contains. Revised model (smart-pike-244): `runner_slice = host − overhead − session_worst_case_slice − headroom`; `runner_count = min(slice / per_job_peak, effective_build_tokens, cpu_cores_available)` — a 3-axis min-over-constraints. Fix this and the model derives > 0 runners. (smart-pike #5674.)
- **T1 — verified-effective slice cap** (construction; primary wall; the srv2 fix): ctrl PR built + signed, awaiting operator GO to open. The authored cap becomes *effective* on the live cgroup (reconcile verifies `MemoryMax != infinity`), closing the inert-cap hole.
- **T2 — per-slot `MemoryMax` caps**: a job that over-caps is cgroup-killed in isolation, not the host. Part of the apply.
- **T3 — budget manifest** (`.github/fleet-runner-deploy.manifest`: per-host count + per-slot cap + tokens): smart-pike #5674 landed; emits typed `UNSOUND` until the caps are sound (correct fail-closed).
- **T4 — per-job peak / build-pool sizing 24 → 8GiB**: cap `rustc`/slot via jobserver tokens; confirm `mem_reserve`. Same problem as T0 from the sizing angle. **manager-owned.**
- **T5 — ctrl apply**: consume the manifest → enable N slots + per-slot caps + tokens on srv1/srv2, verified-effective. Was blocked on #1753; operator now authorizes execute-ASAP. **The live host apply is surfaced to the operator before it executes.**
- **T6 — cpu.weight priority axis**: runner slice = `BatchLatencyTolerant` (weight 100, cgroup v2 default); sessions slice = `InteractiveLatencySensitive` (weight 1000, 10x). cgroup v2 cpu.weight is work-conserving: raising sessions above runners costs zero CI throughput when sessions are idle; under contention interactive sessions preempt batch builds. Fixes dashboard slowness on srv2 under CI load. (merry-stag-459, `WorkloadClass` modeled + emitted on manifest.)

## 2. Wall-clock (faster runs)

- **W1 — floor `spawn_width` memory-aware**: designed in `std.realization_width`; VERIFY it is wired to the executor. **manager-verify.**
- **W2 — split the cargo-test debug monolith** (~16m tentpole; also halves the T4 peak). **manager-owned.**
- **W3 — sccache corruption flakes** (false reds + full rebuilds). **manager-owned.**

## 3. Reliability (no reds / flakes) — folds in the red-rate lockdown lane

- **R1 — floor OOM exit137**: solved by T1+T2+T4 construction; oomd backstop.
- **R2 — gate-trigger false-green** (`.dag`-only changes run 0 tests). **manager-owned.**
- **R3 — merge-result re-run** (catch stale-green merge skew): VERIFY wired.
- The existing **main-red-rate lockdown** (structural floor defects, merge-freshness, anchor-completeness; crisp-carp-603 / quick-carp-124) is the reliability arm of this plan — the CI-humming manager coordinates it as one CI authority rather than letting throughput and reliability drift into unrelated workstreams.

## 4. The carrier (mechanism for T5 + all host config)

- **C1 — std reconcile / verify-ground carrier** (signed signature; home = std): `reconcile(apply_effect, show_read, grounding) → Converged{evidence} | NotConverged{reason}`, with the §5 type-level invariant that the evidence carrier is constructible only via the grounding over a real read — "verify the realization, not the declaration" promoted to a constructor restriction. valiant-pike-233.
- **C2 — `extdeps/os/systemd` unit-mgmt + apt install-effect**: re-homed under `extdeps/os/`, reusing the existing `extdeps.os.systemd` state. valiant-pike-233.
- **C3 — oomd model + install Realization** (now defense-in-depth, off the critical path): valiant-pike-233 (#5677).
- **C4 — PXE/autoinstall + GCP keyless token**: neat-boar-71 (+ children).
- **One emit-from-fleet-model Realization** (§2 regime-2 projection fold): `ci.yml` + `fleet-runner-deploy.manifest` + Ubuntu user-data + `dnsmasq.conf` + the oomd drop-in are all the same projection.

## 5. SessionSliceEnforcement (the safe-apply gate — new)

The hosts co-host the agent-session fleet AND the CI runners. Full safe-by-construction needs **both** slices capped-verified-effective (runner + session), summing all maxes ≤ 125GiB. Today the agent-session cgroups have **no** citable verified-effective cap — that is the residual over-commit vector, and exactly where oomd-as-backstop earns its keep until a `SessionSliceEnforcement` carrier (parallel to the runner-slot enforcement) lands.

**Apply rule:** do NOT apply the runner cap while sessions are *both* uncapped AND oomd-absent — that is an un-backstopped over-commit (the srv2 vector). Pick (a) cap both slices verified-effective → no oomd needed, or (b) cap the runner slice + keep oomd live as the session backstop. The single `resolve_session_slice` figure is the §3 authority: it is subtracted by the runner budget AND drives the dashboard session reservation — one number, passed *down* into the budget, never imported up.

## 6. Auth (separate, near-resolved)

- **A1 — `resolve_auth` `svc_auth_input` realization**: #5661 (8 approvals, CLEAN) — the positive path, live-verified on GCP.
- **A2 — `AuthDeclaredButUnwired` guard** (additive fail-closed delta on #5661; its own PR): re-homed under neat-boar-71.
- **A3 — perturbation confirm** (the §5 done-bar: drop the token → typed error PRE-SEND, never a remote 401): snappy-otter-298.

## Critical path to capacity (do first, both hosts ASAP)

T0 (un-zero the runner slice) → model derives sane N + per-slot caps → T1+T2 verified-effective apply on srv1+srv2, **with the session slice either capped (5) or oomd-backstopped** → safe by construction (Σ caps + overhead + baselines ≤ physical). T4 (peak → 8GiB) multiplies N afterward. oomd is not required on the critical path.

## Dissolution trigger (DESIGN §6)

Delete this doc when CI throughput, wall-clock, and reliability all fall out of the single modeled fleet budget consumed by a verified-effective apply (T0 un-zeroes the runner slice, T1/T2 caps are verified-effective on srv1+srv2, and the session slice is capped or oomd-backstopped) — so capacity is safe by construction rather than hand-managed.
