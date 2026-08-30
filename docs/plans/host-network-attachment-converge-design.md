# Host network attachment as a converged fact — retiring the router click-path

**Status:** design, operator-directed 2026-08-27 ("can we figure out how to do this stuff
via fleet converge in the future? i don't really want to poke at my router anymore").

## 1. The displaced cost

Paid on 2026-08-27: moving two DGX Sparks from ethernet to wifi meant hand-editing DHCP
reservations in the CR1000A web UI — delete two entries, wait on a stale dynamic lease the UI
refuses to remove, re-author two static rows against MACs read off the hosts by hand. Each act
is a fleet fact, none is expressible in the model, and the model's record was wrong in a way
nobody could catch from inside the repo — `dgx_procurement.dag` asserted two
`StaticDhcpReservation` rows while the router had been handing out ordinary dynamic leases the
whole time. A fact the repo cannot read is a fact the repo will eventually misstate.

The cost to displace: **a host's address is currently owned by a device the fleet cannot
observe, cannot converge, and cites only through a user-guide PDF.**

## 2. Two paths, and why one is declined

### Path A — the host owns its address (RECOMMENDED)

Take DHCP out of the loop for fleet hosts. A host's address becomes a netplan fact, authored
from the model and applied over the existing `host_effect_apply` path — the transport that
already converges hostname, toolchain, runner slots and build cache. The router's only remaining
involvement is a **one-time** DHCP-pool shrink so the static range cannot collide; after that it
is never touched.

This fits the machinery already in tree:

- `extdeps.netplan.netplan` already models the applier half, including the load-bearing
  fact that presence in `/etc/netplan` is sufficient for application regardless of writer.
- `host_effect_apply` already delivers typed effects to these exact hosts.
- `membership_reconcile<M,K>` is the spine: desired vs observed attachments, keyed by
  `(host, interface)`. Added/Modified → upsert, Removed → teardown-or-refuse. A new medium is
  a new instantiation, not a forked reconcile.
- **No new transport.** §3 permits one as a Realization handler, but the cheapest correct
  move needs none.

### Path B — model the CR1000A admin transport (DECLINED, recorded)

Converge DHCP reservations by driving the router. `Cr1000aAdminTransport` today is
`WebUiManual | UnknownTransport`, so a real handler is the structurally sanctioned addition.
Declined on cost and on §5:

- The CR1000A's local admin API is undocumented. The module's cited authority is a Verizon
  user-guide PDF describing the UI, not an API — any handler would be reverse-engineered from
  a SPA's traffic and **could not be cited**, the grounding requirement `extdeps/` exists to
  hold.
- Firmware-fragile: a Verizon-pushed update can change it with no notice and no version to
  declare.
- Buys strictly less than Path A: the address still lives in a device that is not the fleet,
  and the model still has to transcribe it back.

Recorded rather than left implicit because the next reader will otherwise re-derive it: the
router *is* the obvious place to converge a DHCP reservation, and the reason not to is not
obvious.

## 3. What is missing

Four gaps, in dependency order. None is large; the first is what everything else waits on.

**(a) A per-host interface roster, and its producer.** No model of "srv6 has `enP7s7`
(ethernet, dark) and `wlP9s9` (wireless, up)". `InterfaceObservation` is a shape with **no live
producer** — `host_network_diagnosis.host_network_observation` returns `none` for every host,
and the module says so, counting the whole fleet as an honest deficit. Landing the producer is
not new scope: an existing model is already waiting for it, and its tally falling is the
receipt.

**(b) Wireless in netplan.** `extdeps.netplan.netplan` models the renderer and the interface,
not config *content* — no `addresses:`, no `wifis:`/`access-points:`. The wifi block needs an
SSID and a key. **The PSK is a secret and must be a `SecretRef`**, never an inline literal, on
the reasoning `nvidia_dgx_spark_setup` already applies to the per-unit sticker credential: a
shared home SSID password in source is the FactoryLogin mistake with a different subject.

**(c) MAC grounding.** `MacAddress` is still the anemic `NonEmptyStr where brand` in
`extdeps.dhcp.v4`, with a named dissolution to an unwritten `extdeps/network/mac.dag`. Two MACs
per host makes this load-bearing: the model must say *which interface's* MAC a row binds, and
today it cannot. The 2026-08-27 incident is the witness — a reservation silently kept naming a
dead port.

**(d) A medium axis on the attachment.** `NetworkPortMedium` carries `WirelessLan`, but as a
*hardware catalog* fact (what the box ships with), not an *attachment* fact (what the host is
using). `product.network_topology.NetworkLocality` has `Lan` with no wired/wireless
distinction, so the topology cannot express the change that just happened.

## 4. Staging

- **Phase 0 — read.** Land the `InterfaceObservation` producer over `host_effect_apply`:
  name, permanent MAC, admin state, carrier, medium, addresses. Discharges the counted
  deficit in `host_network_diagnosis` and makes (c) and (d) authorable from measurement
  instead of transcription. No writes.
- **Phase 1 — ground.** `extdeps/network/mac.dag` (hex-octet parse, dissolving the brand);
  medium on the attachment; the roster keyed by `(host, interface)`.
- **Phase 2 — write, wired first.** Netplan `addresses:` content + apply, converged through
  `membership_reconcile`. Wired is the safe pilot: a mistake leaves the box reachable on
  wifi. Requires the one-time pool shrink.
- **Phase 3 — wireless.** `wifis:`/`access-points:` with the PSK as a `SecretRef`.

**The standing hazard:** every phase after 0 can strand a host. These two boxes have no BMC and
both ethernet ports are now dark, so a bad wireless apply is a physical trip. Wireless converge
must land behind the same read-back-and-revert discipline as the rest of the converge path, and
Phase 2's wired-first ordering exists so there is a second way in before the first is edited.

## 5. Acceptance

- Phase 0 RED: a host whose observation cannot be produced answers `UnknownRefused` and is
  **counted**, never defaulted to a plausible interface list.
- Phase 2/3 RED: an attachment whose desired address collides with the router's live DHCP
  pool **refuses** rather than applying — decidable from the pool bound and the desired set,
  so a construction wall, not a post-check.
- The flagship: srv5/srv6's addresses become derived from converged attachment rows, and
  `dgx_procurement.dag`'s reservation rows stop being the address authority — at which
  point the CR1000A rows are observation-only, and the click-path is retired by having
  nothing left to click.
