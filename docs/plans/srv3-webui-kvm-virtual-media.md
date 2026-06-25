# srv3 nbd-proxy virtual-media install — design for sign-off (architecture B)

**Status:** design-for-review. No seed implementation until §6 is signed.
Lane: `sharp-stag-30` under neat-boar-71 (BMC onboarding). DESIGN refs: §3 (interface ≠ transport ≠
policy; dispatch is realization → peripheral), §4 (one grammar; modeled bytes not authored strings),
§5 (fail-closed; green-by-execution; no fabricated success), §6 (model just-in-time; price in
displaced cost).

## 0. History — what was rejected, and the pivot

Three shapes were rejected before this note, all on the same boundary — **no task-specific JavaScript
may be authored OR emitted**:

1. a hand-written 247-line Playwright `.mjs` runner (#5749);
2. "emit the runner from the model" — still **embedded JS** out of the model (`op_page_js`, dec00d4abe);
3. (this note's own prior draft) a generic `playwright-runner` transport + a `.dag` kernel of WebUI
   click-steps. Refuted on a deeper ground than "is the runner generic enough": **the WebUI is not the
   protocol.** The browser KVM "attach virtual media" button is *sugar* over a documented, non-browser
   wire protocol — OpenBMC's nbd-proxy. Automating the UI to drive a protocol is the N×M-adapter trap
   (§3): one extra moving part (a headless browser) standing in front of a wire we can speak directly.

**Architecture B (operator decision):** speak the protocol. OpenBMC ships a **non-browser reference
client** — `openbmc/jsnbd` (the `nbd-proxy` binary; `nbd.js` is its browser twin). The honest
realization is a **direct wss+NBD client in the Rust seed** — a legitimate transport realization in
exactly the sense `ureq` realizes HTTP and `sh` realizes `shell.Exec`, NOT UI automation. No browser,
no Playwright, no authored JS.

## 1. The protocol (grounded from jsnbd + nbd.js + docs/designs/virtual-media.md)

OpenBMC "proxy mode" virtual media is **NBD carried inside a secure WebSocket**, with the roles
inverted from the usual NBD deployment:

- The **BMC is the NBD _client_** (it exposes `/dev/nbdX` locally and mounts it to the host as a USB
  CD/MSD). **Our program is the NBD _server_** — it serves the ISO's bytes.
- Transport: the BMC's bmcweb exposes a **WebSocket upgrade** at the proxy endpoint (historically
  `/nbd/<N>` or `/vm/<slot>/<idx>`, gated by Redfish session auth). Once upgraded, the socket is a
  **bidirectional byte pipe**; NBD frames flow over it. wss = TLS (self-signed BMC cert → trust-pin).
- NBD dialect (what `nbd-proxy`/`nbd.js` actually speak): **fixed-newstyle handshake**, server-side:
  - server greeting: `NBDMAGIC` (`0x4e42444d41474943`) · `IHAVEOPT` (`0x49484156454f5054`) ·
    handshake flags `FIXED_NEWSTYLE(1) | NO_ZEROES(2)`;
  - client option haggling: only **`NBD_OPT_EXPORT_NAME`** is honored (nbd.js sends exactly this);
  - export reply: 64-bit **export size** + transmission flags incl. **`NBD_FLAG_READ_ONLY`** (the ISO
    is read-only); with `NO_ZEROES`, no padding.
  - transmission phase: request magic `0x25609513`, reply magic `0x67446698`; commands
    `NBD_CMD_READ(0)` (the only one we service), `NBD_CMD_WRITE(1)`/`NBD_CMD_TRIM(4)` → **`EPERM`**
    (read-only export, fail-closed), `NBD_CMD_DISC(2)` → clean teardown.

This is a small, **closed, fully-specified** wire — not a heuristic surface. That is exactly why it
belongs in the seed as a modeled realization rather than behind a browser.

## 2. Read-only srv3 probe — findings (192.168.1.192, factory `root`, GET only)

Run under the operator's GET-only constraint (no media-mount, no ws-upgrade — an upgrade *spawns*
nbd-proxy, which is a side effect, so it is operator-gated, see §6). bmcweb authenticates **before**
routing (401 for any path incl. nonexistent), so all existence checks are **authenticated** GETs.

