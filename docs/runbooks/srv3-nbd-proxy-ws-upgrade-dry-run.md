# srv3 nbd-proxy ws-upgrade dry-run (§6 operator gate)

Operator-gated procedure to settle whether OpenBMC **nbd-proxy virtual media** is compiled into
srv3's bmcweb **before** any Layer-2 seed client work or `CapabilityNbdProxyVirtualMedia` row flip.

**Authority:** [srv3 virtual-media design (architecture B)](../plans/srv3-webui-kvm-virtual-media.md) §6;
modeled interface facts in `dag/extdeps/bmc/webui/nbd_proxy.dag`.

## Why this gate exists

Read-only HTTP GET cannot confirm the nbd-proxy ws route: `/nbd/0` and `/vm/0/0` return **404** on
non-upgrade GET (inconclusive — ws routes commonly 404 without an `Upgrade` header). A ws-upgrade
**spawns nbd-proxy** (side effect), so it is operator-gated.

| Outcome | Meaning | Next path |
| ------- | ------- | --------- |
| **Present** | ws-upgrade succeeds; socket stays open soliciting NBD server greeting | Sign off §6(1); separate PR may flip `CapabilityNbdProxyVirtualMedia` on srv3's cited row and start L2 seed client |
| **Absent** | upgrade fails or closes immediately with no nbd-proxy surface | Honest path is existing solver arm: `PxeHttpInstall` (or `FirmwareUpdateThenVirtualMedia` if catalog gains VM-capable firmware) — **not** a fourth improvised shape |

## Hard constraints (do not violate)

1. **Disconnect without serving an export** — do not complete the NBD handshake or send export size;
   no ISO bytes, no virtual media mounted on the host.
2. **No Layer-2 seed client** in this procedure — use generic ws tooling only (`websocat`, `wscat`, or
   equivalent). No gunbc nbd-proxy client, no authored/emitted JavaScript.
3. **Record a receipt** — paste upgrade status, first bytes observed (if any), and disconnect timing
   into the operator sign-off thread. Ambiguous results stay **absent** until reproduced.

## Prerequisites

- Management-network reachability to srv3 BMC: **192.168.1.192** (cited in
  `dag/gunbc/bmc_onboarding.dag`).
- OpenBMC 2.07.00 (srv3 cited row in `dag/extdeps/bmc/capability.dag`).
- BMC credentials: factory `root` / `0penBmc` until rotated (`dag/extdeps/bmc/openbmc.dag`), or
  operator netrc per [BMC Redfish operator access](bmc-redfish-operator-access.md).
- `websocat` or `wscat` on the operator workstation.
- `curl` and `jq` for bmcweb session token.

## 1. Obtain bmcweb session token

bmcweb authenticates **before** routing; the ws-upgrade requires a Redfish/bmcweb session.

```bash
export BMC_HOST=192.168.1.192

# OpenBMC bmcweb JSON login (adjust password if rotated).
TOKEN=$(
  curl -sk -X POST "https://${BMC_HOST}/login" \
    -H 'Content-Type: application/json' \
    -d '{"username":"root","password":"0penBmc"}' \
  | jq -r '.token // empty'
)

test -n "$TOKEN" || { echo "FAIL: no session token from /login"; exit 1; }
echo "token obtained (${#TOKEN} chars)"
```

If login fails, stop — fix auth before probing ws routes.

## 2. ws-upgrade dry-run to `/nbd/0`

**Side effect warning:** a successful upgrade may spawn `nbd-proxy` on the BMC. This is expected and
acceptable for this gate; step 4 ensures no media is mounted.

```bash
# Primary endpoint (cited in nbd_proxy.dag: /nbd/{slot}, slot=0).
# websocat: exit after 5s idle or first inbound frame; do NOT send NBD server greeting.
timeout 8 websocat -v \
  -H "Authorization: Token ${TOKEN}" \
  "wss://${BMC_HOST}/nbd/0" \
  2>&1 | tee /tmp/srv3-nbd0-ws-dry-run.log
```

Alternate with `wscat`:

```bash
timeout 8 wscat -c "wss://${BMC_HOST}/nbd/0" \
  -H "Authorization: Token ${TOKEN}" \
  2>&1 | tee /tmp/srv3-nbd0-ws-dry-run.log
```

TLS uses the BMC self-signed certificate — `-k` / default wss client trust bypass is expected for
OOB management (same as Redfish curl).

### Pass criteria (nbd-proxy **present**)

Any of:

- WebSocket handshake completes (log shows `101` / `CONNECTED` / `opened`).
- Connection stays open for the probe window without immediate error close.
- Inbound binary/text arrives (optional) — BMC nbd-proxy waiting for **server** NBD greeting
  (`NBDMAGIC` / `IHAVEOPT` per `nbd_fixed_newstyle_server_greeting` in `nbd_proxy.dag`).

### Fail criteria (nbd-proxy **absent**)

Any of:

- HTTP **401** / **403** after token step (auth misconfiguration — fix before re-test).
- HTTP **404** on upgrade, or immediate close with no session.
- TLS/connect failure unrelated to auth.

## 3. Optional secondary probe: `/vm/0/0`

Only if step 2 is inconclusive. Same constraints — upgrade only, no export served.

```bash
timeout 8 websocat -v \
  -H "Authorization: Token ${TOKEN}" \
  "wss://${BMC_HOST}/vm/0/0" \
  2>&1 | tee /tmp/srv3-vm00-ws-dry-run.log
```

## 4. Mandatory clean disconnect

- Close the websocket (Ctrl-C / timeout) **without** sending:
  - NBD server greeting bytes
  - export size / transmission flags
  - any `NBD_CMD_READ` payload
- Confirm host did **not** gain a new USB virtual-media device (no installer mount). If uncertain,
  power-cycle is **not** required for this dry-run; note ambiguity in the receipt.

## 5. Operator receipt template

Paste into the §6 sign-off thread (ctrl / dashboard):

```
srv3 nbd-proxy §6 dry-run receipt
- date:
- operator:
- endpoint tested: /nbd/0 [ ]  /vm/0/0 [ ]
- upgrade result: [101 present | 404 | 401 | other]
- connection held open: [yes | no]
- inbound bytes observed (hex prefix if any):
- export served: NO (required)
- reading: [nbd-proxy PRESENT | ABSENT]
```

## 6. Post-receipt actions (not in this PR)

| Reading | Action |
| ------- | ------ |
| **Present** | Operator signs §6(1). Follow-on work (separate PRs): flip srv3 `openbmc_2_07_00_capabilities` to include `CapabilityNbdProxyVirtualMedia`; implement L2 wss+NBD seed client. |
| **Absent** | Do **not** flip capability row or start L2. Solver stays on `PxeHttpInstall` for srv3 (`dag/test/claim/bmc_capability_solve_witness_test.dag` `srv3_install_mechanism_is_pxe_until_dry_run`). Escalate firmware-update path only if operator chooses `FirmwareUpdateThenVirtualMedia`. |

## Related model files

- `dag/extdeps/bmc/webui/nbd_proxy.dag` — cited NBD-over-wss interface shape (L1)
- `dag/extdeps/bmc/capability.dag` — srv3 row (no `CapabilityNbdProxyVirtualMedia` until receipt)
- `docs/plans/srv3-webui-kvm-virtual-media.md` — architecture B design + §6 gate
