# Host-Converge Control-Plane Inventory (srv1/srv2 lane)

READ-ONLY map. No host mutation, no new vocabulary, no cap/scheduler change. Produced by
the `host-converge-inventory-a` scout for the srvN convergence lane (owner: `bright-crab-27`).
This is a *map of what exists*, not a design; `bright-crab-27` owns the design that consumes it.

Convention in the tables below:
- **Model source** — the `.dag` declaration that owns the desired fact.
- **Emitted artifact** — where the actuation is projected to (today: opaque bash in `.github/fleet-converge.sh`).
- **Live read** — the typed read-back of the *effective* on-host value, if one exists.
- **Ownership** — `model-owned` (desired value derived in `.dag`) vs `hand-patched` (a literal/placeholder).
- **Status** — convergence posture today.

---

## 0. Layer orientation (what is the base, and what is NOT)

The converge base is **`gunbc.host_converge.HostConverge` / `ConvergeKnob`**, NOT `fleet_intent`.
Three separate fact-families feed it and must stay separate (do not fuse):

| Family | Owner module | Role | Relationship to converge |
|---|---|---|---|
| **Physical inventory** | `gunbc.fleet_intent` | `ComputeHost`, `ExecutionSurface`, `MemoryDevice`, `StorageMount`, `CpuFacts`, per-host `srv1_host`/`srv2_host`/`srv3_host` rows | UPSTREAM supply. Feeds placement; is not a converge knob. |
| **Caps / admission / budget** | `gunbc.fleet_host_budget` | `host_allocation_conserves`, `runner_slice_cap(...)`, `gunbc_runner_pool_budget`, `oomd_actuation_floor`, `FleetHostPlan` | Derives the desired cap *values*. A separate soundness fact (conservation), NOT the converge shape. |
| **Placement / deployment** | `gunbc.ci_runner_placement` | `RunnerHostDeployment { runner_count, runner_slice_cap, per_runner_memory_cap, build_tokens, workload_class }`, `RunnerDeploymentPlan` | The SOURCE of each knob's `desired`. `fleet_converge_policy` reads this. |
| **Converge (this lane)** | `gunbc.host_converge` | `HostConverge`, `ConvergeKnob`, `ConvergeTarget` (6 variants), `ConvergePolicy`, `fleet_converge_policy()` | The typed spine. Desired ← placement; effective ← live read (mostly absent). |

`fleet_converge_policy()` = `fleet_converge_policy_from(runner_deployment_plan(), gunbc_ci_session_reservation_mode)`.
It folds each `RunnerHostDeployment` into a `HostConverge` via `host_converge_of` → three knob groups:
`gunbc_pinned_tree_knobs` ++ `runner_host_knobs` ++ `session_host_knobs`.

**Transport is a handler, never fused into the converge shape.** Today the only transport is the
committed bash projection `.github/fleet-converge.sh`; see §3. `#6171 std.resource_namespace`
is a sibling lane, not a dependency here.

---

## 1. Knob table — every live-managed knob

`ConvergeTarget` has 6 variants; every knob is one of them:
`SliceProperty` · `PerSlotMemoryCap` · `RunnerWidth` · `JobserverTokens` · `VerifyOnlyCap` · `GunbcPinnedTree`.

### 1a. Runner slice (`RunnerSlice`) — source: `runner_host_knobs(h: RunnerHostDeployment)` in `host_converge.dag`

