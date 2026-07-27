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

Five categories. "Genuine emitter" = emits bash that actually runs; "oracle/scaffold" = shell text retained only for a test. *(Corrected 2026-07-26: the oracle/scaffold definition read "`serialize_bash` retained only for a test" — that sidecar is deleted and **no** test retains it, so the definition described an empty class by its mechanism instead of by its role. §4.H is the live statement of this category.)*

### A. Foreign-executor shell — PERMANENT framing (category (a), stays shell)

| site | what | invoked by |
| --- | --- | --- |
| `ci_workflow.dag` | 8 GHA `RunStep.run:` bodies + concat-built floor-peak/cgroup runners (`ci_cgroup_peak_locate_shell`, `ci_floor_peak_{pre,post}_script`) | GitHub Actions (byte-consumes `run:`) |
| `local_tidy_spec.dag` | git pre-push hook, **built by `concat`** in `githooks_pre_push_emit.dag` (`:19-31`) — *not* by a serializer | git (foreign executor). *Corrected 2026-07-26 (`review 43501`): this said "via `serialize_bash`", which is doubly wrong — the sidecar is deleted, and this hook never routed through one. It is a live concat-built foreign-executor site, i.e. a genuine §4 punch-list row: stays shell permanently, but should be **emitted** via `emit(intent, Bash)` rather than concat-assembled.* |
| cron entry lines | (per lane D census) | cron |

