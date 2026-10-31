# Shell → DAG: residual census, thin-run scoping, and arc completion

> **Status: dated assessment + scoping draft (witty-ibex-317, 2026-07-12).** Grounded against `origin/main` @ #6507. Builds on the landed slices (Retry #6467, If-band #6475, typed transports #6457/#6478) and lane D's authority [provisioning-window-executor-capability-design.md](https://github.com/gunb-ai/gunbc/blob/d1f6922580962d14870577f1352e62966073c4bb/docs/plans/provisioning-window-executor-capability-design.md). No `.dag` edits — records direction for operator + host-effect-lane sign-off. ⚠ FLAG = a decision needing sign-off.

[Original census, superseded dispatch ledgers and correction receipts](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/plans/shell-to-dag-residual-census-and-arc-completion.md). The scope and remaining obligations below are not a claim that every historical site remains live.

## 0. The one finding

The remainder of the ShellProgram→DAG arc is **not** "migrate N files onto the v2 bash emitter." It splits cleanly:

- **Legitimate shell** (foreign executors + pre-runtime bootstrap) stays shell, emitted through the **v2 bash rows** (grammar-owned), and is *bounded* — a roster, not a growth surface.
- **Runtime-present shell** (fleet-converge steady-state, the srv3 install/reconcile tails, host-identity converge) **all funnels onto one keystone: `host_effect_apply` carrying typed effects.** Two sites already prove the pattern (`floor_diff_observe.dag`, `host_identity_converge.dag`); nearly every srv3 dissolution trigger already *names* `host_effect_apply` as its destination.

So srv1/srv2 subsumption and srv3 bringup are the **same** work seen twice: generalize `host_effect_apply` from `ShellCommand{script:String}` toward typed effects, and add the `EmitArtifactThenThinRun` transport arm. Everything else is consequence.

---

## 0b. Finding — modeled ops whose ONLY realization is a shell escape where a NATIVE handler is correct (transport-decomposition lane; filed 2026-07-24, do-not-fix-here)

A distinct axis from §1's *emission* census: ops whose **interface shape is right** but whose **single hardwired transport is wrong** — the verbatim §3(b) "single hardwired transport is the N×M-adapter trap" tell. The shape belongs to the dependency; the transport is a Realization *handler, one of N* (§2). Each has exactly one handler — `shell` — where a **native in-process handler** is correct when locality is `OnTarget`, and `shell`/`ssh` belongs only to a target that is another process on another host.