| Knob (`name`) | Target variant | systemd property / unit | Desired source | Emitted fn (fleet-converge.sh) | Typed live read | Ownership | Status |
|---|---|---|---|---|---|---|---|
| `runner_slice_cap_bytes` | `SliceProperty` | `MemoryMax` on `system-actions-runner.slice` | `h.runner_slice_cap` (byte_size) | `converge_slice_property` | **none** — `gunbc_runner_slot_show_effective_read = ReadAbsent` | model-owned | **STOP-POINT** (no live read) |
| `per_slot_memory_max_bytes` | `PerSlotMemoryCap` | `MemoryMax` drop-in `20-fleet-width.conf` + per-instance | `h.per_runner_memory_cap` | `converge_per_slot_cap` | **fixture only** — `runner_unit_live_read.dag` parses `MemoryMax`→`SystemdCgroupMemoryLimit` (Terminal, fixture) | model-owned | fixture-typed; **live transport absent** |
| `per_slot_memory_swap_max_bytes` | `PerSlotMemoryCap` | `MemorySwapMax` drop-in `30-fleet-swap.conf` | literal `"0"` | `converge_per_slot_cap` | **fixture only** — same carrier parses `memory_swap_max` | model-owned (literal 0) | fixture-typed; live transport absent |
| `cpu_weight` (runner) | `SliceProperty` | `CPUWeight` on runner slice | `workload_class_cpu_weight(BatchLatencyTolerant)=100` | `converge_slice_property` | **none** | model-owned | **STOP-POINT** |
| `build_tokens` | `JobserverTokens` | env file `/etc/default/ctrl-jobserver` key `CTRL_JOBSERVER_TOKENS`, service `ctrl-jobserver.service` | `hardware_thread_count_value(h.build_tokens)` | `converge_jobserver_tokens` | **none** (grep of env file inside shell, not typed) | model-owned | **STOP-POINT** |
| `runner_count` | `RunnerWidth` | active-unit count of `actions-runner@srvN-*.service` | `h.runner_count` | `converge_runner_width` | **none** (unit-count only in shell) | model-owned | **STOP-POINT** (INCREASE is a no-op in shell — needs slot provisioning) |

Note `runner_unit_live_read.dag` also parses `MemoryHigh` (`memory_high`) into `SystemdCgroupMemoryLimit`,
but **no `ConvergeKnob` emits `MemoryHigh`** — the typed read exists ahead of any knob for it.
`TasksMax` is **not modeled** as a knob (see §5 stop-points).

### 1b. Sessions slice (`SessionsSlice`) — source: `session_host_knobs(slice_max, per_session)` in `host_converge.dag`

Emitted only when `session_knobs_by_host(mode)` resolves (`Regime2AggregateOomd` + `SessionAggregateMeasured`); else `[]`.

| Knob (`name`) | Target variant | systemd property / unit | Desired source | Emitted fn | Typed live read | Ownership | Status |
|---|---|---|---|---|---|---|---|
| `slice_max_bytes` | `SliceProperty` | `MemoryMax` on `sessions.slice` | measured aggregate reservation | `converge_slice_property` | **none** | model-owned | **STOP-POINT** |
| `per_session_max_bytes` | `VerifyOnlyCap` | `MemoryMax` (verify only — spawn path owns the write) | `ceiling` from reservation mode | `converge_verify_only_cap` | **none** (shell reads `ConsistsOf`→scope, untyped) | model-owned (verify-only) | **STOP-POINT** |
| `oom_pressure_kill` | `SliceProperty` | `ManagedOOMMemoryPressure` (`kill`) | `gunbc_oomd_sessions_policy` | `converge_slice_property` | **none** | model-owned | **STOP-POINT** |
| `oom_pressure_limit_pct` | `SliceProperty` | `ManagedOOMMemoryPressureLimit` | `gunbc_oomd_sessions_policy` limit (note desired display `60` vs effective raw `2576980377`) | `converge_slice_property` | **none** | model-owned | **STOP-POINT** (desired/effective representation mismatch — see §5) |
| `cpu_weight` (sessions) | `SliceProperty` | `CPUWeight` on `sessions.slice` | `workload_class_cpu_weight(InteractiveLatencySensitive)=1000` | `converge_slice_property` | **none** | model-owned | **STOP-POINT** |
| *(membership probe)* | — (`SessionsMembershipProbe`) | cgroup `sessions.slice` vs legacy `system.slice` | `sessions_membership_probe` | `emit_sessions_membership` | count-only, untyped | model-owned | signal only, not a knob |

### 1c. Gunbc pinned tree (`GunbcSlice`) — source: `gunbc_pinned_tree_knob(host)` in `host_converge.dag`

