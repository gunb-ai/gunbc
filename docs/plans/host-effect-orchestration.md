# Host-effect orchestration — interface + shell→.dag migration plan

**Status:** design note for sign-off. Co-authored smart-newt-512 (dag-managed-infra) + neat-boar-71 (BMC onboarding). **Neither lane mints the interface until the shape is signed** (operator + bright-stag-194). Until then host effects ride neat-boar's `host_exec()` placeholder seam — *using* it, not minting the interface; every call migrates onto `apply()` in one move once signed.

## 0. Aim (operator, 2026-06-24)

Manage the **whole server lifecycle on `.dag`** — install / manage / decom — coherent and tested-working, moving **off pure shell transport onto real `.dag` orchestration**. Two standing constraints:

- **Minimal central dependencies.** Move toward the simplest representation: the host **bootstraps itself as far as it can** (via BMC or otherwise) — fewer things to manage, no central orchestrator holding SSH keys to every host (the star-topology cost). The end-state is *pull*, not *push*.
- **No ctrl split.** The JS control plane shrinks to zero. Honest metric = **ctrl LOC deleted per responsibility moved**, not lines added anywhere (§7 seed-shrink applied to the control plane).

## 1. The shape (the one thing to sign off)

Both lanes independently converged on this: today's "emit a shell artifact + external runner" and "gunbc-run dispatches `shell.Exec.Run` over SSH" are **not two regimes** — they are realizations of **one interface**, varying along **two orthogonal axes** (DESIGN §3b transport, §3c policy):

```
apply(host: HostIdentity, effect: HostEffect) -> Receipt          # fail-closed, typed, located
   × Transport (§3b):  SshShell | LocalShell | EmitArtifactThenThinRun | RedfishRest | SystemdUnit
   × Policy    (§3c):  OneShotIdempotent | ConvergeToFixpoint
```

**One interface × N transports × N policies.** Dispatch (selecting transport+policy) is *itself* realization → it sits peripheral; only the agnostic effect shape + `apply()` stay central (DESIGN §3 "the dispatch that selects a realization is itself realization").

