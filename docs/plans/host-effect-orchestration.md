# Host-effect orchestration — interface + shell→.dag migration plan

**Status:** design note for sign-off. Co-authored smart-newt-512 (dag-managed-infra) + neat-boar-71 (BMC onboarding). **Neither lane mints the interface until the shape is signed** (operator + bright-stag-194). Until then host effects ride neat-boar's `host_exec()` placeholder seam — *using* it, not minting the interface; every call migrates onto `apply()` in one move once signed.

## 0. Aim (operator, 2026-06-24)

Manage the **whole server lifecycle on `.dag`** — install / manage / decom — coherent and tested-working, moving **off pure shell transport onto real `.dag` orchestration**. Two standing constraints:

- **Minimal central dependencies.** Move toward the simplest representation: the host **bootstraps itself as far as it can** (via BMC or otherwise) — fewer things to manage, no central orchestrator holding SSH keys to every host (the star-topology cost). The end-state is *pull*, not *push*.
- **No ctrl split.** The JS control plane shrinks to zero. Honest metric = **ctrl LOC deleted per responsibility moved**, not lines added anywhere (§7 seed-shrink applied to the control plane).

## 1. The shape (the one thing to sign off)

Both lanes independently converged on this: today's "emit a shell artifact + external runner" and "gunbc-run dispatches `shell.Exec.Run` over SSH" are **not two regimes** — they are realizations of **one interface**, varying along **two orthogonal axes** (DESIGN §3b transport, §3c policy):

```
apply(target: NodeControlPlane, effect: HostEffect, policy: Policy) -> Receipt   # fail-closed, typed, located

   NodeControlPlane = HostOs(node)        # in-band  — transport ∈ {SshShell, LocalShell, SystemdUnit, EmitArtifactThenThinRun}
                    | BmcController(node)  # out-of-band — transport ∈ {RedfishRest}
   HostEffect       = ShellCommand{script} | RedfishAction{resource, verb, body} | …   # COPRODUCT of effect-kinds
   Policy (§3c)     = OneShotIdempotent | ConvergeToFixpoint
   Receipt          = located/typed core + poll-until-converged task state (NOT just exit-0)
```

**One interface × N control-planes × N transports × N effect-kinds × N policies.** Dispatch (selecting transport+policy for a (control-plane, effect-kind)) is *itself* realization → it sits peripheral; only the agnostic effect shape + `apply()` stay central (DESIGN §3 "the dispatch that selects a realization is itself realization").

Three refinements proved by driving srv3's BMC live (neat-boar-71), each load-bearing:

- **Target-duality.** A logical node has *two* control planes — in-band host OS (ssh/systemd) and out-of-band BMC (Redfish) — so `apply` targets a **control plane**, not a host address. Pre-OS lifecycle phases (`FactoryDefault → OsInstalled`) are reachable **out-of-band ONLY**: the OS doesn't exist yet, only the BMC does. Both resolve over `fleet_intent` (which already binds `HostIdentity` + `BmcEndpoint` per node).
- **`HostEffect` is a coproduct, not a shell string.** A Redfish op is `BootSourceOverride{target:Pxe, enabled:Once}` then `Reset{ForceRestart}` — typed actions, not bash. A stringly `command` field would be the §3 anemic leaf; ssh's shell-string is *one arm*. Dispatch on (control-plane × transport × effect-kind) makes an **incompatible combination (e.g. `RedfishAction` over `SshShell`) a typed mismatch, unwritable by construction** (§5) — not a silent failure.
- **Async/polled Receipt.** Redfish `Reset` returns before the machine reboots; the real outcome is observed by **polling** (BootSourceOverrideEnabled flips `Once → Disabled` when firmware consumes it). So the Receipt carries poll-until-converged task state. This *reinforces* the policy axis: BMC ops are **intrinsically converge-shaped** even in a "one-shot" call — `ConvergeToFixpoint` is a property of the effect, not just of ssh-vs-drift.