| Knob (`name`) | Target variant | Managed object | Desired source | Emitted fn | Typed live read | Ownership | Status |
|---|---|---|---|---|---|---|---|
| `pinned_tree_sha` | `GunbcPinnedTree` | pinned gunbc checkout + binary provenance + green-place marker + quiescent reload | desired pin ← ctrl `third_party/gunbc` gitlink (never a constant) | `converge_gunbc_pinned_tree` | **shell-only** pin reads (stamp files / git HEAD / `/proc/<pid>/environ`), NOT typed in `.dag` | model-owned; **binary digest is a hand-patched placeholder** (`gunbc_fleet_pinned_binary_digest = "sha256:BOOTSTRAP_DIGEST_PLACEHOLDER"`, marked Scaffold) | no-op policy typed (`gunbc_pinned_noop_satisfied`), reads untyped |

`GunbcPinnedTree` carries a rich sub-model (`DagRootBinding`, `GunbcBinaryProvenancePolicy`,
`GreenPlaceReadinessMarker`, `ConvergeLiveApplyPolicy`, `SpawnRetryBackstop`). The no-op decision
IS typed in `.dag` (`gunbc_sha_triple_coherent`, `gunbc_process_effective_converged`,
`gunbc_green_place_marker_satisfied`), but the underlying reads are opaque bash.

### 1d. Knobs the brief names that are NOT modeled as `ConvergeKnob` today

Enumerated so later slices know they are greenfield, not existing:
`MemoryHigh` (typed read exists, no knob) · `TasksMax` · runner **labels** (`RunnerSpec.SelfHosted { labels }`
lives in `RunnerHostDeployment`/`fleet_intent`, not projected to a converge knob) · **sudoers grants** ·
**tailscale ACL / deploy route** · **sccache config** · **OOM/swap counters** (only `emit_sessions_membership`
count exists). None of these have a `ConvergeTarget` variant or a typed live read → all §5 stop-points.

### 1e. Last live receipt / source on record (which knobs have ANY real observed value vs pure ReadAbsent)

To keep the §1 tables readable, the observed-value provenance is consolidated here. **Exactly one knob
family has any observed value on record today — and it is a recorded *fixture*, not a live-transport read.**

| Knob | Last observed value on record | Source | Nature | Reads as |
|---|---|---|---|---|
| `per_slot_memory_max_bytes` (srv1-01) | `MemoryMax = infinity` | `gunbc_srv1_runner_unit_live_read_fixture` (`runner_unit_live_read.dag`), `probed_at: "2026-07-01"`, unit `actions-runner@srv1-01.service` | **fixture row** (Terminal disposition), NOT a live transport read | **DRIFTED** — uncapped; the inert-cap OOM-reboot vector `runner_slot_enforcement` refuses to ground |
| `per_slot_memory_swap_max_bytes` (srv1-01) | `MemorySwapMax = 0` | same fixture row | fixture row | converged-shaped (equals desired `"0"`), but off a fixture not a live read |
| `memory_high` (srv1-01) | `MemoryHigh = infinity` | same fixture row | fixture row | no knob emits `MemoryHigh` — observed value exists ahead of any knob |
| **every other knob** | *none* | — | `ShowEffectiveRead = ReadAbsent` (runner slice) / no read carrier at all (sessions, jobserver, width, pinned-tree) | pure `Absent` / unenforced, fail-closed |

Reading of the one receipt: the single on-record observation *already shows drift* (srv1-01 per-slot `MemoryMax`
reads `infinity` = uncapped), which is exactly why `runner_slot_enforcement`'s committed posture is
`RunnerSlotUnenforced`. It is a fixture, so it is not proof of the current live host — but it is the shape a
real receipt will take, and it is the reason the first live mutation (§1f) targets a per-slot MemoryMax cap.

### 1f. First narrow hunk candidate (the deliberately-narrow first live-mutation target)

**Candidate: srv2, one runner unit, `per_slot_memory_max_bytes` (+ its paired `per_slot_memory_swap_max_bytes`).**