**Layer split** (keeps the import arrow pointing toward std; std stays product-free per DESIGN open-decision #2):

| Part | Layer | Why |
|---|---|---|
| `EffectShape` (an effect: inputs/outputs/exit semantics) | **std** (`std.effects`, exists) | framework-level, no fleet knowledge |
| `HostEffect`, `apply(host, effect)`, `Receipt`-typed outcome | **product** | references `HostIdentity` / `fleet_intent` (product); homing in std would drag product upward |
| concrete transports (Redfish / ssh / systemd), each citing its real spec, + the dispatch that selects them | **extdeps** | external authorities; dispatch is realization → peripheral |

**Receipt:** generalize the converge lane's already-byte-locked receipt (#5725, fierce-carp) as the *single* `apply()` outcome — do **not** fork a second receipt grammar (§3 nickname trap). Confirm its located+typed core is **policy-agnostic** (carries `OneShotIdempotent` outcomes, not just drift-deltas); if it has converge-specific fields, generalize = common located/typed core + a policy-specific extension, never a parallel type. **Touch it WITH fierce-carp's lock, not around it.**

## 2. Topology: star (push) → self-converge (pull)

The grid above admits the operator's minimal-dependency aim as a *cell*, not a rewrite:

- **Today / interim:** `transport=SshShell, policy=OneShotIdempotent` — a central actor reaches into hosts over SSH. Works, but it's the star: keys, central availability, passive hosts.
- **End-state:** `policy=ConvergeToFixpoint` running **on the host** — the host pulls its content-addressed `.dag` target-state and reconciles locally (systemd timer). No central SSH poker. The central job shrinks to (a) **emit** target state and (b) **kick BMC** for bare-metal bring-up.
- **Bootstrap chain (how a host gets to self-converge with minimal deps):** BMC PXE/UEFI-HTTP → Ubuntu autoinstall → the autoinstall seed plants the on-host self-converge agent (cloud-init/systemd unit) → host self-converges thereafter. The install/BMC lane (neat-boar) *is* the bootstrap; self-converge composes on top of it. This is the same Realization pattern as everywhere else: content-addressed pure-spec → host-effect.

## 3. Scoped, dependency-ordered plan

Each phase names its **displaced cost** (the pain removed) and its **dissolution/migration trigger**.

- **Phase A — mint the interface (model-before-implement).** Declare `HostEffect` + `apply()` + the two handler axes in **product**, building on `std.effects.EffectShape`; transports as extdeps handlers with the dispatch peripheral. Receipt = generalized #5725 (with fierce-carp). Land with a witness that `apply()` over a `LocalShell`/`OneShotIdempotent` cell runs green by execution + a fail-closed control. *Gate: shape signed.* *Displaced cost: the (1)/(2) fork — two paths for one concept.*
- **Phase B — migrate `host_exec()` → `apply(transport=SshShell, policy=OneShotIdempotent)`.** neat-boar's seam is the single migration point; every `host_exec` call moves in one mechanical pass. Install lane behavior unchanged, now expressed on the interface. *Displaced cost: a bespoke ssh seam that can't express non-ssh effects (e.g. BMC).*
- **Phase C — BMC effects as `RedfishRest` handler.** Power/boot-override/reset (already REST, *un-expressible* in an ssh-only executor) become `apply(transport=RedfishRest)`. Proves the N-transport claim with a second real transport. *Displaced cost: BMC ops stranded outside the model.*
- **Phase D — converge lane → `EmitArtifactThenThinRun` handler; begin ctrl deletion.** `host_converge`/`fleet_converge_emit` become one transport handler bound to the same shape; **delete `runner_host_reconcile.mjs`'s hash/fan-out** (first net ctrl deletion) and `defaultContainerResourceCaps` heuristic (cap inherited from the emitted slice). *Displaced cost: ctrl JS owning host-apply; metric = ctrl LOC deleted.*
- **Phase E — self-converge policy (the minimal-topology payoff).** `policy=ConvergeToFixpoint` realized **on-host** (pull target-state + systemd-timer reconcile); autoinstall seed plants the agent. Retire the central SSH-star for steady-state management; central role shrinks to emit + BMC-kick. *Displaced cost: the star topology — keys, central availability, passive hosts.*
- **Phase F — decom as the reverse fold (stubs now).** Extend the lifecycle `FactoryDefault→CredsRotated→OsInstalled→FabricJoined→Drained→Decommissioned` against `fleet_intent` (§4 one procedure, both directions). Land the decom operations (`drain → deregister-from-offers → revoke-creds → secure-erase-drives → BMC power-off/factory-reset`) as **fail-closed stubs** — typed `Unimplemented`/refuse, **never a plausible no-op** (a silent drive-wipe/power-off "success" is the exact §5 trap). Declares the shape, does nothing, can't be mistaken for working. *Implementation trigger: a real decom use case.*

## 4. Coherence + test (the deliverable's proof)

- One interface, one fleet authority (`fleet_intent`), one receipt grammar — the §3 single-authority test: net concepts must not grow by re-invention.
- **End-to-end lifecycle witness** green by execution over the interface (install slice already proven on the seam; re-prove on `apply()` post-migration), with discriminating fail-closed controls (drop an input → typed refusal; decom stub → typed `Unimplemented`, not success).
- ctrl-LOC-deleted tracked per phase (D/E) as the honest progress metric.

## Ownership

- **smart-newt-512** — interface shape + converge-lane migration (Phase A/D), the management/caps + ctrl-exodus side.
- **neat-boar-71** — install/BMC bootstrap + transports (Phase B/C/E bootstrap), continues srv3 install on `host_exec()` until the shape is signed, then migrates.
- Decom (Phase F) co-owned; stubs land with the interface.
