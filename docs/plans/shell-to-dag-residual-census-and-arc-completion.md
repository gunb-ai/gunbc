# Shell → DAG: residual census, thin-run scoping, and arc completion

> **Status: assessment + sign-ready scoping draft (witty-ibex-317, 2026-07-12).** Grounded against `origin/main` @ #6507. Builds on the landed slices (Retry #6467, If-band #6475, typed transports #6457/#6478) and lane D's authority [provisioning-window-executor-capability-design.md](provisioning-window-executor-capability-design.md). No `.dag` edits — records direction for operator + host-effect-lane sign-off. ⚠ FLAG = a decision needing sign-off.

## 0. The one finding

The remainder of the ShellProgram→DAG arc is **not** "migrate N files onto the v2 bash emitter." It splits cleanly:

- **Legitimate shell** (foreign executors + pre-runtime bootstrap) stays shell, emitted through the **v2 bash rows** (grammar-owned), and is *bounded* — a roster, not a growth surface.
- **Runtime-present shell** (fleet-converge steady-state, the srv3 install/reconcile tails, host-identity converge) **all funnels onto one keystone: `host_effect_apply` carrying typed effects.** Two sites already prove the pattern (`floor_diff_observe.dag`, `host_identity_converge.dag`); nearly every srv3 dissolution trigger already *names* `host_effect_apply` as its destination.

So srv1/srv2 subsumption and srv3 bringup are the **same** work seen twice: generalize `host_effect_apply` from `ShellCommand{script:String}` toward typed effects, and add the `EmitArtifactThenThinRun` transport arm. Everything else is consequence.

---

## 1. Residual-shell census (current tree)

Five categories. "Genuine emitter" = emits bash that actually runs; "oracle/scaffold" = `serialize_bash` retained only for a test.

### A. Foreign-executor shell — PERMANENT framing (category (a), stays shell)

| site | what | invoked by |
| --- | --- | --- |
| `ci_workflow.dag` | 8 GHA `RunStep.run:` bodies + concat-built floor-peak/cgroup runners (`ci_cgroup_peak_locate_shell`, `ci_floor_peak_{pre,post}_script`) | GitHub Actions (byte-consumes `run:`) |
| `local_tidy_spec.dag` | git pre-push hook via `serialize_bash` (`githooks_pre_push_emit_scaffold`) | git (foreign executor) |
| cron entry lines | (per lane D census) | cron |