| Axis | Value |
|---|---|
| Host | `srv2` (existing host, `ExistingHostQuiescentReload` apply mode — not the srv3 fresh-standup path) |
| Knob | `per_slot_memory_max_bytes` on one `actions-runner@srv2-NN.service` instance, paired with `per_slot_memory_swap_max_bytes` |
| Target variant | `PerSlotMemoryCap` (drop-in + per-instance `set-property`) |
| Desired source | `RunnerHostDeployment.per_runner_memory_cap` (already model-owned) |
| Why narrowest | (a) it is the ONLY knob with a typed read shape already built (`RunnerUnitMemoryLiveRead`, §1e); (b) the keyed-delta fold is already exercised on exactly this srv2 + `per_slot_memory_max/swap` pair by `host_converge_delta_witness_test.dag` (2-entry patch, apply-hunk verdicts); (c) `MemoryMax` is drain-free and per-instance-scoped — no runner-width change, no session-slice coupling, no pinned-tree reload. |
| Reuse path | fixture `RunnerUnitLiveReadRow` → `runner_unit_live_read_typed` → `RunnerSliceCapEffectiveness` → `reconcile_runner_slot` → `Reconciliation<RunnerSlotApplyEffect, RunnerSlotEnforcement>`; only the live `systemctl show` transport (currently `ReadAbsent`) is new. |
| Stays narrow by | one host, one instance, one property pair; verify-first (read before write); everything else stays `ReadAbsent`/unenforced. |

This is a *candidate*, not a decision — the owning lane ratifies the actual first target.

---

## 2. Existing converge carriers to REUSE (so later slices extend, never fork)

These are the single authorities a new slice must build on rather than re-mint:

1. **`std.realization_reconcile.Reconciliation<Applied, Evidence>`** — `Converged { evidence, applied }` /
   `NotConverged { reason, applied }`, helper `reconciliation_converged`. The universal apply/read-back
   result. Instantiated in this lane as `Reconciliation<HostEffectIntent, HostEffectEvidence>`.
2. **`std.realization_reconcile.ShowEffectiveRead<T>`** — `ReadObserved { value } | ReadAbsent { reason }`,
   with `Grounding<T>` (`Grounds`/`DoesNotGround`) and `reconcile_grounded`. This is the typed live-read
   carrier. `runner_slot_enforcement.dag` is the copy-me for "a knob whose live read is not wired yet":
   its committed instance is `ReadAbsent`, fail-closed to `RunnerSlotUnenforced`.
3. **`std.change` keyed-delta engine** — `KeyedRow`, `KeyedPatch`, `KeyedThreeWayPatch`, `KeyedLeafVerdict`
   (`KeyedApplyHunk`/`KeyedUnchanged`/`KeyedConflict`/`KeyedVerdictAdded`/`KeyedVerdictRemoved`),
   `keyed_two_way_diff`, `keyed_three_way_fold`, `keyed_leaf_verdict_fold` (`KeyedLeafVerdictFold`).
   Already wired for knobs in **`host_converge_delta.dag`** (`host_converge_two_way_diff`,
   `host_converge_three_way_patch`, `host_converge_apply_patch`, `host_converge_invert_patch`).
4. **`gunbc.host_converge`** — `ConvergeKnob`, `ConvergeTarget` (6 variants), `ConvergeVerdict`
   (`Converged | Drifted | Absent`), `converge_verdict`, `ConvergeNoOpPolicy`, `ConvergeApplyMode`.
   Any new knob is a new row of `ConvergeKnob`, not a new type.
5. **`gunbc.runner_unit_live_read`** — the raw→typed read pattern:
   `RunnerUnitLiveReadRow` (raw `*_raw: String`) → `runner_unit_live_read_typed` →
   `RunnerUnitMemoryLiveRead` (`SystemdCgroupMemoryLimit`), projecting to `ConvergeVerdict`.
   Copy this shape for any new typed read; note it is **fixture-only (Terminal disposition)** — it
   does not yet supply a live transport.