| Probe | Result | Reading |
| ----- | ------ | ------- |
| `/redfish/v1/Systems/system/VirtualMedia` | **404** ResourceNotFound | Redfish `InsertMedia` surface absent |
| `/redfish/v1/Managers/bmc/VirtualMedia` | **404** ResourceNotFound | legacy-HTTPS/CIFS *BMC-mounts-a-share* surface absent |
| `/redfish/v1/Managers/bmc` | 200; has `GraphicalConsole{KVMIP}`, `SerialConsole`, `Oem/OpenBmc` | KVM **video** present; no VirtualMedia member |
| `/nbd/0`, `/vm/0/0` (non-upgrade GET) | 404 | **inconclusive** — ws routes commonly 404 without an `Upgrade` header *and* the feature may not be built |
| `/` , `/index.html` (static WebUI) | Content-Length 0 / 404 | no static WebUI served to cross-check the endpoint |
| port 80 | closed | — |

**Disambiguation answer (parent's question):** the Redfish-driven mount — *both* `InsertMedia`
(proxy/stream) and the simpler **legacy HTTPS/CIFS share-mount** — is **definitively OUT** on srv3's
2.07.00 firmware (collection 404 under both Systems and Managers). So the §1-cheaper "plain Redfish
REST call, BMC pulls a remote URL over Tailscale" path that parent hoped for **does not exist here**;
the Tailscale-reachability question for that path is therefore **moot**.

**New doubt this raises (must be settled before seed code):** the nbd-proxy ws endpoint could not be
confirmed read-only. `/nbd/0` 404s on a non-upgrade GET — indistinguishable from *absent*. Combined
with *no static WebUI served* and *no Redfish VirtualMedia*, there is a live possibility that **this
bmcweb build has the graphical KVM video but NOT the nbd-proxy virtual-media feature compiled in**. If
so, architecture B has no surface either, and the honest install path collapses to
`FirmwareUpdateThenVirtualMedia` (flash a VM-capable build) or `PxeHttpInstall` — which is exactly
what the capability solver already encodes. **Settling this requires one operator-gated ws-upgrade
dry-run** (upgrade `/nbd/0`, observe whether nbd-proxy starts / a server-greeting is solicited, then
disconnect *without* serving an export → no media mounted). That is the §6 gate.

## 3. The §3 framing: nbd-proxy is a *transport*, the install order is *policy*

The host-effect interface (`gunbc.host_effect`, #5756) already models the agnostic shape:

```
apply(target: NodeControlPlane, effect: HostEffect, policy: Policy) -> Reconciliation
   NodeControlPlane = HostOs(node) | BmcController(node)
```

"Mount this ISO as a virtual CD and boot-once from it" is **one agnostic BMC effect-kind**. Its
realizations are transports, *one of N*, selected by dispatch (which is itself realization → peripheral):

- `RedfishInsertMedia` (the `RedfishAction` arm) — **unavailable** on srv3 (§2);
- **`NbdProxyServe`** (new) — the wss+NBD seed client serving the ISO while the BMC's nbd-proxy mounts it.

Per §3's three separable facts: **(a) interface shape** = "serve an export of N bytes, read-only, until
disconnect" (extdeps owns this — it's NBD's contract, cited to the NBD protocol doc + jsnbd); **(b)
transport** = wss-to-`/nbd/0` carrying fixed-newstyle NBD (a Realization handler, the seed client);
**(c) business policy** = which BMC, which export slot, the boot-once + power-cycle ordering around the
mount — a *workflow* fact in `gunbc/`, never in extdeps. The capability solver
(`os_install_mechanism.solve_install_mechanism`) is the dispatch that picks `NbdProxyVirtualMediaInstall`
**because** `CapabilityVirtualMedia` (Redfish) is false and `CapabilityNbdProxyVirtualMedia` is true.

This is the principled win and why B beats the browser draft: the protocol client is the cell
`(BmcController × NbdProxyServe × VirtualMediaBoot)` of the existing host-effect grid — no headless
browser standing between the model and the wire.

## 4. Layers (import arrow points toward std)

| # | Artifact | Layer | Content |
| - | -------- | ----- | ------- |
| 1 | `extdeps/bmc/webui/nbd_proxy.dag` **(new)** | extdeps | **Cited** NBD-over-wss interface shape: the fixed-newstyle handshake fields, transmission framing, read-only export semantics, and the bmcweb ws-upgrade endpoint — as structured facts citing `openbmc/jsnbd` (`nbd-proxy`, `nbd.js`) and the NBD protocol doc + `docs/designs/virtual-media.md`. Carries the formal `ExternalAuthority` citation for openbmc/jsnbd. **No JS.** |
| 2 | seed realization (Rust) | seed | The wss+NBD **client** binary: TLS-pin → ws-upgrade → fixed-newstyle server handshake → serve `NBD_CMD_READ` from the ISO, `EPERM` writes, clean `DISC`. A transport realization like `ureq`/`sh`; **fixed and task-agnostic** (serves *an* export; knows nothing of "install"). Shrinks toward a `.dag` realization as the seed does. |
| 3 | `gunbc/nbd_proxy_virtual_media_install.dag` **(new)** | product/workflow | The install **kernel** = policy: pick slot, open the BmcController effect, serve the ISO export, set boot-once, power-cycle, reconcile. A sequence of modeled effect calls (the `apply(...)` form), zero hand-shell. |

## 5. Witness (DESIGN §5 — green-by-execution)

- **Model-level (lands first, no hardware):** the capability solver witness — already green 8/8 — pins
  srv3 → `NbdProxyVirtualMediaInstall`, with discriminating REDs (a `CapabilityVirtualMedia`-true row
  dispatches to `VirtualMediaInstall` instead; an update-only row → `FirmwareUpdateThenVirtualMedia`;
  bare → `PxeHttpInstall`). This is the dispatch half and it is **done**.
- **Protocol-level (seed, no hardware):** a loopback test — the seed client serves a tiny fixture
  "export" to a local NBD *client* (or a recorded `nbd.js` handshake fixture); assert the exact
  server-greeting bytes + a discriminating RED (a `NBD_CMD_WRITE` must get `EPERM`, not silent accept;
  drop the export size → typed refusal, not a zero-length serve). Green-by-execution, no srv3.
- **Live (operator-gated):** the §6 ws-upgrade dry-run, then a real serve that mounts the ISO and boots
  the installer. Behind the §6 sign-off + the operator's live-mount gate.

## 6. The ONE thing that must be signed before I write seed code

The capability/dispatch modeling (Layers-0 of the grid, the 8/8 witness, this note) is **done and
merge-ready in #5773**. Architecture B's *seed client* (Layer 2) must **not** start until:

1. **An operator-gated ws-upgrade dry-run confirms nbd-proxy is actually present on srv3's 2.07.00
   bmcweb** (§2's new doubt). Read-only GET cannot settle it; an upgrade *spawns* nbd-proxy, which is a
   side effect and so is operator-gated. If it is absent, B has no surface and the honest path is the
   solver's `FirmwareUpdateThenVirtualMedia` / `PxeHttpInstall` arm — not a fourth improvised shape.
2. **Confirmation that a seed-resident wss+NBD client is the accepted realization origin** (the
   `ureq`/`sh` analogy), i.e. the protocol client may live in the Rust seed and shrink toward `.dag`
   like the rest of the seed — distinct from the rejected task-specific JS, which it is (no JS at all).

I will not write the seed client until both are signed, because guessing the boundary is exactly what
got the prior attempts rejected.

## Dissolution trigger (DESIGN §6)

Delete this note when the layer-1 nbd-proxy interface shape + the seed client + the layer-3 kernel land
green-by-execution and `NbdProxyServe` is dispatched-to from the host-effect interface — at which point
the carrier (`.dag` facts + witnesses + the seed realization) is the authority and this note is redundant.
