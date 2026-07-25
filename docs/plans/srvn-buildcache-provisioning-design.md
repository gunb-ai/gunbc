# srvN build-cache provisioning on the host-standup subsumption spine

Status: **DESIGN ANCHOR** — operator review before implementation (2026-07-17, session wise-heron-850). No load-bearing spine edits in this PR; authority sketch + scaffold rows only.

## 0. Displaced cost (§6 — the pain this removes)

- **Legacy host artifact:** CI and session builds depend on a hand-configured `ctrl-sccache.service` on srv1/srv2 self-hosted runners. The operator is deleting `ctrl/`; the service is misconfigured today.
- **Absorbing fallback masks the deficit (§5):** `ci_release_build_script()` (`dag/gunbc/ci_spec.dag`) wraps the release build in a 3-level retry that escalates from full parallelism → `CARGO_BUILD_JOBS=1` (keep sccache) → **`-u RUSTC_WRAPPER`** (local build, no sccache). That last arm is the exact "degradation is disguised fail-open" pattern DESIGN §5 forbids: the misconfig's frequency is zeroed by construction, so it never ranks for fixing.
- **Symptoms already modeled:** `extdeps.cache.sccache` cites fatal-error log fragments (`encountered fatal error`, exit 254, EAGAIN under jobserver concurrency — see `compute-envelope-model.md` §1). `ci_materialization` opportunistically sets `RUSTC_WRAPPER=sccache` only when `sccache --show-stats` succeeds — it does not *provision* the daemon, only detects an already-running one.
- **host-converge inventory gap:** `sccache config` is listed as an **unmodeled knob** (no `ConvergeTarget` variant, no typed live read — `host-converge-inventory.md` §5). Runner-allocation v0 defers it, but CI reliability does not.

## 1. Operator intent (locked)

> "Get ctrl-sccache onto our srvN subsumption process." Fail closed instead of falling back — sccache misconfigured is the source of the problems; falling back to local build covers it up.

**Sequence (operator-signed in brief):**

| Step | Scope | Done when |
|---|---|---|
| **STEP 1** (this lane) | Model build-cache provisioning on the subsumption spine; srvN host provisions its own sccache deterministically | Green-by-execution witness on fixture; spine row lands with dissolution trigger |
| **STEP 2** (separate PR, operator-sequenced) | Remove `CARGO_BUILD_JOBS=1` / `-u RUSTC_WRAPPER` absorbing fallback from `ci_release_build_script()` | Typed/located/counted REFUSE when cache backend absent; fleet stays green because STEP 1 made provisioning reliable |

STEP 2 is **explicitly out of scope** for the first implementation PR. Removing the fallback before provisioning is reliable would red the whole fleet.