6. **`gunbc.host_identity_converge` / `host_identity_assimilation`** — THE adopt→apply→read-back→noop
   copy-me loop on the `Reconciliation` spine:
   - verdict (`host_identity_precondition_verdict`, three-way base=persisted `DeployedIntentV0`,
     observed=live, desired=`DeployedIntentV1`) → `HostIdentityPlan` (`ApplyPlan`/`NoopPlan`/`ConflictPlan`).
   - `host_identity_plan_effect_script` returns `none` on Noop/Conflict (no-shell-on-conflict by construction).
   - apply: `host_identity_apply_gated` → `Reconciliation`; read-back: `host_identity_observation_live`;
     converged: `host_identity_apply_converged`.
   - orchestration: `host_identity_converge_with_observation` runs the loop and re-folds a *reapply verdict*
     to prove `reapply_noop` (idempotence/fixpoint). Receipt: `HostIdentityAssimilationReceipt`
     (`apply_converged`, `read_back_converged`, `reapply_noop`); validity gate
     `host_identity_assimilation_receipt_is_valid`.
   - idempotency declared, not re-derived: `EffectShape`/`UpsertEffect` + `KeySource`/`CompositeKey` +
     `IdempotencyEvidence`/`LatticeEffect` → `is_idempotent_effect` → `host_identity_apply_policy`
     (`OneShotIdempotent` vs `ConvergeToFixpoint`).
7. **`gunbc.host_effect` / `host_effect_realize`** — `HostEffectIntent`, `HostEffectEvidence`,
   `ShellCommand`, `HostEffectTransport`, `NodeControlPlane`, `Policy`; apply via `host_effect_apply_gated`.
   The transport is a bound handler here — keep it out of the converge shape.

Caution flagged by the sub-scan: two `Absent` symbols are in scope — option `Absent` and
`gunbc.host_converge.Absent` (a `ConvergeVerdict` variant). Keep them namespaced in new slices.

---

## 3. `fleet-converge.sh` shell actions → typed-knob candidates

`.github/fleet-converge.sh` is GENERATED by `dag/gunbc/fleet_converge_emit.dag` (do-not-hand-edit banner).
It is a **Regime-2 pure projection** of `fleet_converge_policy`. **All actuation logic is opaque bash
string-blobs** — this is the anti-pattern the lane maps (not fixes here). Each shell fn corresponds to a
`ConvergeTarget` variant; the read-back logic inside each is untyped bash to be lifted into typed reads.

| Shell fn | `ConvergeTarget` it projects | Actuation (opaque) | Read-back (opaque → candidate for typing) |
|---|---|---|---|
| `converge_slice_property` | `SliceProperty` | `systemctl set-property UNIT PROP=VAL` | `systemctl show UNIT --property=PROP --value` → **candidate: `SystemdCgroupMemoryLimit`/typed scalar** |
| `converge_per_slot_cap` | `PerSlotMemoryCap` | write drop-in + `daemon-reload` + per-active-instance `set-property` | `systemctl show <unit> --property` loop → **candidate: reuse `runner_unit_live_read_typed`** |
| `converge_runner_width` | `RunnerWidth` | drain-then-stop surplus (DECREASE only; INCREASE = stderr no-op) | active-unit count via `list-units ... | awk END{print NR}` → **candidate: typed `RunnerWidthLiveRead { active_count }`** |
| `converge_jobserver_tokens` | `JobserverTokens` | write env file + `systemctl restart` | `grep ENV_KEY env_file | cut -d=` → **candidate: typed env-file read** |
| `converge_verify_only_cap` | `VerifyOnlyCap` | none (verify only) | `ConsistsOf`→scope→`show PROP` → **candidate: typed scope read** |
| `converge_gunbc_pinned_tree` | `GunbcPinnedTree` | git checkout / binary pull / quiescent-reload hook | multi-source pin reads (`converge_read_gunbc_host_pin`, `converge_read_ctrl_gunbc_pin`, `converge_dashboard_process_pin`) → **candidate: typed `GunbcPinTriple { binary, dag, ctrl }`** |
| `emit_sessions_membership` | `SessionsMembershipProbe` | none | count `docker-*.scope` in slice vs legacy → **candidate: typed `SessionMembershipCount`** |
| `decide_verdict` / `host_summary` | `converge_verdict` (already single authority) | — | already realizes `gunbc.host_converge.converge_verdict`; the receipt grammar is the alignment point |