**Layer split** (keeps the import arrow pointing toward std; std stays product-free per DESIGN open-decision #2):

| Part | Layer | Why |
| --- | --- | --- |
| `EffectShape` (an effect: inputs/outputs/exit semantics) | **std** (`std.effects`, exists) | framework-level, no fleet knowledge |
| `HostEffect` coproduct, `NodeControlPlane`, `apply(target, effect, policy)`, `Receipt` | **product** | references `HostIdentity`/`BmcEndpoint`/`fleet_intent` (product); homing in std would drag product upward |
| concrete transports (Redfish / ssh / systemd), each citing its real spec, + the dispatch that selects them | **extdeps** | external authorities; dispatch is realization → peripheral |

**Receipt:** generalize the converge lane's already-byte-locked receipt (#5725, fierce-carp) as the *single* `apply()` outcome — do **not** fork a second receipt grammar (§3 nickname trap). Confirm its located+typed core is **policy-agnostic** (carries `OneShotIdempotent` outcomes, not just drift-deltas); if it has converge-specific fields, generalize = common located/typed core + a policy-specific extension, never a parallel type. **Touch it WITH fierce-carp's lock, not around it.**

## 2. Topology: star (push) → self-converge (pull)

The grid above admits the operator's minimal-dependency aim as a *cell*, not a rewrite:

- **Today / interim:** `transport=SshShell, policy=OneShotIdempotent` — a central actor reaches into hosts over SSH. Works, but it's the star: keys, central availability, passive hosts.
- **End-state:** `policy=ConvergeToFixpoint` running **on the host** — the host pulls its content-addressed `.dag` target-state and reconciles locally (systemd timer). No central SSH poker. The central job shrinks to (a) **emit** target state and (b) **kick BMC** for bare-metal bring-up.
- **Bootstrap chain (how a host gets to self-converge with minimal deps):** BMC out-of-band remote-boot (mechanism-agnostic — **VirtualMedia** or PXE, per what the node's BMC actually exposes) → Ubuntu autoinstall → the autoinstall seed plants the on-host self-converge agent (cloud-init/systemd unit) → host self-converges thereafter. The install/BMC lane (neat-boar) *is* the bootstrap; self-converge composes on top of it. This is the same Realization pattern as everywhere else: content-addressed pure-spec → host-effect. (Ground-truth: srv3's OpenBMC 2.07 exposes UpdateService + VirtualMedia paths but not PXE-over-Tailscale — PXE needs BIOS-net-stack + same-L2, can't cross the overlay — so **VirtualMedia is the Tailscale-agnostic primary** there. The interface is agnostic to which: both are `apply(BmcController, RedfishSystemAction)` arms; mechanism is a §3 realization detail, not an interface fact.)

## 3. Scoped, dependency-ordered plan

Each phase names its **displaced cost** (the pain removed) and its **dissolution/migration trigger**.

- **Phase A — mint the interface (model-before-implement). [LANDED #5756]** `gunbc.host_effect` (coproduct + `NodeControlPlane` + `Policy`) + `gunbc.host_effect_realize` (total dispatch fold, no `_ =>`; `ShellCommand@HostOs` realized over LocalShell grounded from observed exit; `RedfishAction@BmcController` fail-closed `Unimplemented` stub; incompatible cells = typed mismatch). Return reuses `std.realization_reconcile.Reconciliation` (no new receipt). `RedfishSystemAction` arm in `extdeps.bmc.types` (4 variants, co-owned neat-boar). Witness green-by-execution (real `sh` execs) + discriminating RED. *Scaffold: `ShellCommand { script: String }` is a raw-string leaf — see Phase B for its dissolution.* *Displaced cost: the (1)/(2) fork — two paths for one concept.*
- **Phase B — migrate `host_exec()` → `apply(target=HostOs, transport=SshShell, policy=OneShotIdempotent)`. [after neat-boar's srv3 `OsInstalled` milestone]** Sequenced *after* the live install reaches `OsInstalled` so B migrates a **proven-working** seam, not a racing install (neat-boar's ask). The seam is the single migration point; every `host_exec` call moves in one mechanical pass. Install behavior unchanged, now on the interface. **Dissolution trigger (the Phase-A `ShellCommand{script:String}` scaffold):** `host_exec` carries script *strings*, so B migrates them 1:1 onto `ShellCommand{script}` — then `ShellCommand`'s payload dissolves *jointly with* those strings onto the modeled bash AST (`extdeps/languages/bash/program.dag` + `serialize_bash`), symmetric to `RedfishAction` being typed. Forcing the bash-AST payload in A would couple two changes; deferring keeps B a clean 1:1 migration. *Displaced cost: a bespoke ssh seam that can't express non-ssh effects (e.g. BMC); + the raw-shell-string anemic leaf.*
- **Phase C — BMC effects as the `RedfishAction` arm over `RedfishRest`.** `apply(target=BmcController, effect=BootSourceOverride{Pxe,Once})` → `Reset{ForceRestart}`, outcome observed via async poll. This is the transport that **proves the interface isn't ssh-with-extra-steps** — it forces target-duality, structured (non-string) effects, and async receipts into the shape; an ssh-only executor literally *can't* express it. *Displaced cost: BMC ops stranded outside the model — and the only path to pre-OS phases.*
- **Phase D — converge lane → `EmitArtifactThenThinRun` handler; begin ctrl deletion.** `host_converge`/`fleet_converge_emit` become one transport handler bound to the same shape; **delete `runner_host_reconcile.mjs`'s hash/fan-out** (first net ctrl deletion) and `defaultContainerResourceCaps` heuristic (cap inherited from the emitted slice). *Displaced cost: ctrl JS owning host-apply; metric = ctrl LOC deleted.*
- **Phase E — self-converge policy (the minimal-topology payoff).** `policy=ConvergeToFixpoint` realized **on-host** (pull target-state + systemd-timer reconcile); autoinstall seed plants the agent. Retire the central SSH-star for steady-state management; central role shrinks to emit + BMC-kick. *Displaced cost: the star topology — keys, central availability, passive hosts.*
- **Phase F — decom as the reverse fold (stubs now).** Extend the lifecycle `FactoryDefault→CredsRotated→OsInstalled→FabricJoined→Drained→Decommissioned` against `fleet_intent` (§4 one procedure, both directions). Land the decom operations (`drain → deregister-from-offers → revoke-creds → secure-erase-drives → BMC power-off/factory-reset`) as **fail-closed stubs** — typed `Unimplemented`/refuse, **never a plausible no-op** (a silent drive-wipe/power-off "success" is the exact §5 trap). Declares the shape, does nothing, can't be mistaken for working. *Implementation trigger: a real decom use case.*

## 4. Coherence + test (the deliverable's proof)

- One interface, one fleet authority (`fleet_intent`), one receipt grammar — the §3 single-authority test: net concepts must not grow by re-invention.
- **End-to-end lifecycle witness** green by execution over the interface (install slice already proven on the seam; re-prove on `apply()` post-migration), with discriminating fail-closed controls (drop an input → typed refusal; decom stub → typed `Unimplemented`, not success).
- ctrl-LOC-deleted tracked per phase (D/E) as the honest progress metric.

## Ownership

The `HostEffect` coproduct must not fork — arms co-design, the shape mints as one unit:

- **smart-newt-512** — the `ShellCommand` arm + `HostOs` control-plane + central `apply()`/dispatch + Receipt generalization (with fierce-carp); converge-lane migration + ctrl-exodus (Phase A/D), management/caps side.
- **neat-boar-71** — the `RedfishAction` arm + `BmcController` control-plane + bmc extdeps (has the live srv3 evidence + the Redfish spec); install/BMC bootstrap + transports (Phase B/C/E bootstrap). Continues srv3 install on `host_exec()` through `OsInstalled`, then migrates.
- **Co-owned, lands together in Phase A:** the `HostEffect` coproduct, `NodeControlPlane`, and the `apply()` signature — so neither arm is minted alone.
- Decom (Phase F) co-owned; fail-closed stubs land with the interface.

## Dissolution trigger (DESIGN §6)

Delete this doc when the `apply(target, effect, policy) -> Receipt` interface is minted in product (the `HostEffect` coproduct + `NodeControlPlane` co-owned as one unit), every `host_exec()` call is migrated onto it, the BMC `RedfishAction` arm over `RedfishRest` and the converge-lane `EmitArtifactThenThinRun` handler have landed, ctrl host-apply LOC (the `runner_host_reconcile.mjs` hash/fan-out + `defaultContainerResourceCaps`) is deleted, and the on-host self-converge policy + the decom reverse-fold are realized (or their fail-closed `Unimplemented` stubs are witnessed) — at which point the lifecycle is one tested interface and this design note is redundant.
