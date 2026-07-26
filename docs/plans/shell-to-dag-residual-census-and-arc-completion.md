# Shell → DAG: residual census, thin-run scoping, and arc completion

> **Status: assessment + sign-ready scoping draft (witty-ibex-317, 2026-07-12).** Grounded against `origin/main` @ #6507. Builds on the landed slices (Retry #6467, If-band #6475, typed transports #6457/#6478) and lane D's authority [provisioning-window-executor-capability-design.md](provisioning-window-executor-capability-design.md). No `.dag` edits — records direction for operator + host-effect-lane sign-off. ⚠ FLAG = a decision needing sign-off.

## 0. The one finding

The remainder of the ShellProgram→DAG arc is **not** "migrate N files onto the v2 bash emitter." It splits cleanly:

- **Legitimate shell** (foreign executors + pre-runtime bootstrap) stays shell, emitted through the **v2 bash rows** (grammar-owned), and is *bounded* — a roster, not a growth surface.
- **Runtime-present shell** (fleet-converge steady-state, the srv3 install/reconcile tails, host-identity converge) **all funnels onto one keystone: `host_effect_apply` carrying typed effects.** Two sites already prove the pattern (`floor_diff_observe.dag`, `host_identity_converge.dag`); nearly every srv3 dissolution trigger already *names* `host_effect_apply` as its destination.

So srv1/srv2 subsumption and srv3 bringup are the **same** work seen twice: generalize `host_effect_apply` from `ShellCommand{script:String}` toward typed effects, and add the `EmitArtifactThenThinRun` transport arm. Everything else is consequence.

---

## 0b. Finding — modeled ops whose ONLY realization is a shell escape where a NATIVE handler is correct (transport-decomposition lane; filed 2026-07-24, do-not-fix-here)

A distinct axis from §1's *emission* census: these are ops whose **interface shape is right** but whose **single hardwired transport is wrong** — the verbatim §3(b) "single hardwired transport is the N×M-adapter trap" tell. The shape belongs to the dependency; the transport is a Realization *handler, one of N* (§2). Each of these has exactly one handler — `shell` — where a **native in-process handler** is the correct realization when locality is `OnTarget`, and `shell`/`ssh` is the handler only when the target is another process on another host.