The shell has invariants worth preserving in any typed lift: **caps-before-widen** (`runner_count` applied
LAST), **DECREASE-only width** (drain, never SIGKILL), **verify-only** sessions cap, **fail-closed** empty reads
(`ABSENT`→`absent` verdict), and the **process-effective** (not disk) pin read for the dashboard.

---

## 4. First proposed `HostConvergeKey` set (key axes for the keyed fold)

Today's keyed fold (`host_converge_delta.dag`) keys on **`ConvergeKnobKey { slice: ConvergeSlice, name: String }`**
— it is **per-host implicitly** (the fold runs within one `HostConverge`). For a *fleet-wide* keyed fold
across hosts, the key must gain the host axis. Proposed (naming for `bright-crab-27` to ratify — not minted here):

```
HostConvergeKey {
  host:  HostIdentity        # srv1 | srv2 | srv3  (from RunnerHostDeployment.identity)
  slice: ConvergeSlice       # GunbcSlice | RunnerSlice | SessionsSlice
  name:  String              # the knob name, e.g. "per_slot_memory_max_bytes"
}
```

Rationale: `(host, slice, name)` uniquely identifies a live-managed knob across the fleet; `slice` and `name`
already form the intra-host key; `host` is the only missing axis and is already present as
`HostConverge.identity` / `RunnerHostDeployment.identity`. The value under the key stays `ConvergeKnob`
(reuse `converge_knob_values_equal`, grounded per-target in #6175). Do **not** put the systemd unit or property
in the key — those are payload on `ConvergeTarget`, and folding on them would fork the key from the knob identity.

Open question for the owner: whether the per-target unit (`system-actions-runner.slice` vs per-instance glob)
needs to be in the key when one `name` fans out to N active instances (`converge_per_slot_cap`). Recorded, not decided.

---

## 5. Stop-points — knobs with NO typed live read (these gate later mutation)

Per the brief: where a live read output cannot be represented typed today, it is recorded, not invented.
**Every knob below is fail-closed to `Absent`/unenforced until a typed live-read transport is wired.**

| Stop-point | Why it blocks | Nearest existing carrier to extend |
|---|---|---|
| **Runner slice `MemoryMax` / `CPUWeight` live read** | `gunbc_runner_slot_show_effective_read = ReadAbsent` (committed posture `RunnerSlotUnenforced`); no live `systemctl show` transport has run on the fleet. | `ShowEffectiveRead<RunnerSliceCapEffectiveness>` (`runner_slot_enforcement.dag`) |
| **Per-slot memory/swap cap live read (real transport)** | `runner_unit_live_read.dag` is Terminal **fixture-only**; explicitly does NOT replace the ReadAbsent transport node. | `RunnerUnitMemoryLiveRead` (typed shape ready; needs live rows) |
| **`build_tokens` / jobserver env live read** | only a bash `grep` of the env file; no typed read. | new, model on `RunnerUnitLiveReadRow` raw→typed pattern |
| **`runner_count` active-width live read** | shell `list-units | awk NR` only; no typed `RunnerWidthLiveRead`. | new typed row |
| **All sessions-slice knobs** (`slice_max`, `per_session` verify, `oom_pressure_kill`, `oom_pressure_limit_pct`, sessions `cpu_weight`) | no typed live read for any; `ManagedOOMMemoryPressureLimit` also has a **desired/effective representation mismatch** (`60` vs raw `2576980377`) that is not yet grounded typed. | `SystemdCgroupMemoryLimit` + a new oomd-pressure typed read |
| **`pinned_tree_sha` reads** | pin triple + process-effective + marker reads are all opaque bash (`/proc/<pid>/environ`, git HEAD, stamp files); no typed `GunbcPinTriple`. Binary digest is a **hand-patched placeholder** (`BOOTSTRAP_DIGEST_PLACEHOLDER`, Scaffold-marked). | typed pin-triple carrier + resolve the digest single-authority |
| **Unmodeled knobs entirely** (no `ConvergeTarget` variant *and* no read): `TasksMax`, `MemoryHigh` knob, runner **labels**, **sudoers grants**, **tailscale ACL / deploy route**, **sccache config**, **OOM/swap counters** | greenfield — not just missing a read, missing the knob. | new `ConvergeTarget` variants (owner's call) |

Representation stop-flag (do-not-invent, per brief): `oom_pressure_limit_pct` emits desired `60` but the
effective wire value is `2576980377` (a byte count). Whether the typed live read grounds a **percent** or a
**derived byte ceiling** is a modeling decision recorded here, not resolved.

### 5a. Which stop-points BLOCK runner-allocation-v0 vs which are deferrable

runner-allocation-v0 is the milestone that derives and *reads back* the runner memory/width allocation on the
fleet. A stop-point BLOCKS it iff a typed live read must exist before the allocation can be verified converged
(fail-closed: no read ⇒ `RunnerSlotUnenforced` ⇒ the allocation cannot claim it landed).

| Stop-point | v0 disposition | Rationale |
|---|---|---|
| Runner slice `MemoryMax` live read | **BLOCKS** | The slice cap is the allocation's top-level envelope; conservation (`host_allocation_conserves`) is premised on it being effective. Must read back typed. |
| Per-slot memory/swap cap live read (real transport) | **BLOCKS** | This is the per-runner allocation itself; the typed shape (`RunnerUnitMemoryLiveRead`) is ready, only the transport is missing. This is the §1f first hunk. |
| `runner_count` active-width live read | **BLOCKS** | The allocation's cardinality; without a typed width read-back v0 cannot confirm the derived `runner_count` matches live active units. |
| Runner slice `CPUWeight` live read | **deferrable** | Work-conserving; wrong/absent CPUWeight does not break the *memory* allocation soundness. Nice-to-verify, not a v0 gate. |
| `build_tokens` / jobserver env live read | **deferrable** | Build parallelism, not a memory-allocation fact; jobserver drift degrades throughput, it does not invalidate the allocation. |
| All sessions-slice knobs | **deferrable** | Separate slice (sessions ≠ runner allocation); coupled only through host conservation arithmetic, which is already model-side. |
| `pinned_tree_sha` reads | **deferrable** | Deploy/version concern, orthogonal to the memory allocation read-back. |
| Unmodeled knobs (`TasksMax`, `MemoryHigh` knob, labels, sudoers, tailscale, sccache, OOM counters) | **deferrable** | Greenfield; not on the runner-allocation-v0 critical path. |

Net: **three typed reads gate runner-allocation-v0** — runner-slice `MemoryMax`, per-slot `MemoryMax`/`MemorySwapMax`,
and `runner_count` width. All three extend carriers that already exist (`ShowEffectiveRead<RunnerSliceCapEffectiveness>`,
`RunnerUnitMemoryLiveRead`, and a new `RunnerWidthLiveRead` row); each needs its live `systemctl`/`list-units`
transport wired. Everything else is deferrable past v0.

---

## Provenance

Files DFS'd (read-only): `dag/gunbc/host_converge.dag`, `host_converge_delta.dag`,
`fleet_converge_emit.dag` (via its emitted `.github/fleet-converge.sh`), `host_identity_converge.dag`,
`host_identity_assimilation.dag`, `host_identity_observation.dag`, `runner_unit_live_read.dag`,
`runner_slot_enforcement.dag`, `fleet_host_budget.dag`, `fleet_intent.dag`, `ci_runner_placement.dag`,
`dag/test/claim/host_converge_delta_witness_test.dag`. No files were modified.