| op (extdeps) | modeled transport | native handler that's missing | receipt |
| --- | --- | --- | --- |
| `shell.Env.Get` (`extdeps.shell` `shell.Env` `Get`) | `shell { argv: ["printenv", "{name}"] }` | `std::env::var` — reading an env var **the process already holds in its own environment**. Realized today as `wet_env_var` spawning `printenv` (`src/v1/stage0/src/v1_interpreter.rs` `wet_env_var`). Reading your own environment isn't a host effect at all. | floor diff-observation spawns `printenv` children per pass (compile-clean scope + discovery selection), repeated per floor pass — pure log clutter; ~ms each, **not a floor-time lever** |
| `shell.Which.Check` (`extdeps.shell` `shell.PosixCommandV` `Check`) | `command -v` (and the ssh `command -v <tool>` at §3's `host_effect_realize`) | native path-search when `OnTarget`; ssh `command -v` only for a remote host | sibling flagged in the transport review (2026-07-24); same root |

**One root, three lanes** (operator, 2026-07-24): these two, the transport review's `test -x`/`command -v` hand-strings for ssh, and the wall worker's fight are all *modeled operations whose only realization is a shell escape*. The dissolution is the transport-decomposition Realization: **same operation, two handlers, native chosen when locality is `OnTarget`** — `Env.Get`'s native read is the lane's **cheapest first consumer** (a pure in-process read, no `host_effect_apply`). Not scheduled here; recorded so the deficit is counted and prioritizable (§6), never absorbed into "it's only a few ms."

---

## 0c. The named boundary — domain operation, realization, CLI surface

This lane owns typed domain/HostEffect semantics and site routing. The CLI-invocation lane (#8467) owns only CLI-backed lowering through cited tool grammar. `ArgvCommand` and handwritten `transport shell { argv }` are migration substrates, never authority. Native REST/filesystem/library/Redfish handlers reach no CLI surface.

```
domain operation → realization choice
  native handler → domain result
  CLI handler → tool-semantic request → admitted process plan
    (tool identity, CLI surface, stdin, cwd/env, observation contract)
    local: ProcessArgv → direct spawn
    SSH: simple-command tree → shell grammar → RFC 4254 command string
  raw ProcessObservation → tool-policy decoder → typed tool observation → domain result
```

Local `transport shell` executes `Command::new(...).args(...)`, with no shell, splitting or quoting; its carrier is `ProcessArgvExpansion`. SSH carries one string, and far-side shell interpretation is a modeled/observed capability. Never append inner argv to an SSH prefix and assume boundaries survive.

Calling an existing semantic op closes this lane's site even while its CLI realization still owes grammar-derived flags. The grains land independently; semantic homing is never blocked on CLI work, but an actual cutover may share its process/SSH target increment. Build that increment once, never bypass it with another string. #8596 supplied the SSH target and `command_runner` removed argv→quoted-text→heredoc for both transports; its `command_runner_dissolution_trigger` still requires `host_effect_apply` to bind ArgvCommand execution.

### Two finish lines, named separately

**SHELL-DAG:** zero runtime-present scripts; foreign/bootstrap shell emitted from typed intent; ShellCommand and runtime retained-script bridges deleted. **CLI-AUTHORITY:** production CLI realizations derive argv from semantic requests and cited surfaces. Naming either finish line is not a closeout receipt.

### Measured residual classification

The 2026-08-19 census covered production `dag/**` and `src/v2/**`, excluding tests/fixtures. A scoped `git grep` of `transport shell` and `argv:` locates sites for a change and establishes no population (re-deriving the populations is retired as a prerequisite by the 2026-10-30 process ruling (`gunbc.design_ruling bad_process_is_worse_than_no_process`)); block counts are process invocations, not shell programs, and file/line/occurrence counts differ. Most transport blocks sit under typed extdeps operations; intent-side argv remains a distinct population. Price work by root consumer × subject population, not each tool/argv/site mention.

The July §4 snapshot missed later `package_delivery`, `codex_app_server_press` and `bmc_netboot_serve` retained bridges. At `98d7147f9e`, package delivery/app-server press remained partial, while `provider_wire_evidence` had migrated to typed shell ops and only its dead retained_foreign import remained. Interleaving typed Mkdir/Find operations with a retained script proves runtime presence by construction: those scripts dissolve to typed effects (`retained_runtime`), not foreign Bash. Package delivery fixed one apostrophe-unsafe find/sort site but retained another JSON find/sort concat with quoting/locale exposure; a fix note must name its actual population.

### Obligations across lanes

CLI lane owes a negative falsifier proving a Redfish/REST path reaches no JqInvocation, CliSurface, ProcessArgvExpansion or shell target, and its scoped supersession statement: argv ceases to be terminal authority only inside CLI-backed handlers. This does not replace domain effects, typed extdeps, host routing or native realizations. DESIGN §3 already owns the interface/realization/policy split; no extra authority paragraph is needed. Existing `retained_runtime`/`retained_srvn` populations and host-effect completion require their own closeout; dated census counts do not discharge them.

## 1. Residual-shell census (dated observations and retained obligations)

Five categories. "Genuine emitter" = emits bash that actually runs; "oracle/scaffold" = shell text retained only for a test. *(Corrected 2026-07-26: the oracle/scaffold definition read "`serialize_bash` retained only for a test" — that sidecar is deleted and **no** test retains it, so the definition described an empty class by mechanism instead of role. §4.H is the live statement of this category.)*

### A. Foreign-executor shell — PERMANENT framing (category (a), stays shell)

| site | what | invoked by |
| --- | --- | --- |
| `ci_workflow.dag` | GHA `RunStep.run:` bodies + ~~the floor-peak/cgroup runners (`ci_cgroup_peak_locate_shell`, `ci_floor_peak_{pre,post}_script`)~~ **DELETED #14467 (Charter 2 carve — no-consumer §3c chain)**. *Corrected 2026-09-05 against `origin/main` @ `70925ee89e`: **"concat-built" is stale for the cgroup-ascent half.** The ascent loop's cond and body (`ci_floor_peak_while_cond_command`, `ci_floor_peak_while_body_command`) and the membership assign (`ci_floor_peak_assign_cgroup_dir_command`) were, at that correction, derived from a typed `NearestAncestorContaining` (`std.path_intent`) plus `extdeps.linux.proc_self_cgroup` membership, lowered by `v2.extdeps.languages.bash_orch_if` / `bash_proc_self_cgroup`, with a located refusal arm rather than a widened default — introduced by `ecbd0863c2a` (#7978, 2026-08-08, ancestor of `origin/main`). *Amended 2026-10-19 (Charter 2 carve): that whole vertical is now DELETED, not climbed. `bash_proc_self_cgroup` went first (no production caller; a raw `String` `Run.command` bridge), then with it the ascent emitters `bash_nearest_ancestor_locate_cond_command` / `_body_command`, their `bash_orch_if_leaf_text` / `_leaf_refusal` helpers, and `std.path_intent`'s `nearest_ancestor_while_predicate` / `_while_ascent` / `_walk_target` + `NearestAncestorContaining` — a §3c chain whose only entry point was a test. What survives is the typed lowering itself (`bash_path_predicate_target_model`, `bash_dirname_ascent_target_model`) plus the typed reader `extdeps.linux.proc_self_cgroup` `read_unified_cgroup_membership`, and the byte-level claims the deleted tests made now execute in `test.claim.path_predicate_bash_lowering_witness` under a discovered scan dir. Nothing in production constructs a `While` step.* **Amended 2026-10-21 (#14467): the sentence naming the peak calibration leaves as still-raw Phase 2 work is retired — the module that held them (`v2.workflow.ci_floor_peak_emit`) and the runtime-scan transport (`bash_proc_self_cgroup`'s `bash_membership_assign_from_source`) were deleted as no-consumer §3c chains, so that raw-String residual no longer exists; the Phase-2 typed-observation trigger now has no subject on this pipeline.** This row dispatched at least one lane onto already-discharged work.* | GitHub Actions (byte-consumes `run:`) |
| `local_tidy_spec.dag` | git pre-push hook, **built by `concat`** in `gunbc.githooks.githooks_pre_push_emit` `expected_githooks_pre_push_sh` — *not* by a serializer | git (foreign executor). *Corrected 2026-07-26 (`review 43501`): this said "via `serialize_bash`", which is doubly wrong — the sidecar is deleted, and this hook never routed through one. It is a live concat-built foreign-executor site, i.e. a genuine §4 punch-list row: stays shell permanently, but should be **emitted** via `emit(intent, Bash)` rather than concat-assembled.* |
| cron entry lines | (per lane D census) | cron |

These are correct as shell — the executor only understands shell text. Target end-state: emitted through v2 bash rows (the concat-built `ci_workflow` runners carry `…_shell_emit_dissolution_trigger` rows to the `emit(intent,Bash)` path), **not** dissolved to a typed transport. The floor-peak runners are the near-term win (they cite `floor_diff_observe.dag` as their pattern, but being foreign-executor they emit rather than `apply()`).

### B. Bootstrap / pre-runtime shell — provisioning window (category (b))

| site | what | genuine? |
| --- | --- | --- |
| `ubuntu_install_media_fetch.dag` | ISO fetch: `curl` mirror loop + `sha256sum -c` + `mv`; `serialize_bash`+`RawLine` | **dissolved** (FLAG B1 below) — in-process typed reconcile, no `ShellProgram`/`RawLine` emitted |
| `ubuntu_seeded_install_media_remaster.dag` | ISO remaster: `xorriso` extract/mkisofs, NoCloud `user-data`/`meta-data`, grub `sed`; heaviest raw shell | **dissolved** (FLAG B1 below) — in-process typed reconcile, no `ShellProgram`/`RawLine` emitted |
| `fleet_converge` fresh-standup arm | `git fetch`/`cargo build`/`cp` greenfield bring-up (frontier rows `orch_construct_procedure`, `…_cmdsubst_assign`, `…_let_assign`) | **DISCHARGED — the arm no longer exists.** Its emitter `dag/gunbc/fleet/fleet_converge_emit.dag` was deleted in `fc47b4f9866` ("P0: delete the fleet-converge paper transport; tighten the vocab wall") and #7121, together with its test module; `fleet_converge_workflow.dag` carries no standup/greenfield site. No census row emits through these frontier rows any more. |

These run before a gunbc runtime exists on the target, so shell is the honest lowest-dependency medium. **None of the rows above still emits**: the fresh-standup arm's emitter was deleted outright (`fc47b4f9866`, #7121 — see its row), and the two ubuntu-media files dissolved to in-process typed reconcile and emit no shell (see their rows and the *Landed* note below; corrected 2026-07-26, since "they emit through the v2 bash rows" read as covering every row above). No census row emits through the v2 bash rows any more (the If-band and the tier-2 band, #6566). The tier-2 Procedure and Call steps are retired outright once #14488 lands (its cgroup scan-loop predecessor was deleted by #14467); the arith arm had no live site and is being pruned by the consumer slice. **✅ FLAG B1 SIGNED (operator, 2026-07-14, relayed via calm-ferret-849):** the ISO remaster/fetch dissolve to a **typed target**, not a permanent `ShellPayloadRequired` bootstrap roster entry — these scripts execute on the provisioning host where the gunbc runtime IS present, so under the bash-minimization rule they are runtime-present, not bootstrap; `curl`/`xorriso`/`sed`/`sha256sum` become typed argv ops (`sha256sum` behind the `Sha256Verify` interface shape, `dag/extdeps/tools/sha256sum.dag`) and the NoCloud files become typed `Filesystem.Write`. Consequence: the for/heredoc emit band is never needed and the tier-2 band stays Procedure/Let-only (and becomes Let-only once #14488 lands). **Landed:** `ubuntu_install_media_fetch.dag` and `ubuntu_seeded_install_media_remaster.dag` no longer import `extdeps.languages.bash.program` / build `ShellProgram`/`RawLine` — both rewritten to in-process typed reconcile (`observe(path) → Observation`), green-by-execution.

### C. Runtime-present shell — MUST become typed via `host_effect_apply` (the real work)

| site | what | current path | dissolves to |
| --- | --- | --- | --- |
| ~~`fleet_converge_emit.dag`~~ **LANDED #7121 (`ac89afbe615`)** | The earlier slice-2 thin emitter and its `.github/fleet-converge.sh` artifact were subsequently deleted together | modeled convergence through `gunbc.fleet.fleet_converge_apply.converge_apply_for_host` and `gunbc.host.host_effect_realize.host_effect_apply` | done — no surviving thin-run golden or bootstrap emitter at this site |
| ~~`host_effect_realize.dag` srv3 shell-string transport helpers~~ **LANDED #8590 (`61bf47bb876`)** | `srv3_transport_witness_bin_success` takes `bin_path` + `List<String>` args; executable and tool-path probes use operation-derived argv. Directory ownership now follows `srv3_ensure_directory_owned_by_current_user` → ownership decision → `srv3_chown_then_read_back` | typed local operations and `typed_argv_exec_over_ssh` / `typed_argv_exec_over_fleet_ssh`; unsupported transports and non-root remote workdirs refuse | done for these helpers — this does not discharge the surviving srv* actuator graph (§4.D) |
| ~~`host_build_cache_provision_script.dag`~~ **FILE DELETED — A5 un-deferred, bounded to the effect-grain swap only** | `EnsureBuildCacheInstance { desired: BuildCacheInstance }` reaches `realize_provision_build_cache_body`, which surveys the endpoint and calls `admit_ensure_build_cache_instance` before mutation (**admission wiring LANDED #8825, `40f2fb60d39`**). The concat-built `build_cache_*_body` functions are ~~deleted #14155~~ — no builder, script body or `retained_srvn` call remains in `host_effect_realize.dag` | admitted changes reach `build_cache_ensure_step`, whose legs are typed argv runs through `build_cache_argv_read_outcome` / `run_argv_over_host_effect_transport` — no `run_shell_transport` on the build-cache path; a second endpoint survey feeds `provision_build_cache_verdict` | **DISCHARGED #14155** — the remaining shell construction is replaced by typed observe/effect operations (see the deferred-runtime-srv* row below for the builders' deletion) |
| ~~`host_hygiene_{reaper,liveness}_script.dag`~~ **LANDED #8583 (`ffa16a55bc9`)** | Both script files deleted; reaper observation reads typed systemd properties and cgroupfs/procfs, while liveness folds typed `systemd.Journalctl.UnitLog` output against a clock observation | `host_hygiene_reaper_observe` / `host_hygiene_liveness_observe` enumerate units through `systemctl_list_units_all_states`; separate reaper remediation uses `posix.Signal.TerminateProcess` and `shell.Remove.RecursiveForce` | done for the shell migration — LocalShell observation; reaper occupant-classification scaffold and wet/live remediation coverage remain separate obligations |
| ~~`host_codex_runtime_provision_script.dag`~~ **LANDED** | concat-built observe/ensure/read-back for `ProvisionCodexRuntime` | typed observe/materialize/read-back argv steps in `host_effect_realize.dag` (`gunbc.WitnessBin.Run` / `gunbc.Cli.Run` locally, `run_typed_argv_transport` over SSH/FleetSsh), verdict folded in `host_codex_runtime_provision_verdict.dag` | done — file deleted in the same cut |
| `live_deploy/emit.dag` | mutation operations remain raw `Run.command` leaves; #7315 deleted #7298's concat-built preflight and routes the roster-derived principal/sudo probes through typed effects before mutation | generated deploy scripts invoked from GHA, with gunbc already present | interpret the remaining mutation operations as typed host effects; the outer GHA thin invocation may emit Bash, the runtime operations may not (§4.J) |
| ~~`host_identity_converge.dag` hostname shell~~ **LANDED #7194** | `SetHostnameCas` carries typed hostname intent | `host_effect_apply_gated` | done — `os.Hostname.Set` local op / typed SSH argv |
| `src/v2/workflow/floor_diff_observe.dag` | typed `git.Core.DiffUnified0`, no `ShellCommand` | interpreter service operation | **target pattern already realized** |
| ~~`host_effect_nbd_proxy_serve.dag` systemd-run `WitnessBin` scaffold~~ **LANDED #8582 (`74b9a6e6a51`)** | `host_effect_nbd_proxy_serve_disposition` is `Terminal`; the former scaffold and dissolution trigger are retired. The 2026-07-14 operator ruling still excludes `trap`/`&`/`$!` `ShellStmt` vocabulary | start through `gunbc.systemd_run_transient.systemd_run_transient_read` (`RunTransient`), stop through `gunbc.systemctl_stop_run.systemctl_stop_run` (`Stop`), observe the transient unit through `gunbc.systemctl_is_active_read.systemctl_is_active_read` (`IsActive`), grounded on the cited `extdeps.systemd` operations | done — typed argv dispatch and unit observation replace the scaffold and unconditional inaccessible-port stub; captured active/inactive/refused results map to `PortObserved` / `PortAbsent` / `PortObservationRefused` |

The surviving srv\* dissolution triggers are unanimous: runtime-present raw transport dissolves into typed operations on `host_effect_apply`; the common runner `gunbc.shell_bash_runner.shell_exec_via_bash` (the `bash <<'GUNBC_BASH_EOF'` heredoc framing) dissolves when no caller passes a raw script. The former `srv3_install_diagnostic_checklist_shell_runner_dissolution_trigger` carrier was deleted with that dead reconcile/checklist subgraph; do not cite it as live.

### C-note — build-cache admission: corrected finding and executed repair

The zero-caller build-cache fold was an **unconsumed incident remedy**, not dead code. `lane_e_srv1_ci_noncompletion_root_cause_receipt` required `placement_verdict` to refuse ServerRunnerOwned endpoints on active or retired lifecycles; deleting green witnesses would have erased a live incident obligation. Catalog-ID validation was not equivalent placement admission. A generic zero-caller census cannot make that retirement decision.

#8825 wired both authorities around actuation: `build_cache_ensure.admit_ensure_build_cache_instance` is the precondition; AlreadyConverged writes/restarts nothing and runner ownership refuses located. `host_build_cache_provision.provision_build_cache_verdict` is the postcondition over a second observation. #14155 deleted the remaining concat builders and replaced ensure legs with typed argv, discharging that shell row. Its deleted prose note's obligations survive in `build_cache_daemon_ensure` and the verified-binary-install dissolution witnesses; #8598's narrower arm/render ruling did not itself claim admission.

`gunbc.build_cache.build_cache_endpoint_observe` purely decodes read bytes into endpoint/path/currency observations. Host realization owns nonmutating reads through its actual transport: boot_id, endpoint stat, `/proc/net/unix`, fd-holder find, then cgroup/stat/principal/exe only for a single resolved owner. The receipt chain is exact path + `St==01` → inode → owner pid → owner facts, never `ss`.

The measured shell-free walk is `find /proc -maxdepth 3 -path "*/fd/*" -lname "socket:?INODE?"`. Live-socket positive and absent-inode negative controls established it; `socket:[INODE]` failed because brackets are glob syntax, not because argv cannot walk procfs. All four stale-socket clauses can therefore produce EndpointStaleSocket and admitted unlink. TCP ownership remains unmodeled (`/proc/net/tcp` hex decode), explicitly refused; no measured BuildCacheInstance used it. A zero without a same-run nonzero control proves no impossibility.

### D. Oracle / scaffold retainers — NOT genuine emitters (mostly my Phase 3a work)

| site | state |
| --- | --- |
| `dag_compile_clean_transport.dag` | **fully migrated** — `serialize_bash` not even imported; scaffolds dissolved ✓ |
| `emit_determinism_transport.dag` | **fully migrated** — same; pairing now via `duplicate_computation` lens ✓ |
| `host_prelude.dag` | **fully migrated** — run path typed (`WitnessBin.Run`/`cargo.Build`). *Corrected 2026-07-26 (`review 43501`): this claimed "residual typed `ShellStmt` builders … for the emit_determinism lane". Verified: `ShellStmt` occurs **zero** times in this file, and the file's own `witness_invocation_doc` records the former builders (`witness_bin_program`, `gunbc_claims_program`) as dissolved with the typed `WitnessBin` path (Phase D, 2026-07-14). Only `toolchain_provision_shell_exec` still uses the module's shell path.* ✓ |
| `build_step_transport.dag` | **dissolved (#6565, 2026-07-14)** — corruption-probe harness reshaped onto typed `shell.Mktemp.Dir`/`Filesystem.Write`/`gunbc.WitnessBin.Run`/`shell.Remove.RecursiveForce`; no `serialize_bash`/`verifications_script` import remains (see `bst_typed_transport_doc` in the file). |
| `build_step.dag` | **corrected 2026-07-26** — not a `ShellStmt` AST library; `ShellStmt` no longer exists anywhere in the tree. It imports `v2.extdeps.languages.bash_build` and constructs plain `Node`, with serialization delegated to `v2.workflow.build_step_emit` via `bash_command_fold_serialize` (its own `build_step_serialize_dissolution_note` says so). Consumed by the above |

### E. Replacement machinery + detectors — not dissolution targets

`bash_command_fold.dag` + `bash_build` (the v2 compositional emitter that *is* the replacement); the lenses `realization_vocabulary_containment` / `medium_structure_containment` / `duplicate_computation` (detectors).

**Corrected 2026-07-26 (`review 43477`).** This block previously named `program.dag` as "the sidecar — the deletion target" and carried a **`bash_program_importer_count_baseline = 19` ratchet** gating that deletion. Both are gone:

- `dag/extdeps/languages/bash/program.dag` was **deleted** (#6831, Phase 0). It is not a target; it does not exist.
- The ratchet was **pruned with it** — `gunbc.plans.shell_emission_model` `shell_emission_model_body` records it as *"the vacuous bash-program importer-count ratchet is pruned with it (resolve fails before the ratchet could fire)"*. Verified: `bash_program_importer_count` now appears **nowhere in the repository except this document**, so the "baseline = 19" figure measured a mechanism that no longer exists.
- `bash_program_emit.dag`, also named here, does not exist either.

The live replacement is `src/v2/extdeps/languages/bash_command_fold.dag` (see §4.G).

---

## 2. `EmitArtifactThenThinRun` scoping — srv1/srv2 subsumption keystone

**Goal — ACHIEVED (2026-07-20 receipts; roadmap `6-shell-slice2` sign-off pending):** move the fleet-converge srv1/srv2 steady-state (category C) *out of emitted bash and into the gunbc binary*, which interprets the typed converge policy in-process and emits typed receipts. The emitted artifact shrank to the fresh-standup bootstrap fragment and a thin `gunbc converge` invocation line; that fragment's emitter was later deleted (`fc47b4f9866`, #7121), so nothing fresh-standup is emitted now. **Landed:** G1+G2 minted in #6572, G3 consumed in #6585, CLI receiver in #6598; witnesses (`fleet_converge_emit_*`, `fleet_converge_apply_holds`) green by execution 2026-07-20.

### What already exists (the pieces)

1. **The typed plan** — `gunbc.host.host_converge.HostConverge` / `ConvergePolicy` / `ConvergeKnob` (`SliceProperty|PerSlotMemoryCap|RunnerWidth|JobserverTokens|VerifyOnlyCap|GunbcPinnedTree`) is a pure typed value with `ConvergeApplyMode = FreshStandup | ExistingHostQuiescentReload`, importing typed `extdeps.os.systemd`/`oomd` property names. Zero shell. **This is the plan the binary interprets.**
2. **The apply interface** — `host_effect_apply(target, effect, evidence, transport) -> Reconciliation<HostEffectIntent, HostEffectEvidence>` is minted (`host_effect_realize.dag`), with `Drive = OneShot | ConvergeLoop` and a redrive directive. Its consumers exist; **`host_identity_converge` is the exact converge precedent** (gated apply → typed effect → read-back → noop).
3. **The receipt** — the fleet-converge receipt grammar (`gunbc_host_converge_receipt_grammar_marker`) is byte-locked; the design says generalize it as the single `apply()` outcome, not fork it.

### The gap (three coupled pieces)

- **G1 — a typed converge effect.** `HostEffect` today = `ShellCommand{script:String}` | `RedfishAction`. The `script:String` is the anemic leaf (shell-emission-model §1: "same anemic leaf as `host_effect.ShellCommand.script`"). Subsumption needs the effect to carry the *typed converge intent*, not bash.
  - **⚠ FLAG 2a — effect shape.** Option (i): add `HostEffect::ConvergePlan{ policy: HostConverge, host: ComputeHost }`, resolved by `resolve_host_effect_cell` to a new `ConvergeOnHost` cell, realized by an in-process interpreter that drives the typed systemd/oomd property reads+writes (the same knobs `host_converge.dag` already names). Option (ii): keep `ShellCommand` for one-shots and make `EmitArtifactThenThinRun` a transport that reinterprets — rejected, because the script is already bash (no typed plan to interpret). **Recommend (i)** — it's the anemic-leaf dissolution the design already calls for, and it makes the interpreter total over the knob coproduct (each `ConvergeKnob` arm → a typed `extdeps.os.systemd` set/read). **✓ SIGNED option (i)** (operator, 2026-07-14 — dispatch order via calm-ferret-849).
- **G2 — the `EmitArtifactThenThinRun` transport arm.** `HostEffectTransport` today = `LocalShell` | `SshShell`. Add `EmitArtifactThenThinRun { bootstrap: BootstrapFragment, invocation: ThinInvocation }`.
  - **Semantics:** for a `SteadyState` host it degenerates to in-process interpretation (`gunbc converge --host X` reads the typed policy and reconciles via typed systemd effects — no emitted script). For a `FreshStandup` host it was designed to emit the bootstrap fragment (git-checkout / cargo-build / cp, bootstrap bash per category B) and then the thin invocation, the `ConvergeApplyMode` split `host_converge.dag` models. **Retired:** the fresh-standup emitter `dag/gunbc/fleet/fleet_converge_emit.dag` was deleted (`fc47b4f9866`, #7121) and `.github/fleet-converge.sh` is gone from main, so no fresh-standup bootstrap fragment is emitted any more (see the `fleet_converge` fresh-standup row).
  - **⚠ FLAG 2b — thin-run guardrail (load-bearing).** The binary must interpret the *typed* `HostConverge`, emitting typed receipts. Replacing the bash with the same lines of imperative Rust in the seed is refused (shell-emission-model §6 / host-effect §Phase-D). The interpreter folds the `ConvergeKnob` coproduct; there is no free-form imperative arm. **✓ SIGNED as the merge bar** (operator, 2026-07-14): total fold over `ConvergeKnob` with a fail-closed, counted `Unimplemented`-per-knob frontier; receipts on the byte-locked receipt grammar generalized as the single `apply()` outcome.
- **G3 — fleet_converge consumes `apply()`.** Point the fleet-converge lane at `converge_apply` (the `ConvergePlan` effect over `EmitArtifactThenThinRun`) instead of `project_fleet_converge_to_doc`. During the transition the committed `.github/fleet-converge.sh` was the byte-locked oracle for the emitted bootstrap fragment; the steady-state functions moved in-process. **Retired:** the fresh-standup emitter `dag/gunbc/fleet/fleet_converge_emit.dag` was deleted (`fc47b4f9866`, #7121) and `.github/fleet-converge.sh` is gone from main, so no fresh-standup bootstrap fragment is emitted any more (see the `fleet_converge` fresh-standup row).

### Sequence (each gated by the byte oracle)

1. **Mint G1+G2 together** — **LANDED (#6572)**: `ConvergePlan` effect + `ConvergeOnHost` cell + `EmitArtifactThenThinRun` transport + the in-process knob interpreter, with a fail-closed `Unimplemented`-per-knob frontier so unmodeled knobs refuse (not fabricate).
2. **Consume from fleet_converge** (G3) — **LANDED (#6585, CLI receiver #6598)**: `converge_apply` for the steady-state arms; the Doc projection then emitted only the fresh-standup bootstrap and the thin invocations, and the committed `.github/fleet-converge.sh` golden shrank to that remainder with the drift gate green. **Retired:** the fresh-standup emitter `dag/gunbc/fleet/fleet_converge_emit.dag` was deleted (`fc47b4f9866`, #7121) and `.github/fleet-converge.sh` is gone from main, so no fresh-standup bootstrap fragment is emitted any more (see the `fleet_converge` fresh-standup row).
3. **Retire** `fleet_converge_steady_state_doc_projection_dissolution_trigger` and the ctrl reconciler's script fan-out (host-effect Phase D's first net ctrl deletion — `runner_host_reconcile.mjs` hash/fan-out). *(Still open: the trigger remains a Scaffold binding `fleet_converge_thin_invocation` — it dissolves when the thin invocation becomes typed argv dispatch rather than a shell line.)*

**⚠ FLAG 2c — ownership.** `host_effect.dag` is a DESIGN-named seam co-owned by smart-newt-512 (dag-managed-infra) / neat-boar-71 (BMC), and its note requires **operator + bright-stag-194 sign-off before any transport arm is minted**. This scoping is the sign-ready shape; the mint is that lane's, not a solo edit from here. **✓ DISCHARGED** (operator, 2026-07-14): operator dispatch order via calm-ferret-849 supplies the sign; sessions bright-stag-194 / smart-newt-512 / neat-boar-71 verified archived at sign time (no live co-owner lane to co-sign or conflict with), so the mint proceeds under a dedicated worker owned by calm-ferret-849.

---

## 3. The remainder of the ShellProgram→DAG arc

> **Retitled and re-anchored 2026-07-26 (`review 43477`).** This section was framed *"to `program.dag` deletion"*, with deletion as the join the two tracks converge on. **That deletion already happened** — #6831, Phase 0 — so the goal is history, not pending. The tracks below are still the live work; only the terminus was stale. Do not dispatch anyone to "delete `program.dag`".

Two tracks run in parallel.

### Track 1 — legitimate shell onto the v2 bash rows (bounded)

- **P1 (ready now, no sign-off):** the `ci_workflow` concat-built floor-peak/cgroup runners → `emit(intent, Bash)` via the landed If-band + word support (they carry `…_shell_emit_dissolution_trigger` rows). Foreign-executor, so they *emit*; no `apply()`.
- **P2 — DISCHARGED:** the Procedure/Let emit band for the fresh-standup frontier rows was the emitter vocabulary this arc needed, scoped by the bootstrap census and fail-closed where a construct isn't modeled; with the fresh-standup emitter deleted (`fc47b4f9866`, #7121) no consumer of it remains, and the tier-2 Procedure and Call steps are retired outright once #14488 lands. *(Tier-2 band LANDED #6566; the consumer slice — route the fresh-standup fragment through `emit(intent,Bash)`, byte-oracle vs `.github/fleet-converge.sh` — LANDED #6573. The former "ubuntu-media files need `for`/heredoc" clause is superseded by the FLAG B1 working default above: ubuntu-media dissolves to typed argv/`Filesystem.Write`, so the for/heredoc band is never built.)*
- **P3:** `local_tidy_spec` pre-push hook + cron lines stay **shell** permanently (foreign-executor roster entries, category (a)). These never dissolve — the honest residue. *(Corrected 2026-07-26: this row said they stay `serialize_bash`. They cannot — `serialize_bash` no longer exists. What is permanent is that these sites **stay shell**, emitted through the v2 bash rows like every other foreign-executor site. "Permanently shell" ≠ "permanently on the deleted sidecar".)*

### Track 2 — runtime-present shell onto `host_effect_apply` (the §2 keystone cascade)

- **P4 = the G1+G2+G3 keystone above** (srv1/srv2 subsumption).
- **P5:** the srv3 tails follow the *same* interface. Each `srv3_*_observe_script` becomes a typed `…Observe` effect on `host_effect_apply` (their dissolution triggers already name this); the receipt echoes become typed receipts; `shell_exec_via_bash` (the heredoc runner) dissolves once no caller passes a raw script. The two heavy files: `srv3_install_diagnostic_checklist` is FROZEN/terminal (typed observe effect retires it), `srv3_os_install_actuator_toolchain_ensure` → typed `extdeps.package_managers.apt`/`curl` argv effects.
- **P6:** `host_identity_converge` drops its `sudo hostnamectl` script for a typed hostname effect (it's already on `apply_gated`). *(P6 Part 2 corrected 2026-07-26: `nbd_proxy_serve_program`'s `RawLine` body is **already gone** — the operator's 2026-07-14 no-`trap`/`&`/`$!` ruling routed it to a systemd transient-unit transport instead of "first-class typed background/trap statements", which are now explicitly **not** a target. What remains is the observe-side read-back grounding, §1.C.)*
- **P7:** `build_step_transport` — **LANDED (#6565, 2026-07-14)**: corruption-probe harness reshaped onto typed `shell.Mktemp`/`Filesystem.Write`/`WitnessBin.Run`, dropping the `verifications_script` `serialize_bash` execution.

### The join — ~~delete `program.dag`~~ **already done (#6831); the real terminus is the wall**

**Superseded 2026-07-26 (`review 43477`).** This section described the arc's terminal step as: importers drain → the `bash_program_importer_count` ratchet hits its floor → `program.dag` + `serialize_bash` delete. **None of that is pending.** The sidecar was deleted outright in #6831 Phase 0 and the ratchet pruned with it as vacuous (resolve now fails before it could fire); the identifier survives nowhere but this document. The join is not a deletion the tracks build toward — it already happened, ahead of them.

The v2 bidirectional bash language **is** the single bash authority today. The **Phase-3 wall** that turns *"no sidecar exists"* into *"a hand-built transport string is unwritable"* (§5 construction over validation) has **partially** landed (#7184, operator acceptance 2026-07-24): the free minter is deleted and `RetainedShellScript` — a **record** — is the sole mint **for the `ShellOnHost` realization edge**.

The construction half of the global transport wall has **since closed too**: `TransportScript` is a **`sole_constructor` record** whose single mint `transport_script_seal` is `admit_callers`-sealed to exactly two production declarations (`gunbc.retained_shell_script` `retained_shell_script_to_transport`, `gunbc.bash_materialized_transport` `bash_materialized_transport`), and the cast form closed with it — `04_infer` `sole_constructor_construction_diags` judges a cast into a sealed type (#7962) — so neither a record literal nor a cast can produce one outside those bridges. What remains is not a construction hole but a **String residue** — both admitted bridges take a bare body `String`, so arbitrary text still reaches a transport through a counted, reason-bearing, `dissolves_to`-carrying call. That is conspicuous rather than impossible, and dissolves with the per-site migration, not more wall work. So Track 1's **deletion** terminal and Track 2's **construction** terminal are both behind us; what remains is the per-site migration itself.

*(Corrected 2026-07-27, `review 43608`. This read "has also **already landed** … is the sole mint. So Tracks 1 and 2 feed neither a deletion nor a wall; both terminals are behind us." That declared the narrower `ShellOnHost` wall to be the completed **global** transport wall — a §3 violation, the planning authority overstating a landed scope, with concrete misdispatch risk: a worker reading this routes away from the still-open meta-exec confinement work, exactly the harm this document exists to prevent. This is the **fifth** site of this claim corrected in this PR; the first four were fixed while this one survived because each pass swept for sentences already seen rather than enumerating every sentence asserting the fact. Standing rule, stated so it outlives the individual corrections: **enumerate by claim across the whole authority set, then verify each; a correction that fixes only the site under review is not finished.**)*

*(Corrected 2026-07-26, `review 43486`. The previous sentence here — written by me one round earlier while fixing the `program.dag` staleness — said the terminus was to "brand `TransportScript` … and activate the inert lens". Both halves were already dead, for *different* reasons — the brand **refuted by execution** (transparent brand, §5.E's ruling block), the lens directive **overtaken by completion** (#7184 had already activated it). A refuted plan and a completed one both read as "stale directive", but only the first means the idea was wrong. Fixing a stale directive with a differently-stale directive is the failure mode this document keeps reproducing; see §4.G's method note.)*

**Critical-path summary:** everything non-foreign converges on **P4 (the `host_effect_apply` typed-effect + `EmitArtifactThenThinRun` mint)** — **P4 is LANDED** (#6572/#6585/#6598; FLAGs 2a(i)/2b/2c signed/discharged 2026-07-14, see §2). The critical path is now P5/P6 (mechanical on the landed interface) and the operator sign of roadmap `6-shell-slice2`; P1/P2 (emitter side) run independently. **Dispatch note (2026-07-20):** do not re-dispatch workers onto P4/slice 2 from the old line-count framing — that staleness produced two misdispatches onto finished work.

## 4. Exhaustive instance census @ `78f43c38` — the tracked punch-list (calm-ferret-849, 2026-07-22)

§1–§3 record *direction* at #6507; this section is the **per-instance snapshot at the stated base** so every shell-string-construction site is tracked to closure. Grounded against `origin/main` @ `78f43c38`.

**Why now:** a "migration" wave (#7004, #7006, and the closed srv* cluster) counted **relocations** as progress — it moved `ShellCommand{script: concat(...)}` out of the intent file into a new `*_script.dag` file and realized a typed variant by *calling that concat and stuffing the raw string back into `ShellOnHost{script}`*. Net raw-shell-string construction: unchanged; new §3 coproduct-nickname debt added. This census exists so no site is counted done until the concat is **gone**, not homed elsewhere.

> **Anchoring (addresses review 41399 / DESIGN §6):** rows are keyed on **file + symbol name** (variant, fn), NOT line numbers — a line-numbered prose ledger drifts from the tree within a commit (it did: the first draft was mis-grounded on a divergent worktree that still had the pre-#7006 `ShellCommand{script}` sites; on `origin/main` those are already the nickname variants). Symbol anchors are stable across line moves. Grounded at `origin/main` @ `78f43c38`; an interim tracker that dissolves into the enforcement lens (§4.F), the drift-proof authority.

### 4.0 Completeness method (proves this is exhaustive, not sampled)

Every way a shell string is constructed or carried, over `dag/**` (excluding `*_test.dag`). **The search must be multiline-aware** — construction is frequently `ShellCommand {` on one line and `script:` on the next, which a single-line regex misses (this gap hid the `readiness.dag` sites in the first draft — review 41467). Use a slurped/multiline match (e.g. `perl -0777 -ne '/ShellCommand\s*\{\s*script:/'`), not line-at-a-time `grep`.

| # | pattern (multiline) | what it finds |
| --- | --- | --- |
| P1 | `ShellCommand\s*\{\s*script:` | `HostEffect.ShellCommand{script}` construction — **live sites on main** (`live_deploy/readiness.dag`, see §4.B); the rest became nickname variants. Residue = the `host_effect_plan.dag` `{script: ""}` placeholder + the `host_effect.dag` type def. (Match-arms `ShellCommand { script: s\|_ } =>` in `host_effect_realize`/`fleet_converge_cli`/`ci_deploy_access_observe` are destructuring, not construction.) |
| P1b | `effect: <Variant>` for the shell-backed `HostEffect` variants | the nickname-variant construction sites that *replaced* P1 (the real §4.A rows) |
| P2 | `BootstrapFragment *{` | bootstrap-script carrier (0 live construction sites today) |
| P3 | `command:` in `Run{}` / `Do{run:}` | `std.orchestration.Run.command` string |
| P4 | `shell.Exec.Run` / `.Check` / `shell_exec_via_bash` | meta-exec bottom-transport calls |
| P5 | `fn … -> String` builders — in `*_script.dag` **and inline** (e.g. `fleet_show_effective_read.dag`'s `fleet_runner_unit_property_read_script`/`fleet_runner_width_count_read_script`, `host_converge_slice1.dag`'s `…_memory_max_read_script`/`…_memory_max_set_script`/`…_enumerate_units_script`) — plus any `concat("systemctl …"/"…")` inline | the concat/relocation script builders, wherever they live (not only the `*_script.dag` glob) |
| P6 | `transport_script_from_body` | the (then-porous) `TransportScript` boundary — its sites **at `78f43c38`**. *Historical: the minter is now deleted and the boundary is the `RetainedShellScript` record (§4.F). This row records what the probe found then, not a live count.* |
| P7 | `serialize_bash`/`ShellProgram`/`RawLine`/`ShellStmt` | bash-AST emit vocab (emit-internal) **at `78f43c38`**. *Historical, like P6: this vocabulary is now **extinct** — zero construction sites, declaring module deleted (§4.G). This row records what the probe searched for then, not a live surface.* |
| P8 | `ssh.Session.Exec(` (non-`ExecArgv`) | ssh command-string transport |

The classes below partition every hit. **Class letters = the ACTION**, not the file.

### 4.A — RELOCATION REGRESSION + systemctl-read cluster (dissolve properly)

Keyed on the **construction site (variant/fn name)** and the modeled op it *should* call. Done = the concat gone (deleted, not moved) and, where the variant is a nickname, the variant gone too.

| construction site (file · variant/fn) | builder(s) — `*_script.dag` and/or inline | should call | sub-class |
| --- | --- | --- | --- |
| ~~`fleet_show_effective_read.dag` · `SystemdUnitMemoryPropertiesRead`, `SystemdUnitMemoryMaxRead`~~ **LANDED** | ~~`fleet_show_effective_read_script.dag`~~ deleted; ~~inline `fleet_runner_width_count_read_script`, `fleet_runner_unit_memory_props_read_script`~~ deleted (census true-up, 2026-07-26) | `systemd.Systemctl.ShowProperty` via `gunbc.systemctl_show_read`; `list-units` via `gunbc.systemctl_list_units` | **A1 DONE** — see the dead-scaffold note below |
| ~~`host_converge_slice1.dag` (via `shell.Exec.Run`)~~ **LANDED** | ~~inline `…_memory_max_read/set_script`, `…_enumerate_units_script`~~ deleted | `systemctl_show_read` + `systemctl_list_units_active_services` (both imported and called; `gunbc.host.host_converge_slice1` imports and its `systemctl_list_units_active_services` call) | **A1 DONE** — verified 2026-07-26: zero `_script` fns, zero `shell.Exec.Run`, zero `systemctl` concats in the file |
| ~~`host_identity_observation.dag` · `HostIdentityShortHostnameRead`~~ **LANDED** | ~~`host_identity_short_hostname_script`~~ deleted | `os.Hostname.ReadShort` · `dag/extdeps/tools/hostname.dag` | **A2 DONE** — realized through `gunbc.hostname_read` (LocalShell → op, SshShell → typed argv) |
| ~~`host_effect.dag` · `SetHostnameCas`~~ **LANDED (#7194)** | ~~`host_effect_set_hostname_cas_script`~~ deleted (no such symbol anywhere in tree) | `os.Hostname.Set` · `dag/extdeps/tools/hostname.dag` | **A2 DONE** — `host_effect_realize.dag`'s `SetHostnameCas` arm calls `gunbc.hostname_set` `hostname_set_cas`, whose `hostname_set_local` is `os.Hostname.Set(desired:)`; the SSH arm goes through `typed_argv_exec_over_ssh`, and the CAS read reuses the landed `os.Hostname.ReadShort`. Verified on main 2026-07-25 (bucket A): zero concat builders, zero `shell.Exec` on the path. |
| ~~`host_effect.dag` · `ReadEffectivePosixPrincipal`, `SudoNopasswdExecuteProbe`, `SudoNopasswdGrantListProbe`~~ **LANDED (#6946/#7315)** | ~~`host_effect_deploy_access_probe_script.dag`~~ and ~~`ci_deploy_access_emit.dag`~~ deleted | `access.PosixEffectivePrincipal`, `sudo.NopasswdExecuteProbe`, `sudo.NopasswdGrantListProbe` | **A3 DONE.** #7315 corrected #7298's later raw-string regression: roster-derived typed effects execute at the runtime-present access gate; the generated deploy artifacts contain no embedded authorization probes. |
| ~~`live_deploy/` effect variants~~ **LANDED (D2 #7192)** | ~~`live_deploy/host_effect_script.dag`~~ **file deleted** — no such path in tree | decomposed (multi-op) | **A4 DONE** — verified 2026-07-26: `dag/gunbc/live_deploy/` is `apply · emit · intent · operations · readiness · service_ready · spec` only, and the directory contains **zero** `ShellCommand` occurrences. This closes do-not-miss item 1 below (the #7004/#7006 relocation debt) |
| ~~`host_effect_realize.dag` · `ProvisionBuildCache`~~ **ARM SWAPPED, builders DELETED #14155** | ~~`host_build_cache_provision_script.dag`~~ **file deleted**; the `build_cache_*_body` fns are ~~deleted #14155~~ — the ensure legs are typed argv operations in `gunbc.host.build_cache_daemon_ensure` and `host_effect_realize`, matched on `EnsureBuildCacheInstance { desired: BuildCacheInstance }` | decompose | **A5 DISCHARGED #14155** — the effect grain swapped (bounded operator clearance) and the shell body converted to typed argv; `admit_ensure_build_cache_instance` is wired into realize (#8825), and no `retained_srvn` build-cache call remains; the remaining `#5828`-style conversion debt does not include a build-cache subject |
| ~~`host_effect_realize.dag` · `ProvisionCodexRuntime`~~ **LANDED** | ~~`host_codex_runtime_provision_script.dag`~~ **file deleted** — no such path in tree | typed observe/materialize/read-back argv steps (`gunbc.WitnessBin.Run` / `gunbc.Cli.Run` locally, `run_typed_argv_transport` over SSH/FleetSsh); verdict folded in `host_codex_runtime_provision_verdict.dag` | **A5 DONE** for this row — the other srv* A5 rows above remain deprioritized |
| ~~hygiene reaper/liveness~~ **DELETED #8583** | ~~`host_hygiene_reaper_script.dag` (`…_body`), `host_hygiene_liveness_script.dag`~~ **file deleted** — migrated to typed `host_hygiene_reaper_observe.dag` / `host_hygiene_reaper_remediate.dag` / `host_hygiene_liveness_observe.dag` | typed observe/remediate modules | **A5 DISSOLVED #8583** — CASE 2 (file deletion upstream): script file deleted, construction migrated to typed sub-modules. No direct successor body names. |
| `srv3_host_effect_apply.dag` · `Srv3*` variants, `srv3_install_diagnostic_checklist.dag` · `Srv3InstallDiagnosticObserve`, `nbd_proxy_virtual_media_install.dag` · `Srv3NbdProxyServe` | `srv3_host_effect_script.dag`, `srv3_install_diagnostic_observe_script.dag` | typed observe/receipt effects | **A5** srv* deprioritized |

**A1 correction:** the live typed migration left two dead concat builders (`fleet_runner_unit_memory_props_read_script`, `fleet_runner_width_count_read_script`), subsequently deleted. One had a tautological witness pinning “list-units” text while claiming typed authority. Keep the actual typed argv/local-read controls; do not preserve a bypass because its string assertion is green. A module-wide “no concat” claim must cover the module, not only its migrated path.

### 4.B — DIRECT `ShellCommand{script}` still constructed in intent

**STATUS 2026-07-26 (census true-up): both rows below are DISCHARGED — there are now ZERO direct `ShellCommand{script}` construction sites in `dag/gunbc/live_deploy/`** (verified: `grep -rn ShellCommand dag/gunbc/live_deploy/` returns 0). `live_deploy_unit_diagnosis_command` and `live_deploy_healthz_probe_script_for_port` no longer exist anywhere in tree — D2 #7192 (tree_sync `| tail` + `exit 0` absorbing fallback, converted to a typed `Systemctl.Status` exit-as-data refusal) and D3 #7193 (readiness routed through `gunbc.systemctl_status_read` + `http.Client.Get`) closed them. The table is kept struck-through as the audit trail.

~~The relocation wave turned most direct construction into nickname variants (4.A), but **live direct sites remain** on `origin/main` — the multiline form my first-draft single-line P1 missed (review 41467):~~

| site (file · construction) | builder (file · fn) | runs on | dissolve to |
| --- | --- | --- | --- |
| `live_deploy/readiness.dag` · `ShellCommand { script: live_deploy_healthz_probe_script_for_port(port) }` | `readiness.dag` · `live_deploy_healthz_probe_script_for_port` → `intent.dag` · `live_deploy_health_probe_curl_command` (curl localhost `/healthz`; uses the typed `curl_bounded_get_argv`) | srv1 LocalShell via `host_effect_apply_gated` | `http.Client.Get` · `extdeps/http/client.dag` (or the typed curl argv end-to-end) — see §4.C / §5.B |
| `live_deploy/readiness.dag` · `ShellCommand { script: live_deploy_unit_diagnosis_command(unit) }` | `intent.dag` · `live_deploy_unit_diagnosis_command` (`systemctl status --no-pager --full <unit> \| tail`) | srv1 LocalShell | a `systemd.Systemctl.Status` op (§5.A add) — **and the `\| tail` + defensive `exit 0` is a §5 absorbing fallback** (intent.dag's own comment admits it masks systemctl's nonzero); the typed op models exit-3-for-dead-unit instead |

Residue only (not construction to migrate):

| site (file · symbol) | what | action |
| --- | --- | --- |
| `gunbc.host.host_effect_plan` (no longer present at this revision) · `ShellCommand { script: "" }` | empty placeholder | delete with the type — **still present** (verified 2026-07-26) |
| `gunbc.host.host_effect` (no longer present at this revision) · `ShellCommand { script: String }` | the **type variant** itself | delete at arc close (DESIGN §5, escalated) — terminal; **still present** |
| `gunbc.fleet.fleet_converge_cli` (no longer present at this revision) · `ShellCommand { script: _ } => fallback` | a **match arm**, not a construction site — consumes the variant, builds no string | *(row added by the 2026-07-26 true-up; previously uncensused)* dissolves with the type variant above. Listed so the arc-close delete has a complete consumer list — a missed match arm is what turns the terminal delete into a non-exhaustive-match break |

*(The first draft mis-grounded on a divergent worktree — `fleet_show`/`host_identity` corrected to 4.A per review 41399 — and then under-scoped P1 to single-line, missing these `readiness.dag` sites per review 41467. Both fixed.)*

### 4.C — RUNTIME-PRESENT `shell.Exec.Run` with a string/`_script` body (dissolve to typed op)

| site (file · fn) | body | should call | class |
| --- | --- | --- | --- |
| `host_converge_slice1.dag` · `…_memory_max_read/set/enumerate_units_script` | `systemctl show`/`set-property`/`list-units` | `systemctl.ShowProperty`/`SetProperty` + new `list-units` op | A1 (same cluster as 4.A) |
| ~~`host_converge_slice1.dag`, `host_identity_adopt.dag`, `host_runner_memory_cap_verify.dag`~~ **LANDED** | `"date -Iseconds"` | `Clock.Now` · `dag/extdeps/clock.dag`, via the single authority `gunbc.clock_read` `clock_now_probed_at` | **A2 DONE** — the last holdout `host_runner_memory_cap_verify.dag` dissolved in bucket A (2026-07-25); its `retained_runtime` bridge, its `extdeps.shell` import and its transport-script scaffold all deleted with it |
| ~~`ci_deploy_target_host.dag`~~ **LANDED** | `"hostname -s 2>/dev/null \|\| hostname"` | `os.Hostname.ReadShort` via `gunbc.hostname_read` `hostname_short_read_local` | **A2 DONE** — the `\|\| hostname` widen is gone with the string; the file's remaining `extdeps.shell` import is `shell.Env.Get`, which is not the confined `extdeps.shell.exec` module (see `non_exec_import_is_not_leak_holds`) |
| `host_identity_assimilation.dag`, `host_identity_adopt.dag`, `srv3_install_diagnostic_checklist.dag` | `"echo <receipt>"` | typed receipt emit (a stdout write, not shell) | A4 |
| `host_effect_realize.dag` · `shell_exec_via_bash` dispatch | realization-core script dispatch | the `LocalShell`/`SshShell` typed-argv edge (C5) | A4 realization core — confirm before edit |
| `dag/gunbc/instruments/{host_prelude,gunbc_ci,~~emit_host_gate~~,merge_admission_stamp}`, `gunbc/tools/review.dag` | witness/CI transports invoked from `claim_executor` | typed `WitnessBin.Run`/argv (host_prelude precedent) | A4 — `emit_host_gate` **DISSOLVED 2026-09-13 (gentle-gull-238), CASE 1** — its narration shell-out (`shell.Exec.Run` of `shell_printf_line`, the module's only `retained_foreign` site) is deleted outright, no successor transport: the per-smoke verdict line it printed is carried typed by the `ExitFailure.reason` field end to end (`cli_run.rs` `ClaimOutcome::ExitFailure` → claim_batch FAIL row → terminal-ledger detail under the `exit-failure` tag), which is the first named arm of its own dissolution trigger — the scaffold and trigger are deleted with the site |

### 4.D — `ssh.Session.Exec` command-string (vs typed `ExecArgv`)

**RECLASSIFIED by the 2026-07-26 true-up — read before picking this up.** Every `ssh_session_exec(command:)` site below sits inside an **`srv3_*` function**, i.e. inside the srv\* cluster the operator wound down 2026-07-22 and authorized for retirement 2026-07-24 (§5.D roadmap item). So 4.D is **not** an independently-dispatchable A1 bucket: dissolving these to `ExecArgv` would be typed-argv work on a subgraph slated for deletion — motion, not progress. **4.D inherits A5's deferred status** and is picked up only as part of the srv3 retirement, or if that retirement is abandoned.

Exhaustive site list, verified on `origin/main` @ `efe67794cd` (enclosing fn in bold — this is what the original table did not record):

| module | enclosing fn | verb | disposition |
| --- | --- | --- | --- |
| `gunbc.host.host_effect_realize` | **`srv3_transport_witness_bin_success`** | arbitrary command (`cd … && bin args`) | A5-deferred (srv3) |
| `gunbc.host.host_effect_realize` | **`srv3_transport_test_executable`** | `test -x <path>` | A5-deferred (srv3) |
| `gunbc.host.host_effect_realize` | **`srv3_apt_tool_present`** | `command -v <tool>` | A5-deferred (srv3) |
| `gunbc.host.host_effect_realize` | **`srv3_tool_bin_path`** | `command -v <tool>` | A5-deferred (srv3) |
| ~~`gunbc.host.host_effect_realize`~~ | ~~**`srv3_chown_directory_to_current_user`**~~ **RENAMED+CLIMBED #8796** | ~~`id -u` / `id -g`~~ | **CASE 4** — successor is `gunbc.host.host_effect_realize.srv3_ensure_directory_owned_by_current_user`. New name has a stronger guarantee (readback-based, not chown exit-status). |
| `gunbc.host.host_effect_realize` | **`run_shell_transport`** | `ssh_session_exec_script(script: script.body)` | **NOT srv3** — this is the realization core's `RetainedShellScript` → SSH path, i.e. the §5.E counted frontier's own transport. It dissolves when the frontier empties, not by a per-site migration. Do not edit under a 4.D brief |

The original table's rows are retained below for provenance:

| construction | verb | should call | class |
| --- | --- | --- | --- |
| `ssh_session_exec(command: cmd)` | arbitrary command | `ssh.Session.ExecArgv` (C5) | A1 |
| `ssh_session_exec(command: concat("test -x ", path))` | file-exists probe | `ssh.Session.ExecArgv` | A1 |
| `ssh_session_exec(command: concat("command -v ", tool))`  | tool-presence probe | `ssh.Session.ExecArgv` | A1 |
| `ssh_session_exec(command: "id -u")`, `"id -g"` | uid/gid read | `ssh.Session.ExecArgv` (an `id` op) | A1 |
| `ssh_session_exec_script(script:)` | script-over-ssh | typed-argv splice or decompose | A4 |
| `extdeps.diagnostic.ssh.dag` · `ssh.Session.Exec(command:)` | the transport itself | keep as the ONE command-string transport, or fold into `ExecArgv` | transport-decision — escalate |

### 4.E — FOREIGN-EXECUTOR / BOOTSTRAP emit (LEGIT shell — route through `emit(intent,Bash)`, stays shell but bounded)

> **True-up @ post-#7216 (wise-crane-222, 2026-07-26):** fresh bucket-D re-census; supersedes the 07-22 rows below for CI placement files. Authority: `shell-intent-emit-realization-design.md`, `host-effect-orchestration.md`. Invariant: intent imports no `bash_build` / `ShellProgram` / `serialize_bash` (`realization_vocabulary_containment`).

These are correct as shell (GHA `run:`, cron, git hooks, pre-runtime bootstrap — the executor only understands shell text). Target = emitted through the v2 bash rows via `orch_emit_pipeline(medium: bash_orchestration_emit_medium())`, a **roster not a growth surface** — NOT dissolved to a typed op unless a row names one.

**Already on emit (do not re-migrate):**

| module | symbols | executor |
| --- | --- | --- |
| `v2.workflow.ci_workflow_run_emit` | `ci_isolate_toolchain_script`, `ci_pin_rustup_default_script`, ~~`ci_selection_control_script`~~ **DELETED #8283-CI-FLOOR-CUT** | GHA `run:` |
| ~~`v2.workflow.ci_floor_peak_emit`~~ | ~~`ci_cgroup_peak_locate_shell`, `ci_floor_peak_pre_script`, `ci_floor_peak_post_script`~~ | GHA `run:` (**AMENDED 2026-10-21 #14467: the whole module went — with its wet witness — as a no-consumer §3c chain (zero importers; its calibration rows fed no live emitter), so this row records a module that no longer exists; the peak-calibration Phase-2 residual it named is discharged by deletion, not by typing — see §1.A and the Phase-1 row in §4.J**) |
| `v2.workflow.ci_retry_emit` | `ci_cargo_eagain_retry_script` | GHA `run:` (via `ci_release_build_emit`) |
| `v2.workflow.ci_release_build_emit` | `ci_release_build_script`, ~~`gunbc_ci_run_script`~~ **DELETED #9252-PLAN-WALK-DELETE** | GHA `run:` (partial — still concat-wraps verify script) |
| `v2.workflow.orchestration_bash_emit_support` | `orch_bash_run`, `orch_bash_do`, `orch_bash_emit_pipeline` | shared emit plumbing (**#7265**; pre-existing `ci_*_emit` forks migrate in PR2) |
| `v2.workflow.ci_materialization_emit` | `ci_sccache_provider_shell_injection` | GHA `run:` (**LANDED #7265**) |
| `v2.workflow.ci_merge_admission_emit` | `ci_floor_disposition_marker_init_script` | GHA `run:` (**LANDED #7265**) |
| `v2.workflow.ci_regen_rustfmt_path_emit` | ~~`ci_regen_ensure_rustfmt_path_script`~~ **DELETED #8406-REGEN-ROOT-CUT** | GHA regen `run:` (**LANDED #7290**; typed control flow, but FileExists/session-env leaves remain raw as named in §4.J) |
| `gunbc.assimilate.bmc_token_federation` | `gcp_token_smoke_script` | GHA `run:` |
| `gunbc.live_deploy.emit` | `expected_live_deploy_apply_script`, ~~`expected_live_deploy_retract_script`~~ **DELETED #7909-RELEASE-IDENTITY-REFACTOR** | **RECLASSIFIED runtime-present, not an emit-completion receipt.** GHA supplies the outer `run:` medium, but gunbc is already executing and must interpret typed effects; its raw command leaves are the §4.J terminal row. |
| `gunbc.host.host_effect` | ~~`fresh_standup_bootstrap_intent` → `EmitArtifactThenThinRun` bootstrap arm~~ | **DISCHARGED / NEVER EXISTED AS NAMED — there is no `fresh_standup_bootstrap_intent` function in the tree**; the name occurs only inside the `reason:` of `gunbc.host.host_effect` `bootstrap_fragment_script_carrier_scaffold` and in this census. `EmitArtifactThenThinRun { bootstrap: BootstrapFragment }` has **no production constructor** — every production site only matches and refuses it. Retiring that dead arm is a separate decision, not a pre-runtime bootstrap row. **Follow-up (next trigger):** delete the `EmitArtifactThenThinRun` arm with its match-and-refuse sites, and with it the `bootstrap_fragment_script_carrier_scaffold` row whose `reason:` still claims emission through `fresh_standup_bootstrap_intent`. |

**Remaining concat-built foreign-executor punch-list** — per-symbol table and PR batching in §4.J.

**Deferred / out of bucket D:**

| site | reason |
| --- | --- |
| `roadmap_static_site.dag` body fns | HTML/JSON content emit, not shell — belt B (`gunbc serve`) |
| `runner_host_deploy.dag` · `runner_host_docker_provision_script` | fleet manual-gap lane |
| `bmc_virtual_media.dag` srv4 gadget scripts | BMC provisioning, separate lane |
| Runtime-present `host_effect_apply` / srvN tails | bucket B (typed effects), not foreign-executor emit |

**Legacy 07-22 snapshot rows** (superseded for CI sites; non-CI rows unchanged):

| site | executor | class |
| --- | --- | --- |
| ~~`fleet_converge_emit.dag` `fresh_standup_bootstrap_script` + fresh-standup `Run{command: concat(...)}` rows~~ **DELETED (`fc47b4f9866`, #7121)** | ~~pre-runtime bootstrap~~ | discharged: no emitter remains |
| `roadmap_static_site.dag` body fns | srv1 dashboard (belt B) | E-emit / dissolves with belt B |
| `fleet_posix_accounts.dag` `probe_command: "id <user>"`  | account-existence probe | **NOT a shell emit site** — authored provenance metadata; live proof is `deploy_access_check_observed` via typed `SudoNopasswdExecuteProbe` |

### 4.F — bottom transport (Phase-3 WALL — **LANDED at construction on both edges**: `ShellOnHost` record wall #7184, `TransportScript` seal #7962)

> **Trued up 2026-07-26 (`review 43486`).** This table described the wall as *pending* and directed Phase 3 to "brand `TransportScript`" across its sites. **The `ShellOnHost` half of the wall landed on 2026-07-24 and branding was proven ineffective by execution** (the `shell.Exec.Run` sink stayed open until #7962 sealed it — see the row below) — both facts were already recorded in §5.E's ruling block below, which superseded this table in place without updating it. A worker reading the action column would have executed superseded work. Corrected against the tree.

| surface | status | action |
| --- | --- | --- |
| `transport_script_from_body(body: String)` | **DELETED** from `extdeps.shell.exec`. Not live sites — **zero**. Surviving occurrences are a note recording its replacement (the leading note of `gunbc.retained_shell_script`), two compile-**RED** controls asserting the deleted-minter fake *fails to compile* (`transport_script_wall_compile_red.dag`), and v1-seed Rust detector code | **none — done for the `ShellOnHost` edge.** Its realization takes a counted `RetainedShellScript` record. Direct `shell.Exec.Run` closed at construction with #7962: `TransportScript` is a **`sole_constructor` record** whose single mint `transport_script_seal` is `admit_callers`-sealed to exactly two production declarations (`gunbc.retained_shell_script` `retained_shell_script_to_transport`, `gunbc.bash_materialized_transport` `bash_materialized_transport`), and the cast form closed with it — `04_infer` `sole_constructor_construction_diags` judges a cast into a sealed type (#7962). The residue is the bridges' bare-`String` body parameter, counted at every call, not a construction route |
| `TransportScript` branding | **abandoned deliberately, and now historical.** It *was* `= String where brand(…)`, a transparent brand `expand_nominal_alias_rhs` peeled to its base, so branding alone could never make a hand-join a type error | **superseded twice.** The record wall closed `ShellOnHost`; #7962 then made `TransportScript` itself a `sole_constructor` record with an `admit_callers`-sealed mint, closing direct `shell.Exec.Run` and the cast form. Nothing here is pending |
| `shell_exec_via_bash` (`gunbc.shell_bash_runner` `shell_exec_via_bash`) | heredoc runner scaffold | dissolves when no caller passes a raw script *(still open)* |
| `host_language_transport_script` lens | **LIVE — activated by #7184**, not inert. `v2.test.claim.host_language_transport_script.corpus.wall_residue_live`: *"the previously-inert … lens is wired as a per-PR live consumer over the `shell.Exec.Run` anchor sites"*, disposition `ReadsLiveTree`. It reds a **new raw literal** at an enrolled Run position, and stays **deliberately green** on `ComputedApplication` (the counted bridge calls) — which is why it never caught the #7064 computed-join fake | **none — done as a literal-blob backstop.** `RetainedShellScript` closed the computed-join class at the `ShellOnHost` record field and #7962 closed the direct `shell.Exec.Run` sink at the type; the lens deliberately permits computed applications, which are now exactly the counted bridge calls. Do not "promote" the lens — that action presumed a brand that does not work and a lens that was already switched on |
| meta-exec module `extdeps.shell.exec` | not walled | module-isolate / symbol-visibility confinement (meta-exec-confinement lane) *(still open — the genuinely remaining Phase-3 item)* |

### 4.G — bash-AST vocabulary is extinct; the replacement is live

#6831 deleted `program.dag` and its ShellProgram/serialize_bash/ShellStmt/RawLine constructors. Remaining occurrences are prose, absence controls, substring collisions or frozen `.diff` fixtures; never edit those fixtures to improve a census. Live `bash_command_fold`/`bash_build` constructs Node and is replacement machinery, confined by `realization_vocabulary_containment` (#6854), not a dissolution target.

A hit is not a construction. Check declarations, read every hit across all file types, exclude this census's self-reference, and distinguish matching lines (`grep -c`) from occurrences (`grep -o`). Deletion lines in frozen patches are evidence of removal. Delete a superseded directive, or retain its exact correction where needed: **refuted** means never retry; **completed** means do not redo; **partially superseded** preserves named residual work. Never append a mutually incompatible instruction beside its correction.

The live `v2.std.orchestration_emit.OrchestrationEmitMedium.realize_run_rawline` is a different concept from the extinct RawLine AST node. `bash_fold_raw_line_target_model`, reached by `orch_emit_run_core` for env-free Run, remains live. A non-command PipelineStep can use this existing medium method; it does not require a restored ShellProgram or new grammar row merely because the names resemble one another.

### 4.H — oracle / test retainers (NOT live construction — skip)

`gunbc.live_deploy.emit` (no longer present at this revision) `expected_*_script` (drift-gate oracles), `*_test.dag` fixtures. These are test expectations, not runtime construction; they follow their subject's dissolution.

### Ledger true-up @ 2026-07-26 (§4.A/4.B/4.D, snappy-moth-330) — read this FIRST

Re-censused §4.A, §4.B and §4.D against `origin/main` @ `efe67794cd` by execution over the tree, not by trusting the rows. Four corrections, in descending order of how badly the stale row would mislead:

1. **§4.D is not a dispatchable bucket — it is A5-deferred.** Every `ssh_session_exec(command:)` site is inside an **`srv3_*`** fn (the wound-down srv\* cluster), except `run_shell_transport`, the realization core's own `RetainedShellScript` transport. The old table listed verbs without enclosing fns, so it read like an independent typed-argv bucket. It is not; picking it up would be typed-argv work on a subgraph slated for retirement.
2. **§4.A4 and §4.B are DONE.** `live_deploy/host_effect_script.dag` is deleted and `dag/gunbc/live_deploy/` contains zero `ShellCommand`. This closes "do-not-miss" item 1 (the #7004/#7006 relocation debt), which had been the loudest open warning in this doc.
3. **§4.A1 is DONE, but left two dead concat builders behind** — deleted in this true-up, with the witness pinning one of them as contract. See the A1 dead-scaffold note in §4.A; the pattern (a file note claiming "no concat shell strings" while two sat below it, plus a tautological witness conjunct asserting the bypass still had the right shape) is the reusable finding.
4. **One uncensused consumer added** — `gunbc.fleet.fleet_converge_cli`'s (no longer present at this revision) `ShellCommand` **match arm**, so the terminal type delete has a complete consumer list.

**Net at the 2026-07-26 snapshot:** with bucket D (§4.E/§4.I) in flight, the non-deferred remainder of this arc was meta-exec confinement. Subsequent truth: merged #7298 added a runtime-present raw-`Run.command` preflight, #7315 corrected it onto roster-derived typed effects, and #7303 initially left source `Run.env` as a Retry silent-drop before #7318 discharged it by explicit nonempty-env refusal with an executing empty-env control. The current owner/trigger roster in §4.J supersedes this snapshot; the srv\* cluster remains operator-deferred.

*(Corrected 2026-07-26, `review 43494`: this "Net" dispatched **`host_language_transport_script` lens promotion** as remaining §4.F work. §4.F now says explicitly **do not** promote that lens — the construction wall landed (#7184) on the `ShellOnHost` edge, and the lens is already **live**, deliberately green on the computed concats that were the actual fake. Same document, opposite dispatch; the wall half of "§4.F wall-green" is discharged, leaving only meta-exec confinement.)*

### Ledger true-up @ 2026-07-25 (bucket A, calm-pike-837) — read this before the 07-22 snapshot below

The 07-22 snapshot is several merges stale. What has landed since, keyed to the rows it discharges:

| PR | what | census effect |
| --- | --- | --- |
| #7184 | the **transport-script construction wall** (§5.E keystone) | §5.B per-op migration UNPAUSED — relocation is now a type error, so the §4 tables are again the live punch-list |
| #7192 / #7193 | **D2 / D3** — `systemd.Systemctl.ShowProperty` consumers + `systemctl` op cluster | 4.A1 `fleet_show_effective_read`, `host_converge_slice1` memory-property reads |
| #7194 | **D1** — `os.Hostname.ReadShort` + `os.Hostname.Set` on `dag/extdeps/tools/hostname.dag` | 4.A2 rows for `HostIdentityShortHostnameRead` **and** `SetHostnameCas` — both now DONE (see §4.A) |
| #7215 | census doc pass | — |
| #7231 | **extdeps positioning restructure** | import paths repointed: `extdeps.os.systemctl` → `extdeps.systemd`, `extdeps.os.hostname` → `extdeps.tools.hostname`, `extdeps.os.exec_arg_limit` → `extdeps.exec`, `os.os` → `extdeps.os` hub-loose. Older rows in §4/§5 still spell the pre-restructure paths; read them through this mapping. |
| #7233 | srv3 `Record` | 4.A5 srv\* cluster (still deferred) |
| **#7265** | bucket D PR1 — `ci_sccache_provider_shell_injection` + `ci_floor_disposition_marker_init_script` → `ci_materialization_emit` / `ci_merge_admission_emit` | 4.E/4.I/4.J rows for those two symbols DONE; §4.E/4.J true-up folded here (orphan doc deleted) |
| **this PR** | **bucket A** — the last `date -Iseconds` in `host_runner_memory_cap_verify.dag` onto `Clock.Now`; `meta_exec_confinement_exception_roster` **→ 0** | 4.C `date -Iseconds` and `hostname -s` rows DONE; the meta-exec roster is now empty, so §4.F's wall has no exceptions left to grant |

**Meta-exec roster: → 0, and the dark lane that hid it.** Two of the three rows (`dag/gunbc/tools/review.dag`, `dag/gunbc/ci/ci_deploy_target_host.dag`) were **stale** — their sites had stopped importing `extdeps.shell.exec` merges earlier and nobody noticed, because `meta_exec_roster_sound_live` existed in the lens but ran on **no** per-PR cadence; the only enrolled roster RED was synthetic (hand-written fact rows, blind to the live roster). Measured this PR: the live receipt costs **13.5s cold** against a 5s fast-lane budget, so it cannot be enrolled per-PR as-is. The per-PR enforcement is instead a **construction wall** — `stale_count` is bounded above by roster length, so the empty roster proves soundness with no scan, and any re-added row reds `meta_exec_roster_shrunk_to_empty_holds` in the same PR that adds it. The live receipt stays as a backstop in `src/v2/test/claim/long/meta_exec_confinement_clean_tree.dag` (with a non-degeneracy control, since a clean tree makes a live scan vacuously true if the walk reads nothing). **Named residue:** that long lane is still dark (not roster-enrolled); its dissolve-on is a falsifier batch admitting live-tree lens receipts.

### Historical wind-down ledger

The [2026-07-22 PR/session snapshot](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/plans/shell-to-dag-residual-census-and-arc-completion.md) is retained in immutable history, not presented as current dispatch. The final corrections are: #6946 established ExecArgv; #7004/#7006 relocated scripts until #7192/#7194 removed them; the finite new-op list closed; #7184/#7962 closed the retained-script construction edges, not the remaining typed-op path. §4.J owns the later site/trigger roster; operator-deferred work remains deferred unless its specific later receipt discharges it. PR/branch state is read from the PR, never this dated ledger.

### 4.I — CI foreign-executor sites the census missed (2026-07-24 audit @ `879c5a2699`)

§4 was grounded at `78f43c38`; a three-facet CI audit (2026-07-24) found **five CI `run:` shell sites that landed after that snapshot and are not in the tables above.** Each carries a rich in-code note — tracked *at the site*, never rolled up here. Keyed on file + symbol (the §4 convention). All are foreign-executor (GitHub Actions `run:`), so category **E-emit** (route through the v2 bash rows, a roster not a growth surface) unless a typed-op dissolution is named.

| construction site (file · symbol) | what | executor | class / dissolves to |
| --- | --- | --- | --- |
| `ci_spec.dag` · `ci_fmt_gate_line` | `"$CARGO_BIN" fmt --all --check` (build job, first step) | GitHub Actions | **5.B call-the-op** — `cargo.Build.Fmt` · `extdeps/rust/cargo_build.dag` (its `ci_fmt_gate_note` already names this dissolve-on — the same seam `ci_release_build_line` awaits). NOT E-permanent. |
| `ci_spec.dag` · `gunbc_ci_deploy_invoke` | `ROOT=$(git rev-parse … \|\| pwd); "$ROOT/target/release/gunbc" run … live_deploy_apply_prod_role_wet` (deploy job) | GitHub Actions | E-emit (thin `gunbc` invocation wrapped in bash); the `git rev-parse … \|\| pwd` prelude is a §5 fallback (below) |
| `ci_spec.dag` · `gunbc_ci_heal_regen_invoke` | `gunbc run … generated_artifact_gate main_wet` (heal job) | GitHub Actions | E-emit (same thin-invoke shape; `ci_heal_regen_note` documents it) |
| `ci_spec.dag` · `ci_heal_git_add_lines` + commit/push | `git add` over `committed_generated_artifact_paths()` (each `[ -e ]`-guarded), `git diff --cached --quiet` gate, `git commit`, `git push origin HEAD:<branch>` (heal job) | GitHub Actions | E-emit — the git verbs dissolve to typed `git.Core.*` ops (Add/Commit/Push); `ci_spec.dag` `ci_heal_commit_push_shell_emit_scaffold` carries the typed `Scaffold` disposition and its dissolution trigger |
| `ci_materialization.dag` · `ci_sccache_provider_shell_injection` | `if sccache --show-stats >/dev/null 2>&1; then RUSTC_WRAPPER=sccache; …; fi` (embedded in "Isolate toolchain dirs") | GitHub Actions | **DONE (#7265)** — migrated to `v2.workflow.ci_materialization_emit`; still carries §5 ABSORBING-FALLBACK debt (opportunistic guard silently skips caching when daemon is down → dissolve-on: mandatory provisioning-ensure via srvN build-cache STEP-2) |

**§5 absorbing-fallbacks INSIDE the CI scripts** (§5.B rule: each becomes a *modeled outcome*, never re-appended). The census tracked the scripts but not these internal widens:

| site | fallback | fail-closed / modeled form |
| --- | --- | --- |
| `run:` bodies | `git rev-parse --show-toplevel 2>/dev/null \|\| pwd` | scope-widen on non-repo → typed refusal (`git.Core.ShowToplevel` with a `nonzero =>` refuse, not `pwd`) |
| regen + floor steps | `git fetch --no-tags origin main … \|\| true` | stale-baseline on fetch failure silently corrupts the affected-set diff base → typed refusal or an explicit `DiffBaseUnavailable` marker downstream logic can gate on |
| ~~`v2.workflow.ci_workflow_run_emit` · `ci_native_cache_root_toolchain_segment_command`~~ **DELETED #7436** | `rustc -V 2>/dev/null … ; if empty: toolchain-unresolved` slug | cache-poison fallback — **RESOLVED #7436**: toolchain segment computation reordered after setup-rust-toolchain; refusal instead of fabricated segment. |
| `gunbc.ci.ci_spec` · `ci_retry_escalation_level1` | `CARGO_BUILD_JOBS=1` on EAGAIN | masks a fleet-pressure *resource deficit* rather than emitting a typed counted diagnostic (its level-2 sibling `-u RUSTC_WRAPPER` was removed as STEP-2, #7138); recover parallelism-cap as a fleet-admission control, not a per-run silent degrade |

**Adjacent (NOT shell-string) — cross-ref, tracked elsewhere.** The same audit found a second class the CI shares that is *not* a `ShellProgram`/string sink and so does **not** belong in this census: hand-spelled **config** that should DERIVE from an existing model. The exemplar is toolchain isolation — `ToolchainEnvIsolation = SharedHomeAcrossJobs \| PerJobCargoHome{…} \| HermeticContainer` (`extdeps/toolchain/types.dag`) is modeled and consumed by `sccache`/`fleet_intent`, but the workflow emit never reads `env_isolation`; it hand-spells per-job isolation (build isolates, the ci/gate job shares → the rustup `ETXTBSY` race, documented in PROSE in `ci_floor_gate_toolchain_note` on `ci_workflow.dag` rather than modeled/gated). Same shape for the GHA **cache key** (modeled `gha_actions_cache_facts` key-derivation vs the hand-spelled `ci_cache_key_template`), **artifact** upload/download (un-modeled; the dissolve-on artifact-CacheProvider named in `ci_materialization.dag` is undone), and **permissions** (`std.effect_grant` vs per-job `WorkflowPermissions`). These are the model↔realization-fork open thread + roadmap `2-emit-partition`, not the ShellProgram arc; noted so the audit's findings aren't lost across the seam.

### 4.J — Bucket D foreign-executor emit punch-list @ post-#7216 (wise-crane-222, 2026-07-26)

> **STALENESS BANNER — the CI half of this section is grounded on a workflow that no longer exists (filed 2026-09-05, jolly-deer-148, against `origin/main` @ `70925ee89e`).**
> `.github/workflows/ci.yml` was **deleted** by `611fd027708` (#8283, 'FLOOR-Y cutover: delete the CI floor, rebuild from the `run_required_floor` seed'), confirmed an ancestor of `origin/main`. CI today is one emission, `gunbc.witness.witness_floor_workflow` → `.github/workflows/witnesses.yml`. This document does not mention `witness_floor_workflow`, `witnesses.yml`, or #8283 anywhere, and still names `ci.yml` drift+parse as the byte-oracle in four places, so **every CI row below states its dissolve trigger in terms of a gate that cannot fire.**
> A symbol-existence check over the exact production symbols this section's dispatch roster names found the following are absent from the whole `.dag` corpus (`dag/**`, `src/v2/**`) at that head: `gunbc_ci_floor_only_script`, `gunbc_ci_regen_floor_only_script`, `gunbc_ci_run_script`, `ci_selection_control_script`, `ci_regen_floor_skip_shortcut_script`, `git_fetch_script`, `scheduler_invoke`, `scheduler_invoke_with`, `ci_floor_materialization_receipt_gate_script`, `ci_floor_resolve_receipt_gate_script`, `ci_floor_stamp_merge_admission_script`, `ci_floor_stamp_ambient_exit_command`, `ci_floor_stamp_root_command`, `ci_regen_ensure_rustfmt_path_script` and its named raw leaves, `host_effect_plan_placeholder_effect`, the `host_hygiene_reap_*_body` builders, and `srv3_chown_directory_to_current_user`. `ci_fmt_gate_line` and `ci_fmt_gate_note` DO still exist, so this is not a blanket sweep of the section.
> **WORKED EXAMPLES — the absent list resolves to a MIX, and that is the point.** Several of the names above already have verdicts, and they do not all point the same way. This is written out rather than left to be re-derived, because the per-row lane's whole difficulty is that these three outcomes are indistinguishable by grep:
>
> | absent name | why it is absent | introducing commit (on `main`) |
> | --- | --- | --- |
> | `srv3_chown_directory_to_current_user` | **renamed AND climbed — both at once, which is the case a two-way discharge/rename split cannot express.** The surviving shell-string helpers took typed argv over SSH/FleetSsh, and directory ownership now follows a live successor under a different name, `srv3_ensure_directory_owned_by_current_user` (`gunbc.host.host_effect_realize`). Classifying this as plain "discharged" — as an earlier draft of this table did — loses the successor a reader needs to find | `61bf47bb876` (#8590) |
> | `host_build_cache_provision_script` | **discharged** — the realize path is wired through both admission authorities with the observation producer they were waiting on | `40f2fb60d39` (#8825) |
> | `host_hygiene_reap_install_units_body`, `host_hygiene_reap_classify_and_act_body`, `host_hygiene_reap_stale_override_body`, `host_hygiene_reap_script_body` | **absent because their FILES were deleted** — reaper and liveness migrated onto typed observation and `host_hygiene_*_script` went with them. The body names never appear in the correcting diff at all, so a name-level check sees only absence and cannot see the cause | `ffa16a55bc9` (#8583) |
> | `git_fetch_script` | **rename, NOT discharge** — `git_fetch_no_tags_shell` and `git_fetch_prune_shell` exist. Identical grep signature to the rows above, opposite verdict | (unadjudicated) |
>
> Read that table before touching the rest of the list. Four different causes — dissolution, file deletion upstream of the name, bare rename, and rename-plus-climb — and no textual property of the name distinguishes any of them. The `host_hygiene` row is the sharpest: the correcting commit does not mention the absent symbols, so the evidence that discharges them is not reachable from the symbol at all. The SS1.C rows for these subjects were corrected separately (#10529); this table exists so the per-row lane inherits the worked examples instead of rediscovering them, since a finding that must be rediscovered is a finding that will be.
>
> **WHAT THIS BANNER ESTABLISHES AND WHAT IT DOES NOT.** It establishes that those names have no declaration in the corpus, which is a fact about spelling, and that the CI workflow the section's triggers cite is gone. It does **not** establish that each of those rows is discharged: a name can vanish by rename as easily as by dissolution — `git_fetch_script` is absent while `git_fetch_no_tags_shell` and `git_fetch_prune_shell` exist, which is exactly the ambiguity a per-row adjudication has to resolve. Nothing below has been re-verified at construction grain except the `ci_floor_peak_*` rows corrected in this same change. **Row-by-row adjudication of these rows by a dedicated lane is not a prerequisite (retired by the 2026-10-30 process ruling (`gunbc.design_ruling bad_process_is_worse_than_no_process`); this does not establish which rows are discharged). A row in §4.E/§4.J is not used to dispatch work without first re-deriving that row's own claim against current `main`** — this document has now dispatched several lanes onto already-discharged rows (#10529 corrected five; this change corrects the `ci_floor_peak` set), and that recurrence is the reason for the banner rather than a silent per-row edit.
> Population for that recurrence, stated as a denominator **measured on the merge of `70925ee89e` with `origin/main` `16a702eac3d`, and valid only there** — every later edit that strikes a row through changes what the closed-marker grep counts, so this split moves under you and is never quoted: the census carries data rows across its tables, of which some already carry a closed marker (LANDED / DELETED / RETIRED / dissolved / MIGRATED / EXTINCT), leaving **the rest asserting live state**, none of them re-verified here.
> That this split MOVED is itself the demonstration, not a bookkeeping detail: measured one merge earlier the same recipe returned a different closed / live split, and #10529 (`16a702eac3d`) striking two more SS1.C rows through is the entire difference. A number transcribed into this banner would have been wrong within one merge of being written.
>
> **DO NOT TRUST THESE NUMBERS.** They are transcribed counts, which is the same defect this banner is about, one layer up — so the checks that produce them are named here and the counts are valid only for the head they cite (`70925ee89e`). Re-deriving them is not a prerequisite (retired by the 2026-10-30 process ruling (`gunbc.design_ruling bad_process_is_worse_than_no_process`); this does not establish coverage); they are recorded as history, not as a standing requirement:
>
> ```sh
> # denominator: data rows, and the subset already marked closed
> awk '/^\| /{ if ($0 !~ /^\| ---/) print }' docs/plans/shell-to-dag-residual-census-and-arc-completion.md \
>   | grep -vE '\| (op \(extdeps\)|site|symbol|module|row|#|population|construction site|state|what)' > /tmp/rows
> wc -l < /tmp/rows                                                              # at 70925ee89e
> grep -cE 'LANDED|DELETED|RETIRED|dissolved|DONE|MIGRATED|fully migrated|extinct|EXTINCT' /tmp/rows
>
> # absent-symbol list: every backticked identifier in SS4.J with no declaration in the corpus
> sed -n '/^### 4\.J /,/^### Dissolution trigger/p' docs/plans/shell-to-dag-residual-census-and-arc-completion.md \
>   | grep '^|' | grep -v '^| ---' | grep -o '`[a-z_][a-z0-9_]\{6,\}`' | tr -d '`' | sort -u \
>   | while read -r sym; do grep -rqE "^ *(fn|data|type) $sym\b" --include='*.dag' dag src/v2 || echo "$sym"; done
> ```
>
> The second check over-reports by construction and its raw output must be read, not counted: at `70925ee89e` it emits candidates, and the production symbols listed above are what survives discarding the module names (`ci_spec`, `extdeps`, `host_effect_realize`, `bash_orch_if`, `orchestration_bash_emit_support`, `realization_vocabulary_containment`, `ci_materialization_emit`, `host_hygiene_reaper_script`, `host_build_cache_provision_script`), the fn-fragment and commit-sha spellings this banner itself introduced, and rows already struck through as retired. A symbol declared under a spelling the anchor does not match also reads as absent. That is why its output is evidence for a per-row adjudication and never a verdict. The `ci.yml` deletion is the one claim here that is not count-shaped and needs no re-derivation beyond `git log --diff-filter=D -- .github/workflows/ci.yml`.

The per-symbol remaining work and batching plan for §4.E CI sites. **Load-bearing at the time of writing:** every `ci_spec.dag` composer change regenerated `.github/workflows/ci.yml`; drift+parse gate was the byte-oracle. **That oracle is gone with `ci.yml` (#8283); the current byte-oracle is the `witnesses.yml` emission from `gunbc.witness.witness_floor_workflow`, and no row below has been restated against it.**

**Typed-op candidates (route to extdeps op, NOT bash emit):**

| site | current | target op | notes |
| --- | --- | --- | --- |
| `ci_spec.dag` · `ci_fmt_gate_line` | `"$CARGO_BIN" fmt --all --check` | `cargo.Build.Fmt` | `ci_fmt_gate_note` already names dissolve-on; **load-bearing ci.yml** |
| ~~`ci_deploy_access_emit.dag` · raw preflight builders~~ **DELETED (#7315)** | ~~`"whoami"`, concat-built `sudo -n` / `sudo -n -l`, raw `echo` / `exit` in `Run.command`~~ | `access.PosixEffectivePrincipal.Read` + `sudo.NopasswdExecuteProbe.Check` + `sudo.NopasswdGrantList.Read` with exact `NOPASSWD:` row parsing | **LANDED.** The fleet grant roster selects execute-check versus greenfield-safe grant-list verification; list-query exit zero alone never admits a grant. `deploy_mutation_gate` realizes those typed effects before `live_deploy_fold` applies mutation. Both generated deploy-script goldens exclude the former embedded preflight. |

**Remaining concat-built foreign-executor punch-list:**

#### A — `merge_admission_produce.dag` (single live concat builder plus the emitted floor tail)

| symbol | GHA consumer | status / emit complexity |
| --- | --- | --- |
| ~~`ci_floor_disposition_marker_init_script`~~ | ~~floor opener (via `gunbc_ci_floor_only_script`)~~ | **RETIRED 2026-07-27** — its only purpose was clearing stale state for an unreachable zero-writer docs-only disposition family |
| ~~`ci_documentation_only_gate_skip_prefix`~~ | ~~receipt gates, merge gate, selection control~~ | **RETIRED 2026-07-27** — all its readers, persistent marker writer, stamp branch, and admission override deleted atomically |
| ~~`ci_merge_admission_gate_script`~~ (concat builder) | merge-admission gate step | **MIGRATED 2026-07-28 (#7363, Wave B)** — the builder is deleted; the step's `run:` is materialized from a typed `EffectPlan<BoundOperationInvocation>` (`v2.workflow.ci_merge_admission_gate_emit`) whose argv derives from `git.Core.FetchNoTags` and `gunbc.WitnessBin.Run`. The `ROOT=$(git rev-parse …)` line went with it; `FailFast` is now the carrier's policy (`Continue` refuses) rather than the runner's `-e` flag. Gate policy (merge base, tool entry/function, source roots) stays in `merge_admission_produce.dag` as language-blind rows |
| ~~`ci_floor_stamp_merge_admission_script`~~ | ~~floor tail~~ | **STRIKE — the raw leaves dissolved #7522 (87a4af3ad0).** `ci_floor_stamp_ambient_exit_command`, `ci_floor_stamp_root_command`, `merge_admission_stamp_command()` all deleted. CASE 1 dissolution. The "PARTIAL #7293" status is stale. |

#### B — `ci_materialization.dag` (script surfaces)

| symbol | GHA consumer | status / emit complexity |
| --- | --- | --- |
| `ci_sccache_provider_shell_injection` | isolate-toolchain pipeline | **DONE (#7265)** → `ci_materialization_emit` |
| ~~`ci_floor_materialization_receipt_gate_script`~~ | ~~ci job gate~~ | **DELETED #7470 (b01cdf4d89).** CASE 1 dissolution — receipt gate dissolved with WalkPlan success stages finalization. No direct successor. |
| ~~`ci_floor_resolve_receipt_gate_script`~~ | ~~ci job gate~~ | **DELETED #7470 (b01cdf4d89).** CASE 1 dissolution — receipt gate dissolved with WalkPlan success stages finalization. No direct successor. |

#### C — `ci_spec.dag` (distinct script composers feeding GHA)

| symbol | GHA consumer | route |
| --- | --- | --- |
| `ci_release_build_line` | build (via retry emit) | emit (partially done) |
| `ci_fmt_gate_line` | build fmt step | **typed op** |
| `ci_floor_build_verify_script` | build verify + unpack | emit (build_step transport pattern) |
| ~~`ci_release_bins_pack_script`~~ **DELETED #12422** (replaced by `bash_build` node trees in `gunbc.fleet.fleet_release_bins_key`) | build pack artifact | emit |
| ~~`ci_release_bins_unpack_verify_script`~~ **DELETED #12422** (replaced by `bash_build` node trees in `gunbc.fleet.fleet_release_bins_key`) | ci unpack step | emit |
| ~~`gunbc_ci_floor_only_script`~~ | ~~ci floor `run:`~~ | **DELETED #9252 (489346fff7).** CASE 1 dissolution — plan/walk CLI surface deleted, including its floor-only composer. |
| ~~`ci_regen_floor_skip_shortcut_script`~~ | ~~regen skip~~ | **DELETED #8406 (3b431f34a9).** CASE 1 dissolution — regen_stage0 root cut; regen skip shortcut dissolved with it. |
| ~~`gunbc_ci_regen_floor_only_script`~~ | ~~regen job~~ | **DELETED #8406 (3b431f34a9).** CASE 1 dissolution — regen_stage0 root cut; regen composer dissolved. |
| `gunbc_ci_deploy_invoke` | deploy job | emit (thin gunbc run) |
| `gunbc_ci_heal_regen_invoke` | heal regen | emit |
| `gunbc_ci_heal_commit_push_script` | heal commit/push | emit (git verbs; future typed git ops) |
| ~~`scheduler_invoke` / `scheduler_invoke_with`~~ | ~~floor/regen~~ | **DELETED #9252 (489346fff7).** CASE 1 dissolution — plan/walk CLI surface deleted; scheduler invoke removed. |
| ~~`git_fetch_script`~~ | ~~floor/regen/deploy~~ | **RENAMED #6833 (a5477cf5b4).** CASE 3 bare rename — successor is `git_fetch_no_tags_shell` / `git_fetch_prune_shell` in `dag/extdeps/git.dag`. Old name cleaned up by #9252. |

#### D — permanent roster (emit target, never typed-op dissolve)

| site | executor |
| --- | --- |
| `gunbc.githooks.githooks_pre_push_emit` · `expected_githooks_pre_push_sh` | git pre-push hook |
| `gunbc.tools.cron_tag` · `build_cron_entry_line` | cron |

**Current-main ownership and dissolve triggers after #7293/#7298/#7303, including the later #7290 merge (verified at `c629a4ff0d`; this is the dispatch roster):**

| row | exact production symbols | owner | dissolve trigger |
| --- | --- | --- | --- |
| Merge-admission foreign executor | ~~concat builder `ci_merge_admission_gate_script`~~ (deleted #7363); ~~raw leaves remaining inside the already-emitted floor tail: `ci_floor_stamp_ambient_exit_command`, `ci_floor_stamp_root_command`, `merge_admission_stamp_command()`~~ **STRIKE — the raw leaves DELETED #7522 (87a4af3ad0). CASE 1 dissolution.** | roadmap `2-emit-partition`; ~~gate half **CLOSED #7363 (Wave B)**; floor-tail half unassigned~~ **CLOSED — entire merge-admission raw-leaf surface dissolved #7522.** | ~~Gate half done: the live gate composer is a typed `EffectPlan<BoundOperationInvocation>` materialized through the canonical Bash medium (`effect_plan_bash_materialize`, Wave A #7356) and its concat builder is deleted; argv derives from the referenced `extdeps` declarations, and a materialization refusal renders a located marker rather than substitute shell. The floor-tail leaves remain separate raw-`Run.command` work and were explicitly out of Wave B's scope. The zero-consumer stamp-script orphan and the zero-writer documentation-only disposition reader/marker/policy family retired rather than gaining invented consumers or dead FileExists/typed-read models.~~ **All raw leaves deleted; the "floor-tail half" is now closed.** |
| CI materialization foreign executor | ~~`ci_floor_materialization_receipt_gate_script`, `ci_floor_resolve_receipt_gate_script`~~ **DELETED #7470 (b01cdf4d89). CASE 1 dissolution — WalkPlan success stages finalization dissolved both receipt gates.** | ~~roadmap `2-emit-partition`; unassigned~~ **CLOSED.** | ~~same, with receipt parsing/refusal represented by typed predicates rather than `sed`/test strings~~ **Rendered moot — both symbols gone; receipt gates no longer exist.** |
| CI-spec foreign executor | `ci_release_build_line`, `ci_fmt_gate_line`, `ci_floor_build_verify_script`, ~~`ci_release_bins_pack_script`~~ **DELETED #12422**, ~~`ci_release_bins_unpack_verify_script`~~ **DELETED #12422**, ~~`gunbc_ci_floor_only_script`~~ **DELETED #9252**, ~~`ci_regen_floor_skip_shortcut_script`~~ **DELETED #8406**, ~~`gunbc_ci_regen_floor_only_script`~~ **DELETED #8406**, `gunbc_ci_deploy_invoke`, `gunbc_ci_heal_regen_invoke`, `gunbc_ci_heal_commit_push_script`, ~~`scheduler_invoke`, `scheduler_invoke_with`~~ **DELETED #9252**, ~~`git_fetch_script`~~ **RENAMED #6833** | roadmap `2-emit-partition`; unassigned; `ci_spec.dag` is load-bearing | `ci_fmt_gate_line` calls `cargo.Build.Fmt`; the remaining rows become typed intent → canonical Bash emission; old composers delete and `ci.yml` drift+parse stays green |
| Phase-1 orchestration fidelity | ~~`ci_floor_peak_while_cond_command`, `ci_floor_peak_while_body_command`~~ **DISCHARGED #7978 (`ecbd0863c2a`, 2026-08-08)** — both leaves, and `ci_floor_peak_assign_cgroup_dir_command` with them, are derived from typed `std.path_intent` / `extdeps.linux.proc_self_cgroup` models via `bash_orch_if`, refusing with a located marker rather than widening. **SUPERSEDED 2026-10-21 (#14467): the still-open peak calibration leaves (`ci_floor_peak_test_f_memory_peak_command`, `ci_floor_peak_echo_reset_redirect_command`, `ci_floor_peak_pre_reset_ok_command`, `ci_floor_peak_pre_reset_unsupported_command`, `ci_floor_peak_pre_unavailable_command`, `ci_floor_peak_post_ok_command`, `ci_floor_peak_post_unavailable_command`) and `bash_membership_assign_from_source` were DELETED with `v2.workflow.ci_floor_peak_emit` and `v2.extdeps.languages.bash_proc_self_cgroup` as no-consumer §3c chains — the Phase 2 typed-observation work on this module is closed by deletion, not by typing** | roadmap `6-shell-intent-phase1`; While-fidelity half CLOSED #7978, Phase 2 calibration half unassigned | ~~model the two live While leaves and move their discriminating witness onto normal cadence~~ (done); the residual trigger is Phase 2 typed filesystem observation through `host_effect_apply` for the calibration leaves; #7318 explicitly refuses nonempty Retry body `Run.env`, BoundedPoll was deleted 2026-08-07 (zero-production-constructor census), and three-or-more (N-level) Retry is refused, so the remaining scope/adoption claim either gains a need-driven consumer with faithful lowering or narrows explicitly |
| Already-routed foreign emit, terminal raw leaves | the §4.E exact output roster: `ci_isolate_toolchain_script`, `ci_pin_rustup_default_script`, ~~`ci_selection_control_script`~~ **DELETED #8283**, ~~`ci_cgroup_peak_locate_shell`, `ci_floor_peak_pre_script`, `ci_floor_peak_post_script`~~ **DELETED #14467**, `ci_cargo_eagain_retry_script`, `ci_release_build_script`, ~~`gunbc_ci_run_script`~~ **DELETED #9252**, `ci_sccache_provider_shell_injection`, ~~`ci_regen_ensure_rustfmt_path_script`~~ **DELETED #8406** (and these named raw leaves similarly deleted: `ci_regen_rustfmt_test_x_cargo_home`, `ci_regen_rustfmt_test_x_opt_cargo`, `ci_regen_rustfmt_test_x_home_cargo`, `ci_regen_rustfmt_test_x_rf`, `ci_regen_rustfmt_command_v`, `ci_regen_rustfmt_export_cargo_home_path`, `ci_regen_rustfmt_export_opt_cargo_path`, `ci_regen_rustfmt_export_home_cargo_path`, `ci_regen_rustfmt_export_rf_dir_path`, `ci_regen_rustfmt_error_echo`, `ci_regen_rustfmt_rustup_which_assign`), `gcp_token_smoke_script`, and ~~`fresh_standup_bootstrap_intent`~~ **DISCHARGED — no such function exists; the name survives only in the `reason:` of `gunbc.host.host_effect` `bootstrap_fragment_script_carrier_scaffold` and in this roster.** | roadmap `2-emit-partition`; Phase-1 owns the While fidelity and scope/adoption subset | retain the correct foreign/bootstrap Bash target, but replace ambient `Run.command` strings with modeled `Do{effect}` leaves and delete each raw producer; for #7290 this is the cited `FileExists` + session-`EnvExport` carrier trigger; do not re-home them in another String carrier |
| Permanent foreign media | `expected_githooks_pre_push_sh`, `build_cron_entry_line` | roadmap `2-emit-partition`; unassigned | their **builders** route through typed intent → Bash; the emitted shell artifacts remain permanently because git/cron require that medium |
| Runtime `Run.command` / `ShellCommand` terminal | `live_deploy.emit.deploy_raw` fed by `ensure_apt_package_step`, `apply_systemd_unit_write_step`, `apply_tree_sync_unit_write_step`, `tree_sync_restart_step_with_diagnosis`, `tailscale_serve_apply_step`, `install_d_owned_command`, `deploy_apply_preamble_steps`, `emit_artifact_upsert`, `emit_artifact_teardown`, `emit_deploy_teardown_dependency_refusal`, `apply_intent_from_effects`, `retract_intent_from_effects`; ~~`host_effect_plan_placeholder_effect`~~ **DELETED #10509**; the `gunbc.host.host_effect.ShellCommand` carrier and its total match consumers `host_effect_realize.resolve_host_effect_cell` and `fleet_converge_cli.converge_cli_applied_knob_count` | roadmap `2-emit-partition`, coordinated with the host-effect lane | replace `Run.command` / `ShellCommand` with `Do{effect}` typed operations, migrate every total match, then delete the carriers; never count Pipeline wrapping as completion |
| Deferred runtime srv\* | `host_effect_realize` · `srv3_transport_witness_bin_success`, `srv3_transport_test_executable`, `srv3_apt_tool_present`, `srv3_tool_bin_path`, ~~`srv3_chown_directory_to_current_user`~~ **DELETED #8796 (see §4.D)**; `host_effect_realize` (relocated from the now-deleted `host_build_cache_provision_script`, unit body builder dropped in favor of `gunbc.build_cache.build_cache_unit.render_build_cache_unit`) · ~~`build_cache_observe_stats_body`, `build_cache_classify_failure_body`, `build_cache_ensure_daemon_body`, `build_cache_read_back_body`, `build_cache_provision_script_body`, `build_cache_provision_shell_script`~~ **ALL DELETED #14155**; the ensure legs run as typed argv through `gunbc.host.build_cache_daemon_ensure`; ~~`host_hygiene_reaper_script` · `host_hygiene_reap_install_units_body`, `host_hygiene_reap_classify_and_act_body`, `host_hygiene_reap_stale_override_body`, `host_hygiene_reap_script_body`~~ **ALL DELETED #8583 (see §4.A)**; ~~`host_hygiene_liveness_script.host_hygiene_liveness_read_body`~~ **DELETED #8583** | operator-deferred surviving srv\* actuator/runtime graph | un-defer a typed-effect migration with its live consumer; the separate dead install/reconcile subgraph is already deleted (#7233/#7237), so do not polish or resurrect it; build-cache's own trigger is `#5828` — the relocation into `host_effect_realize` is not that trigger firing |
| Direct meta-exec construction | `tools.gunbc_ci.main`, `tools.merge_admission_stamp.pr_number_from_event`, ~~`gunbc.instruments.emit_host_gate.gate_failure_exit`~~ **DELETED 2026-09-13 (gentle-gull-238) — the function's only direct `shell.Exec.Run` construction was the verdict-narration printf (§4.C row above); its body now builds a typed `exit_failure(reason)` whose reason carries the per-smoke verdict line, so the site is gone rather than migrated**, `gunbc.instruments.host_prelude.toolchain_provision_shell_exec`, `gunbc.bmc.bmc_netboot_serve.bmc_netboot_run_bmc`, `gunbc.bmc.bmc_netboot_serve.bmc_netboot_put_path`, `gunbc.command_runner.run_shell_command`, `gunbc.host.host_effect_realize.run_shell_transport` — direct `shell.Exec.Run` sites through the two counted bridges | per-site runtime/foreign owners delete their bodies | **construction is closed at both edges** (#7184 record wall, #7962 seal); these rows are migration work, not wall work, and the `Reference`-verb lane is no longer their blocker |

**srv3 deferral — premise measured false, disposition still owed (2026-08-20, measured at `origin/main` `4cec10f6`).** The deferral of the `srv3_*` runtime helpers rested on the premise that the subgraph was pending subsumption by the `EmitArtifactThenThinRun` transport, so migrating its call sites would be wasted work. That premise is false, and the refutation is a typed refusal rather than an absence: all `EmitArtifactThenThinRun` match arms in `gunbc.host.host_effect_realize` refuse, none accept, and the refusal text names the boundary — *"reserved for ConvergePlan (host-effect Phase D)."*

The control widens the finding past the one file, but **not** to "the transport is unrealized" — that stronger claim was asserted here on 2026-08-20 and falsified the same day by reading every arm body rather than pattern-matching refusal shapes. `EmitArtifactThenThinRun` has **exactly one realizer**: `gunbc.host.host_effect_realize` `realize_converge_on_host` splits on `gunbc_apply_mode_for_host` — `FreshStandup` refuses as fail-closed Unimplemented; `ExistingHostQuiescentReload` calls `realize_converge_in_process`, *the identical call the `LocalShell` arm makes*, so for an existing quiescent host the transport takes the shell transport's own path. The neighbouring `SshShell`/`FleetSsh` refusals confirm the direction: their text directs the caller to "use EmitArtifactThenThinRun."

Two disguises defeated the pattern-based reads that produced the wrong count, each surviving a scan written for the other: a **refusal shaped as a success-valued record** (`gunbc.hostname_set` answers `HostnameSetCasApplyFailed { stderr: "EmitArtifactThenThinRun unsupported..." }`, which no `NotConverged` scan sees), and a **realizer behind a refusing first sub-arm** (the arm above, whose opening lines read as a refusal). Only reading arm bodies rather than matching shapes survived.

**The denominator both audits used was itself wrong, and the correction belongs here rather than in a footnote.** An earlier revision gave an arm-body count inherited from the manual audits. Measured at `4cec10f66a3d84cbde631e20ca9feaa31457d88c`, the tree carries `EmitArtifactThenThinRun` occurrences: the variant declaration (`gunbc.host.host_effect`), **production match arms** across many modules, and some in `dag/test/claim/`. Neither manual audit reached the production arm count, and neither mentioned `gunbc.host.host_effect_codex_supervised_turn`, whose arm answers `CodexSupervisedSessionApplyRefused` — another refusal wearing a record name rather than a refusal type. The **conclusion is unchanged**: exactly one arm realizes, `realize_converge_on_host` under `ExistingHostQuiescentReload`. What was wrong was the count of what had been examined, in a paragraph about examining carefully.

That is the point, not an aside. Two readers who had just corrected each other on method still could not state their own denominator: a hand-assembled roster cannot prove its own completeness, only report what someone remembered to look at. A derived census is not a stylistic preference; it is the only kind that can be wrong *loudly*. The arm partition above is therefore a **dated measurement at a named SHA, never a fixed law**; a derived detector is not a prerequisite (retired by the 2026-10-30 process ruling (`gunbc.design_ruling bad_process_is_worse_than_no_process`)); migrations are expected to change it.

Everything else does refuse: `gunbc.systemd_run_transient`, `gunbc.systemctl_stop_run`, `gunbc.systemctl_status_read`, `gunbc.systemctl_show_read`, `gunbc.systemctl_is_active_read`, `gunbc.hostname_set`, `gunbc.hostname_read`, `gunbc.spark.serving_converge_apply`, `gunbc.live_deploy.readiness`. The sites that appear to accept — `gunbc.ci.ci_deploy_access`, `gunbc.ci.ci_deploy_target_host`, `gunbc.host.host_identity_adopt` — are classifiers whose arms return a `Bool` or the variant's own name for totality; none executes the transport.

**The srv3 finding is unaffected by the correction.** The one realizing path is for `HostConverge`, not any srv3 effect: every `Srv3*` arm still refuses, including `Srv3BootOnceReadBackSleep`, `Srv3SeededInstallMediaToolchainEnsure`, and the `srv3_actuator_toolchain_*` helper refusals. What changes is the supporting sentence, not the conclusion — no longer "the transport is realized nowhere, so nothing is pending cutover" but **"the transport has exactly one realizer and srv3 is not on it."**

What this settles and what it does not. **Settled by measurement:** the srv3 subgraph is not subsumed today and not scheduled for subsumption by any live mechanism, so the deferral has no standing premise. **Not settled — a decision, not a measurement:** whether the `srv*` actuator graph survives at all. The tree cannot answer that; migrating the call sites (`srv3_transport_witness_bin_success`, `srv3_tool_bin_path`, `srv3_apt_tool_present`, `srv3_chown_directory_to_current_user`) in a subgraph the operator intends to delete by another route would be wasted work under a different premise than the original. The deferral is **due for re-decision, not automatically expired**, and that re-decision is the last blocking input on the largest remaining block in this census.

**Prior art any derived census would supersede rather than extend — `v2.lens.effect_reach` (measured 2026-08-20 at `4cec10f6`).** A lens answering approximately this question exists and may be mistaken for coverage: it classifies host-effect sinks and carries a `ShellExecRunSink` variant, so its subject overlaps the shell census directly.

Its selection is the forbidden seed form. `sink_kind_for_callee` takes a `callee_text: String` and compares it by name equality against a remembered list — `"Run"`, `"shell.Exec.Run"`, `"Exec.Run"`, `"WitnessBin.Run"`, `"Read"` — everything else falling to `UnknownHostEffectSink`. That is *where we currently remember doing so*, not *what interprets bytes as shell*: it cannot see `sh -c`, `sh -s` on stdin, or a wrapper stack such as `sudo -S sh -s` — **the same blind spot this census carried before Spark was found**, where `gunbc.spark.managed_access_apply`'s script builders reach a host through `stdin_payload` behind a wrapper, unreachable by any callee-name comparison.

Two further facts bound how much weight it can hold. Its classification derives from callee *text* rather than resolved callee identity — the text-scanning substitution the census brief's stop condition forbids. And `v2.test.claim.enforcement_live_witness_test` asserts `lens_contract_is_unbound(c: lens_contract_effect_reach)` — the corpus's own record that the contract has no enrolled consumer. Whatever else it is, it is not gating.

The disposition is therefore **supersede, not extend**: a census seeded from interpretation semantics subsumes this lens's entire sink vocabulary, while extending it would inherit its selection principle. Recorded so the derived cut treats it as prior art with a known-narrow frontier, not an available foundation — and so its `ShellExecRunSink` rows are never read as an existing shell-reach population.
**Proposed batching (3 PRs):**

| PR | scope |
| --- | --- |
| **PR 1 (#7265)** | `ci_sccache_provider_shell_injection` + `ci_floor_disposition_marker_init_script` emit migrations; `orchestration_bash_emit_support` shared plumbing; semijoin scaffold dissolved onto block-bodied `if_else` (#7277); witness tests + `realization_vocabulary_containment` roster; regen `ci.yml`. Does not touch `ci_spec.dag` composers beyond import delegation. |
| **PR 2** | merge-admission cluster completion + receipt gates (A/B remaining rows); the dead docs-only disposition family retired separately without new read models. |
| **PR 3** | `ci_spec` composer migration + `ci_fmt_gate_line` → typed `cargo.Build.Fmt`. **Operator review required** — load-bearing CI generator. |

**PR-2 scope change @ 2026-07-26 (operator ruling, relayed snappy-moth-330; discharged by #7293, 2026-07-27).** The two concats #7265 left standing were about to be recorded here as accepted residue with dissolution triggers. The operator ruled the opposite: *dissolve them now, along with all other instances*. Both are **LANDED in PR 2a #7293**, not tracked residue; the table records the carrier construction that discharged them:

| residue | site | prerequisite carrier | why it was standing |
| --- | --- | --- | --- |
| `ci_merge_admission_floor_disposition_stamp_command` | `v2.workflow.ci_merge_admission_emit` (no longer present at this revision) | `RedirectSpec.ToFile` (`src/v2/std/orchestration.dag`) — **LANDED #7293 with this live consumer** | `echo X > path` was unexpressible before #7293; the carrier and consumer landed together, never speculatively. |
| `orch_bash_comment_line` | `v2.workflow.orchestration_bash_emit_support` `orch_bash_comment_line` | `PipelineStep.Comment`, wired to the **existing** `realize_run_rawline` medium method — **LANDED #7293** | A comment routed through `Do{Run{command}}` was a category error, not a missing grammar row — see the §4.G name-collision trap above. |

The governing rule for the rest of the sweep, so "all other instances" stays decidable: **concat-assembled dissolves now; a constant command string waits for arc close.** `"test -f " ++ path` is a concat and earns a typed carrier; `"exit 0"` or `"mkdir -p target"` is a constant and belongs to the `Run.command` → `Do{effect}` migration, not here. Gold-plating constants now is the §6 purity trap.

**Adjacent finding — `CaptureSpec` capture is a §3 dual representation (raised wise-crane-222, corrected and verified snappy-moth-330, 2026-07-26).** Not part of either dissolution above. First reported as *"TeeTo and CmdSubst are declared but unreachable"*; that is **wrong for `TeeTo`** and the corrected form is what matters:

- **`CaptureSpec.CmdSubst`** — genuinely dead: zero constructors tree-wide. (`ExprCmdSubst` and `CmdSubstLines` are different types.)
- **`CaptureSpec.TeeTo`** — **was live but inert; fixed by merged #7293.**
  - *Before #7293:* `ci_retry_body_run` (`gunbc.ci.ci_spec` `ci_retry_body_run`) constructed `capture: TeeTo { log: "BUILD_LOG" }`, but `orch_retry_step_command` returned only `r.command` and `orch_emit_retry_run` rebuilt the `Run` with `redirect`/`capture` `Absent`. The identical tee semantics were **hardcoded as fixed tokens** in `v2.extdeps.languages.bash` (the `retry_eagain_seg_*` fragments), so the emitted bytes were right and the *model* was the copy that did nothing — editing `log: "BUILD_LOG"` changed no output. Exactly the tell §5 names: the spec could be edited while the realizer kept doing its own thing.
  - *Current main after #7293:* `ci_retry_body_run` is `redirect: Absent, capture: Absent` — the lying declaration is **deleted** — and the retry path refuses any `Present` redirect/capture instead of emitting while dropping it. The discriminating RED (`ci_merge_admission_emit_test.dag`, `orch_retry_body_tee_to_refused_holds`) proves a retry-body `TeeTo` is rejected. `TeeTo` now has **zero production constructors**; the only constructor is that refusal control.
- **It was a divergence, not one bug:** the ordinary `Run` emit path already refused the same field fail-closed and typed (`^orch_emit_run_capture_unsupported`, in `v2.compiler.emit_orchestration` `orch_emit_apply_capture`), while the retry path silently dropped it. **#7293 closes that** — both paths now refuse.

**Disposition — this `TeeTo` dual-representation defect is DISCHARGED on the current tree by #7293.** Verified after merging current `origin/main`: `ci_retry_body_run` carries `Absent`/`Absent`, a retry-body `TeeTo` rejects, and no production constructor claims the hardcoded template tee is controlled by intent.

- **Fix direction changed, and the change is right:** originally filed as "thread `redirect`/`capture` through the retry path". Deleting the lying declaration is better — an inert declaration is worse than an absent one, because it lies.
- **Current honest modelling gap:** the intent layer cannot express "tee this run to a log" at all, leaving the tee solely as fixed tokens in the grammar row — an honest gap rather than a dual representation. **Trigger:** dissolves when the bash rows grow a modelled tee/redirect construct that `Retry` can carry. `CmdSubst`'s deletion rides with that work.
- **#7303 is merged on current main (`5b229afd65`):** it replaces the `Optional<String>` conflation with `Outcome<Run>`, refuses multi-step bodies and distinct redirect/capture cases, validates the template-hardcoded log source, and refuses the 3+ escalation shape whose old template dropped `level1.env`.
- **#7318 is merged on current main (`e8b7117f4d`):** it refuses every nonempty Retry body `Run.env` before dispatch can discard it. The executing `orch_retry_body_env_present_refused_holds` RED proves the nonempty shape rejects; `orch_retry_body_env_empty_accepted_holds` plus the unchanged ordinary Retry goldens prove the empty shape keeps its existing bytes. No accepted Retry body field is silently discarded at this boundary.

### Dissolution trigger for §4

**The construction wall this trigger waited on has PARTIALLY landed (#7184, 2026-07-24)** — it record-walls the **`ShellOnHost` realization edge** (`ResolvedHostEffectCell.ShellOnHost { script: RetainedShellScript }`, `gunbc.host.host_effect_realize` `ResolvedHostEffectCell` (no longer present at this revision)) and deletes the free minter `extdeps.shell.exec.transport_script_from_body(body: String)`, so a hand-assembled `String` no longer typechecks *on that edge*, with compile-fail REDs (§4.F, §5.E).

It did not close the `shell.Exec.Run` sink; **#7962 did.** `TransportScript` is a **`sole_constructor` record** whose single mint `transport_script_seal` is `admit_callers`-sealed to exactly two production declarations (`gunbc.retained_shell_script` `retained_shell_script_to_transport`, `gunbc.bash_materialized_transport` `bash_materialized_transport`), and the cast form closed with it — `04_infer` `sole_constructor_construction_diags` judges a cast into a sealed type (#7962). The `meta_exec_confinement` scan and `host_language_transport_script` remain as validation backstops, but construction is no longer what they guard. The leak fixture at `src/v2/test/fixture/meta_exec_confinement_scan/leak/plant.dag` is now **scan input and historical record only** — no longer citable as evidence that the cast compiles.

So *new* instances of the `ShellOnHost` class can no longer be authored; the punch-list is **not** closed at the top.

*(Corrected 2026-07-27, `review 43600`. This read "a raw-string `shell.Exec.Run` / hand-built transport is already unwritable … so *new* instances of this class can no longer be authored, and to that extent the punch-list is closed at the top." **Overstated on both counts** — the wall is real but scoped to `ShellOnHost`, and new instances can still be authored through the brand cast. Declaring the top closed while a writable sink remains is precisely the misdispatch this document exists to prevent, and the third "wall landed" claim in this lane to overstate its scope: the recurring error is checking that a carrier exists rather than that every route into it is closed.)*

What the trigger does **not** discharge is the **existing** rows: the wall stops new concats **on the `ShellOnHost` edge**; it does not dissolve the ones already written, and does not stop new ones at the still-open `shell.Exec.Run` sink. Each remaining row is still discharged the same way — *deletion of the concat*, verified green-by-execution + an injection-RED, never by relocation. This section retires when that list empties, not on further wall work.

*(Corrected 2026-07-26, `review 43494` then `43501`: this trigger read "folds into the `host_language_transport_script` lens **going live**". Superseded on both counts — the lens **was already activated** by #7184 (§4.F), and it was never what closed this class. It is **live but deliberately green** on `ComputedApplication`, redding only a raw literal at an enrolled `Run` position: a backstop for literal blobs while **construction** closes the computed-join class. My first correction here said the lens "is inert by design" — itself wrong, contradicting §4.F in the same edit pass. A lens is *validation*, the `RetainedShellScript` record edge is *construction*, and §5 prefers the second because it makes the state unwritable rather than merely flagged.)*

---

## 5. Method of Action — the bounded path to bash-free user space (calm-ferret-849, 2026-07-22)

**End state:** no user-space `.dag` constructs a shell string. Every "this `.dag` wants to call a bash script" instance resolves to exactly one of the four paths below. The `realization_vocabulary_containment` lens (#6854, LIVE) already forbids bash-AST vocab (`ShellProgram`/`ShellStmt`/`serialize_bash`) in user space. #7184 retyped the **`ShellOnHost` realization edge** to the `RetainedShellScript` record (`gunbc.host.host_effect_realize` `ResolvedHostEffectCell` (no longer present at this revision), `gunbc.retained_shell_script` `RetainedShellScript`) and deleted the free minter `transport_script_from_body`, so a hand-assembled `String` no longer typechecks *there*; what remains possible on that edge is **conspicuous**, not silent — authoring a counted `RetainedShellScript` row with a hand-built body is the *reviewed* escape, carrying a `reason` and a `dissolves_to`, and that visibility is the mechanism, not a leak in it. #7184 did not retype `shell.Exec.Run`; #7962 **closed the `shell.Exec.Run(script: TransportScript)` sink at construction** (§5.E), at the type rather than by the lens: `TransportScript` is a **`sole_constructor` record** whose single mint `transport_script_seal` is `admit_callers`-sealed to exactly two production declarations (`gunbc.retained_shell_script` `retained_shell_script_to_transport`, `gunbc.bash_materialized_transport` `bash_materialized_transport`), and the cast form closed with it — `04_infer` `sole_constructor_construction_diags` judges a cast into a sealed type — so the meta-exec confinement milestone is met at construction; what survives is the bridges' bare-`String` body, dissolved by migration rather than a further wall. *(Rewritten 2026-07-26, `review 43544`. This sentence previously called that sink "the remaining hole", and my first correction **appended** a supersession notice below it instead of rewriting — two mutually exclusive current-state claims in one paragraph, misapplying this document's own rule: strike-in-place is for corrections **illegible without the original**, and this one is not — the four-path end state survives the rewrite intact, so only the blocker claim needed replacing.)*

**The headline (verified by op inventory @ `78f43c38`):** the arc needs only **a few new typed ops**. Almost every site calls an op that already exists.

### 5.A — the FINITE new-op list (the ONLY new modeling the whole arc needs)

| new op | home file | covers verb | consumers |
| --- | --- | --- | --- |
| `extdeps.tools.hostname` · ~~`Read`~~ **LANDED** + ~~`Set`~~ **LANDED, then migrated** | `dag/extdeps/tools/hostname.dag` (moved from `dag/extdeps/os/hostname.dag`) | ~~`hostname -s`~~ **done**; ~~`hostnamectl set-hostname`~~ **done** | **Read done** — `os.Hostname.ReadShort` realized via typed cell `HostnameReadOnHost` + `gunbc.hostname_read` (LocalShell → op, SshShell → typed argv); both read consumers dissolved: `host_identity_observation` (`HostIdentityShortHostnameRead`, the `host_identity_short_hostname_script` concat DELETED) and `ci_deploy_target_host` (runtime-present read; the GHA preflight *block* stays E-emit). **Set done at the §5.B grain, then migrated at the #8467 grain** (CLI-invocation lane): `os.Hostname.Set`'s hand-authored `transport shell { argv: ["hostnamectl", "set-hostname", "{desired}"] }` row is **deleted**; the local leg realizes through `extdeps.tools.hostname`'s `HostnameSetInvocation` → `hostname_set_invocation_process_plan` → `hostnamectl.Process.Run` (argv derived from the cited `hostnamectl_set_hostname_verb` row), and the SSH legs' privileged argv reads the same cited verb. |
| `systemd.Systemctl.ListUnits` | **add to** `dag/extdeps/os/systemctl.dag` | `systemctl list-units --state=active` | `host_converge_slice1` (was `_enumerate_units_script`; now calls `systemctl_list_units_active_services`). *The `fleet_show_effective_read` consumer named here — `fleet_runner_width_count_read_script` — was never migrated: it was superseded and left dead, and is deleted by the 2026-07-26 true-up.* |
| `extdeps.os.id` · `Read` (uid/gid/user) — **ops landed; row NOT done** | `dag/extdeps/tools/id.dag` (created; landed #7194; post-#7231 home) | `id -u`, `id -g`, `id <user>` — service `os.Id` ops `Uid`/`Gid`/`Lookup` | **Operation model + hermetic operation witnesses have landed** (`test.claim.shell_dag_census_5a_typed_ops_witness`, `witness_os_id_uid_realizes_hermetically`, `witness_os_id_gid_realizes_hermetically`, `witness_os_id_lookup_realizes_hermetically`; declared scope is model-before-implement, call-site migration §5.B a later lane). **NOT "Done"** — that word requires the production consumers to have moved AND the superseded string/argv path to be deleted. Remaining argv/SSH callers are dispositioned independently, not by this row: `host_effect_realize` srv3 probes (raw argv rows `extdeps.tools.id.id_uid_argv`/`id_gid_argv`, absolute-path variants), `fleet_posix_accounts` `probe_command` (NOT-a-shell-site, §4), `spark/managed_access_apply` (deferred, dissolve-on). `id_lookup_argv` RETIRED by ruling (below). |
| `ssh.Session.ExecArgv` | **add to** `dag/extdeps/diagnostic/ssh.dag` | typed argv over ssh (`ssh host -- argv`) | `host_effect_realize` ssh probes — **IN FLIGHT, C5 #6946** |
| `systemd.Systemctl.Status` | **add to** `dag/extdeps/os/systemctl.dag` | `systemctl status --no-pager --full <unit>` (models exit-3-for-dead-unit — retires the `\| tail`+`exit 0` absorbing fallback) | `live_deploy/readiness.dag` unit diagnosis |

**RULING — `extdeps.tools.id.id_lookup_argv` RETIRED and deleted (ruled 2026-09-03).** The retirement is on the ruling, not on the zero-reference count: `os.Id.Lookup` (the semantic operation) already names the intended request; a future consumer should consume `os.Id.Lookup` and choose a typed SSH realization (`ssh.Session.ExecArgv` on the `id` binary with a `{user}` argument, or an `IdLookup` invocation modeled on the `os.Hostname.Set`/`HostnameSetInvocation` pattern) — not resurrect a raw `List<String>` argv helper because it sat in inventory. Keeping it "reserved" would preserve exactly the lower-level interface the migration exists to make unavailable. The measured denominator (filed 2026-09-03) found no production reference — the §5.A witness calls `os.Id.Lookup`, NOT this helper — but a future need is the trigger to add a typed remote realization, not evidence that the helper should survive.

That is the entire new-modeling surface. (A sixth, optional: a typed stdout/receipt emit for the two `echo <receipt>` sites, or reuse `Filesystem.Write`.) The healthz probe needs **no** new op — `http.Client.Get` already exists.

### 5.B — CALL AN EXISTING OP (receipt: the op is already modeled on main)

| site (file · symbol) | current shell | call this op — receipt (file · service.Op) |
| --- | --- | --- |
| `host_converge_slice1` · `_memory_max_read_script` | `systemctl show --property=MemoryMax --value` | `systemd.Systemctl.ShowProperty` · `extdeps/os/systemctl.dag` |
| `host_converge_slice1` · `_memory_max_set_script` | `systemctl set-property … MemoryMax=` | `systemd.Systemctl.SetProperty` · `extdeps/os/systemctl.dag` |
| `host_converge_slice1` · `date`, `host_identity_adopt` · `date` | `date -Iseconds` (local ISO w/ offset) | `Clock.Now` · `extdeps/clock/clock.dag` — **⚠ semantics differ**: `Clock.Now` is wired to `date -u +%Y-%m-%dT%H:%M:%SZ` (UTC `…Z`), not local `-Iseconds`. Migrate by adopting UTC-`Z` (canonical for receipts) — a deliberate reconciliation, NOT a silent drop-in; if a site truly needs local offset, add a `Clock.NowLocal` variant rather than fork `date` (review 41476) |
| `fleet_show_effective_read` · `SystemdUnitMemory*Read`, `fleet_runner_unit_property_read_script` | `systemctl show --property` | `systemd.Systemctl.ShowProperty` · `extdeps/os/systemctl.dag` |
| `host_effect_realize` · ssh `test -x <path>` | `test -x` | `shell.Find.IsExecutable` · `extdeps/shell/shell.dag` (spliced via `ssh.Session.ExecArgv`) |
| `host_effect_realize` · ssh `command -v <tool>`  | `command -v` | `shell.Which.Check` · `extdeps/shell/*` (via `ExecArgv`) |
| `tools/review` · `design`, `algebra_ref` | `git fetch` + `git show origin:FILE` | `git.Core.FetchNoTags` + `git.Core.Show` · `extdeps/git/git.dag` (`|| echo '(not found)'` → typed error→Absent) |
| `merge_admission_stamp` · `mkdir -p` | `mkdir -p` | `Filesystem` / `shell.Find.Dir` · `extdeps/filesystem/filesystem_io.dag` |
| `dag/gunbc/instruments` · `host_prelude`/`gunbc_ci`/`emit_host_gate` witness+build transports | witness/build run | `gunbc.WitnessBin.Run` · `extdeps/gunbc/gunbc.dag`; `cargo.Build.*` · `extdeps/rust/cargo_build.dag` (`host_prelude` already has the typed precedent) |
| `live_deploy/readiness.dag` · `live_deploy_healthz_probe_script_for_port` → `intent.dag` · `live_deploy_health_probe_curl_command` | `curl` localhost `/healthz` | `http.Client.Get` · `extdeps/http/client.dag` (already exists) |
| `host_identity_assimilation`/`adopt` · `echo <receipt>` | `echo <msg>` | typed receipt/stdout emit (`Filesystem.Write` or a print op) — not a shell need |

Each row's `2>/dev/null || true` / `|| echo` fallback becomes a **modeled outcome** (`nonzero => …` mapped to `Absent`), never re-appended (§5 absorbing-fallback rule).

### 5.C — EMIT via the bash backend (foreign executors ONLY — bounded roster, legitimately stays shell)

Bash-as-target lives in one isolated backend: `src/v2/extdeps/languages/bash*` + `src/v2/workflow/bash*` (confined by the containment lens). These sites emit *through* it because the executor's input contract IS shell text:

| site (file · symbol) | executor | path |
| --- | --- | --- |
| `ci_spec` · `ci_floor_build_verify_script`/~~`ci_release_bins_pack_script`~~/~~`…unpack_verify_script`~~ (DELETED #12422)/`ci_regen_floor_skip_shortcut_script` | GitHub Actions `run:` | `emit(intent, Bash)` via the bash backend |
| `ci_materialization` · `ci_floor_materialization_receipt_gate_script`/`…resolve_receipt_gate_script` | GitHub Actions | same |
| `merge_admission_produce` · `ci_*_script` | GitHub Actions | same |
| ~~`fleet_converge_emit` · `fresh_standup_bootstrap_script`/`_arm_golden`~~ **DELETED (`fc47b4f9866`, #7121)** | ~~pre-runtime bootstrap~~ | discharged: no emitter remains |
| cron entry lines, `local_tidy_spec` pre-push hook | cron / git | emit, **permanent** roster (the honest residue) |
| `dispatch_pipe_pane_emit` · `dispatch_event_pipe_emission` | tmux `pipe-pane` (its input contract IS a shell command) | **RESOLVED** (`review 44063`, refusal shape corrected per `review 44113`). The payload is built as a bash AST (`bash_build_command_from_lits`) and rendered through `v2.workflow.bash_command_fold_serialize` — quoting is the lit-word production's job, so `shell_quote` no longer appears at this site. Reaching the backend required moving the construction onto a realization edge: `dag/gunbc/dispatch_pipe_pane_emit.dag` is enrolled in `realization_edge_path_prefixes` (precedent: `dag/gunbc/live_deploy/emit`). The *seam* stays permanent, as the pre-push hook row above; the hand-joined *construction* is gone. A serializer refusal is a **typed refusal, not a command**: `PipePaneEmission = PipeEmitted { command } \| PipeEmitRefused { detail }` has no arm that yields a command on refusal, so it propagates as `DispatchSpawnRefused { step: "tmux-event-pipe-emit", detail }` and surfaces through the belt's existing `SpawnFailed` carrier — dispatching an un-emittable intent is unwritable, not merely unlikely. Witness: `witness_event_pipe_emits_grammar_quoted_command` |
| `roadmap_provider_events` · `codex_provider_event_projection_filter` | jq (a foreign DSL, not shell) | **declared scaffold** (`review 44058`), trigger in the leading annotation on `codex_provider_event_projection_filter`: the filter exists only to bound a JSONL line before the substrate reads it, so the dissolution is the substrate-side bounded read (witness-realization lane) deleting BOTH the filter and its `ProviderEventProjectionCapability` jq dependency — not a jq grammar. Interpolated values are the `CharacterCount` budget rows, never caller input |

`roadmap_static_site` · `roadmap_site_*_body` is HTML/JSON content emit (not shell) for the srv1 dashboard — dissolves with belt B (`gunbc serve`), tracked there, not here.

### 5.D — DEFERRED (each with its own trigger)

| bucket | sites | trigger to un-defer |
| --- | --- | --- |
| C5 access probes | ~~`host_effect_deploy_access_probe_script`~~ and ~~`ci_deploy_access_emit`~~ **deleted** | **DISCHARGED by C5 #6946 + corrective #7315.** The runtime access gate uses typed effective-principal, execute-probe, and grant-list-probe operations; no embedded deploy-script preflight remains. |
| surviving srv* actuator/runtime cluster | `host_effect_realize` srv3 command-string helpers, `host_effect_realize` build-cache shell builders (relocated from the deleted `host_build_cache_provision_script`), `host_hygiene_*` | operator un-defers the surviving actuator graph; the dead install/reconcile subgraph named below is already retired (#7233/#7237) |
| nbd backgrounding | **`RawLine` already dissolved** (corrected 2026-07-26); now a systemd transient-unit transport with `WitnessBin` `systemd-run` scaffolding | observe-side port/unit read-back on cited `extdeps.systemd`/`systemctl` + typed argv retiring the scaffold (trigger in-file; operator ruled no trap/&/$! vocab) |

#### Roadmap item — srv3 install/reconcile subgraph retirement — **LANDED #7233/#7237**

**Finding (liveness audit, snappy-moth-330 @ the §5.E wall):** the srv3 install/reconcile cluster was dead — `srv3_os_install_reconcile_apply` was reached from no `gunbc` subcommand, CLI, or CI path; srv3 was already installed. The audit named `srv3_host_effect_script.dag`, `srv3_install_diagnostic_checklist.dag`, the `Srv3InstallDiagnosticObserve` realization arm, and the `srv3_os_install_reconcile*` scaffold subgraph.

**Landing receipt:** #7233 performed the coordinated load-bearing coproduct surgery: deleted the dead script/checklist/reconcile/SOL files, removed their `HostEffect` variants and realization arms, updated every total match and witness, and removed the corresponding retained-shell frontier rows. #7237 then deleted the orphaned `DurableApprovalGrantRead` variant. Current tree has none of the retired files or variants.

This receipt is deliberately narrower than “all srv3 is gone.” `Srv3NbdProxyServe`, the surviving actuator/toolchain helpers in `host_effect_realize`, build-cache provisioning, and host hygiene are different live/deferred rows with their own triggers above; #7233/#7237 must not be used to mark those complete.

### 5.E — the enabler that had to come first — **LANDED in two steps**: #7184 closed the `ShellOnHost` string sink, #7962 sealed `TransportScript` and closed direct `shell.Exec.Run`

The construction wall landed in two steps: #7184 made the host script edge a RetainedShellScript record and #7962 sealed TransportScript, including casts. Counted string-body bridges remain. The three original proposals below have different dispositions; do not repeat completed/refuted work or retire the surviving typed-argv path.

> ⚠ **The three bullets immediately below are the ORIGINAL plan, superseded by the ruling + design-correction block that follows.** Kept because the correction is only legible against them. **Each bullet failed differently, and the difference decides what a reader should do with it** — a *refuted* plan must never be retried; a *completed* one already happened; a *partially superseded* one still has live residue. Do not action them as written; read on. *(Flagged `review 43486`; classifications corrected `review 43522` — the earlier version lumped bullets 1 and 3 together as "refuted by execution", wrong for bullet 3 and exactly the conflation §4.G's method note warns about.)*

- ~~Brand `TransportScript` so it is produced ONLY by `emit(intent, Bash)`~~ — **REFUTED.** The brand is transparent (`expand_nominal_alias_rhs` peels it to its base), so branding cannot make a hand-join a type error. The idea does not work; replaced by the `RetainedShellScript` **record**. Never retry this.
- **Make `ShellOnHost{script}` take typed argv (`List<String>`), not `String`** — **PARTIALLY SUPERSEDED, not landed.** What landed is a *different* sink: `ShellOnHost.script` is now `RetainedShellScript`, a **record whose `body` is still a `String`** (`gunbc.host.host_effect_realize` `ResolvedHostEffectCell` (no longer present at this revision), `gunbc.retained_shell_script` `RetainedShellScript`). That closes the hand-assembled-`String`-at-the-edge hole by construction, but is **not** the typed-argv retype this bullet describes — the ruling block below says so explicitly: *"the typed-argv `List<String>` path of bullet 2 stays §5.A/§5.B's job — the two are distinct sinks, not one."* **Live residue.** *(Corrected `review 43522`: this previously read "held … **landed**", marking open §5.A/§5.B work as finished.)*
- ~~Activate the `host_language_transport_script` lens~~ — **COMPLETED (#7184), not refuted.** The lens *was* activated and is live, redding a raw literal at an enrolled `Run` position. Activation alone was never **sufficient**: it stays deliberately green on `ComputedApplication`, so it could not catch the #7064 computed-join fake — construction closed that class. Do not re-do it; do not read "insufficient" as "wrong".

> **RULING — wall-first, one PR (operator, 2026-07-24). STOP per-op migration.** The §5.A/§5.B per-op
> migration PRs are paused. Build §5.E's construction wall first, in ONE PR (large PRs fine); §5.B
> resumes on top of it as the *only* writable path. Recorded here so the sequencing is a durable fact,
> not re-litigated per conversation.
>
> **Design correction found by execution while building the wall (snappy-moth-330):** "brand
> `TransportScript`" (bullet 1 above) is **not** a `.dag`-level wall. `TransportScript = String where
> brand("TransportScript")` is a **transparent** brand — `expand_nominal_alias_rhs` peels it to its
> base, so a bare `String` (or a computed concat) flows into a `TransportScript` position with **no
> cast**, and `x as TransportScript` is always allowed. Branding alone cannot make a hand-join a type
> error. The genuine compile-wall is a **record**: a `String` cannot fill a record-typed field. So the
> wall is built as:
>
> - **`gunbc.retained_shell_script.RetainedShellScript`** — a RECORD `{ body: String; reason: NonEmptyStr;
>   dissolves_to: DeclarationRef }` — is the retype target of the host-effect realization SCRIPT edge
>   (`host_effect_realize.ShellOnHost.script`, and the `run_shell_transport`/`realize_shell_on_host`
>   seams). A hand-assembled `String` at that edge no longer typechecks. (This is the *retained-script*
>   path; the typed-argv `List<String>` path of bullet 2 stays §5.A/§5.B's job — the two are distinct
>   sinks, not one.)
> - **The free minter `extdeps.shell.exec.transport_script_from_body(body: String)` is DELETED.** The
>   foreign/runtime `shell.Exec.Run(script:)` sites route through counted bridges
>   (`retained_foreign` / `retained_runtime` / `retained_srvn`), each authoring a `reason` + a
>   `dissolves_to` ref — a new bridge call is a conspicuous, counted review event, with two dissolve
>   buckets (srvN-takeover typed effects vs. the `v2.workflow.bash_emit` foreign roster).
> - **The `host_language_transport_script` lens is activated** as a per-PR `ReadsLiveTree` witness
>   consumer over the `shell.Exec.Run` anchor sites (`wall_residue_live.dag`) — the backstop for a
>   *raw literal* at a Run position (the lens deliberately stays green on `ComputedApplication`, i.e. the
>   counted bridge calls; construction, not the lens, closes the computed-join class).
> - **REDs prove the wall by execution:** the two documented fakes (#7064's
>   `transport_script_from_body(argv_join(...) + " 2>/dev/null || true")` and the vivid-wolf string-into-edge
>   near-miss), reconstructed as inline sources through `compile_dag_rust_emit_check`, **fail to compile**
>   (`transport_script_wall_compile_red.dag`) — not fail review.
> - **Surviving residue (filed here, CLOSED LATER by #7962):** at the time of this ruling `x as TransportScript`
>   remained writable anywhere, and the block below judged it not closable by a record, needing the
>   **`Reference` verb**. **That prediction was wrong in a useful way.** #7962 closed it without any
>   visibility feature, by making `TransportScript` a `sole_constructor` record AND extending
>   `04_infer` `sole_constructor_construction_diags` to judge the cast form — the language grew the
>   check rather than the carrier growing a guard. Kept because the reasoning is only legible against
>   the original: the error was assuming the cast form was outside what construction could reach.

### Sequence

1. **§5.A** — add the ops (each cited, typed `exit`, typed-argv transport). Small, finite.
2. **§5.E** — **the construction wall LANDED #7184**, but this step is *not wholly* discharged and the original wording overstated it. What landed: the `RetainedShellScript` **record** edge + free-minter deletion + counted bridges + compile-fail REDs. The branding half was **refuted** (a transparent brand), and the sink it left open was closed separately by #7962's seal. The **typed-argv realization edge is still open** — `RetainedShellScript.body` is a `String`, not `List<String>`, so that path remains §5.A/§5.B work (see §5.E bullet 2). §5.B is already the only writable path for the *retained-script* sink. *(Corrected `review 43522`: previously struck through entirely and marked "LANDED", which retired open work.)*
3. **§5.B** — migrate by construction (call the op), green-by-execution + injection-RED, deleting each concat.
4. **§5.C** — route foreign-executor sites through the bash backend (bounded roster).
5. **§5.D** — un-defer per trigger.

### Receipts

- **Op inventory verified present @ `78f43c38`** (Pass 1 enumeration of every `service`/`operation` under `dag/extdeps/`): `systemd.Systemctl` (incl. `ShowProperty`/`SetProperty`/`IsActive`), `Clock.Now`, `shell.Which.Check`, `shell.Find.*` (incl. `IsExecutable`/`Dir`), `git.Core.*` (incl. `Show`/`FetchNoTags`), `Filesystem.{Write,Read,Delete,List}`, `apt.PackageManager.Install`, `sleep.Delay.Seconds`, `gunbc.WitnessBin.Run`, `cargo.Build.*`, `sha256sum`/`jq`/`sed`/`grep`/`xorriso`.
- **New ops verified ABSENT @ `78f43c38`**: no `hostname`/`hostnamectl` op, no `id`/`getent` op, no `systemctl list-units` op; `ssh.Session.ExecArgv` absent on main (in flight in C5 #6946).

---

## Dissolution trigger

**P4 has LANDED** (#6572/#6585/#6598 — the `ConvergePlan` effect + `EmitArtifactThenThinRun` transport minted and consumed by fleet_converge; see §2), so the original "delete when P4 lands" criterion is met.

**⚠ Both remaining criteria are ALSO now met — this trigger has fired and needs an operator decision (flagged 2026-07-26, `review 43494`).** The stated condition: dissolve when the `host_language_transport_script` lens **goes live** *and* `program.dag` **deletes**. Verified: the lens was activated by **#7184** (`v2.test.claim.host_language_transport_script.corpus.wall_residue_live`) and `program.dag` deleted by **#6831**. By its own terms this document should already have dissolved.

It has not, and the honest reason is that the trigger was **mis-specified rather than unmet**. It names two *mechanism* events, but what keeps this document alive is the **per-site punch-list** (§4.A–§4.J): the wall stops *new* concats **on the `ShellOnHost` edge** — not at the still-open `shell.Exec.Run` sink — and does not dissolve the ones already there, whose rows still have owners and open PRs. A correct trigger reads: *dissolves when the §4 punch-list empties* — mechanism landing was necessary, not sufficient. (The punch-list-empties condition is a census-conditioned prerequisite and is retired by the 2026-10-30 process ruling (`gunbc.design_ruling bad_process_is_worse_than_no_process`); this does not establish corpus-wide coverage, and rows are repaired by scoped review and focused tests.)

**No unilateral dissolution here.** Sessions are actively working rows in this document; deleting it mid-flight would strand them, and re-specifying a dissolution trigger is an operator call, not a census edit. Recorded for that decision.