These are correct as shell — the executor only understands shell text. Target end-state: emitted through v2 bash rows (the concat-built `ci_workflow` runners carry `…_shell_emit_dissolution_trigger` rows to the `emit(intent,Bash)` path), **not** dissolved to a typed transport. The floor-peak runners are the near-term win (they cite `floor_diff_observe.dag` as their pattern, but they're foreign-executor so they emit rather than `apply()`).

### B. Bootstrap / pre-runtime shell — provisioning window (category (b))

| site | what | genuine? |
| --- | --- | --- |
| `ubuntu_install_media_fetch.dag` | ISO fetch: `curl` mirror loop + `sha256sum -c` + `mv`; `serialize_bash`+`RawLine` | **dissolved** (FLAG B1 below) — in-process typed reconcile, no `ShellProgram`/`RawLine` emitted |
| `ubuntu_seeded_install_media_remaster.dag` | ISO remaster: `xorriso` extract/mkisofs, NoCloud `user-data`/`meta-data`, grub `sed`; heaviest raw shell | **dissolved** (FLAG B1 below) — in-process typed reconcile, no `ShellProgram`/`RawLine` emitted |
| `fleet_converge` fresh-standup arm | `git fetch`/`cargo build`/`cp` greenfield bring-up (3 frontier rows: `orch_construct_procedure`, `…_cmdsubst_assign`, `…_let_assign`) | genuine emitter |

These run before a gunbc runtime exists on the target, so shell is the honest lowest-dependency medium. **Of the three rows above, only the fresh-standup arm still emits** — the two ubuntu-media files dissolved to in-process typed reconcile and emit no shell at all (see their rows and the *Landed* note below; corrected 2026-07-26, since "they emit through the v2 bash rows" read as covering all three). The surviving emitter goes through the v2 bash rows (the If-band + the tier-2 Procedure/Let band, #6566; the arith arm had no live site and is being pruned by the consumer slice). **✅ FLAG B1 SIGNED (operator, 2026-07-14, relayed via calm-ferret-849):** the ISO remaster/fetch dissolve to a **typed target**, not a permanent `ShellPayloadRequired` bootstrap roster entry — these scripts execute on the provisioning host where the gunbc runtime IS present, so under the bash-minimization rule they're runtime-present, not bootstrap; `curl`/`xorriso`/`sed`/`sha256sum` become typed argv ops (`sha256sum` behind the `Sha256Verify` interface shape, `dag/extdeps/tools/sha256sum.dag`) and the NoCloud files become typed `Filesystem.Write`. Consequence: the for/heredoc emit band is never needed and the tier-2 band stays Procedure/Let-only. **Landed:** `ubuntu_install_media_fetch.dag` and `ubuntu_seeded_install_media_remaster.dag` no longer import `extdeps.languages.bash.program` / build `ShellProgram`/`RawLine` — both rewritten to in-process typed reconcile (`observe(path) → Observation`), green-by-execution.

### C. Runtime-present shell — MUST become typed via `host_effect_apply` (the real work)

| site | what | current path | dissolves to |
| --- | --- | --- | --- |
| `fleet_converge_emit.dag` | **dissolved** (slice 2 keystone LANDED: #6572 `ConvergePlan`+`EmitArtifactThenThinRun`+`converge_apply` · #6585 golden shrink 323→21 lines · #6598 `gunbc converge --host` CLI receiver) — the census-time ~275 lines / 12+ fn defs / 4 for-loops / while-read drain / arithmetic / verdict if-elif are gone; the emitter now projects only the fresh-standup bootstrap intent + thin invocation lines | committed `.github/fleet-converge.sh` (21-line thin-run golden), steady-state interpreted in-process | done — `EmitArtifactThenThinRun` (slice 2) — §2 |
| `host_effect_realize.dag` srv3 transport helpers | `srv3_transport_witness_bin_success`, `srv3_transport_test_executable`, `srv3_apt_tool_present`, `srv3_tool_bin_path`, `srv3_chown_directory_to_current_user` still pass command strings to SSH; the old `srv3_host_effect_script` / diagnostic-checklist builders are deleted | typed local/SSH argv operations; operator-deferred with the surviving srv\* actuator graph (§4.D), never polish a deletion-bound subgraph in isolation |
| `host_build_cache_provision_script.dag`, `host_hygiene_{reaper,liveness}_script.dag` | concat-built runtime host mutation/observation bodies | `host_effect_apply_gated` retained-script transport | typed observe/effect operations; operator-deferred A5 |
| `live_deploy/emit.dag` + merged #7298 `ci_deploy_access_emit.dag` | `Pipeline` sequencing is typed, but mutation/preflight operations remain raw `Run.command` leaves; #7298 added concat-built principal/sudo probes | generated deploy scripts invoked from GHA, with gunbc already present | interpret typed host effects before/inside `live_deploy_fold`; the outer GHA thin invocation may emit Bash, the runtime operations may not (§4.J) |
| ~~`host_identity_converge.dag` hostname shell~~ **LANDED #7194** | `SetHostnameCas` carries typed hostname intent | `host_effect_apply_gated` | done — `os.Hostname.Set` local op / typed SSH argv |
| `src/v2/workflow/floor_diff_observe.dag` | typed `git.Core.DiffUnified0`, no `ShellCommand` | interpreter service operation | **target pattern already realized** |
| `host_effect_nbd_proxy_serve.dag` | **`RawLine` body dissolved** (corrected 2026-07-26 — no nbd file carries `RawLine`/`ShellProgram`/`serialize_bash`). Operator ruled 2026-07-14 there is to be **no `trap`/`&`/`$!` `ShellStmt` vocabulary**, so the backgrounding became a systemd transient-unit realize transport | typed effect; residual `WitnessBin` `systemd-run` scaffolding | observe-side port/unit query grounded on cited `extdeps.systemd`/`systemctl` read-back + typed argv dispatch retiring the `systemd-run` scaffold (trigger in-file at `host_effect_nbd_proxy_serve.dag:39`) |

The surviving srv\* dissolution triggers are unanimous: runtime-present raw transport dissolves into typed operations on `host_effect_apply`; the common runner `gunbc.shell_bash_runner.shell_exec_via_bash` (the `bash <<'GUNBC_BASH_EOF'` heredoc framing) dissolves when no caller passes a raw script. The former `srv3_install_diagnostic_checklist_shell_runner_dissolution_trigger` carrier was deleted with that dead reconcile/checklist subgraph; do not cite it as a live row.

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
- The ratchet was **pruned with it** — `plans/shell_emission_model.dag:50` records it as *"the vacuous bash-program importer-count ratchet is pruned with it (resolve fails before the ratchet could fire)"*. Verified: the identifier `bash_program_importer_count` now appears **nowhere in the repository except this document**, so the "baseline = 19" figure was measuring a mechanism that no longer exists.
- `bash_program_emit.dag`, also named here, does not exist either.

The live replacement is `src/v2/extdeps/languages/bash_command_fold.dag` (see §4.G).

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

## 3. The remainder of the ShellProgram→DAG arc

> **Retitled and re-anchored 2026-07-26 (`review 43477`).** This section was framed *"to `program.dag` deletion"*, with deletion as the join the two tracks converge on. **That deletion already happened** — #6831, Phase 0 — so the goal it pointed at is not pending, it is history. The tracks below are still the live work; only the terminus was stale. Do not dispatch anyone to "delete `program.dag`".

Two tracks run in parallel.

### Track 1 — legitimate shell onto the v2 bash rows (bounded)

- **P1 (ready now, no sign-off):** the `ci_workflow` concat-built floor-peak/cgroup runners → `emit(intent, Bash)` via the landed If-band + word support (they carry `…_shell_emit_dissolution_trigger` rows). Foreign-executor, so they *emit*; no `apply()`.
- **P2:** the Procedure/Let emit band for the 3 fresh-standup frontier rows — the one genuinely new *emitter* vocabulary the arc still needs, scoped by the bootstrap census, fail-closed where a construct isn't modeled. *(Tier-2 band LANDED #6566; the consumer slice — route the fresh-standup fragment through `emit(intent,Bash)`, byte-oracle vs `.github/fleet-converge.sh` — LANDED #6573. The former "ubuntu-media files need `for`/heredoc" clause is superseded by the FLAG B1 working default above: ubuntu-media dissolves to typed argv/`Filesystem.Write`, so the for/heredoc band is never built.)*
- **P3:** `local_tidy_spec` pre-push hook + cron lines stay **shell** permanently (foreign-executor roster entries, category (a)). These never dissolve — they're the honest residue. *(Corrected 2026-07-26: this row said they stay `serialize_bash`. They cannot — `serialize_bash` no longer exists. What is permanent is that these sites **stay shell**; the shell they get is emitted through the v2 bash rows like every other foreign-executor site. "Permanently shell" ≠ "permanently on the deleted sidecar".)*

### Track 2 — runtime-present shell onto `host_effect_apply` (the §2 keystone cascade)

- **P4 = the G1+G2+G3 keystone above** (srv1/srv2 subsumption).
- **P5:** the srv3 tails follow the *same* interface. Each `srv3_*_observe_script` becomes a typed `…Observe` effect on `host_effect_apply` (their dissolution triggers already name this); the receipt echoes become typed receipts; `shell_exec_via_bash` (the heredoc runner) dissolves once no caller passes a raw script. The two heavy files: `srv3_install_diagnostic_checklist` is FROZEN/terminal (typed observe effect retires it), `srv3_os_install_actuator_toolchain_ensure` → typed `extdeps.apt`/`curl` argv effects.
- **P6:** `host_identity_converge` drops its `sudo hostnamectl` script for a typed hostname effect (it's already on `apply_gated`). *(P6 Part 2 corrected 2026-07-26: `nbd_proxy_serve_program`'s `RawLine` body is **already gone** — the operator's 2026-07-14 no-`trap`/`&`/`$!` ruling routed it to a systemd transient-unit transport instead of "first-class typed background/trap statements", which are now explicitly **not** a target. What remains is the observe-side read-back grounding, §1.C.)*
- **P7:** `build_step_transport` — **LANDED (#6565, 2026-07-14)**: corruption-probe harness reshaped onto typed `shell.Mktemp`/`Filesystem.Write`/`WitnessBin.Run`, dropping the `verifications_script` `serialize_bash` execution.

### The join — ~~delete `program.dag`~~ **already done (#6831); the real terminus is the wall**

**Superseded 2026-07-26 (`review 43477`).** This section described the arc's terminal step as: importers drain → the `bash_program_importer_count` ratchet hits its floor → `program.dag` + `serialize_bash` delete. **None of that is pending.** The sidecar was deleted outright in #6831 Phase 0 and the ratchet was pruned with it as vacuous (resolve now fails before it could fire); the identifier survives nowhere but this document. So the join is not a deletion the tracks build toward — it already happened, ahead of them.

The v2 bidirectional bash language **is** the single bash authority today. The **Phase-3 wall** that turns *"no sidecar exists"* into *"a hand-built transport string is unwritable"* (§5 construction over validation) has **partially** landed (#7184, operator acceptance 2026-07-24): the free minter is deleted and `RetainedShellScript` — a **record** — is the sole mint **for the `ShellOnHost` realization edge**.

It is **not** the global transport wall. `shell.Exec.Run` still declares `input { script: TransportScript }` over a transparent brand (`dag/extdeps/shell/exec.dag:16,26`), so `String as TransportScript` stays writable outside the record wall — demonstrated in-tree at `src/v2/test/fixture/meta_exec_confinement_scan/leak/plant.dag:6`, and detailed in §4's dissolution trigger and §5's end-state row. So Track 1's **deletion** terminal is behind us; Track 2's **wall** terminal is **not**. What remains is **meta-exec confinement** (§4.F, last row) plus the per-site migration itself.

*(Corrected 2026-07-27, `review 43608`. This read "has also **already landed** … is the sole mint. So Tracks 1 and 2 feed neither a deletion nor a wall; both terminals are behind us." That declared a narrower `ShellOnHost` wall to be the completed **global** transport wall — a §3 violation in which the planning authority overstates a landed scope, and the misdispatch risk is concrete: a worker reading this routes away from the still-open meta-exec confinement work, which is exactly the harm this document exists to prevent. This is the **fifth** site of this same claim corrected in this PR. The first four were fixed while this one survived because each pass swept for the sentences already seen rather than enumerating every sentence that asserts the fact — the standing rule, stated here so it outlives the individual corrections: **enumerate by claim across the whole authority set, then verify each; a correction that fixes only the site under review is not finished.**)*

*(Corrected 2026-07-26, `review 43486`. The previous sentence here — written by me one round earlier while fixing the `program.dag` staleness — said the terminus was to "brand `TransportScript` … and activate the inert lens". Both halves were already dead when I wrote it, for *different* reasons — the brand was **refuted by execution** (transparent brand, §5.E's ruling block), while the lens directive was **overtaken by completion** (#7184 had already activated it). Worth keeping distinct: a refuted plan and a completed one both read as "stale directive", but only the first means the idea was wrong. Fixing a stale directive by writing a differently-stale directive is the failure mode this document keeps reproducing; see §4.G's method note.)*

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
| P6 | `transport_script_from_body` | the (then-porous) `TransportScript` boundary — 26 sites **at `78f43c38`**. *Historical: the minter is now deleted and the boundary is the `RetainedShellScript` record (§4.F). This row records what the probe found then, not a live count.* |
| P7 | `serialize_bash`/`ShellProgram`/`RawLine`/`ShellStmt` | bash-AST emit vocab (emit-internal) **at `78f43c38`**. *Historical, like P6: this vocabulary is now **extinct** — zero construction sites, declaring module deleted (§4.G). This row records what the probe searched for then, not a live surface.* |
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
| ~~`host_effect.dag` · `ReadEffectivePosixPrincipal`, `SudoNopasswdExecuteProbe`~~ **LANDED (#6946)** | ~~`host_effect_deploy_access_probe_script.dag`~~ deleted | `access.PosixEffectivePrincipal`, `sudo.NopasswdExecuteProbe` | **A3 DONE.** Merged #7298 later authored a different raw-string deploy preflight in `ci_deploy_access_emit`; that regression is current §4.J work, not surviving C5 debt. |
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
| `v2.workflow.orchestration_bash_emit_support` | `orch_bash_run`, `orch_bash_do`, `orch_bash_emit_pipeline` | shared emit plumbing (**#7265**; pre-existing four `ci_*_emit` forks migrate in PR2) |
| `v2.workflow.ci_materialization_emit` | `ci_sccache_provider_shell_injection` | GHA `run:` (**LANDED #7265**) |
| `v2.workflow.ci_merge_admission_emit` | `ci_floor_disposition_marker_init_script` | GHA `run:` (**LANDED #7265**) |
| `gunbc.assimilate.bmc_token_federation` | `gcp_token_smoke_script` | GHA `run:` |
| `gunbc.live_deploy.emit` | `expected_live_deploy_apply_script`, `expected_live_deploy_retract_script` | **RECLASSIFIED runtime-present, not an emit-completion receipt.** GHA supplies the outer `run:` medium, but gunbc is already executing and must interpret typed effects; its raw command leaves are the §4.J terminal row. |
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

### 4.F — bottom transport (Phase-3 WALL — **PARTIALLY LANDED** on the `ShellOnHost` edge, operator acceptance 2026-07-24; the `shell.Exec.Run` sink stays open)

> **Trued up 2026-07-26 (`review 43486`).** This table described the wall as *pending* and directed Phase 3 to "brand `TransportScript`" across "26 sites". **The `ShellOnHost` half of the wall landed on 2026-07-24 and branding was proven ineffective by execution** (the `shell.Exec.Run` sink stays open — see the row below) — both facts were already recorded in §5.E's ruling block ~275 lines below, which superseded this table in place without updating it. A worker reading the action column would have executed superseded work. Corrected against the tree.

| surface | status | action |
| --- | --- | --- |
| `transport_script_from_body(body: String)` | **DELETED** from `extdeps.shell.exec`. Not 26 live sites — **zero**. Surviving occurrences are a note recording its replacement (`retained_shell_script.dag:15`), two compile-**RED** controls asserting the deleted-minter fake *fails to compile* (`transport_script_wall_compile_red_test.dag`), and v1-seed Rust detector code | **none — done for the `ShellOnHost` edge.** Its realization takes a counted `RetainedShellScript` record. Direct `shell.Exec.Run` remains open: `retained_foreign` / `retained_runtime` accept a bare body `String`, and an authored `String as TransportScript` cast is still legal pending meta-exec construction confinement |
| `TransportScript` branding | **abandoned deliberately, not pending.** `TransportScript = String where brand(…)` is a *transparent* brand: `peel_nominal_alias_identity` peels it to its base, so a bare or computed `String` flows into a `TransportScript` position with no cast. Branding alone cannot make a hand-join a type error | **superseded on `ShellOnHost`** by the record wall — a `String` cannot fill its record-typed field. It does not wall direct `shell.Exec.Run`; see §5.E's design-correction block |
| `shell_exec_via_bash` (`shell_bash_runner.dag:32`) | heredoc runner scaffold | dissolves when no caller passes a raw script *(still open)* |
| `host_language_transport_script` lens | **LIVE — activated by #7184**, not inert. `wall_residue_live_test.dag:4`: *"the previously-inert … lens is wired as a per-PR live consumer over the `shell.Exec.Run` anchor sites"*, disposition `ReadsLiveTree`. It reds a **new raw literal** at an enrolled Run position, and stays **deliberately green** on `ComputedApplication` (the counted bridge calls) — which is why it never caught the #7064 computed-join fake | **none — done as a literal-blob backstop.** `RetainedShellScript` closes the computed-join class only at the `ShellOnHost` record field; the lens deliberately permits computed applications and the transparent cast path at direct `shell.Exec.Run` stays open. Do not "promote" the lens — that action presumed a brand that does not work and a lens that was already switched on |
| meta-exec module `extdeps.shell.exec` | not walled | module-isolate / symbol-visibility confinement (meta-exec-confinement lane) *(still open — the genuinely remaining Phase-3 item)* |

### 4.G — bash-AST emit vocab (emit-internal — **EXTINCT**, not merely confined)

**Corrected @ 2026-07-26 (snappy-moth-330), in two passes — read the method note below before trusting or editing this section.** The original row read *"in 11 files … + the two ubuntu-media files + `nbd_proxy_serve`"*, which overstated the live surface and cost an audit a wrong prerequisite (see the trap note). The first correction pass narrowed it but still listed seven files as live `ShellProgram`/`serialize_bash` construction sites — **also wrong**, caught in review (`review 43441`), and wrong the same way: every one of those hits is prose. The second pass fixed that but scoped the occurrence count to `.dag` files, missing the frozen `.diff` fixtures (`review 43455`). Re-censused hit-by-hit, unscoped, over `origin/main`:

| vocabulary | live construction sites | where the remaining text occurrences are |
| --- | --- | --- |
| `RawLine` (bash-AST node) | **zero** — tree-wide | **six** occurrences **excluding this document** (see the self-reference note below), none a construction site: one prose string at `ubuntu_seeded_install_media_remaster.dag:43` (a dissolution `reason:` attesting to its own removal), and **five in two frozen `.diff` fixtures** — `src/v1/stage0/testdata/module_grain_affected_{dag_only_6edafbb,v2_only_bb6e656}.diff` (4 + 1). Two of those five are `-` **deletion** lines (`-  RawLine,`, `-      RawLine { text: … }`) — i.e. the recorded removal itself |
| `ShellProgram` / `serialize_bash` / `ShellStmt` | **zero** — the defining module `dag/extdeps/languages/bash/program.dag` was **DELETED** (#6831, Phase 0). There is no bash-AST sidecar left to construct from | prose and frozen fixtures only — `design_document.dag:157`, `DESIGN.md` (its rendered twin), `plans/{shell_emission_model,emission_ingestion_inverse,host_convergence_circuit_residue}.dag`, the two ubuntu-media `reason:` fields, and `src/v1/stage0/testdata/module_grain_affected_dag_only_6edafbb.diff` — plus `srv3_host_effect_apply_witness_test.dag:71`, a witness asserting the string is **absent**. (`lens_module_gate`'s `ProjectionShellProgram` is a different identifier — a lens projection variant, not this type.) |
| `bash_command_fold` / `bash_build` | **live, and this is the actual replacement machinery** — `src/v2/extdeps/languages/bash_command_fold.dag`, consumed by `bash_orchestration_emit.dag`, `dag/tools/build_step.dag` (which constructs plain `Node`, **not** `ShellStmt`), `realization_vocabulary_containment.dag`, and the fold tests | — |
| `nbd_proxy_serve` | **none of it** | (§1.C, §3 P6 and §5.D each carried a pre-dissolution copy of that row; all three corrected in the same pass) |

**Method note, because this row has now been wrong three times and every error had the same cause.** `git grep -l` answers *"which files contain this string"*, which is **not** *"which files construct this thing"*. **Four** distinct classes match the string and none of them is a construction site:

- **(a) prose** — `data …_note: String` / `text:` / `reason:` fields, including dissolution notes whose content says the builder was *removed*.
- **(b) absence-asserting witnesses** — e.g. `!string_contains(s: argv_surface, pattern: "serialize_bash")`, which matches the grep precisely *because* it proves the opposite.
- **(c) different identifiers sharing a substring** — `ProjectionShellProgram` vs `ShellProgram`; `ExprCmdSubst`/`CmdSubstLines` vs the `CaptureSpec.CmdSubst` variant.
- **(d) frozen historical artifacts** — recorded `.diff` fixtures under `src/v1/stage0/testdata/`, which capture the tree as it *was*. These match forever and must **never** be "cleaned up": they are test data, and editing them corrupts the fixture. The sharpest case is a `-` **deletion** line (`-  RawLine,`): it matches a search for the symbol *because it records the symbol's removal* — the strongest available evidence of absence, and the easiest to miscount as presence.

Error history: the original row overstated the surface at "11 files"; the first correction pass listed seven files as live sites when the true count is **zero and the declaring module does not exist**; the second pass fixed that but scoped the occurrence census to `dag/**` and `src/v2/**`, missing class (d) entirely (caught by `review 43455`). **Scope note, so the claim is checkable — and it must exclude this file (`review 43530`).** The counts above are over the *whole repository, all file types*, **minus this document**, which is a census *of* these names and therefore full of them. Stating an unscoped `git grep -- .` was not reproducible: run it and you get a larger number, because this file's own corrections now contribute 11 `RawLine` lines of pure self-reference. The reproducible command is:

```
git grep -o RawLine -- . ":!docs/plans/shell-to-dag-residual-census-and-arc-completion.md" | wc -l   # → 6
```

Two further reproducibility traps worth stating, since a census's whole value is that its numbers can be re-derived: `git grep -c` counts **matching lines** while `git grep -o | wc -l` counts **occurrences** — they differ wherever a line mentions a name twice — and a count taken before a later commit will not match one taken after. The figure above is occurrences, and the per-file split is `ubuntu_seeded_install_media_remaster.dag` 1 + `module_grain_affected_dag_only_6edafbb.diff` 4 + `module_grain_affected_v2_only_bb6e656.diff` 1. Before editing this section again: confirm the *declaring* module still exists, search unscoped, and read each hit rather than counting filenames.

**The second rule, which cost more rounds than the grep rule and generalises past greps: a correction is not done until the text it supersedes is GONE.** Across eight review rounds on this document, the dominant failure was not getting a fact wrong — it was writing the corrected fact *beside* the stale one and leaving both. A reader then has two mutually exclusive instructions and no way to tell which is live, which is strictly worse than the original error, because the stale half now looks reviewed. Instances: §4.F's table kept "brand `TransportScript`" while §5.E already recorded it refuted; §1.A/§1.D kept sibling rows citing deleted vocabulary; and the `TeeTo` finding carried two contradictory **dispositions** in adjacent paragraphs, one directing the very fix the other says was rejected (`review 43513`). So, when correcting:

- **Delete** the superseded text when it is purely a directive — a stale action row is a misdispatch waiting to happen and preserves nothing worth reading.
- **Strike it in place, with the reason**, only when the correction is *illegible without it* — e.g. §5.E's three original bullets, where a verdict like "refuted" means nothing unless you can see what was refuted.
- Never simply append. If you cannot tell which of the two applies, you do not yet understand the correction well enough to write it.

**Third rule, and it is the one that survived longest undetected: say HOW a directive died, because the three ways are not interchangeable.** A stale directive is *refuted* (the idea does not work — never retry it), *completed* (it already happened — do not re-do it), or *partially superseded* (part landed, part is live residue — the dangerous one, because striking it whole retires open work). All three read identically as "this row is out of date", so a single strike-through loses exactly the information a reader needs. `review 43522` caught two instances at once in §5.E: the lens directive was labelled **refuted** when it had been **completed** by #7184, and the typed-argv retype was labelled **landed** when what landed was a *different sink* (`RetainedShellScript.body` is a `String`, not `List<String>`) with the typed-argv path still open. The first would send someone to re-argue a settled design; the second would delete live §5.A/§5.B work from the plan.

Confined by `realization_vocabulary_containment` (LANDED #6854). **No dissolution action, but note the two halves are no longer the same kind of thing:** the bash-AST *sidecar* vocabulary (`ShellProgram`/`serialize_bash`/`ShellStmt`/`RawLine`) is **extinct** — nothing to dissolve and nothing to confine, because the declaring module is gone; only `bash_command_fold`/`bash_build` remains, and that is the replacement machinery proper. This section is now a *negative* result — kept so the extinct names aren't mistaken for construction sites, and so the next reader doesn't re-derive the same false positives from a filename grep.

**⚠ Name-collision trap — do not read this section as covering the bare-line path.** The live "emit one raw line" path is `realize_run_rawline` (`src/v2/std/orchestration_emit.dag:35`), a **method on the `OrchestrationEmitMedium` interface**, realized by `bash_fold_raw_line_target_model` (`bash_command_fold.dag:922`) and reached from `05_emit_orchestration.dag:192` for env-free `Run`s. It merely *echoes the name* of the retired bash-AST `RawLine` node; it is a different thing at a different layer, and it is very much alive. Consequence, recorded because an audit hit it: a non-command `PipelineStep` (e.g. dissolving `orch_bash_comment_line`) needs **only** a new variant wired to this existing medium method — **not** a new grammar row, and **not** a restored `ShellProgram`. Reading §4.G's old "RawLine is confined emit-internal vocab" line as if it described this path is what produced the wrong prerequisite.

### 4.H — oracle / test retainers (NOT live construction — skip)

`live_deploy/emit.dag:448,452` `expected_*_script` (drift-gate oracles), `*_test.dag` fixtures. These are test expectations, not runtime construction; they follow their subject's dissolution.

### Ledger true-up @ 2026-07-26 (§4.A/4.B/4.D, snappy-moth-330) — read this FIRST

Re-censused §4.A, §4.B and §4.D against `origin/main` @ `efe67794cd` by execution over the tree, not by trusting the rows. Four corrections, in descending order of how badly the stale row would mislead:

1. **§4.D is not a dispatchable bucket — it is A5-deferred.** Every `ssh_session_exec(command:)` site is inside an **`srv3_*`** fn (the wound-down srv\* cluster), except `:1169` which is the realization core's own `RetainedShellScript` transport. The old table listed the verbs without the enclosing fns, which made it read like an independent typed-argv bucket. It is not; picking it up would be typed-argv work on a subgraph slated for retirement.
2. **§4.A4 and §4.B are DONE.** `live_deploy/host_effect_script.dag` is deleted and `dag/gunbc/live_deploy/` contains zero `ShellCommand`. This closes "do-not-miss" item 1 (the #7004/#7006 relocation debt), which had been the loudest open warning in this doc.
3. **§4.A1 is DONE, but left two dead concat builders behind** — deleted in this true-up, with the witness that was pinning one of them as contract. See the A1 dead-scaffold note in §4.A; the pattern (a file note claiming "no concat shell strings" while two sat below it, plus a tautological witness conjunct asserting the bypass still had the right shape) is the reusable finding.
4. **One uncensused consumer added** — `fleet_converge_cli.dag:57`'s `ShellCommand` **match arm**, so the terminal type delete has a complete consumer list.

**Net at the 2026-07-26 snapshot:** with bucket D (§4.E/§4.I) in flight, the non-deferred remainder of this arc was meta-exec confinement. That statement is no longer current-main truth: merged #7298 added the runtime-present `ci_deploy_access_emit` raw-`Run.command` preflight described in §4.J, and merged #7303 left source `Run.env` as a live Retry silent-drop. The current owner/trigger roster in §4.J supersedes this snapshot; the srv\* cluster remains operator-deferred.

*(Corrected 2026-07-26, `review 43494`: this "Net" dispatched **`host_language_transport_script` lens promotion** as remaining §4.F work. §4.F now says explicitly **do not** promote that lens — the construction wall landed (#7184) on the `ShellOnHost` edge, and the lens is already **live**, deliberately green on the computed concats that were the actual fake. Same document, opposite dispatch; the wall half of "§4.F wall-green" is discharged, leaving only meta-exec confinement.)*

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
- **§5.B** — the call-the-op migrations are **UNPAUSED** (the `ShellOnHost` wall landed, #7184; the still-open direct `shell.Exec.Run` sink does not re-pause them). D1/D2/D3 (#7192/#7193/#7194) and bucket A discharged the hostname, systemctl-read and clock clusters. **The 2026-07-26 “non-deferred queue is empty” conclusion is superseded by merged #7298:** its deploy-access preflight wraps already-modeled C5 operations back into raw `Run.command` strings at a runtime-present edge. The bounded correction is the first current §4.J row. Outside that regression, only the **operator-deferred srv\* cluster** (4.A5, including 4.D) remains in §5.B.
- **§5.E** — the **transport-script construction wall** — **PARTIALLY LANDED #7184** (the `ShellOnHost` record edge only; the `shell.Exec.Run` sink remains open — see §5.E). Built as a `RetainedShellScript` RECORD edge + free-minter deletion + counted bridges + lens activation + compile-fail REDs (see §5.E ruling block; "brand `TransportScript`" was found non-walling because the brand is transparent). The 2026-07-24 wall-first ruling that paused §5.A/§5.B is therefore discharged.

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
| `ci_deploy_access_emit.dag` · `deploy_access_emit_principal_read_script`, `deploy_access_emit_sudo_execute_probe_script`, `deploy_access_emit_sudo_grant_list_probe_script`, `deploy_access_emit_refusal_echo_step`, `deploy_access_emit_exit_1_step`, `deploy_access_emit_bootstrap_preflight_steps` | `"whoami"`, concat-built `sudo -n` / `sudo -n -l`, raw `echo` / `exit`, and comments placed in `Run.command` | existing C5 principal/execute service ops + a typed grant-list probe op, composed as runtime host effects | **merged #7298 regression, not foreign-executor emit**: a `Pipeline` around strings is still medium-as-string. Owner: `silent-gull-602` corrective follow-up. Dissolve-on: the typed access gate executes before `live_deploy_fold`, this module deletes, and generated deploy scripts contain no embedded authorization probes. |

**Remaining concat-built foreign-executor punch-list:**

#### A — `merge_admission_produce.dag` (5 script surfaces)

| symbol | GHA consumer | status / emit complexity |
| --- | --- | --- |
| `ci_floor_disposition_marker_init_script` | floor opener (via `gunbc_ci_floor_only_script`) | **DONE (#7265)** → `ci_merge_admission_emit` |
| `ci_documentation_only_gate_skip_prefix` | receipt gates, merge gate, selection control | open — medium (nested if/test + cmdsubst) |
| `ci_merge_admission_stamp_script` | merge-admission stamp step | open — low |
| `ci_merge_admission_gate_script` | merge-admission gate step | open — medium |
| `ci_floor_stamp_merge_admission_script` | floor tail | **PARTIAL #7293** — control/capture now emit through typed `ExitStatus`/`IntNe`/`Exit`; remaining raw leaves are `ci_floor_stamp_ambient_exit_command`, `ci_floor_stamp_root_command`, and `merge_admission_stamp_command()` |

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
| `gunbc.tools.cron_tag` · `build_cron_entry_line` | cron |

**Current-main ownership and dissolve triggers after #7293/#7298/#7303 (verified at `5b229afd65`; this is the dispatch roster):**

| row | exact production symbols | owner | dissolve trigger |
| --- | --- | --- | --- |
| Runtime deploy-access regression | the six `ci_deploy_access_emit` symbols in the typed-op table above | `silent-gull-602` corrective follow-up | execute roster-derived typed principal / sudo grant effects before `live_deploy_fold`; delete `ci_deploy_access_emit`; deploy-script goldens lose the embedded probes |
| Merge-admission foreign executor | concat builders `ci_documentation_only_gate_skip_prefix`, `ci_merge_admission_stamp_script`, `ci_merge_admission_gate_script`; raw leaves inside the already-emitted floor tail: `ci_floor_stamp_ambient_exit_command`, `ci_floor_stamp_root_command`, `merge_admission_stamp_command()` | roadmap `2-emit-partition`; **unassigned after wise-crane’s #7303 charter closed** | construct typed orchestration intent, emit through the canonical Bash medium, then delete each concat builder / raw `Run.command` leaf |
| CI materialization foreign executor | `ci_floor_materialization_receipt_gate_script`, `ci_floor_resolve_receipt_gate_script` | roadmap `2-emit-partition`; unassigned | same, with receipt parsing/refusal represented by typed predicates rather than `sed`/test strings |
| CI-spec foreign executor | `ci_release_build_line`, `ci_fmt_gate_line`, `ci_floor_build_verify_script`, `ci_release_bins_pack_script`, `ci_release_bins_unpack_verify_script`, `gunbc_ci_floor_only_script`, `ci_regen_floor_skip_shortcut_script`, `gunbc_ci_regen_floor_only_script`, `gunbc_ci_deploy_invoke`, `gunbc_ci_heal_regen_invoke`, `gunbc_ci_heal_commit_push_script`, `scheduler_invoke`, `scheduler_invoke_with`, `git_fetch_script` | roadmap `2-emit-partition`; unassigned; `ci_spec.dag` is load-bearing | `ci_fmt_gate_line` calls `cargo.Build.Fmt`; the remaining rows become typed intent → canonical Bash emission; old composers delete and `ci.yml` drift+parse stays green |
| Phase-1 orchestration fidelity | `ci_floor_peak_while_cond_command`, `ci_floor_peak_while_body_command`; accepted Retry body `Run.env` | roadmap `6-shell-intent-phase1`; unassigned | model the live While leaves; preserve or explicitly refuse Retry body env; BoundedPoll/N-level either gain a need-driven consumer with faithful lowering or narrow out of scope |
| Already-routed foreign emit, terminal raw leaves | the §4.E exact output roster: `ci_isolate_toolchain_script`, `ci_pin_rustup_default_script`, `ci_selection_control_script`, `ci_cgroup_peak_locate_shell`, `ci_floor_peak_pre_script`, `ci_floor_peak_post_script`, `ci_cargo_eagain_retry_script`, `ci_release_build_script`, `gunbc_ci_run_script`, `ci_sccache_provider_shell_injection`, `ci_floor_disposition_marker_init_script`, `gcp_token_smoke_script`, and `fresh_standup_bootstrap_intent` | roadmap `2-emit-partition`; Phase-1 owns the While/Retry fidelity subset | retain the correct foreign/bootstrap Bash target, but replace ambient `Run.command` strings with modeled `Do{effect}` leaves and delete each raw producer; do not re-home them in another String carrier |
| Permanent foreign media | `expected_githooks_pre_push_sh`, `build_cron_entry_line` | roadmap `2-emit-partition`; unassigned | their **builders** route through typed intent → Bash; the emitted shell artifacts remain permanently because git/cron require that medium |
| Runtime `Run.command` / `ShellCommand` terminal | `live_deploy.emit.deploy_raw` fed by `ensure_apt_package_step`, `apply_systemd_unit_write_step`, `apply_tree_sync_unit_write_step`, `tree_sync_restart_step_with_diagnosis`, `tailscale_serve_apply_step`, `install_d_owned_command`, `deploy_apply_preamble_steps`, `emit_artifact_upsert`, `emit_artifact_teardown`, `emit_deploy_teardown_dependency_refusal`, `apply_intent_from_effects`, `retract_intent_from_effects`; plus `host_effect_plan`'s placeholder and the `HostEffect.ShellCommand` type/total matches | roadmap `2-emit-partition`, coordinated with the host-effect lane | replace `Run.command` / `ShellCommand` with `Do{effect}` typed operations, migrate every total match, then delete the carriers; never count Pipeline wrapping as completion |
| Deferred runtime srv\* | `host_effect_realize` · `srv3_transport_witness_bin_success`, `srv3_transport_test_executable`, `srv3_apt_tool_present`, `srv3_tool_bin_path`, `srv3_chown_directory_to_current_user`; `host_build_cache_provision_script` · six `build_cache_*_body`/wrapper builders; `host_hygiene_reaper_script` · four bodies; `host_hygiene_liveness_script.host_hygiene_liveness_read_body` | operator-deferred surviving srv\* actuator/runtime graph | un-defer a typed-effect migration with its live consumer; the separate dead install/reconcile subgraph is already deleted (#7233/#7237), so do not polish or resurrect it |
| Direct meta-exec construction | `dag/tools/gunbc_ci`, `merge_admission_stamp`, `emit_host_gate`, `host_prelude`; `gunbc.bmc_netboot_serve`; `gunbc.command_runner`; `host_effect_realize.run_shell_transport` — direct `shell.Exec.Run` sites through transparent/counted carriers | node/subtree visibility-grants `Reference` lane; per-site runtime/foreign owners still delete their bodies | confine who may form base→brand/meta-exec edges; `ShellOnHost` is already record-walled, direct `shell.Exec.Run` is not |

**Proposed batching (3 PRs):**

| PR | scope |
| --- | --- |
| **PR 1 (#7265)** | `ci_sccache_provider_shell_injection` + `ci_floor_disposition_marker_init_script` emit migrations; `orchestration_bash_emit_support` shared plumbing; semijoin scaffold dissolved onto block-bodied `if_else` (#7277); witness tests + `realization_vocabulary_containment` roster; regen `ci.yml`. Does not touch `ci_spec.dag` composers beyond import delegation. |
| **PR 2** | merge-admission cluster completion + receipt gates (A/B remaining rows). |
| **PR 3** | `ci_spec` composer migration + `ci_fmt_gate_line` → typed `cargo.Build.Fmt`. **Operator review required** — load-bearing CI generator. |

**PR-2 scope change @ 2026-07-26 (operator ruling, relayed snappy-moth-330; discharged by #7293, 2026-07-27).** The two concats #7265 left standing were about to be recorded here as accepted residue with dissolution triggers. The operator ruled the opposite: *dissolve them now, along with all other instances*. Both are now **LANDED in PR 2a #7293**, not tracked residue; the table records the carrier construction that discharged them:

| residue | site | prerequisite carrier | why it was standing |
| --- | --- | --- | --- |
| `ci_merge_admission_floor_disposition_stamp_command` | `src/v2/workflow/ci_merge_admission_emit.dag:22` | `RedirectSpec.ToFile` (`src/v2/std/orchestration.dag`) — **LANDED #7293 with this live consumer** | `echo X > path` was unexpressible before #7293; the carrier and consumer landed together, never speculatively. |
| `orch_bash_comment_line` | `src/v2/workflow/orchestration_bash_emit_support.dag:24` | `PipelineStep.Comment`, wired to the **existing** `realize_run_rawline` medium method — **LANDED #7293** | A comment routed through `Do{Run{command}}` was a category error, not a missing grammar row — see the §4.G name-collision trap above. |

The governing rule for the rest of the sweep, so "all other instances" stays decidable: **concat-assembled dissolves now; a constant command string waits for arc close.** `"test -f " ++ path` is a concat and earns a typed carrier; `"exit 0"` or `"mkdir -p target"` is a constant and belongs to the `Run.command` → `Do{effect}` migration, not here. Gold-plating constants now is the §6 purity trap.

**Adjacent finding — `CaptureSpec` capture is a §3 dual representation (raised wise-crane-222, corrected and verified snappy-moth-330, 2026-07-26).** Not part of either dissolution above. The finding was first reported as *"TeeTo and CmdSubst are declared but unreachable"*; that is **wrong for `TeeTo`** and the corrected form is what matters:

- **`CaptureSpec.CmdSubst`** — genuinely dead: zero constructors tree-wide. (`ExprCmdSubst` and `CmdSubstLines` are different types.)
- **`CaptureSpec.TeeTo`** — **was live but inert; fixed by merged #7293.**
  - *Before #7293:* `ci_retry_body_run` (`dag/gunbc/ci_spec.dag:224`) constructed `capture: TeeTo { log: "BUILD_LOG" }`, but `orch_retry_step_command` returned only `r.command` and `orch_emit_retry_run` rebuilt the `Run` with `redirect`/`capture` `Absent`. The identical tee semantics were **hardcoded as fixed tokens** at `bash.dag:1093-1133`, so the emitted bytes were right and the *model* was the copy that did nothing — editing `log: "BUILD_LOG"` changed no output. Exactly the tell §5 names: the spec could be edited while the realizer kept doing its own thing.
  - *Current main after #7293:* `ci_retry_body_run` is `redirect: Absent, capture: Absent` — the lying declaration is **deleted** — and the retry path refuses any `Present` redirect/capture instead of emitting while dropping it. The discriminating RED (`ci_merge_admission_emit_test.dag`, `orch_retry_body_tee_to_refused_holds`) proves a retry-body `TeeTo` is rejected. `TeeTo` now has **zero production constructors**; the only constructor is that refusal control.
- **It was a divergence, not one bug:** the ordinary `Run` emit path already refused the same field fail-closed and typed (`^orch_emit_run_capture_unsupported`, `:230`/`:249`), while the retry path silently dropped it. **#7293 closes that** — both paths now refuse.

**Disposition — this `TeeTo` dual-representation defect is DISCHARGED on the current tree by #7293.** Verified after merging current `origin/main`: `ci_retry_body_run` carries `Absent`/`Absent`, a retry-body `TeeTo` rejects, and no production constructor claims the hardcoded template tee is controlled by intent.

- **Fix direction changed, and the change is right:** I originally filed this as "thread `redirect`/`capture` through the retry path". Deleting the lying declaration is better — an inert declaration is worse than an absent one, because it lies.
- **Current honest modelling gap:** the intent layer cannot express "tee this run to a log" at all, leaving the tee solely as fixed tokens in the grammar row — an honest gap rather than a dual representation. **Trigger:** dissolves when the bash rows grow a modelled tee/redirect construct that `Retry` can carry. `CmdSubst`'s deletion rides with that work.
- **#7303 is merged on current main (`5b229afd65`):** it replaces the `Optional<String>` conflation with `Outcome<Run>`, refuses multi-step bodies and distinct redirect/capture cases, validates the template-hardcoded log source, and refuses the 3+ escalation shape whose old template dropped `level1.env`. One separate silent narrowing remains: accepted one/two-level Retry takes the source `Run` but never threads `body_run.env` into `orch_emit_retry_dispatch`.

### Dissolution trigger for §4

**The construction wall this trigger waited on has PARTIALLY landed (#7184, 2026-07-24)** — it record-walls the **`ShellOnHost` realization edge** (`ResolvedHostEffectCell.ShellOnHost { script: RetainedShellScript }`, `dag/gunbc/host_effect_realize.dag:167`) and deletes the free minter `extdeps.shell.exec.transport_script_from_body(body: String)`, so a hand-assembled `String` no longer typechecks *on that edge*, with compile-fail REDs (§4.F, §5.E).

It does **not** close the `shell.Exec.Run` sink. `TransportScript` is a **transparent brand** over `String` (`type TransportScript = String where brand("TransportScript")`, `dag/extdeps/shell/exec.dag:16`) and `shell.Exec.Run` still declares `input { script: TransportScript }` (`:26`) — so `"…" as TransportScript` remains writable from any module, demonstrated live in-tree at `src/v2/test/fixture/meta_exec_confinement_scan/leak/plant.dag:6`. What guards that sink today is **validation, not construction** — the `meta_exec_confinement` scan the fixture feeds, plus `host_language_transport_script` — and DESIGN §5 is explicit that a lens *concedes* the bad state is writable. The code says so itself: `retained_shell_script_wall_note` files the surviving cast-mint's full closure on the node/subtree visibility-grants lane, where the `Reference` verb governs who may cast base→brand.

So *new* instances of the `ShellOnHost` class can no longer be authored; the punch-list is **not** closed at the top.

*(Corrected 2026-07-27, `review 43600`. This read "a raw-string `shell.Exec.Run` / hand-built transport is already unwritable … so *new* instances of this class can no longer be authored, and to that extent the punch-list is closed at the top." **Overstated on both counts** — the wall is real but scoped to `ShellOnHost`, and new instances can still be authored through the brand cast. Declaring the top closed while a writable sink remains is precisely the misdispatch this document exists to prevent, and it is the third "wall landed" claim in this lane to overstate its scope: the recurring error is checking that a carrier exists rather than that every route into it is closed.)*

What the trigger does **not** discharge is the **existing** rows: the wall stops new concats **on the `ShellOnHost` edge**, it does not dissolve the ones already written — and it does not stop new ones at the still-open `shell.Exec.Run` sink. Each remaining row is still discharged the same way — *deletion of the concat*, verified green-by-execution + an injection-RED, never by relocation. This section retires when that list empties, not on any further wall work.

*(Corrected 2026-07-26, `review 43494` then `43501`: this trigger read "folds into the `host_language_transport_script` lens **going live**". Superseded on both counts — the lens **was already activated** by #7184 (§4.F), so the event it waited on has happened; and it was never what closed this class anyway. It is **live but deliberately green** on `ComputedApplication`, redding only a raw literal at an enrolled `Run` position, so it backstops literal blobs while **construction** closes the computed-join class. My first correction here said the lens "is inert by design" — that was itself wrong, and contradicted §4.F in the same edit pass. The two mechanisms are not interchangeable: a lens is *validation*, the `RetainedShellScript` record edge is *construction*, and §5 prefers the second because it makes the state unwritable rather than merely flagged.)*

---

## 5. Method of Action — the bounded path to bash-free user space (calm-ferret-849, 2026-07-22)

**End state:** no user-space `.dag` constructs a shell string. Every "this `.dag` wants to call a bash script" instance resolves to exactly one of four paths below. The `realization_vocabulary_containment` lens (#6854, LIVE) already forbids bash-AST vocab (`ShellProgram`/`ShellStmt`/`serialize_bash`) in user space — the `shell.Exec.Run(script: TransportScript)` sink remains **open** (§5.E). #7184 retyped the **`ShellOnHost` realization edge** to the `RetainedShellScript` record (`host_effect_realize.dag:167`, `retained_shell_script.dag:9-10`) and deleted the free minter `transport_script_from_body`, so a hand-assembled `String` no longer typechecks *there* — but it did not retype `shell.Exec.Run`, which still declares `input { script: TransportScript }` over a transparent brand (`dag/extdeps/shell/exec.dag:16,26`). On the walled `ShellOnHost` edge what is still possible is **conspicuous**, not silent: authoring a counted `RetainedShellScript` row with a hand-built body is the *reviewed* escape, carrying a `reason` and a `dissolves_to` — that visibility is the mechanism, not a leak in it. The `shell.Exec.Run` sink has no such construction gate; it is lens-guarded only, and closing it is the meta-exec confinement milestone (module-isolate the bottom transport, or symbol-level import visibility). *(Rewritten 2026-07-26, `review 43544`. This sentence previously called that sink "the remaining hole", and my first correction **appended** a supersession notice below it instead of rewriting — leaving two mutually exclusive current-state claims in the same paragraph. That misapplied this document's own rule: strike-in-place is for corrections that are **illegible without the original**, and this one is not — the four-path end state survives the rewrite intact, so only the blocker claim needed replacing.)*

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
| C5 access probes | ~~`host_effect_deploy_access_probe_script`~~ **deleted** | **DISCHARGED by C5 #6946**. Merged #7298 later reintroduced an embedded raw-string preflight through a different module; that non-deferred regression is owned in §4.J, not here. |
| surviving srv* actuator/runtime cluster | `host_effect_realize` srv3 command-string helpers, `host_build_cache_provision_script`, `host_hygiene_*` | operator un-defers the surviving actuator graph; the dead install/reconcile subgraph named below is already retired (#7233/#7237) |
| nbd backgrounding | **`RawLine` already dissolved** (corrected 2026-07-26); now a systemd transient-unit transport with `WitnessBin` `systemd-run` scaffolding | observe-side port/unit read-back on cited `extdeps.systemd`/`systemctl` + typed argv retiring the scaffold (trigger in-file; operator ruled no trap/&/$! vocab) |

#### Roadmap item — srv3 install/reconcile subgraph retirement — **LANDED #7233/#7237**

**Finding (liveness audit, snappy-moth-330 @ the §5.E wall):** the srv3 install/reconcile cluster was dead — `srv3_os_install_reconcile_apply` was reached from no `gunbc` subcommand, CLI, or CI path; srv3 was already installed. The audit named `srv3_host_effect_script.dag`, `srv3_install_diagnostic_checklist.dag`, the `Srv3InstallDiagnosticObserve` realization arm, and the `srv3_os_install_reconcile*` scaffold subgraph.

**Landing receipt:** #7233 performed the coordinated load-bearing coproduct surgery: deleted the dead script/checklist/reconcile/SOL files, removed their `HostEffect` variants and realization arms, updated every total match and witness, and removed the corresponding retained-shell frontier rows. #7237 then deleted the orphaned `DurableApprovalGrantRead` variant. Current tree has none of the retired files or variants.

This receipt is deliberately narrower than “all srv3 is gone.” `Srv3NbdProxyServe`, the surviving actuator/toolchain helpers in `host_effect_realize`, build-cache provisioning, and host hygiene are different live/deferred rows with their own triggers above; #7233/#7237 must not be used to mark those complete.

### 5.E — the enabler that had to come first — **PARTIALLY LANDED #7184** (closes the `ShellOnHost` string sink; `shell.Exec.Run` still open)

*(Heading and premise trued up 2026-07-26, `review 43537`. This read "THE ENABLER THAT **MUST COME** FIRST" in the imperative, and the paragraph below was written in the present tense — "can be faked" — as though the sink were still open. The `ShellOnHost` sink is closed, so both are past tense **for that edge** — but the `shell.Exec.Run` sink is **not** closed (transparent `TransportScript` brand, §4.F/§5.E), so the present tense still applies there. (Re-corrected 2026-07-27, `review 43614`.) The §5.A/§5.B rows it gates are the work that remains, not this.)*

Every §5.A/§5.B row **could be** faked at the former `ShellOnHost{script}` edge, and **can still be** faked at direct `shell.Exec.Run(script)`, by joining argv back into a string — sleek-crab #7064 did exactly this (`argv_join(...) + " 2>/dev/null || true"`). While the `ShellOnHost` sink was reachable from intent, relocation was the path of least resistance and a brief alone could not stop it; the same risk remains live at direct `shell.Exec.Run`. So a construction wall was **not cleanup-after** — it was the enabler. The `ShellOnHost` record wall is now built; direct `shell.Exec.Run` remains writable through the transparent cast and counted string-taking bridges, so its construction/meta-exec confinement is still open. The original plan follows, with each bullet's actual fate marked:

> ⚠ **The three bullets immediately below are the ORIGINAL plan, superseded by the ruling + design-correction block that follows.** Kept because the correction is only legible against them. **Each bullet failed differently, and the difference decides what a reader should do with it** — a *refuted* plan must never be retried; a *completed* one simply already happened; a *partially superseded* one still has live residue. Do not action them as written; read on. *(Flagged `review 43486`; classifications corrected `review 43522` — the earlier version lumped bullets 1 and 3 together as "refuted by execution", which was wrong for bullet 3 and is exactly the conflation §4.G's method note warns about.)*

- ~~Brand `TransportScript` so it is produced ONLY by `emit(intent, Bash)`~~ — **REFUTED.** The brand is transparent (`peel_nominal_alias_identity` peels it to its base), so branding cannot make a hand-join a type error. The idea does not work; replaced by the `RetainedShellScript` **record**. Never retry this.
- **Make `ShellOnHost{script}` take typed argv (`List<String>`), not `String`** — **PARTIALLY SUPERSEDED, not landed.** What landed is a *different* sink: `ShellOnHost.script` is now `RetainedShellScript`, a **record whose `body` is still a `String`** (`host_effect_realize.dag:167`, `retained_shell_script.dag:9-10`). That closes the hand-assembled-`String`-at-the-edge hole by construction, but it is **not** the typed-argv retype this bullet describes — the ruling block below says so explicitly: *"the typed-argv `List<String>` path of bullet 2 stays §5.A/§5.B's job — the two are distinct sinks, not one."* **Live residue.** *(Corrected `review 43522`: this previously read "held … **landed**", which marked open §5.A/§5.B work as finished.)*
- ~~Activate the `host_language_transport_script` lens~~ — **COMPLETED (#7184), not refuted.** The lens *was* activated and is live, redding a raw literal at an enrolled `Run` position. What is true is that activation alone was never **sufficient**: it stays deliberately green on `ComputedApplication`, so it did not and could not catch the #7064 computed-join fake — construction closed that class. Do not re-do it; do not read "insufficient" as "wrong".

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
2. **§5.E** — **the construction wall LANDED #7184**, but this step is *not wholly* discharged and the original wording overstated it. What landed: the `RetainedShellScript` **record** edge + free-minter deletion + counted bridges + compile-fail REDs. The branding half was **refuted** (transparent brand). The **typed-argv realization edge is still open** — `RetainedShellScript.body` is a `String`, not `List<String>`, so that path remains §5.A/§5.B work (see §5.E bullet 2). §5.B is already the only writable path for the *retained-script* sink. *(Corrected `review 43522`: previously struck through entirely and marked "LANDED", which retired open work.)*
3. **§5.B** — migrate by construction (call the op), green-by-execution + injection-RED, deleting each concat.
4. **§5.C** — route foreign-executor sites through the bash backend (bounded roster).
5. **§5.D** — un-defer per trigger.

### Receipts

- **Op inventory verified present @ `78f43c38`** (Pass 1 enumeration of every `service`/`operation` under `dag/extdeps/`): `systemd.Systemctl` (8 ops incl. `ShowProperty`/`SetProperty`/`IsActive`), `Clock.Now`, `shell.Which.Check`, `shell.Find.*` (incl. `IsExecutable`/`Dir`), `git.Core.*` (incl. `Show`/`FetchNoTags`), `Filesystem.{Write,Read,Delete,List}`, `apt.PackageManager.Install`, `sleep.Delay.Seconds`, `gunbc.WitnessBin.Run`, `cargo.Build.*`, `sha256sum`/`jq`/`sed`/`grep`/`xorriso`.
- **New ops verified ABSENT @ `78f43c38`**: no `hostname`/`hostnamectl` op, no `id`/`getent` op, no `systemctl list-units` op; `ssh.Session.ExecArgv` absent on main (in flight in C5 #6946).

---

## Dissolution trigger

**P4 has LANDED** (#6572/#6585/#6598 — the `ConvergePlan` effect + `EmitArtifactThenThinRun` transport minted and consumed by fleet_converge; see §2), so the original "delete when P4 lands" criterion is met.

**⚠ Both remaining criteria are ALSO now met — this trigger has fired and needs an operator decision (flagged 2026-07-26, `review 43494`).** The stated condition was: dissolve when the `host_language_transport_script` lens **goes live** *and* `program.dag` **deletes**. Verified: the lens was activated by **#7184** (`wall_residue_live_test.dag:4`) and `program.dag` was deleted by **#6831**. So by its own terms this document should already have dissolved.

It has not, and the honest reason is that the trigger was **mis-specified rather than unmet**. It names two *mechanism* events, but what actually keeps this document alive is the **per-site punch-list** (§4.A–§4.J): the wall stops *new* concats from being written **on the `ShellOnHost` edge** — not at the still-open `shell.Exec.Run` sink — and it does not dissolve the ones already there, and those rows still have owners and open PRs. A correct trigger reads: *dissolves when the §4 punch-list empties* — mechanism landing was necessary, not sufficient.

**No unilateral dissolution here.** Three sessions are actively working rows in this document; deleting it mid-flight would strand them, and re-specifying a dissolution trigger is an operator call, not a census edit. Recorded for that decision.