**STEP 2 status (2026-07-23): the cache-drop half LANDED.** `ci_retry_escalation_level2` (`-u RUSTC_WRAPPER`) is deleted — the escalation now stops at `CARGO_BUILD_JOBS=1` (keeps sccache) and then fails loud via verify-build-artifacts, so no arm drops the cache any more. The sketch `provision_build_cache_absorbing_fallback_widen_sketch` stays as the RED control guarding the verdict fold against re-introduction. Level 1 was kept as EAGAIN containment rather than converted to a refusal (§4.4's open decision, resolved toward "keep"). Still open under STEP 2: `ci_sccache_opportunistic_detect` — CI's `if sccache --show-stats` is still an opportunistic detect rather than a fail-closed require, and §5.4 Finding 2 sharpens why that flip needs a supervisor read and not a stats read to key on.

## 2. Reuse map — single authorities, never fork (§3 DFS)

| Concern | Single authority (reuse verbatim) | What's NEW |
|---|---|---|
| what sccache *is* | `extdeps.cache.sccache` (`sccache_local_facts`, cited vendor rows) | nothing — interface shape only |
| catalog → ladder projection | `extdeps.cache.materialization.provider_from_catalog` | role row: host-scope + coverage = runner/session compile frames |
| CI consumer (detect-only today) | `gunbc.ci_materialization.ci_sccache_provider_shell_injection` | STEP 2: flip from opportunistic detect → fail-closed require (dissolution trigger) |
| host-effect apply seam | `gunbc.host_effect` + `gunbc.host_effect_realize.host_effect_apply_gated` | new `ProvisionBuildCache` effect variant + `realize_provision_build_cache` handler |
| toolchain-ensure pattern | `gunbc.srv3_os_install_actuator_toolchain_ensure` (routes through `host_effect_apply`; per-tool ensure + synthesized receipt) | **generalize** to srvN-agnostic `host_toolchain_ensure` — build-cache is a sibling ensure, not a srv3 fork |
| subsumption spine | `gunbc.host_standup.host_standup_spine` | one new assimilation phase row (below) |
| level-triggered verdict | `std.upsert_decision` (`ObservationVerdict`, `UpsertDecision`, `Refuse`) | `BuildCacheProvisionVerdict` instantiates the same fold for cache daemon observation |
| absorbing fallback to kill | `gunbc.ci_spec.ci_cargo_eagain_retry_intent` (levels 1–2) | STEP 2 dissolution only |

**Fork-traps avoided:**

- (a) NOT minting a parallel "build cache" concept — `CacheInterfaceId = sccache_local_id` is the identity; provisioning is a host-effect *realization* of that catalog row.
- (b) NOT keeping srv3-only `WorkflowEnsureActuatorToolchain` as the only toolchain-ensure entry — horizontal generalization (§2) to `HostToolchainEnsure { host, catalog_id }`.
- (c) NOT modeling transport (systemd unit, user service, container sidecar) in extdeps — transport is a Realization handler bound to the agnostic `ProvisionBuildCache` shape.
- (d) NOT folding sccache into `ConvergeKnob` grain in pass 1 — membership/knob grains layer; build-cache provision is a **host standup assimilation act**, not a per-runner slice cap. Converge may gain a read-back row later (`host-converge-inventory` deferrable list).

## 3. Spine attachment — where it lands

### 3.1 Phase placement

Insert a new assimilation phase **`BuildCacheProvision`** on `HostStandupAssimilationPhase`, **after `RunnerDeploySlot` (P1) and before `ComputeFabricEnroll` (P2)**:

```
… P1:runner-deploy-slot
→ P1b:build-cache-provision          ← NEW
→ P2:compute-fabric-enroll
…
```

**Rationale:**

- P1 deploys runner slots; CI builds on those slots need a working per-host compile cache (`PerHostFilesystem` locality in `sccache_local_facts`).
- Build-cache is a **host capability**, not a runner-unit knob — sessions (`ctrl-build`) and runners share the same host-local sccache daemon.
- Must complete before P5 `AssimilationCompleteGate` can honestly claim `RunnerSlotHealthy` (future tightening: runner health may require cache provision receipt).

### 3.2 Spine row shape (implementation PR)

```dag
host_standup_assimilation_step(
  phase: BuildCacheProvision,
  label: "P1b:build-cache-provision",
  disposition: ModeledCompose {
    authorities: [
      decl_ref("gunbc.host_build_cache_provision", "provision_build_cache"),
      decl_ref("gunbc.host_toolchain_ensure", "host_toolchain_ensure"),
      decl_ref("extdeps.cache.sccache", "sccache_local_facts"),
    ]
  },
  effect_evidence: host_effect_identity_evidence,
)
```

Design-anchor PR does **not** edit `host_standup.dag` — this row is the implementation target.

### 3.3 srv3 generalization of toolchain-ensure

Today `WorkflowEnsureActuatorToolchain` exists only in `gunbc.srv3_os_install_actuate_workflow` (srv3 install prefix path). The pattern to generalize:

| srv3-specific today | srvN target |
|---|---|
| `OsInstallActuatorToolchainEnsure {}` host effect | `ProvisionBuildCache { catalog: CacheInterfaceId }` host effect (new) |
| `srv3_os_install_actuator_toolchain_ensure()` | `host_toolchain_ensure(host: HostIdentity, kind: HostToolchainKind)` |
| `WorkflowEnsureActuatorToolchain` workflow step | `WorkflowEnsureHostToolchain { kind: BuildCache \| OsInstallActuator }` |
| `srv3_realize_os_install_actuator_toolchain_ensure_body` | `realize_provision_build_cache` + keep srv3 body as `OsInstallActuator` arm |

`srv3_os_install_actuator_toolchain_ensure.dag` becomes a thin caller of `host_toolchain_ensure(host: srv3, kind: OsInstallActuator)` — no behavioral change, §3 de-fork.

## 4. Authority sketch — `ProvisionBuildCache` HostEffect

### 4.1 Effect shape (extends `gunbc.host_effect.HostEffect`)

```dag
| ProvisionBuildCache {
    catalog_id: CacheInterfaceId,   // witness fixtures pin sccache_local_id; not a String nickname
  }
```

- Targets `HostOs { node: ComputeHost }` only (same cell rule as `OsInstallActuatorToolchainEnsure` — refuse `BmcController` with typed `IncompatibleCell`).
- Transports: `LocalShell` | `SshShell` only; `EmitArtifactThenThinRun` → `NotConverged` (same fail-closed pattern as actuator toolchain ensure).
- Routes through `host_effect_apply_gated` — proof by consumption, not a parallel shell script.

### 4.2 Provision verdict (level-triggered, fail-closed)

```dag
type BuildCacheProvisionRefusal
  = DaemonAbsent                    // no sccache binary or unit
  | DaemonUnresponsive              // --show-stats fails / timeout
  | DaemonMisconfigured { detail: NonEmptyStr }  // EAGAIN / fatal-error fragments from extdeps rows
  | PrivilegeDenied                 // cannot install/start unit
  | CatalogUnknown { id: CacheInterfaceId }      // catalog_id not in cache_catalog

type BuildCacheProvisionVerdict
  = ProvisionConverged { catalog_id: CacheInterfaceId, stats_line: String }
  | ProvisionRefused { cause: BuildCacheProvisionRefusal, reason: NonEmptyStr }

fn provision_build_cache_verdict(
  catalog_id: CacheInterfaceId,
  observed: BuildCacheDaemonObservation,
) -> BuildCacheProvisionVerdict
```

**§5 wall:** there is **no** `ProvisionDegraded` / `ProvisionFallbackLocal` arm. A misconfigured daemon is `ProvisionRefused`, never "proceed without wrapper."

### 4.3 Realization handlers (transport = one of N)

The agnostic ensure sequence (all catalog-driven):

1. **Observe** — `sccache --show-stats` (or typed equivalent); classify against `sccache_fatal_error_log_fragment` rows.
2. **Ensure** (if Absent/Drifted) — install sccache if missing; write config; enable+start daemon.
3. **Read-back** — independent second `show-stats`; converge only on observed evidence (`reconcile_grounded` pattern).

| Handler | When | Dissolution |
|---|---|---|
| `SystemdUserService` | srvN runners (initial target) | default for P1b |
| `SystemdSystemService` | if operator rules shared daemon | alternate row, not forked shape |
| `ContainerSidecar` | future session isolation | YAGNI until displaced cost |

Handler selection is a **dispatch row** in `gunbc.host_build_cache_provision`, not a field on the effect.

### 4.4 Relationship to CI materialization

`ci_sccache_provider_shell_injection` today:

```bash
if sccache --show-stats >/dev/null 2>&1; then
  echo "RUSTC_WRAPPER=sccache" >> "$GITHUB_ENV"
  …
fi
```

STEP 1 makes the `if` condition **true by construction** on subsumed hosts (provision act ran during standup).

STEP 2 replaces the `if` with fail-closed:

- Missing/unresponsive → job refuses with typed diagnostic citing `BuildCacheProvisionRefusal` class (not silent skip).
- Delete `ci_retry_escalation_level2` (`-u RUSTC_WRAPPER`) entirely.
- Level 1 (`CARGO_BUILD_JOBS=1`) — operator decision at STEP 2 cutover: keep as containment-only (EAGAIN) or also refuse; **default design: refuse** — EAGAIN under misconfig is `DaemonMisconfigured`, not a widen.

## 5. Discriminating witness plan

### 5.1 RED control (non-tautological)

The STEP-2 forbidden behavior is modeled as a **separate perturbation sketch**, not asserted from the authority fold:

- `provision_build_cache_absorbing_fallback_widen_sketch` — fabricates `ProvisionConverged` on `DaemonAbsent` (models `ci_retry_escalation_level2` / `-u RUSTC_WRAPPER` widen).
- `build_cache_provision_gate_accepts` — the gate STEP 2 will enforce; returns `false` when absent observation pairs with a Converged verdict.
- `witness_red_control_widen_on_absent_rejected_by_gate` — **discriminating control**: widen sketch *does* fabricate Converged on absent, gate *rejects* it, authority verdict *passes* gate. Perturbation is independent of the property checked.

Dropped: `provision_refused_has_no_fallback_arm` (tautology — both coproduct arms returned `true`). Verdict exhaustiveness is structural (two-arm coproduct, no third arm).

### 5.2 P1b spine ordering justification

Walk of `host_standup_spine` phases that could touch compile/cache **before** P1b:

| Phase | Compiles Rust during standup? | Needs sccache? |
|---|---|---|
| prefix:bmc-credential-converge | No (BMC probe) | No |
| prefix:os-install-mechanism | No (solver) | No |
| prefix:os-install-actuated | No during standup spine itself | No |
| prefix:host-converge | No (policy emit) | No |
| P0:host-identity-converge | No (hostnamectl) | No |
| P0:reach-secrets-network | No (ACL model) | No |
| P1:runner-deploy-slot | No (runner registration — no cargo build in spine) | No |

**After P1b**, consumers that need sccache: CI jobs and ctrl-build sessions run **post-assimilation**, not during earlier spine phases. P2–P5 are fabric enrollment, green-place pin, session placement, and composite gate — none compile Rust during standup execution.

**Conclusion:** no standup phase before P1b performs a Rust compile needing the cache. P1b after runner slot deploy provisions the shared per-host cache (`PerHostFilesystem` in `sccache_local_facts`) before post-assimilation CI/session workloads consume it.

### 5.3 Witness tiers

| Tier | Witness | Green | RED (perturb) |
|---|---|---|---|
| **T1** | `host_build_cache_provision_design_witnesses` | authority refuses absent; widen sketch rejected by gate | widen sketch passes gate → witness false |
| **T1** | catalog authority | `catalog_id == sccache_local_id` pins to cited row | wrong id → `CatalogUnknown` |
| **T2** | spine consumer | `host_standup_spine` includes `P1b:build-cache-provision` row | remove row → `host_standup_gap_count` witness fails |
| **T3** | srv1 dry-run | `host_toolchain_ensure(host: srv1, kind: BuildCache)` dry-run receipt on operator host | — |
| **T4** | live read-back | **superseded — see §5.4.** A post-apply stats read cannot discharge this tier | hand-stop daemon → next standup run → `ProvisionRefused`, counted |
| **T5** | CI consumer | STEP 2: build job without `RUSTC_WRAPPER` unset arm stays green for 7d on main | disable daemon on one host → build job typed refuse, not local fallback |

Design-anchor PR lands **T1** only (scaffold + type witnesses). T2+ follow implementation PR.

### 5.4 T4 restated — a stats read-back cannot discharge it (2026-07-25)

T4 was written as "post-apply `sccache --show-stats`, independent of our write." Live execution on srv3 falsified **both** halves of that, and the tier is restated here rather than quietly re-scoped.

**Finding 1 — the unit still died, for a second reason.** #7206 foregrounded the server (`Type=simple` + bare `ExecStart` + `SCCACHE_START_SERVER=1` / `SCCACHE_NO_DAEMON=1`), which fixed the daemonize-and-exit no-op. A read-back six hours later found the unit `inactive (dead) since 06:38:56`, `Main PID … (code=exited, status=0/SUCCESS)`, `Duration: 10min 8.215s`, and **no sccache process at all**. Root cause: `SCCACHE_IDLE_TIMEOUT` defaults to 600s, so the foreground process exits *cleanly* on the first quiet ten minutes, and `Restart=on-failure` cannot fire on a success exit. Same end state as the bug #7206 fixed, one level deeper — and #7206's own claim that "the live srv3 re-provision carries the runtime is-active grain" was the overclaim that hid it, because that read was taken in the same minutes as the write.

- **Fix:** `Environment=SCCACHE_IDLE_TIMEOUT=0` — cited to sccache `docs/Configuration.md`: *"how long the local daemon process waits for more client requests before exiting, in seconds. Set to 0 to run sccache permanently."*
- **Not `Restart=always`:** that would relaunch *through* the clean exit and make the deficit invisible — §5's absorbing fallback. With the timeout disabled a status-0 exit is unreachable by configuration, so `on-failure` is the honest policy and a dead unit stays a real, refusable state.
- **Receipt (green):** modeled unit rendered from `build_cache_systemd_user_unit_body()`, installed on srv3, `active (running)`, `NRestarts=0`, same `Main PID`, read back **13 minutes** after the write — past the 600s timer that killed the previous form.
- **Receipt (red control):** the identical unit *without* the knob — i.e. exactly what #7206 shipped — exits 0 after 10min 8s. This is the discriminating pair; the construction check `build_cache_unit_daemon_runs_until_stopped` is parameterized over a unit body so the witness runs both sides.
- **srv1/srv2 are not the counter-example they look like.** The production `ctrl-sccache.service` does not set the knob either; its month-plus uptime is CI traffic keeping the daemon below the idle threshold — accidental coverage, not a durable unit. So this line is a **deduction** from the cited doc plus the srv3 receipt, not a swap toward production.

**Finding 2 — `--show-stats` is not an independent observation.** Any sccache invocation auto-starts a server when none is listening, so the probe manufactures the evidence it reports. On srv3 it printed a full stats table while the unit had been dead for six hours. Read through the verdict fold as it stood, that host observed `DaemonStatsOk` and **converged** — a provisioning act reporting success over a host with no durable cache (§5 fabricated plausible output, the same shape as `DaemonPathShadowed`).

- **Model:** `DaemonUnsupervised { unit, unit_state }` → `RefusalDaemonUnsupervised`. "A daemon answered" and "the modeled supervisor is running it" are different states with different remedies, so they get different names rather than one `Ok`.
- **Live observation STAGED**, on the PATH-shadow precedent (review 42343): the `systemctl --user is-active` read is *not* emitted as hand-shell into the A5 census row; it lands with the typed-argv dissolve-on (#5828). The model carries the state and its refusal today.
- Construction covers hosts *this lane* provisions (the self-exiting unit is unwritable); the observation's standing job is the residue construction cannot reach — a unit stopped, masked or replaced out of band.

**T4 restated.** The tier is discharged by a **delayed supervisor read**, not a post-apply stats read: after provisioning, read `systemctl --user is-active` (and the main PID) **separated from the write by longer than every self-exit timer in the supervised process**, on each subsumed host. An is-active read taken at provisioning time proves the unit *started*, never that it *stays*.

**T4 status:** srv3 **green** at the restated bar (13-minute delayed read, modeled unit, PID stable). srv1 and srv2 remain **open** — not for a modeling reason: neither host is reachable from a session container with the fleet key today (`Permission denied (publickey)` for `ubuntu`/`briansrls`/`node-orch`), which is the A1/A2/B2 reach-and-identity gap already typed in `gunbc.plans.fleet_subsumption_manual_gaps`. The downstream triggers (`ctrl_sccache_service_retired`, and the `ci_sccache_opportunistic_detect` cutover) stay blocked on those two hosts, and the srv1/srv2 units should be expected to carry the same latent defect until re-provisioned from the model.

## 6. Dissolution triggers (every scaffold names its exit)

| Scaffold | Dissolves to | Trigger |
|---|---|---|
| `host_build_cache_provision_scope` | `SingleAuthority` on `provision_build_cache` | P1b spine row merged + T2 witness green |
| `host_toolchain_ensure_srv3_fork` | `host_toolchain_ensure` generic | srv3 module thinned to one-line caller |
| `ctrl_sccache_service_retired` | deleted host artifact | T4 on both srv1+srv2 + operator confirms ctrl/ deletion |
| `ci_sccache_opportunistic_detect` | fail-closed require | STEP 2 operator cutover after T4 |
| `ci_cargo_eagain_fallback_level2` | typed `ProvisionRefused` propagate | STEP 2 — same cutover PR as row above |
| `host_converge_sccache_knob` (future) | `ConvergeTarget` + live read | optional; P1b assimilation act is sufficient for v0 |

## 7. Implementation phases (post-design)

- **P-A (model):** `gunbc.host_build_cache_provision` types + fixture verdict fold + T1 witnesses (this design's scaffold module grows here).
- **P-B (realize):** `realize_provision_build_cache` in `host_effect_realize.dag`; systemd handler for srv1/srv2; `host_toolchain_ensure` generic entrypoint.
- **P-C (spine):** `BuildCacheProvision` row in `host_standup.dag`; gap ledger update; `host_standup_spine_witness_test` extended.
- **P-D (srv3 de-fork):** srv3 actuator toolchain ensure calls generic `host_toolchain_ensure`.
- **P-E (STEP 2 — operator-gated):** remove CI fallback; `ci_spec_witness_test` golden updated; discriminating RED proves refuse.

## 8. Open questions (operator)

1. **Systemd scope:** user service (`systemd --user`, runner uid) vs system service for sccache daemon on srv1/srv2?
2. **EAGAIN at STEP 2:** refuse outright vs retain `CARGO_BUILD_JOBS=1` containment without `-u RUSTC_WRAPPER`?
3. **AssimilationCompleteGate:** should `RunnerSlotHealthy` require `BuildCacheProvisionConverged` explicitly, or stay implicit through CI?

## 9. Provenance

Files DFS'd (read-only): `gunbc/host_standup.dag`, `gunbc/host_standup_assimilation_deduction.dag`, `gunbc/srv3_os_install_actuator_toolchain_ensure.dag`, `gunbc/srv3_os_install_actuate_workflow.dag`, `gunbc/host_effect.dag`, `gunbc/host_effect_realize.dag` (actuator toolchain realize), `gunbc/ci_spec.dag` (retry levels), `gunbc/ci_materialization.dag` (shell injection), `extdeps/cache/{sccache,cache,materialization}.dag`, `docs/plans/host-converge-inventory.md`.

Prior lanes (do not re-derive): session-loyal-bear-281 (srvN subsumption), session-neat-dove-454 (assimilation-deduction PR1).