These are correct as shell — the executor only understands shell text. Target end-state: emitted through v2 bash rows (the concat-built `ci_workflow` runners carry `…_shell_emit_dissolution_trigger` rows to the `emit(intent,Bash)` path), **not** dissolved to a typed transport. The floor-peak runners are the near-term win (they cite `floor_diff_observe.dag` as their pattern, but they're foreign-executor so they emit rather than `apply()`).

### B. Bootstrap / pre-runtime shell — provisioning window (category (b))

| site | what | genuine? |
| --- | --- | --- |
| `ubuntu_install_media_fetch.dag` | ISO fetch: `curl` mirror loop + `sha256sum -c` + `mv`; `serialize_bash`+`RawLine` | **dissolved** (FLAG B1 below) — in-process typed reconcile, no `ShellProgram`/`RawLine` emitted |
| `ubuntu_seeded_install_media_remaster.dag` | ISO remaster: `xorriso` extract/mkisofs, NoCloud `user-data`/`meta-data`, grub `sed`; heaviest raw shell | **dissolved** (FLAG B1 below) — in-process typed reconcile, no `ShellProgram`/`RawLine` emitted |
| `fleet_converge` fresh-standup arm | `git fetch`/`cargo build`/`cp` greenfield bring-up (3 frontier rows: `orch_construct_procedure`, `…_cmdsubst_assign`, `…_let_assign`) | genuine emitter |

These run before a gunbc runtime exists on the target, so shell is the honest lowest-dependency medium. They emit through the v2 bash rows (the If-band + the tier-2 Procedure/Let band, #6566; the arith arm had no live site and is being pruned by the consumer slice). **✅ FLAG B1 SIGNED (operator, 2026-07-14, relayed via calm-ferret-849):** the ISO remaster/fetch dissolve to a **typed target**, not a permanent `ShellPayloadRequired` bootstrap roster entry — these scripts execute on the provisioning host where the gunbc runtime IS present, so under the bash-minimization rule they're runtime-present, not bootstrap; `curl`/`xorriso`/`sed`/`sha256sum` become typed argv ops (`sha256sum` behind the `Sha256Verify` interface shape, `dag/extdeps/tools/sha256sum.dag`) and the NoCloud files become typed `Filesystem.Write`. Consequence: the for/heredoc emit band is never needed and the tier-2 band stays Procedure/Let-only. **Landed:** `ubuntu_install_media_fetch.dag` and `ubuntu_seeded_install_media_remaster.dag` no longer import `extdeps.languages.bash.program` / build `ShellProgram`/`RawLine` — both rewritten to in-process typed reconcile (`observe(path) → Observation`), green-by-execution.

### C. Runtime-present shell — MUST become typed via `host_effect_apply` (the real work)

| site | what | current path | dissolves to |
| --- | --- | --- | --- |
| `fleet_converge_emit.dag` | **dissolved** (slice 2 keystone LANDED: #6572 `ConvergePlan`+`EmitArtifactThenThinRun`+`converge_apply` · #6585 golden shrink 323→21 lines · #6598 `gunbc converge --host` CLI receiver) — the census-time ~275 lines / 12+ fn defs / 4 for-loops / while-read drain / arithmetic / verdict if-elif are gone; the emitter now projects only the fresh-standup bootstrap intent + thin invocation lines | committed `.github/fleet-converge.sh` (21-line thin-run golden), steady-state interpreted in-process | done — `EmitArtifactThenThinRun` (slice 2) — §2 |
| srv3 tails (9 files) | observe scripts, receipt echoes, token-extract, sleep glue; **2 heavy**: `srv3_install_diagnostic_checklist` (curl/redfish/sol probing, FROZEN/terminal), `srv3_os_install_actuator_toolchain_ensure` (apt/curl/websocat) | `shell_exec_via_bash` heredoc → `shell.Exec.Run` | typed observe/effect on `host_effect_apply` (their scaffolds already say so) |
| `host_identity_converge.dag` | 1–2 line `sudo hostnamectl set-hostname` as `ShellCommand{script}` | **already** `host_effect_apply_gated` | typed hostname effect (drop the shell string) |
| `floor_diff_observe.dag` | `git diff` observe as `ShellCommand{script}` | **already** `host_effect_apply` LocalShell | **this is the target pattern already realized** |
| `nbd_proxy_serve.dag` | `nbd_proxy_serve_program` body is `RawLine` (`&`/`trap`/`$!` backgrounding) | typed `ShellProgram` → `serialize_bash` | first-class typed background/trap statements or a typed long-running-process effect |

The srv3 dissolution triggers are unanimous: `srv3_install_diagnostic_checklist_shell_runner_dissolution_trigger` = *"DISSOLVES WHEN host_effect_apply binds Srv3InstallDiagnosticObserve without concat runner glue"*; the same shape repeats across the reconcile files. The common runner `gunbc.shell_bash_runner.shell_exec_via_bash` (the `bash <<'GUNBC_BASH_EOF'` heredoc framing) is itself a scaffold that *"dissolves when host_effect_apply … binds multiline shell transport without concat heredoc framing."*

### D. Oracle / scaffold retainers — NOT genuine emitters (mostly my Phase 3a work)

| site | state |
| --- | --- |
| `dag_compile_clean_transport.dag` | **fully migrated** — `serialize_bash` not even imported; scaffolds dissolved ✓ |
| `emit_determinism_transport.dag` | **fully migrated** — same; pairing now via `duplicate_computation` lens ✓ |
| `host_prelude.dag` | run path typed (`WitnessBin.Run`/`cargo.Build`); residual typed `ShellStmt` builders are scaffold-only for the emit_determinism lane ✓ |
| `build_step_transport.dag` | **dissolved (#6565, 2026-07-14)** — corruption-probe harness reshaped onto typed `shell.Mktemp.Dir`/`Filesystem.Write`/`gunbc.WitnessBin.Run`/`shell.Remove.RecursiveForce`; no `serialize_bash`/`verifications_script` import remains (see `bst_typed_transport_doc` in the file). |
| `build_step.dag` | typed `ShellStmt` AST library (no `RawLine`); consumed by the above |

### E. Replacement machinery + detectors — not dissolution targets

`bash_program_emit.dag` + `bash_command_fold.dag` (the v2 compositional emitter that *is* the replacement); the lenses `realization_vocabulary_containment` / `medium_structure_containment` / `duplicate_computation` (detectors); `program.dag` (the sidecar — the deletion target).

**Ratchet:** `bash_program_importer_count_baseline = 19`. The genuine live emitters that must clear before `program.dag` deletes: the two ubuntu-media files, `nbd_proxy_serve`, `floor_diff_observe`, `build_step`(+transport), `bash_command_fold`(replacement). Categories D-fully-migrated and the detectors don't emit; they can drop their import or are the replacement itself.

---

## 2. `EmitArtifactThenThinRun` scoping — srv1/srv2 subsumption keystone

**Goal — ACHIEVED (2026-07-20 receipts; roadmap `6-shell-slice2` sign-off pending):** move the fleet-converge srv1/srv2 steady-state (category C, ~275 lines at census time) *out of emitted bash and into the gunbc binary*, which interprets the typed converge policy in-process and emits typed receipts. The emitted artifact shrinks to the fresh-standup bootstrap fragment + a thin `gunbc converge` invocation line. **Landed:** G1+G2 minted in #6572, G3 consumed in #6585, CLI receiver in #6598; witnesses (`fleet_converge_emit_*`, `fleet_converge_apply_holds`) green by execution 2026-07-20.

### What already exists (the pieces)

1. **The typed plan** — `gunbc.host_converge.HostConverge` / `ConvergePolicy` / `ConvergeKnob` (`SliceProperty|PerSlotMemoryCap|RunnerWidth|JobserverTokens|VerifyOnlyCap|GunbcPinnedTree`) is a pure typed value with `ConvergeApplyMode = FreshStandup | ExistingHostQuiescentReload`, importing typed `extdeps.os.systemd`/`oomd` property names. Zero shell. **This is the plan the binary interprets.**
2. **The apply interface** — `host_effect_apply(target, effect, evidence, transport) -> Reconciliation<HostEffectIntent, HostEffectEvidence>` is minted (`host_effect_realize.dag`), with `Drive = OneShot | ConvergeLoop` and a redrive directive. Four consumers exist; **`host_identity_converge` is the exact converge precedent** (gated apply → typed effect → read-back → noop).
3. **The receipt** — the fleet-converge receipt grammar (`gunbc_host_converge_receipt_grammar_marker`) is byte-locked; the design says generalize it as the single `apply()` outcome, not fork it.

### The gap (three coupled pieces)

- **G1 — a typed converge effect.** `HostEffect` today = `ShellCommand{script:String}` | `RedfishAction`. The `script:String` is the anemic leaf (shell-emission-model §1: "same anemic leaf as `host_effect.ShellCommand.script`"). Subsumption needs the effect to carry the *typed converge intent*, not bash.
  - **⚠ FLAG 2a — effect shape.** Option (i): add `HostEffect::ConvergePlan{ policy: HostConverge, host: ComputeHost }`, resolved by `resolve_host_effect_cell` to a new `ConvergeOnHost` cell, realized by an in-process interpreter that drives the typed systemd/oomd property reads+writes (the same knobs `host_converge.dag` already names). Option (ii): keep `ShellCommand` for one-shots and make `EmitArtifactThenThinRun` a transport that reinterprets — rejected, because the script is already bash (no typed plan to interpret). **Recommend (i)** — it's the anemic-leaf dissolution the design already calls for, and it makes the interpreter total over the knob coproduct (each `ConvergeKnob` arm → a typed `extdeps.os.systemd` set/read). **✓ SIGNED option (i)** (operator, 2026-07-14 — dispatch order via calm-ferret-849).
- **G2 — the `EmitArtifactThenThinRun` transport arm.** `HostEffectTransport` today = `LocalShell` | `SshShell`. Add `EmitArtifactThenThinRun { bootstrap: BootstrapFragment, invocation: ThinInvocation }`.
  - **Semantics:** for a `SteadyState` host it degenerates to in-process interpretation (`gunbc converge --host X` reads the typed policy and reconciles via typed systemd effects — no emitted script). For a `FreshStandup` host it *first* emits the bootstrap fragment (the 3 frontier rows: git-checkout / cargo-build / cp — legitimately bootstrap bash per category B) and *then* the thin invocation. This is exactly the `ConvergeApplyMode` split `host_converge.dag` already models.
  - **⚠ FLAG 2b — thin-run guardrail (load-bearing).** The binary must interpret the *typed* `HostConverge`, emitting typed receipts. Replacing 275 lines of bash with 275 lines of imperative Rust in the seed is refused (shell-emission-model §6 / host-effect §Phase-D). The interpreter folds the `ConvergeKnob` coproduct; there is no free-form imperative arm. **✓ SIGNED as the merge bar** (operator, 2026-07-14): total fold over `ConvergeKnob` with a fail-closed, counted `Unimplemented`-per-knob frontier; receipts on the byte-locked receipt grammar generalized as the single `apply()` outcome.
- **G3 — fleet_converge consumes `apply()`.** Point the fleet-converge lane at `converge_apply` (the `ConvergePlan` effect over `EmitArtifactThenThinRun`) instead of `project_fleet_converge_to_doc`. The committed `.github/fleet-converge.sh` **stays byte-locked as the oracle** during the transition (the emitted bootstrap fragment must still match its fresh-standup arm byte-for-byte); the steady-state functions are what move in-process.

### Sequence (each gated by the byte oracle)

1. **Mint G1+G2 together** — **LANDED (#6572)**: `ConvergePlan` effect + `ConvergeOnHost` cell + `EmitArtifactThenThinRun` transport + the in-process knob interpreter, with a fail-closed `Unimplemented`-per-knob frontier so unmodeled knobs refuse (not fabricate).
2. **Consume from fleet_converge** (G3) — **LANDED (#6585, CLI receiver #6598)**: `converge_apply` for the steady-state arms; the Doc projection emits *only* the fresh-standup bootstrap + the thin invocations; the committed `.github/fleet-converge.sh` golden shrank 323→21 lines with the drift gate green.
3. **Retire** `fleet_converge_steady_state_doc_projection_dissolution_trigger` and the ctrl reconciler's script fan-out (host-effect Phase D's first net ctrl deletion — `runner_host_reconcile.mjs` hash/fan-out). *(Still open: the trigger remains a Scaffold binding `fleet_converge_thin_invocation` — it dissolves when the thin invocation becomes typed argv dispatch rather than a shell line.)*

**⚠ FLAG 2c — ownership.** `host_effect.dag` is a DESIGN-named seam co-owned by smart-newt-512 (dag-managed-infra) / neat-boar-71 (BMC), and its note requires **operator + bright-stag-194 sign-off before any transport arm is minted**. This scoping is the sign-ready shape; the mint is that lane's, not a solo edit from here. **✓ DISCHARGED** (operator, 2026-07-14): operator dispatch order via calm-ferret-849 supplies the sign; sessions bright-stag-194 / smart-newt-512 / neat-boar-71 verified archived at sign time (no live co-owner lane to co-sign or conflict with), so the mint proceeds under a dedicated worker owned by calm-ferret-849.

---

## 3. The remainder of the ShellProgram→DAG arc (to `program.dag` deletion)

Two tracks run in parallel; deletion is the join.

### Track 1 — legitimate shell onto the v2 bash rows (bounded)

- **P1 (ready now, no sign-off):** the `ci_workflow` concat-built floor-peak/cgroup runners → `emit(intent, Bash)` via the landed If-band + word support (they carry `…_shell_emit_dissolution_trigger` rows). Foreign-executor, so they *emit*; no `apply()`.
- **P2:** the Procedure/Let emit band for the 3 fresh-standup frontier rows — the one genuinely new *emitter* vocabulary the arc still needs, scoped by the bootstrap census, fail-closed where a construct isn't modeled. *(Tier-2 band LANDED #6566; the consumer slice — route the fresh-standup fragment through `emit(intent,Bash)`, byte-oracle vs `.github/fleet-converge.sh` — LANDED #6573. The former "ubuntu-media files need `for`/heredoc" clause is superseded by the FLAG B1 working default above: ubuntu-media dissolves to typed argv/`Filesystem.Write`, so the for/heredoc band is never built.)*
- **P3:** `local_tidy_spec` pre-push hook + cron lines stay `serialize_bash`/foreign-executor **permanently** (roster entries, category (a)). These never dissolve — they're the honest residue.

### Track 2 — runtime-present shell onto `host_effect_apply` (the §2 keystone cascade)

- **P4 = the G1+G2+G3 keystone above** (srv1/srv2 subsumption).
- **P5:** the srv3 tails follow the *same* interface. Each `srv3_*_observe_script` becomes a typed `…Observe` effect on `host_effect_apply` (their dissolution triggers already name this); the receipt echoes become typed receipts; `shell_exec_via_bash` (the heredoc runner) dissolves once no caller passes a raw script. The two heavy files: `srv3_install_diagnostic_checklist` is FROZEN/terminal (typed observe effect retires it), `srv3_os_install_actuator_toolchain_ensure` → typed `extdeps.apt`/`curl` argv effects.
- **P6:** `host_identity_converge` drops its `sudo hostnamectl` script for a typed hostname effect (it's already on `apply_gated`); `nbd_proxy_serve_program`'s `RawLine` body gets first-class typed background/trap statements or a typed long-running-process effect.
- **P7:** `build_step_transport` — **LANDED (#6565, 2026-07-14)**: corruption-probe harness reshaped onto typed `shell.Mktemp`/`Filesystem.Write`/`WitnessBin.Run`, dropping the `verifications_script` `serialize_bash` execution.

### The join — delete `program.dag`

When Track 1's emitters route through the v2 bash rows and Track 2's runtime-present shell is on `apply()` with typed effects, the `bash_program_importer_count` reaches the permanent-residue floor (the v2 replacement emitter + the foreign-executor roster). At that point `bash.program`/`serialize_bash` has no runtime importer, the ratchet's baseline hits the floor, and `program.dag` + `serialize_bash` delete — the arc's terminal step. The v2 bidirectional bash language is the single bash authority.

**Critical-path summary:** everything non-foreign converges on **P4 (the `host_effect_apply` typed-effect + `EmitArtifactThenThinRun` mint)** — **P4 is LANDED** (#6572/#6585/#6598; FLAGs 2a(i)/2b/2c signed/discharged 2026-07-14, see §2). The critical path is now P5/P6 (mechanical on the landed interface) and the operator sign of roadmap `6-shell-slice2`; P1/P2 (emitter side) run independently. **Dispatch note (2026-07-20):** do not re-dispatch workers onto P4/slice 2 from the old ~275-line framing — that staleness produced two misdispatches onto finished work.

## 4. Exhaustive instance census @ `78f43c38` — the tracked punch-list (calm-ferret-849, 2026-07-22)

§1–§3 record *direction* at #6507; this section is the **complete, current, per-instance list** so every single shell-string-construction site is tracked to closure. Grounded against `origin/main` @ `78f43c38`.

**Why now:** a "migration" wave (#7004, #7006, and the closed srv* cluster) counted **relocations** as progress — it moved `ShellCommand{script: concat(...)}` out of the intent file into a new `*_script.dag` file and realized a typed variant by *calling that concat and stuffing the raw string back into `ShellOnHost{script}`*. Net raw-shell-string construction: unchanged; new §3 coproduct-nickname debt added. This census exists so no site is counted done until the concat is **gone**, not homed elsewhere.

> **Anchoring (addresses review 41399 / DESIGN §6):** rows below are keyed on **file + symbol name** (variant, fn), NOT line numbers — a line-numbered prose ledger drifts from the tree within a commit (it did: the first draft was mis-grounded on a divergent worktree that still had the pre-#7006 `ShellCommand{script}` sites; on `origin/main` those are already the nickname variants). Symbol anchors are stable across line moves. This section is grounded at `origin/main` @ `78f43c38`; it is an interim tracker that dissolves into the enforcement lens (§4.F), which is the drift-proof authority.

### 4.0 Completeness method (proves this is exhaustive, not sampled)

Every way a shell string is constructed or carried, grep-complete over `dag/**` (excluding `*_test.dag`):

| # | pattern | what it finds |
| --- | --- | --- |
| P1 | `ShellCommand *{ *script:` | `HostEffect.ShellCommand{script}` construction — **0 live sites on main** (all became nickname variants); residue = the `host_effect_plan.dag` `{script: ""}` placeholder + the `host_effect.dag` type def |
| P1b | `effect: <Variant>` for the shell-backed `HostEffect` variants | the nickname-variant construction sites that *replaced* P1 (the real §4.A rows) |
| P2 | `BootstrapFragment *{` | bootstrap-script carrier (0 live construction sites today) |
| P3 | `command:` in `Run{}` / `Do{run:}` | `std.orchestration.Run.command` string |
| P4 | `shell.Exec.Run` / `.Check` / `shell_exec_via_bash` | meta-exec bottom-transport calls |
| P5 | `fn … -> String` builders — in `*_script.dag` **and inline** (e.g. `fleet_show_effective_read.dag`'s `fleet_runner_unit_property_read_script`/`fleet_runner_width_count_read_script`, `host_converge_slice1.dag`'s `…_memory_max_read_script`/`…_memory_max_set_script`/`…_enumerate_units_script`) — plus any `concat("systemctl …"/"…")` inline | the concat/relocation script builders, wherever they live (not only the `*_script.dag` glob) |
| P6 | `transport_script_from_body` | the (porous) `TransportScript` brand boundary — 26 sites |
| P7 | `serialize_bash`/`ShellProgram`/`RawLine`/`ShellStmt` | bash-AST emit vocab (emit-internal) |
| P8 | `ssh.Session.Exec(` (non-`ExecArgv`) | ssh command-string transport |

The classes below partition every hit. **Class letters = the ACTION**, not the file.

### 4.A — RELOCATION REGRESSION + systemctl-read cluster (dissolve properly)

Keyed on the **construction site (variant/fn name)** and the modeled op it *should* call. Done = the concat gone (deleted, not moved) and, where the variant is a nickname, the variant gone too.

| construction site (file · variant/fn) | builder(s) — `*_script.dag` and/or inline | should call | sub-class |
| --- | --- | --- | --- |
| `fleet_show_effective_read.dag` · `SystemdUnitMemoryPropertiesRead`, `SystemdUnitMemoryMaxRead` | `fleet_show_effective_read_script.dag` relocated fns **+ inline** `fleet_runner_unit_property_read_script`, `fleet_runner_width_count_read_script` (`systemctl show`/`list-units`) | `extdeps.os.systemctl.ShowProperty` (exists) + a `list-units` op | **A1** modeled |
| `host_converge_slice1.dag` (via `shell.Exec.Run`) | inline `…_memory_max_read_script`, `…_memory_max_set_script` (`systemctl set-property` concat), `…_enumerate_units_script` (`systemctl list-units` concat) | `systemctl.ShowProperty` / `SetProperty` / new `list-units` op | **A1** modeled |
| `host_identity_observation.dag` · `HostIdentityShortHostnameRead` | `host_identity_observation_script.dag` · `host_identity_short_hostname_script` | **new** `extdeps.os.hostname` read | **A2** model-first |
| `host_effect.dag` · `SetHostnameCas` (realized via `host_effect_hostname_script.dag`) | `host_effect_set_hostname_cas_script` | **new** hostnamectl set op | **A2** model-first |
| `host_effect.dag` · `ReadEffectivePosixPrincipal`, `SudoNopasswdExecuteProbe` | `host_effect_deploy_access_probe_script.dag` (whoami / `sudo -n` / `sudo -n -l`) | `access.PosixEffectivePrincipal`, `sudo.NopasswdExecuteProbe` (exist) | **A3** OWNED BY C5 #6946 — do not touch |
| `live_deploy/` effect variants | `live_deploy/host_effect_script.dag` (`…apply`/`…retract`/`…ensure_dependency`/`…digest_readback`) | decompose (multi-op) | **A4** decompose |
| `host_effect_realize.dag` · `ProvisionBuildCache` | `host_build_cache_provision_script.dag` (6 `build_cache_*_body`) | decompose | **A5** srv* deprioritized |
| hygiene reaper/liveness | `host_hygiene_reaper_script.dag` (4 `…_body`), `host_hygiene_liveness_script.dag` | decompose | **A5** srv* deprioritized |
| `srv3_host_effect_apply.dag` · `Srv3*` variants, `srv3_install_diagnostic_checklist.dag` · `Srv3InstallDiagnosticObserve`, `nbd_proxy_virtual_media_install.dag` · `Srv3NbdProxyServe` | `srv3_host_effect_script.dag` (5 fns), `srv3_install_diagnostic_observe_script.dag` (5 fns) | typed observe/receipt effects | **A5** srv* deprioritized |

### 4.B — DIRECT `ShellCommand{script}` — RESIDUE ONLY (0 live construction sites on main)

The relocation wave left **no direct `ShellCommand{script}` construction sites** on `origin/main` — every one became a nickname variant (tracked in 4.A). What remains:

| site (file · symbol) | what | action |
| --- | --- | --- |
| `host_effect_plan.dag` · `ShellCommand { script: "" }` | empty placeholder | delete with the type |
| `host_effect.dag` · `ShellCommand { script: String }` | the **type variant** itself | delete at arc close (DESIGN §5, escalated) — terminal |

*(The first draft's `fleet_show_effective_read.dag:147,295` and `host_identity_observation.dag:66` rows were mis-grounded on a divergent worktree — corrected to their real nickname-variant construction sites in 4.A per review 41399.)*

### 4.C — RUNTIME-PRESENT `shell.Exec.Run` with a string/`_script` body (dissolve to typed op)

| site (file · fn) | body | should call | class |
| --- | --- | --- | --- |
| `host_converge_slice1.dag` · `…_memory_max_read/set/enumerate_units_script` | `systemctl show`/`set-property`/`list-units` | `systemctl.ShowProperty`/`SetProperty` + new `list-units` op | A1 (same cluster as 4.A) |
| `host_converge_slice1.dag`, `host_identity_adopt.dag` | `"date -Iseconds"` | typed clock/date op (DFS `extdeps` time) | A2 |
| `ci_deploy_target_host.dag` | `"hostname -s 2>/dev/null \|\| hostname"` | new `extdeps.os.hostname` (the `\|\| hostname` fallback becomes modeled, §5) | A2 |
| `host_identity_assimilation.dag`, `host_identity_adopt.dag`, `srv3_install_diagnostic_checklist.dag` | `"echo <receipt>"` | typed receipt emit (a stdout write, not shell) | A4 |
| `host_effect_realize.dag` · `shell_exec_via_bash` dispatch | realization-core script dispatch | the `LocalShell`/`SshShell` typed-argv edge (C5) | A4 realization core — confirm before edit |
| `dag/tools/{host_prelude,gunbc_ci,emit_host_gate,merge_admission_stamp}`, `gunbc/tools/review.dag` | witness/CI transports invoked from `claim_executor` | typed `WitnessBin.Run`/argv (host_prelude precedent) | A4 |

### 4.D — `ssh.Session.Exec` command-string (vs typed `ExecArgv`)

All in `host_effect_realize.dag`, via `ssh_session_exec(command:)` / `ssh_session_exec_script(script:)`:

| construction | verb | should call | class |
| --- | --- | --- | --- |
| `ssh_session_exec(command: cmd)` | arbitrary command | `ssh.Session.ExecArgv` (C5) | A1 |
| `ssh_session_exec(command: concat("test -x ", path))` | file-exists probe | `ssh.Session.ExecArgv` | A1 |
| `ssh_session_exec(command: concat("command -v ", tool))` (×2) | tool-presence probe | `ssh.Session.ExecArgv` | A1 |
| `ssh_session_exec(command: "id -u")`, `"id -g"` | uid/gid read | `ssh.Session.ExecArgv` (an `id` op) | A1 |
| `ssh_session_exec_script(script:)` | script-over-ssh | typed-argv splice or decompose | A4 |
| `extdeps.diagnostic.ssh.dag` · `ssh.Session.Exec(command:)` | the transport itself | keep as the ONE command-string transport, or fold into `ExecArgv` | transport-decision — escalate |

### 4.E — FOREIGN-EXECUTOR / BOOTSTRAP emit (LEGIT shell — route through `emit(intent,Bash)`, stays shell but bounded)

These are correct as shell (a GHA runner / cron / git / pre-runtime host only understands shell text). Target = emitted through the v2 bash rows, a **roster not a growth surface** — NOT dissolved to a typed op.

| site | executor | class |
| --- | --- | --- |
| `ci_spec.dag` `ci_floor_build_verify/ci_release_bins_pack/…unpack_verify/ci_regen_floor_skip_shortcut` + retry `command:` | GitHub Actions | E-emit |
| `ci_materialization.dag:217,246` gate scripts | GitHub Actions | E-emit |
| `merge_admission_produce.dag:192,199,261,320` stamp/gate scripts | GitHub Actions | E-emit |
| `fleet_converge_emit.dag` `fresh_standup_bootstrap_script` + the fresh-standup `Run{command: concat(...)}` rows (`:112,120,133,136,140,147,153,161,171`) | pre-runtime bootstrap | E-emit (byte-oracle vs `.github/fleet-converge.sh`) |
| `roadmap_static_site.dag:76,80,144,209` body fns | srv1 dashboard (belt B — being replaced by `gunbc serve`) | E-emit / dissolves with belt B |
| `fleet_posix_accounts.dag` `probe_command: "id <user>"` (×4) | account-existence probe | A2 (an `id` argv op) or E if roster — DFS |
| `assimilate/bmc_token_federation.dag:65` `gcp_token_smoke_script` | GCP token smoke | DFS — likely A (typed gcp op exists) |

### 4.F — bottom transport & brand (Phase-3 WALL — the construction guard)

| surface | status | action |
| --- | --- | --- |
| `transport_script_from_body(body: String)` — 26 sites | the **porous** `TransportScript` brand: `shell.Exec.Run` already takes `TransportScript` (`extdeps/shell/exec.dag:53`) but the constructor accepts any `String` | Phase-3: brand `TransportScript` so it is produced ONLY by `emit(intent,Bash)`/`serialize_bash` — a hand-concat becomes a type error (§5 construction wall) |
| `shell_exec_via_bash` (`shell_bash_runner.dag:32`) | heredoc runner scaffold | dissolves when no caller passes a raw script |
| `host_language_transport_script` lens | **inert** (`fail_closed_lockdown.dag`) — no gate reds a bare-string `shell.Exec.Run` | activate once the brand lands |
| meta-exec module `extdeps.shell.exec` | not walled | module-isolate / symbol-visibility confinement (meta-exec-confinement lane) |

### 4.G — bash-AST emit vocab (emit-internal — NOT fraud, already confined)

`serialize_bash`/`ShellProgram`/`RawLine`/`ShellStmt`/`bash_command_fold` in 11 files (the v2 emitter itself + the two ubuntu-media files + `nbd_proxy_serve` + `build_step`(+transport) + design/roadmap prose). Confined by `realization_vocabulary_containment` (LANDED #6854). No dissolution action — this is the replacement machinery. Tracked only so it isn't mistaken for a construction site.

### 4.H — oracle / test retainers (NOT live construction — skip)

`live_deploy/emit.dag:448,452` `expected_*_script` (drift-gate oracles), `*_test.dag` fixtures. These are test expectations, not runtime construction; they follow their subject's dissolution.

### Scope in flight (2026-07-22)

- **sleek-crab-621** (dispatched): 4.A1/4.A2 in-mission rows — `fleet_show_effective_read` (`ShowProperty` + `SystemdUnitProperty` enum), `host_converge_slice1` systemctl reads/sets, the new `extdeps.os.hostname` model for the hostname rows, and the `ssh.Session.ExecArgv` conversions in `host_effect_realize`. Staged, exemplar-first, classify-before-edit.
- **keen-deer-531 / C5 #6946**: owns 4.A3 (access probes) — blocked on a `cli_run.rs` conflict.
- **DEFERRED** (operator set aside): all 4.A5 srv* rows, and 4.A4 multi-op decompositions pending a routing call (`authorize_shell_emission`).
- **Phase-3 wall (4.F)**: the durable fix — until it lands, this census is the tracking authority; after, the lens reds any new site by construction.

### Dissolution trigger for §4

This punch-list folds into the **`host_language_transport_script` lens going live** (4.F): once a compile gate reds any raw-string `shell.Exec.Run` / hand-built transport, new instances are unwritable by construction (§5) and a prose punch-list is redundant. Until then, every row here is discharged by *deletion of the concat*, verified green-by-execution + an injection-RED — never by relocation.

---

## 5. Method of Action — the bounded path to bash-free user space (calm-ferret-849, 2026-07-22)

**End state:** no user-space `.dag` constructs a shell string. Every "this `.dag` wants to call a bash script" instance resolves to exactly one of four paths below. The `realization_vocabulary_containment` lens (#6854, LIVE) already forbids bash-AST vocab (`ShellProgram`/`ShellStmt`/`serialize_bash`) in user space — the remaining hole is the `shell.Exec.Run(script: TransportScript)` / `ShellOnHost{script: String}` sink (§5.E), which is what makes relocation possible.

**The headline (verified by op inventory @ `78f43c38`):** the arc needs only **~4 new typed ops**. Almost every site calls an op that already exists.

### 5.A — the FINITE new-op list (the ONLY new modeling the whole arc needs)

| new op | home file | covers verb | consumers |
| --- | --- | --- | --- |
| `extdeps.os.hostname` · `Read` + `Set` | **new** `dag/extdeps/os/hostname.dag` | `hostname -s`, `hostnamectl set-hostname` | `host_identity_observation` (`HostIdentityShortHostnameRead`), `ci_deploy_target_host`, `host_effect_hostname` (`SetHostnameCas`) |
| `systemd.Systemctl.ListUnits` | **add to** `dag/extdeps/os/systemctl.dag` | `systemctl list-units --state=active` | `host_converge_slice1` (`_enumerate_units_script`), `fleet_show_effective_read` (`fleet_runner_width_count_read_script`) |
| `extdeps.os.id` · `Read` (uid/gid/user) | **new** `dag/extdeps/os/id.dag` | `id -u`, `id -g`, `id <user>` | `host_effect_realize` (ssh probes), `fleet_posix_accounts` (`probe_command`) |
| `ssh.Session.ExecArgv` | **add to** `dag/extdeps/diagnostic/ssh.dag` | typed argv over ssh (`ssh host -- argv`) | `host_effect_realize` ssh probes — **IN FLIGHT, C5 #6946** |

That is the entire new-modeling surface. (A fifth, optional: a typed stdout/receipt emit for the two `echo <receipt>` sites, or reuse `Filesystem.Write`.)

### 5.B — CALL AN EXISTING OP (receipt: the op is already modeled on main)

| site (file · symbol) | current shell | call this op — receipt (file · service.Op) |
| --- | --- | --- |
| `host_converge_slice1` · `_memory_max_read_script` | `systemctl show --property=MemoryMax --value` | `systemd.Systemctl.ShowProperty` · `extdeps/os/systemctl.dag` |
| `host_converge_slice1` · `_memory_max_set_script` | `systemctl set-property … MemoryMax=` | `systemd.Systemctl.SetProperty` · `extdeps/os/systemctl.dag` |
| `host_converge_slice1` · `date`, `host_identity_adopt` · `date` | `date -Iseconds` | `Clock.Now` · `extdeps/clock/clock.dag` |
| `fleet_show_effective_read` · `SystemdUnitMemory*Read`, `fleet_runner_unit_property_read_script` | `systemctl show --property` | `systemd.Systemctl.ShowProperty` · `extdeps/os/systemctl.dag` |
| `host_effect_realize` · ssh `test -x <path>` | `test -x` | `shell.Find.IsExecutable` · `extdeps/shell/shell.dag` (spliced via `ssh.Session.ExecArgv`) |
| `host_effect_realize` · ssh `command -v <tool>` (×2) | `command -v` | `shell.Which.Check` · `extdeps/shell/*` (via `ExecArgv`) |
| `tools/review` · `design`, `algebra_ref` | `git fetch` + `git show origin:FILE` | `git.Core.FetchNoTags` + `git.Core.Show` · `extdeps/git/git.dag` (`|| echo '(not found)'` → typed error→Absent) |
| `merge_admission_stamp` · `mkdir -p` | `mkdir -p` | `Filesystem` / `shell.Find.Dir` · `extdeps/filesystem/filesystem_io.dag` |
| `dag/tools` · `host_prelude`/`gunbc_ci`/`emit_host_gate` witness+build transports | witness/build run | `gunbc.WitnessBin.Run` · `extdeps/gunbc/gunbc.dag`; `cargo.Build.*` · `extdeps/rust/cargo_build.dag` (`host_prelude` already has the typed precedent) |
| `host_identity_assimilation`/`adopt` · `echo <receipt>` | `echo <msg>` | typed receipt/stdout emit (`Filesystem.Write` or a print op) — not a shell need |

Each row's `2>/dev/null || true` / `|| echo` fallback becomes a **modeled outcome** (`nonzero => …` mapped to `Absent`), never re-appended (§5 absorbing-fallback rule).

### 5.C — EMIT via the bash backend (foreign executors ONLY — bounded roster, legitimately stays shell)

Bash-as-target lives in one isolated backend: `src/v2/extdeps/languages/bash*` + `src/v2/workflow/bash*` (confined by the containment lens). These sites emit *through* it because the executor's input contract IS shell text:

| site (file · symbol) | executor | path |
| --- | --- | --- |
| `ci_spec` · `ci_floor_build_verify_script`/`ci_release_bins_pack_script`/`…unpack_verify_script`/`ci_regen_floor_skip_shortcut_script` | GitHub Actions `run:` | `emit(intent, Bash)` via the bash backend |
| `ci_materialization` · `ci_floor_materialization_receipt_gate_script`/`…resolve_receipt_gate_script` | GitHub Actions | same |
| `merge_admission_produce` · 4 `ci_*_script` | GitHub Actions | same |
| `fleet_converge_emit` · `fresh_standup_bootstrap_script`/`_arm_golden` | pre-runtime bootstrap | emit, byte-oracle vs `.github/fleet-converge.sh` (largely done #6572/#6585) |
| cron entry lines, `local_tidy_spec` pre-push hook | cron / git | emit, **permanent** roster (the honest residue) |

`roadmap_static_site` · `roadmap_site_*_body` is HTML/JSON content emit (not shell) for the srv1 dashboard — dissolves with belt B (`gunbc serve`), tracked there, not here.

### 5.D — DEFERRED (each with its own trigger)

| bucket | sites | trigger to un-defer |
| --- | --- | --- |
| C5 access probes | `host_effect_deploy_access_probe_script` | C5 #6946 merges |
| srv* cluster | `srv3_host_effect_script`, `srv3_install_diagnostic_observe_script`, `host_build_cache_provision_script`, `host_hygiene_*` | operator un-defers srv* |
| nbd backgrounding | `host_effect_nbd_proxy_serve` `RawLine` (`&`/trap/`$!`) | typed systemd transient-unit effect + `Filesystem.Read` token + typed argv (dissolution trigger already in-file; operator ruled no trap/&/$! vocab) |

### 5.E — THE ENABLER THAT MUST COME FIRST (close the string sink, or every row above can be faked)

Every §5.A/§5.B row can be **faked by joining argv back into a string** and feeding `shell.Exec.Run(script)` / `ShellOnHost{script}` — sleek-crab #7064 did exactly this (`argv_join(...) + " 2>/dev/null || true"`). As long as that sink is reachable from intent, relocation is the path of least resistance and a brief alone won't stop it. So the wall is **not cleanup-after** — it's the enabler:

- Brand `TransportScript` so it is produced ONLY by `emit(intent, Bash)`/`serialize_bash` (today `transport_script_from_body(body: String)` accepts any string — the porous boundary).
- Make `ShellOnHost{script}` / the runtime-present realization edge take **typed argv (`List<String>`), not `String`** — a hand-join becomes a type error.
- Activate the `host_language_transport_script` lens (inert today, `fail_closed_lockdown.dag`).

### Sequence

1. **§5.A** — add the ~4 ops (each cited, typed `exit`, typed-argv transport). Small, finite.
2. **§5.E** — brand `TransportScript` + typed-argv realization edge → §5.B becomes the *only* writable path.
3. **§5.B** — migrate by construction (call the op), green-by-execution + injection-RED, deleting each concat.
4. **§5.C** — route foreign-executor sites through the bash backend (bounded roster).
5. **§5.D** — un-defer per trigger.

### Receipts

- **Op inventory verified present @ `78f43c38`** (Pass 1 enumeration of every `service`/`operation` under `dag/extdeps/`): `systemd.Systemctl` (8 ops incl. `ShowProperty`/`SetProperty`/`IsActive`), `Clock.Now`, `shell.Which.Check`, `shell.Find.*` (incl. `IsExecutable`/`Dir`), `git.Core.*` (incl. `Show`/`FetchNoTags`), `Filesystem.{Write,Read,Delete,List}`, `apt.PackageManager.Install`, `sleep.Delay.Seconds`, `gunbc.WitnessBin.Run`, `cargo.Build.*`, `sha256sum`/`jq`/`sed`/`grep`/`xorriso`.
- **New ops verified ABSENT @ `78f43c38`**: no `hostname`/`hostnamectl` op, no `id`/`getent` op, no `systemctl list-units` op; `ssh.Session.ExecArgv` absent on main (in flight in C5 #6946).

---

## Dissolution trigger

Delete this doc when P4 lands (the `ConvergePlan` effect + `EmitArtifactThenThinRun` transport minted and consumed by fleet_converge), the census rows fold into `provisioning-window-executor-capability-design.md`'s table, and `program.dag` deletes — at which point the arc is complete and this scoping is redundant.