| op (extdeps) | modeled transport | native handler that's missing | receipt |
| --- | --- | --- | --- |
| `shell.Env.Get` (`extdeps/shell/shell.dag:42`) | `shell { argv: ["printenv", "{name}"] }` | `std::env::var` — reading an env var **the process already holds in its own environment**. Realized today as `wet_env_var` spawning `printenv` (`v1_interpreter.rs:5096`). Reading your own environment isn't a host effect at all. | floor diff-observation spawns 5 `printenv` children per pass (compile-clean scope + discovery selection), repeated per floor pass — pure log clutter; ~ms each, **not a floor-time lever** |
| `shell.Which.Check` (`extdeps/shell/shell.dag:57`) | `command -v` (and the ssh `command -v <tool>` ×2 at §3's `host_effect_realize`) | native path-search when `OnTarget`; ssh `command -v` only for a remote host | sibling flagged in the transport review (2026-07-24); same root |

**One root, three lanes** (operator, 2026-07-24): these two, the transport review's `test -x`/`command -v` hand-strings for ssh, and the wall worker's fight are all *modeled operations whose only realization is a shell escape*. The dissolution is the transport-decomposition Realization: **same operation, two handlers, native chosen when locality is `OnTarget`** — `Env.Get`'s native read is the lane's **cheapest first consumer** (a pure in-process read, no `host_effect_apply` even). Not scheduled here; recorded so the deficit is counted and prioritizable (§6), never absorbed into "it's only a few ms."

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

Every way a shell string is constructed or carried, over `dag/**` (excluding `*_test.dag`). **The search must be multiline-aware** — construction is frequently `ShellCommand {` on one line and `script:` on the next, which a single-line regex misses (this gap hid the `readiness.dag` sites in the first draft — review 41467). Use a slurped/multiline match (e.g. `perl -0777 -ne '/ShellCommand\s*\{\s*script:/'`), not `grep` line-at-a-time.

| # | pattern (multiline) | what it finds |
| --- | --- | --- |
| P1 | `ShellCommand\s*\{\s*script:` | `HostEffect.ShellCommand{script}` construction — **2 live sites on main** (`live_deploy/readiness.dag`, see §4.B); the rest became nickname variants. Residue = the `host_effect_plan.dag` `{script: ""}` placeholder + the `host_effect.dag` type def. (Match-arms `ShellCommand { script: s\|_ } =>` in `host_effect_realize`/`fleet_converge_cli`/`ci_deploy_access_observe` are destructuring, not construction.) |
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
| ~~`fleet_show_effective_read.dag` · `SystemdUnitMemoryPropertiesRead`, `SystemdUnitMemoryMaxRead`~~ **LANDED** | ~~`fleet_show_effective_read_script.dag`~~ deleted; ~~inline `fleet_runner_width_count_read_script`, `fleet_runner_unit_memory_props_read_script`~~ deleted (census true-up, 2026-07-26) | `systemd.Systemctl.ShowProperty` via `gunbc.systemctl_show_read`; `list-units` via `gunbc.systemctl_list_units` | **A1 DONE** — see the dead-scaffold note below |
| ~~`host_converge_slice1.dag` (via `shell.Exec.Run`)~~ **LANDED** | ~~inline `…_memory_max_read/set_script`, `…_enumerate_units_script`~~ deleted | `systemctl_show_read` + `systemctl_list_units_active_services` (both imported and called; `host_converge_slice1.dag:15,49,287`) | **A1 DONE** — verified 2026-07-26: zero `_script` fns, zero `shell.Exec.Run`, zero `systemctl` concats in the file |
| ~~`host_identity_observation.dag` · `HostIdentityShortHostnameRead`~~ **LANDED** | ~~`host_identity_short_hostname_script`~~ deleted | `os.Hostname.ReadShort` · `dag/extdeps/tools/hostname.dag` | **A2 DONE** — realized through `gunbc.hostname_read` (LocalShell → op, SshShell → typed argv) |
| ~~`host_effect.dag` · `SetHostnameCas`~~ **LANDED (#7194)** | ~~`host_effect_set_hostname_cas_script`~~ deleted (no such symbol anywhere in tree) | `os.Hostname.Set` · `dag/extdeps/tools/hostname.dag` | **A2 DONE** — `host_effect_realize.dag`'s `SetHostnameCas` arm calls `gunbc.hostname_set` `hostname_set_cas`, whose `hostname_set_local` is `os.Hostname.Set(desired:)`; the SSH arm goes through `typed_argv_exec_over_ssh`, and the CAS read reuses the landed `os.Hostname.ReadShort`. Verified on main 2026-07-25 (bucket A): zero concat builders, zero `shell.Exec` on the path. |
| `host_effect.dag` · `ReadEffectivePosixPrincipal`, `SudoNopasswdExecuteProbe` | `host_effect_deploy_access_probe_script.dag` (whoami / `sudo -n` / `sudo -n -l`) | `access.PosixEffectivePrincipal`, `sudo.NopasswdExecuteProbe` (exist) | **A3** OWNED BY C5 #6946 — do not touch |
| ~~`live_deploy/` effect variants~~ **LANDED (D2 #7192)** | ~~`live_deploy/host_effect_script.dag`~~ **file deleted** — no such path in tree | decomposed (multi-op) | **A4 DONE** — verified 2026-07-26: `dag/gunbc/live_deploy/` is `apply · emit · intent · operations · readiness · service_ready · spec` only, and the directory contains **zero** `ShellCommand` occurrences. This closes do-not-miss item 1 below (the #7004/#7006 relocation debt) |
| `host_effect_realize.dag` · `ProvisionBuildCache` | `host_build_cache_provision_script.dag` (6 `build_cache_*_body`) | decompose | **A5** srv* deprioritized |
| hygiene reaper/liveness | `host_hygiene_reaper_script.dag` (4 `…_body`), `host_hygiene_liveness_script.dag` | decompose | **A5** srv* deprioritized |
| `srv3_host_effect_apply.dag` · `Srv3*` variants, `srv3_install_diagnostic_checklist.dag` · `Srv3InstallDiagnosticObserve`, `nbd_proxy_virtual_media_install.dag` · `Srv3NbdProxyServe` | `srv3_host_effect_script.dag` (5 fns), `srv3_install_diagnostic_observe_script.dag` (5 fns) | typed observe/receipt effects | **A5** srv* deprioritized |

**A1 dead-scaffold finding (census true-up, 2026-07-26).** The A1 typed migration (D2 #7192) dissolved the *live* read path onto `ShowProperty`/`ListUnits` but left two `systemctl` concat builders behind in `fleet_show_effective_read.dag`, orphaned:

- `fleet_runner_unit_memory_props_read_script` — **zero references tree-wide** beyond its own definition. Pure dead code.
- `fleet_runner_width_count_read_script` — **zero production callers**; its only other reference was a witness assertion.

Both are deleted in this true-up (deletion is the receipt; a row claiming "done" beside a live concat is the relocation pattern this census exists to catch). Two things make this worth recording rather than quietly fixing:

1. **The file's own note claimed the property the tree did not have.** `fleet_show_effective_read.dag:62` asserts "no per-command HostEffect nicknames, **no concat shell strings**" while two concat shell strings sat 110 lines below it. The note was true of the *live path* and false of the *file* — the §3 tell that a prose claim had drifted from its carrier.
2. **A witness was pinning the dead scaffold as contract.** `fleet_show_effective_read_witness_test.dag` · `witness_transport_reads_use_show_property_authority` — a witness whose *name* asserts reads use the typed authority — carried a third conjunct asserting `fleet_runner_width_count_read_script(...)` contains `"list-units"`. That conjunct (a) is tautological (a `concat` of a literal containing `list-units` always contains `list-units`; it can only fail if someone edits the literal), (b) asserts nearly the **opposite** of the witness's stated purpose — that a shell-concat *bypass* still exists with the right shape, and (c) kept a dead symbol referenced, so it never read as dead code. Removing it leaves the witness asserting exactly what its name claims (typed argv match + local read), which is **stronger**, not weaker. This is the DESIGN §5 "tests *asserting the string* so the degradation was enshrined as the contract" shape, in the small.

### 4.B — DIRECT `ShellCommand{script}` still constructed in intent

**STATUS 2026-07-26 (census true-up): both rows below are DISCHARGED — there are now ZERO direct `ShellCommand{script}` construction sites in `dag/gunbc/live_deploy/`** (verified: `grep -rn ShellCommand dag/gunbc/live_deploy/` returns 0). `live_deploy_unit_diagnosis_command` and `live_deploy_healthz_probe_script_for_port` no longer exist anywhere in tree — D2 #7192 (tree_sync `| tail` + `exit 0` absorbing fallback, converted to a typed `Systemctl.Status` exit-as-data refusal) and D3 #7193 (readiness routed through `gunbc.systemctl_status_read` + `http.Client.Get`) closed them. The table is kept struck-through as the audit trail.

~~The relocation wave turned most direct construction into nickname variants (4.A), but **two live direct sites remain** on `origin/main` — the multiline form my first-draft single-line P1 missed (review 41467):~~

| site (file · construction) | builder (file · fn) | runs on | dissolve to |
| --- | --- | --- | --- |
| `live_deploy/readiness.dag` · `ShellCommand { script: live_deploy_healthz_probe_script_for_port(port) }` | `readiness.dag` · `live_deploy_healthz_probe_script_for_port` → `intent.dag` · `live_deploy_health_probe_curl_command` (curl localhost `/healthz`; already uses the typed `curl_bounded_localhost_get_argv_prefix`) | srv1 LocalShell via `host_effect_apply_gated` | `http.Client.Get` · `extdeps/http/client.dag` (or the typed curl argv end-to-end) — see §4.C / §5.B |
| `live_deploy/readiness.dag` · `ShellCommand { script: live_deploy_unit_diagnosis_command(unit) }` | `intent.dag` · `live_deploy_unit_diagnosis_command` (`systemctl status --no-pager --full <unit> \| tail`) | srv1 LocalShell | a `systemd.Systemctl.Status` op (§5.A add) — **and the `\| tail` + defensive `exit 0` is a §5 absorbing fallback** (intent.dag's own comment admits it masks systemctl's nonzero); the typed op models exit-3-for-dead-unit instead |

Residue only (not construction to migrate):

| site (file · symbol) | what | action |
| --- | --- | --- |
| `host_effect_plan.dag:39` · `ShellCommand { script: "" }` | empty placeholder | delete with the type — **still present** (verified 2026-07-26) |
| `host_effect.dag:28` · `ShellCommand { script: String }` | the **type variant** itself | delete at arc close (DESIGN §5, escalated) — terminal; **still present** |
| `fleet_converge_cli.dag:57` · `ShellCommand { script: _ } => fallback` | a **match arm**, not a construction site — consumes the variant, builds no string | *(row added by the 2026-07-26 true-up; previously uncensused)* dissolves with the type variant above. Listed so the arc-close delete has a complete consumer list — a missed match arm is what turns the terminal delete into a non-exhaustive-match break |

*(The first draft mis-grounded on a divergent worktree — `fleet_show`/`host_identity` corrected to 4.A per review 41399 — and then under-scoped P1 to single-line, missing these `readiness.dag` sites per review 41467. Both fixed.)*

### 4.C — RUNTIME-PRESENT `shell.Exec.Run` with a string/`_script` body (dissolve to typed op)

| site (file · fn) | body | should call | class |
| --- | --- | --- | --- |
| `host_converge_slice1.dag` · `…_memory_max_read/set/enumerate_units_script` | `systemctl show`/`set-property`/`list-units` | `systemctl.ShowProperty`/`SetProperty` + new `list-units` op | A1 (same cluster as 4.A) |
| ~~`host_converge_slice1.dag`, `host_identity_adopt.dag`, `host_runner_memory_cap_verify.dag`~~ **LANDED** | `"date -Iseconds"` | `Clock.Now` · `dag/extdeps/clock/clock.dag`, via the single authority `gunbc.clock_read` `clock_now_probed_at` | **A2 DONE** — the last holdout `host_runner_memory_cap_verify.dag` dissolved in bucket A (2026-07-25); its `retained_runtime` bridge, its `extdeps.shell` import and its transport-script scaffold all deleted with it |
| ~~`ci_deploy_target_host.dag`~~ **LANDED** | `"hostname -s 2>/dev/null \|\| hostname"` | `os.Hostname.ReadShort` via `gunbc.hostname_read` `hostname_short_read_local` | **A2 DONE** — the `\|\| hostname` widen is gone with the string; the file's remaining `extdeps.shell` import is `shell.Env.Get`, which is not the confined `extdeps.shell.exec` module (see `non_exec_import_is_not_leak_holds`) |
| `host_identity_assimilation.dag`, `host_identity_adopt.dag`, `srv3_install_diagnostic_checklist.dag` | `"echo <receipt>"` | typed receipt emit (a stdout write, not shell) | A4 |
| `host_effect_realize.dag` · `shell_exec_via_bash` dispatch | realization-core script dispatch | the `LocalShell`/`SshShell` typed-argv edge (C5) | A4 realization core — confirm before edit |
| `dag/tools/{host_prelude,gunbc_ci,emit_host_gate,merge_admission_stamp}`, `gunbc/tools/review.dag` | witness/CI transports invoked from `claim_executor` | typed `WitnessBin.Run`/argv (host_prelude precedent) | A4 |

### 4.D — `ssh.Session.Exec` command-string (vs typed `ExecArgv`)

**RECLASSIFIED by the 2026-07-26 true-up — read before picking this up.** Every `ssh_session_exec(command:)` site in the table below sits inside an **`srv3_*` function**, i.e. inside the srv\* cluster the operator wound down 2026-07-22 and authorized for retirement 2026-07-24 (§5.D roadmap item). So 4.D is **not** an independently-dispatchable A1 bucket: dissolving these to `ExecArgv` would be typed-argv work on a subgraph slated for deletion — motion, not progress. **4.D inherits A5's deferred status** and should be picked up only as part of the srv3 retirement, or if that retirement is abandoned.

Exhaustive site list, verified on `origin/main` @ `efe67794cd` (enclosing fn in bold — this is what the original table did not record):

| line | enclosing fn | verb | disposition |
| --- | --- | --- | --- |
| `:731` | **`srv3_transport_witness_bin_success`** | arbitrary command (`cd … && bin args`) | A5-deferred (srv3) |
| `:742` | **`srv3_transport_test_executable`** | `test -x <path>` | A5-deferred (srv3) |
| `:752` | **`srv3_apt_tool_present`** | `command -v <tool>` | A5-deferred (srv3) |
| `:775` | **`srv3_tool_bin_path`** | `command -v <tool>` | A5-deferred (srv3) |
| `:821`, `:822` | **`srv3_chown_directory_to_current_user`** | `id -u` / `id -g` | A5-deferred (srv3) |
| `:1169` | **`run_shell_transport`** | `ssh_session_exec_script(script: script.body)` | **NOT srv3** — this is the realization core's `RetainedShellScript` → SSH path, i.e. the §5.E counted frontier's own transport. It dissolves when the frontier empties, not by a per-site migration. Do not edit under a 4.D brief |

The original table's rows are retained below for provenance:

| construction | verb | should call | class |
| --- | --- | --- | --- |
| `ssh_session_exec(command: cmd)` | arbitrary command | `ssh.Session.ExecArgv` (C5) | A1 |
| `ssh_session_exec(command: concat("test -x ", path))` | file-exists probe | `ssh.Session.ExecArgv` | A1 |
| `ssh_session_exec(command: concat("command -v ", tool))` (×2) | tool-presence probe | `ssh.Session.ExecArgv` | A1 |
| `ssh_session_exec(command: "id -u")`, `"id -g"` | uid/gid read | `ssh.Session.ExecArgv` (an `id` op) | A1 |
| `ssh_session_exec_script(script:)` | script-over-ssh | typed-argv splice or decompose | A4 |
| `extdeps.diagnostic.ssh.dag` · `ssh.Session.Exec(command:)` | the transport itself | keep as the ONE command-string transport, or fold into `ExecArgv` | transport-decision — escalate |

### 4.E — FOREIGN-EXECUTOR / BOOTSTRAP emit (LEGIT shell — route through `emit(intent,Bash)`, stays shell but bounded)

> **True-up @ post-#7216 (wise-crane-222, 2026-07-26):** fresh bucket-D re-census; supersedes the 07-22 rows below for CI placement files. Authority: `shell-intent-emit-realization-design.md`, `host-effect-orchestration.md`. Invariant: intent imports no `bash_build` / `ShellProgram` / `serialize_bash` (`realization_vocabulary_containment`).

These are correct as shell (GHA `run:`, cron, git hooks, pre-runtime bootstrap — the executor only understands shell text). Target = emitted through the v2 bash rows via `orch_emit_pipeline(medium: bash_orchestration_emit_medium())`, a **roster not a growth surface** — NOT dissolved to a typed op unless a row names one.

**Already on emit (do not re-migrate):**

| module | symbols | executor |
| --- | --- | --- |
| `v2.workflow.ci_workflow_run_emit` | `ci_isolate_toolchain_script`, `ci_pin_rustup_default_script`, `ci_selection_control_script` | GHA `run:` |
| `v2.workflow.ci_floor_peak_emit` | `ci_cgroup_peak_locate_shell`, `ci_floor_peak_pre_script`, `ci_floor_peak_post_script` | GHA `run:` |
| `v2.workflow.ci_retry_emit` | `ci_cargo_eagain_retry_script` | GHA `run:` (via `ci_release_build_emit`) |
| `v2.workflow.ci_release_build_emit` | `ci_release_build_script`, `gunbc_ci_run_script` | GHA `run:` (partial — still concat-wraps verify script) |
| `v2.workflow.ci_materialization_emit` | `ci_sccache_provider_shell_injection` | GHA `run:` (**LANDED #7265**) |
| `v2.workflow.ci_merge_admission_emit` | `ci_floor_disposition_marker_init_script` | GHA `run:` (**LANDED #7265**) |
| `gunbc.assimilate.bmc_token_federation` | `gcp_token_smoke_script` | GHA `run:` |
| `gunbc.live_deploy.emit` | `expected_live_deploy_apply_script`, `expected_live_deploy_retract_script` | GHA deploy `run:` |
| `gunbc.host_effect` | `fresh_standup_bootstrap_intent` → `EmitArtifactThenThinRun` bootstrap arm | pre-runtime bootstrap |

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
| `fleet_converge_emit.dag` `fresh_standup_bootstrap_script` + fresh-standup `Run{command: concat(...)}` rows | pre-runtime bootstrap | E-emit (byte-oracle vs `.github/fleet-converge.sh`) |
| `roadmap_static_site.dag` body fns | srv1 dashboard (belt B) | E-emit / dissolves with belt B |
| `fleet_posix_accounts.dag` `probe_command: "id <user>"` (×4) | account-existence probe | **NOT a shell emit site** — authored provenance metadata; live proof is `deploy_access_check_observed` via typed `SudoNopasswdExecuteProbe` |

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

### Ledger true-up @ 2026-07-26 (§4.A/4.B/4.D, snappy-moth-330) — read this FIRST

Re-censused §4.A, §4.B and §4.D against `origin/main` @ `efe67794cd` by execution over the tree, not by trusting the rows. Four corrections, in descending order of how badly the stale row would mislead:

1. **§4.D is not a dispatchable bucket — it is A5-deferred.** Every `ssh_session_exec(command:)` site is inside an **`srv3_*`** fn (the wound-down srv\* cluster), except `:1169` which is the realization core's own `RetainedShellScript` transport. The old table listed the verbs without the enclosing fns, which made it read like an independent typed-argv bucket. It is not; picking it up would be typed-argv work on a subgraph slated for retirement.
2. **§4.A4 and §4.B are DONE.** `live_deploy/host_effect_script.dag` is deleted and `dag/gunbc/live_deploy/` contains zero `ShellCommand`. This closes "do-not-miss" item 1 (the #7004/#7006 relocation debt), which had been the loudest open warning in this doc.
3. **§4.A1 is DONE, but left two dead concat builders behind** — deleted in this true-up, with the witness that was pinning one of them as contract. See the A1 dead-scaffold note in §4.A; the pattern (a file note claiming "no concat shell strings" while two sat below it, plus a tautological witness conjunct asserting the bypass still had the right shape) is the reusable finding.
4. **One uncensused consumer added** — `fleet_converge_cli.dag:57`'s `ShellCommand` **match arm**, so the terminal type delete has a complete consumer list.

**Net:** with bucket D (§4.E/§4.I) in flight, the non-deferred remainder of this arc is **§4.F wall-green only** (the `host_language_transport_script` lens promotion + meta-exec confinement) — and that is coupled to the node/subtree visibility-grants lane, not independently schedulable here. Everything else open is operator-deferred srv\*.

### Ledger true-up @ 2026-07-25 (bucket A, calm-pike-837) — read this before the 07-22 snapshot below

The 07-22 snapshot is ~7 merges stale. What has landed since, keyed to the rows it discharges:

| PR | what | census effect |
| --- | --- | --- |
| #7184 | the **transport-script construction wall** (§5.E keystone) | §5.B per-op migration UNPAUSED — relocation is now a type error, so the §4 tables are again the live punch-list |
| #7192 / #7193 | **D2 / D3** — `systemd.Systemctl.ShowProperty` consumers + `systemctl` op cluster | 4.A1 `fleet_show_effective_read`, `host_converge_slice1` memory-property reads |
| #7194 | **D1** — `os.Hostname.ReadShort` + `os.Hostname.Set` on `dag/extdeps/tools/hostname.dag` | 4.A2 rows for `HostIdentityShortHostnameRead` **and** `SetHostnameCas` — both now DONE (see §4.A) |
| #7215 | census doc pass | — |
| #7231 | **extdeps positioning restructure** | import paths repointed: `extdeps.os.systemctl` → `extdeps.systemd`, `extdeps.os.hostname` → `extdeps.tools.hostname`, `extdeps.os.exec_arg_limit` → `extdeps.exec`, `os.os` → `extdeps.os` hub-loose. Older rows in §4/§5 still spell the pre-restructure paths; read them through this mapping. |
| #7233 | srv3 `Record` | 4.A5 srv\* cluster (still deferred) |
| **#7265** | bucket D PR1 — `ci_sccache_provider_shell_injection` + `ci_floor_disposition_marker_init_script` → `ci_materialization_emit` / `ci_merge_admission_emit` | 4.E/4.I/4.J rows for those two symbols DONE; §4.E/4.J true-up folded here (orphan doc deleted) |
| **this PR** | **bucket A** — the last `date -Iseconds` in `host_runner_memory_cap_verify.dag` onto `Clock.Now`; `meta_exec_confinement_exception_roster` **3 → 0** | 4.C `date -Iseconds` and `hostname -s` rows DONE; the meta-exec roster is now empty, so §4.F's wall has no exceptions left to grant |

**Meta-exec roster: 3 → 0, and the dark lane that hid it.** Two of the three rows (`dag/gunbc/tools/review.dag`, `dag/gunbc/ci_deploy_target_host.dag`) were **stale** — their sites had stopped importing `extdeps.shell.exec` merges earlier and nobody noticed, because `meta_exec_roster_sound_live` existed in the lens but ran on **no** per-PR cadence; the only enrolled roster RED was synthetic (hand-written fact rows, blind to the live roster). Measured this PR: the live receipt costs **13.5s cold** against a 5s fast-lane budget, so it cannot be enrolled per-PR as-is. The per-PR enforcement is instead a **construction wall** — `stale_count` is bounded above by roster length, so the empty roster proves soundness with no scan, and any re-added row reds `meta_exec_roster_shrunk_to_empty_holds` in the same PR that adds it. The live receipt stays as a backstop in `src/v2/test/claim/long/meta_exec_confinement_clean_tree_test.dag` (with a non-degeneracy control, since a clean tree makes a live scan vacuously true if the walk reads nothing). **Named residue:** that long lane is still dark (not roster-enrolled); its dissolve-on is a falsifier batch admitting live-tree lens receipts.

### Wind-down PR ledger — snapshot @ 2026-07-22 (calm-ferret-849 subtree)

The state of every PR in this arc, so nothing is missed if work pauses here. **A task's real state is its branch/PR** (ROADMAP.md rule); this is that ledger.

**Landed (merged to main):**

| PR | owner (session) | what | note |
| --- | --- | --- | --- |
| #6946 | keen-deer-531 *(archived)* | **Wave C5** — typed-argv exec machinery + access-probe dissolution (4.A3) + `ssh.Session.ExecArgv` | **the foundation.** `ExecArgv` is now the shared authority every SshShell dissolution splices through. Genuine. |
| #7007 | valiant-deer-438 *(archived)* | fleet-converge bootstrap emitter dissolution + byte-oracle vs `.github/fleet-converge.sh` | genuine (emit path, §5.C) |
| #7004 | nimble-carp-340 *(archived)* | live_deploy/apply "Phase-2" | **⚠ RELOCATION, not dissolution** — moved the concats to `live_deploy/host_effect_script.dag`; the raw shell string still exists (4.A4). **Not truly done.** |
| #7006 | zesty-crane-129 *(archived)* | fleet_show/host_identity "Phase-2" | **⚠ RELOCATION** — `fleet_show` is now properly dissolved by #7064 (supersedes this); `host_identity` still needs real dissolution (4.A2). **Not truly done.** |

**Open (in review):**

| PR | owner | what | next action |
| --- | --- | --- | --- |
| **#7065** | **calm-ferret-849 (me)** | **THIS doc** — §4 census + §5 Method of Action (the tracking authority) | 2 REQUEST_CHANGES fixed (wrong-tree #41399, multiline-pattern #41467); re-review pending, then merge |
| **#7064** | sleek-crab-621 | **PROPER `fleet_show` + `host_converge_slice1` dissolution — THE EXEMPLAR** (call the op directly; verified genuine, not relocated) | reconcile onto merged C5 (drop #7064's duplicate `ExecArgv`), re-review, merge |

**Closed / deprioritized (srv\* cluster — operator wound down 2026-07-22):**

- Sessions wise-crab-547 + crisp-deer-871 + still-ant-534 + sharp-heron-884 closed. PRs #7019, #7025, #7044 closed; #7020, #7026, #7011 merged before closure.
- **Their relocations remain on main** (4.A5 srv\* rows: `srv3_*`, `host_build_cache_provision`, `host_hygiene_*`) — real debt, **deferred by operator, NOT done.**

**Not started — the remaining arc (bounded, fully specified in §5; safe to pause):**

- **§5.A** — **COMPLETE.** All four ops landed on #7194 and are verified in tree at their post-#7231 homes: `os.Hostname.ReadShort`/`Set` (`dag/extdeps/tools/hostname.dag`), `systemd.Systemctl.ListUnits` and `.Status` (`dag/extdeps/systemd/systemctl.dag:185,219`), `os.Id.Uid` (`dag/extdeps/tools/id.dag:26`). (`ssh.Session.ExecArgv` landed via C5; `Clock.Now` in `dag/extdeps/clock/clock.dag` already existed and is now the single authority for every `date -Iseconds` site.) **The finite new-op list this whole arc needed is closed** — everything remaining in §5.B is calling ops that now exist.
- **§5.B** — the call-the-op migrations, **UNPAUSED** (the wall landed, #7184). D1/D2/D3 (#7192/#7193/#7194) and bucket A discharged the hostname, systemctl-read and clock clusters. **Updated 2026-07-26:** `live_deploy` (4.A4) and the `fleet_show`/`host_converge_slice1` systemctl cluster (4.A1) are **also done** — only the **operator-deferred srv\* cluster (4.A5, and 4.D which is inside it)** remains in §5.B. With bucket D (4.E/4.I foreign-executor emit) in flight, the non-deferred §5.B queue is **empty**.
- **§5.E** — the **transport-script construction wall** — **LANDED #7184.** Built as a `RetainedShellScript` RECORD edge + free-minter deletion + counted bridges + lens activation + compile-fail REDs (see §5.E ruling block; "brand `TransportScript`" was found non-walling because the brand is transparent). The 2026-07-24 wall-first ruling that paused §5.A/§5.B is therefore discharged.

**⚠ Do-not-miss for wind-down:**

1. ~~**#7004 and #7006 merged as "progress" but are relocations**~~ — **CLOSED by the 2026-07-26 true-up.** Both relocation sites are now genuinely dissolved: `live_deploy/host_effect_script.dag` is **deleted** (D2 #7192) with zero `ShellCommand` left in the directory, and `host_identity`'s hostname path went typed on D1 #7194. The relocation debt this item tracked no longer exists on main.
2. **Two open PRs to land:** #7065 (census/plan) and #7064 (exemplar).
3. **Nothing is lost by pausing** — §5 is the durable, bounded plan (4 ops + call-existing-op + emit-roster + wall). Resume from §5 whenever.

### 4.I — CI foreign-executor sites the census missed (2026-07-24 audit @ `879c5a2699`)

§4 was grounded at `78f43c38`; a three-facet CI audit (2026-07-24) found **five CI `run:` shell sites that landed after that snapshot and are not in the tables above.** Each already carries a rich in-code note — they were tracked *at the site*, never rolled up here. Keyed on file + symbol (the §4 convention). All are foreign-executor (GitHub Actions `run:`), so category **E-emit** (route through the v2 bash rows, a roster not a growth surface) unless a typed-op dissolution is named.

| construction site (file · symbol) | what | executor | class / dissolves to |
| --- | --- | --- | --- |
| `ci_spec.dag` · `ci_fmt_gate_line` | `"$CARGO_BIN" fmt --all --check` (build job, first step) | GitHub Actions | **5.B call-the-op** — `cargo.Build.Fmt` · `extdeps/rust/cargo_build.dag` (its `ci_fmt_gate_note` already names this dissolve-on — the same seam `ci_release_build_line` awaits). NOT E-permanent. |
| `ci_spec.dag` · `gunbc_ci_deploy_invoke` | `ROOT=$(git rev-parse … \|\| pwd); "$ROOT/target/release/gunbc" run … live_deploy_apply_srv1_wet` (deploy job) | GitHub Actions | E-emit (thin `gunbc` invocation wrapped in bash); the `git rev-parse … \|\| pwd` prelude is a §5 fallback (below) |
| `ci_spec.dag` · `gunbc_ci_heal_regen_invoke` | `gunbc run … generated_artifact_gate main_wet` (heal job) | GitHub Actions | E-emit (same thin-invoke shape; `ci_heal_regen_note` documents it) |
| `ci_spec.dag` · `ci_heal_git_add_lines` + commit/push | `git add` over `committed_generated_artifact_paths()` (each `[ -e ]`-guarded), `git diff --cached --quiet` gate, `git commit`, `git push origin HEAD:<branch>` (heal job, 76 lines) | GitHub Actions | E-emit — the git verbs dissolve to typed `git.Core.*` ops (Add/Commit/Push); `ci_heal_commit_push_note` already frames it as "the foreign-executor case that renders through bash" |
| `ci_materialization.dag` · `ci_sccache_provider_shell_injection` | `if sccache --show-stats >/dev/null 2>&1; then RUSTC_WRAPPER=sccache; …; fi` (embedded in "Isolate toolchain dirs") | GitHub Actions | **DONE (#7265)** — migrated to `v2.workflow.ci_materialization_emit`; still carries §5 ABSORBING-FALLBACK debt (opportunistic guard silently skips caching when daemon is down → dissolve-on: mandatory provisioning-ensure via srvN build-cache STEP-2) |

**§5 absorbing-fallbacks INSIDE the CI scripts** (§5.B rule: each becomes a *modeled outcome*, never re-appended). The census tracked the scripts but not these internal widens:

| site | fallback | fail-closed / modeled form |
| --- | --- | --- |
| ~17 `run:` bodies | `git rev-parse --show-toplevel 2>/dev/null \|\| pwd` | scope-widen on non-repo → typed refusal (`git.Core.ShowToplevel` with a `nonzero =>` refuse, not `pwd`) |
| regen + floor steps | `git fetch --no-tags origin main … \|\| true` | stale-baseline on fetch failure silently corrupts the affected-set diff base → typed refusal or an explicit `DiffBaseUnavailable` marker downstream logic can gate on |
| `ci_workflow_run_emit.dag:31` · `ci_native_cache_root_toolchain_segment_command` | `rustc -V 2>/dev/null … ; if empty: toolchain-unresolved` slug | cache-poison fallback (all native-cache writes land in one poisoned root) → typed refusal on `rustc -V` failure |
| `ci_spec.dag:219` · `ci_retry_escalation_level1` | `CARGO_BUILD_JOBS=1` on EAGAIN | masks a fleet-pressure *resource deficit* rather than emitting a typed counted diagnostic (its level-2 sibling `-u RUSTC_WRAPPER` was removed as STEP-2, #7138); recover parallelism-cap as a fleet-admission control, not a per-run silent degrade |

**Adjacent (NOT shell-string) — cross-ref, tracked elsewhere.** The same audit found a second class the CI shares that is *not* a `ShellProgram`/string sink and so does **not** belong in this census: hand-spelled **config** that should DERIVE from an existing model. The exemplar is toolchain isolation — `ToolchainEnvIsolation = SharedHomeAcrossJobs \| PerJobCargoHome{…} \| HermeticContainer` (`extdeps/toolchain/types.dag`) is modeled and consumed by `sccache`/`fleet_intent`, but the workflow emit never reads `env_isolation`; it hand-spells per-job isolation (build isolates, the ci/gate job shares → the rustup `ETXTBSY` race, documented in PROSE in `ci_floor_gate_toolchain_note` on `ci_workflow.dag` rather than modeled/gated). Same shape for the GHA **cache key** (modeled `gha_actions_cache_facts` key-derivation vs the hand-spelled `ci_cache_key_template`), **artifact** upload/download (un-modeled; the dissolve-on artifact-CacheProvider named in `ci_materialization.dag` is undone), and **permissions** (`std.effect_grant` vs per-job `WorkflowPermissions`). These are the model↔realization-fork open thread + roadmap `2-emit-partition`, not the ShellProgram arc; noted here only so the audit's findings aren't lost across the seam.

### 4.J — Bucket D foreign-executor emit punch-list @ post-#7216 (wise-crane-222, 2026-07-26)

The per-symbol remaining work and batching plan for §4.E CI sites. **Load-bearing:** every `ci_spec.dag` composer change regenerates `.github/workflows/ci.yml`; drift+parse gate is the byte-oracle.

**Typed-op candidates (route to extdeps op, NOT bash emit):**

| site | current | target op | notes |
| --- | --- | --- | --- |
| `ci_spec.dag` · `ci_fmt_gate_line` | `"$CARGO_BIN" fmt --all --check` | `cargo.Build.Fmt` | `ci_fmt_gate_note` already names dissolve-on; **load-bearing ci.yml** |
| `ci_deploy_access_emit.dag` · `deploy_access_emit_principal_read_script` | `"whoami"` | `os.Id.Lookup` / effective-principal read | deploy preflight; Wave C typed argv |

**Remaining concat-built foreign-executor punch-list:**

#### A — `merge_admission_produce.dag` (5 script surfaces)

| symbol | GHA consumer | status / emit complexity |
| --- | --- | --- |
| `ci_floor_disposition_marker_init_script` | floor opener (via `gunbc_ci_floor_only_script`) | **DONE (#7265)** → `ci_merge_admission_emit` |
| `ci_documentation_only_gate_skip_prefix` | receipt gates, merge gate, selection control | open — medium (nested if/test + cmdsubst) |
| `ci_merge_admission_stamp_script` | merge-admission stamp step | open — low |
| `ci_merge_admission_gate_script` | merge-admission gate step | open — medium |
| `ci_floor_stamp_merge_admission_script` | floor tail | open — medium (`$?` capture) |

#### B — `ci_materialization.dag` (3 script surfaces)

| symbol | GHA consumer | status / emit complexity |
| --- | --- | --- |
| `ci_sccache_provider_shell_injection` | isolate-toolchain pipeline | **DONE (#7265)** → `ci_materialization_emit` |
| `ci_floor_materialization_receipt_gate_script` | ci job gate | open — high (sed parses + if ladder) |
| `ci_floor_resolve_receipt_gate_script` | ci job gate | open — high (sed + numeric compare) |

#### C — `ci_spec.dag` (~15 distinct script composers feeding GHA)

| symbol | GHA consumer | route |
| --- | --- | --- |
| `ci_release_build_line` | build (via retry emit) | emit (partially done) |
| `ci_fmt_gate_line` | build fmt step | **typed op** |
| `ci_floor_build_verify_script` | build verify + unpack | emit (build_step transport pattern) |
| `ci_release_bins_pack_script` | build pack artifact | emit |
| `ci_release_bins_unpack_verify_script` | ci unpack step | emit |
| `gunbc_ci_floor_only_script` | ci floor `run:` | emit (composes A rows) |
| `ci_regen_floor_skip_shortcut_script` | regen skip | emit (`if` + cmdsubst) |
| `gunbc_ci_regen_floor_only_script` | regen job | emit (composer) |
| `gunbc_ci_deploy_invoke` | deploy job | emit (thin gunbc run) |
| `gunbc_ci_heal_regen_invoke` | heal regen | emit |
| `gunbc_ci_heal_commit_push_script` | heal commit/push | emit (git verbs; future typed git ops) |
| `scheduler_invoke` / `scheduler_invoke_with` | floor/regen | emit (thin claim_executor) |
| `git_fetch_script` | floor/regen/deploy | emit (`git fetch` + fallback) |

#### D — permanent roster (emit target, never typed-op dissolve)

| site | executor |
| --- | --- |
| `gunbc.githooks_pre_push_emit` · `expected_githooks_pre_push_sh` | git pre-push hook |
| cron entry lines (REST transport witness corpus) | cron |

**Proposed batching (3 PRs):**

| PR | scope |
| --- | --- |
| **PR 1 (#7265)** | `ci_sccache_provider_shell_injection` + `ci_floor_disposition_marker_init_script` emit migrations; witness tests + `realization_vocabulary_containment` roster; regen `ci.yml`. Does not touch `ci_spec.dag` composers beyond import delegation. |
| **PR 2** | merge-admission cluster completion + receipt gates (A/B remaining rows). |
| **PR 3** | `ci_spec` composer migration + `ci_fmt_gate_line` → typed `cargo.Build.Fmt`. **Operator review required** — load-bearing CI generator. |

### Dissolution trigger for §4

This punch-list folds into the **`host_language_transport_script` lens going live** (4.F): once a compile gate reds any raw-string `shell.Exec.Run` / hand-built transport, new instances are unwritable by construction (§5) and a prose punch-list is redundant. Until then, every row here is discharged by *deletion of the concat*, verified green-by-execution + an injection-RED — never by relocation.

---

## 5. Method of Action — the bounded path to bash-free user space (calm-ferret-849, 2026-07-22)

**End state:** no user-space `.dag` constructs a shell string. Every "this `.dag` wants to call a bash script" instance resolves to exactly one of four paths below. The `realization_vocabulary_containment` lens (#6854, LIVE) already forbids bash-AST vocab (`ShellProgram`/`ShellStmt`/`serialize_bash`) in user space — the remaining hole is the `shell.Exec.Run(script: TransportScript)` / `ShellOnHost{script: String}` sink (§5.E), which is what makes relocation possible.

**The headline (verified by op inventory @ `78f43c38`):** the arc needs only **~4 new typed ops**. Almost every site calls an op that already exists.

### 5.A — the FINITE new-op list (the ONLY new modeling the whole arc needs)

| new op | home file | covers verb | consumers |
| --- | --- | --- | --- |
| `extdeps.os.hostname` · ~~`Read`~~ **LANDED** + `Set` | `dag/extdeps/os/hostname.dag` (created) | ~~`hostname -s`~~ **done**; `hostnamectl set-hostname` remains | **Read done** — `os.Hostname.ReadShort` realized via typed cell `HostnameReadOnHost` + `gunbc.hostname_read` (LocalShell → op, SshShell → typed argv); both read consumers dissolved: `host_identity_observation` (`HostIdentityShortHostnameRead`, the `host_identity_short_hostname_script` concat DELETED) and `ci_deploy_target_host` (runtime-present read; the GHA preflight *block* stays E-emit). **Set** (`host_effect_hostname` `SetHostnameCas`, its own `host_effect_set_hostname_cas_script` concat) still pending — reuses this op file with a `SetHostname` op. |
| `systemd.Systemctl.ListUnits` | **add to** `dag/extdeps/os/systemctl.dag` | `systemctl list-units --state=active` | `host_converge_slice1` (was `_enumerate_units_script`; now calls `systemctl_list_units_active_services` at `:287`). *The `fleet_show_effective_read` consumer named here — `fleet_runner_width_count_read_script` — was never migrated: it was superseded and left dead, and is deleted by the 2026-07-26 true-up.* |
| `extdeps.os.id` · `Read` (uid/gid/user) | **new** `dag/extdeps/os/id.dag` | `id -u`, `id -g`, `id <user>` | `host_effect_realize` (ssh probes), `fleet_posix_accounts` (`probe_command`) |
| `ssh.Session.ExecArgv` | **add to** `dag/extdeps/diagnostic/ssh.dag` | typed argv over ssh (`ssh host -- argv`) | `host_effect_realize` ssh probes — **IN FLIGHT, C5 #6946** |
| `systemd.Systemctl.Status` | **add to** `dag/extdeps/os/systemctl.dag` | `systemctl status --no-pager --full <unit>` (models exit-3-for-dead-unit — retires the `\| tail`+`exit 0` absorbing fallback) | `live_deploy/readiness.dag` unit diagnosis |

That is the entire new-modeling surface. (A sixth, optional: a typed stdout/receipt emit for the two `echo <receipt>` sites, or reuse `Filesystem.Write`.) The healthz probe needs **no** new op — `http.Client.Get` already exists.

### 5.B — CALL AN EXISTING OP (receipt: the op is already modeled on main)

| site (file · symbol) | current shell | call this op — receipt (file · service.Op) |
| --- | --- | --- |
| `host_converge_slice1` · `_memory_max_read_script` | `systemctl show --property=MemoryMax --value` | `systemd.Systemctl.ShowProperty` · `extdeps/os/systemctl.dag` |
| `host_converge_slice1` · `_memory_max_set_script` | `systemctl set-property … MemoryMax=` | `systemd.Systemctl.SetProperty` · `extdeps/os/systemctl.dag` |
| `host_converge_slice1` · `date`, `host_identity_adopt` · `date` | `date -Iseconds` (local ISO w/ offset) | `Clock.Now` · `extdeps/clock/clock.dag` — **⚠ semantics differ**: `Clock.Now` is wired to `date -u +%Y-%m-%dT%H:%M:%SZ` (UTC `…Z`), not local `-Iseconds`. Migrate by adopting UTC-`Z` (canonical for receipts) — a deliberate reconciliation, NOT a silent drop-in; if a site truly needs local offset, add a `Clock.NowLocal` variant rather than fork `date` (review 41476) |
| `fleet_show_effective_read` · `SystemdUnitMemory*Read`, `fleet_runner_unit_property_read_script` | `systemctl show --property` | `systemd.Systemctl.ShowProperty` · `extdeps/os/systemctl.dag` |
| `host_effect_realize` · ssh `test -x <path>` | `test -x` | `shell.Find.IsExecutable` · `extdeps/shell/shell.dag` (spliced via `ssh.Session.ExecArgv`) |
| `host_effect_realize` · ssh `command -v <tool>` (×2) | `command -v` | `shell.Which.Check` · `extdeps/shell/*` (via `ExecArgv`) |
| `tools/review` · `design`, `algebra_ref` | `git fetch` + `git show origin:FILE` | `git.Core.FetchNoTags` + `git.Core.Show` · `extdeps/git/git.dag` (`|| echo '(not found)'` → typed error→Absent) |
| `merge_admission_stamp` · `mkdir -p` | `mkdir -p` | `Filesystem` / `shell.Find.Dir` · `extdeps/filesystem/filesystem_io.dag` |
| `dag/tools` · `host_prelude`/`gunbc_ci`/`emit_host_gate` witness+build transports | witness/build run | `gunbc.WitnessBin.Run` · `extdeps/gunbc/gunbc.dag`; `cargo.Build.*` · `extdeps/rust/cargo_build.dag` (`host_prelude` already has the typed precedent) |
| `live_deploy/readiness.dag` · `live_deploy_healthz_probe_script_for_port` → `intent.dag` · `live_deploy_health_probe_curl_command` | `curl` localhost `/healthz` | `http.Client.Get` · `extdeps/http/client.dag` (already exists) |
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
| srv* cluster | `srv3_host_effect_script`, `srv3_install_diagnostic_observe_script`, `host_build_cache_provision_script`, `host_hygiene_*` | operator un-defers srv* — **see the srv3-retirement roadmap item below (audit-confirmed dead, 2026-07-24)** |
| nbd backgrounding | `host_effect_nbd_proxy_serve` `RawLine` (`&`/trap/`$!`) | typed systemd transient-unit effect + `Filesystem.Read` token + typed argv (dissolution trigger already in-file; operator ruled no trap/&/$! vocab) |

#### Roadmap item — srv3 install/reconcile subgraph retirement (operator-authorized in principle 2026-07-24; gated on load-bearing coproduct surgery)

**Finding (liveness audit, snappy-moth-330 @ the §5.E wall):** the srv3 install/reconcile cluster is **dead** — `srv3_os_install_reconcile_apply` is reached from *no* `gunbc` subcommand, CLI, or CI path (verified: zero hits in `dag/tools/`, `src/v1/`, `cli_run.rs`); `fleet_converge_cli`'s only reference is a `Srv3InstallDiagnosticObserve => fallback` match arm; srv3 is already installed and the srv* cluster is wound down. The three concrete carriers: `dag/gunbc/srv3_host_effect_script.dag` (whole file), `dag/gunbc/srv3_install_diagnostic_checklist.dag` (whole file), and the `Srv3InstallDiagnosticObserve` realization arm in `host_effect_realize.dag`, plus the `srv3_os_install_reconcile_{,_dry_run,_record_approval,_apply}.dag` scaffold subgraph.

**Why it is NOT an "easy" delete (the gate):** the effects these produce scripts for — `Srv3InstallReconcileObserve`, `DurableApprovalGrantRecord`, `Srv3SolConsoleCapture`, `Srv3InstallDiagnosticObserve` — are **`HostEffect` coproduct variants**. Removing them ripples through *every* total `match` on `HostEffect` (`host_effect.dag`, `host_effect_realize`, `live_deploy/host_effect_script`, `fleet_converge_cli`, `ci_deploy_access_observe`) and the srv3 witness tests. `host_effect.dag` is a **DESIGN-named load-bearing file** (escalate-first). So this is a coordinated coproduct-surgery pass, not a quick `rm`, and it is kept out of the §5.E wall PR deliberately (a different concern; the wall's `retained_*` wraps correctly *count* these dead arms in the interim).

**Acceptance (its own scoped PR):** the srv3 install/reconcile subgraph + its now-orphaned `HostEffect` variants deleted together; every total `match` updated; the `retained_srvn` wraps on those arms removed with them; srv3 witness tests deleted or repointed; green by execution; and the `retained_srvn_takeover_ref` dissolve-count for the srv3 arms drops to zero. Operator authorized the deletion in principle (2026-07-24) but asked it be tracked here and land apart from the wall.

### 5.E — THE ENABLER THAT MUST COME FIRST (close the string sink, or every row above can be faked)

Every §5.A/§5.B row can be **faked by joining argv back into a string** and feeding `shell.Exec.Run(script)` / `ShellOnHost{script}` — sleek-crab #7064 did exactly this (`argv_join(...) + " 2>/dev/null || true"`). As long as that sink is reachable from intent, relocation is the path of least resistance and a brief alone won't stop it. So the wall is **not cleanup-after** — it's the enabler:

- Brand `TransportScript` so it is produced ONLY by `emit(intent, Bash)`/`serialize_bash` (today `transport_script_from_body(body: String)` accepts any string — the porous boundary).
- Make `ShellOnHost{script}` / the runtime-present realization edge take **typed argv (`List<String>`), not `String`** — a hand-join becomes a type error.
- Activate the `host_language_transport_script` lens (inert today, `fail_closed_lockdown.dag`).

> **RULING — wall-first, one PR (operator, 2026-07-24). STOP per-op migration.** The §5.A/§5.B per-op
> migration PRs are paused. Build §5.E's construction wall first, in ONE PR (large PRs fine); §5.B
> resumes on top of it as the *only* writable path. Recorded here so the sequencing is a durable fact,
> not re-litigated per conversation.
>
> **Design correction found by execution while building the wall (snappy-moth-330):** "brand
> `TransportScript`" (bullet 1 above) is **not** a `.dag`-level wall. `TransportScript = String where
> brand("TransportScript")` is a **transparent** brand — `peel_nominal_alias_identity` peels it to its
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
>   ~23 foreign/runtime `shell.Exec.Run(script:)` sites route through counted bridges
>   (`retained_foreign` / `retained_runtime` / `retained_srvn`), each authoring a `reason` + a
>   `dissolves_to` ref — a new bridge call is a conspicuous, counted review event, with two dissolve
>   buckets (srvN-takeover typed effects vs. the `v2.workflow.bash_emit` foreign roster).
> - **The `host_language_transport_script` lens is activated** as a per-PR `ReadsLiveTree` witness
>   consumer over the `shell.Exec.Run` anchor sites (`wall_residue_live_test.dag`) — the backstop for a
>   *raw literal* at a Run position (the lens deliberately stays green on `ComputedApplication`, i.e. the
>   counted bridge calls; construction, not the lens, closes the computed-join class).
> - **REDs prove the wall by execution:** the two documented fakes (#7064's
>   `transport_script_from_body(argv_join(...) + " 2>/dev/null || true")` and the vivid-wolf string-into-edge
>   near-miss), reconstructed as inline sources through `compile_dag_rust_emit_check`, **fail to compile**
>   (`transport_script_wall_compile_red_test.dag`) — not fail review.
> - **Surviving residue (filed, not closed here):** `x as TransportScript` (transparent-brand cast-mint)
>   remains writable anywhere — the wall funnels the sanctioned mint through one
>   `retained_shell_script_to_transport` body, but nothing *prevents* a stray cast in a fresh module. That
>   is not closable by a record; it needs the **`Reference` verb** to govern who may form a base→brand cast
>   edge. Filed as a named row on the node/subtree visibility-grants lane
>   ([node-subtree-visibility-grants.md](node-subtree-visibility-grants.md) §3.1), the first brand with a
>   concrete displaced cost — NOT an ad-hoc seed feature.

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

**P4 has LANDED** (#6572/#6585/#6598 — the `ConvergePlan` effect + `EmitArtifactThenThinRun` transport minted and consumed by fleet_converge; see §2), so the original "delete when P4 lands" criterion is met. The doc's remaining life is the same one §4/§5 name: it dissolves when the **`host_language_transport_script` lens goes live** (§4.F/§5.E) and `program.dag` deletes — i.e. when relocation is unwritable by construction and the per-instance census/plan is redundant. One criterion, consistent across the doc (this reconciles the two triggers per review 41476).
