# BMC media share naming: what was read, and what it rules out (2026-10-27)

Trigger: boot run 40140037654 refused at media attach. Work item `adhoc-37e2c372-ad5`.
Everything below was READ; nothing was written to the BMC, to srv2, or to any host.

## What was claimed, and what the reading found

The brief's premises were checked rather than assumed, and two of them did not survive.

| Claim | Verdict | Evidence |
|---|---|---|
| The BMC's NFS server is `192.168.1.188` | **already false in the corpus** | `mtcollins1_media_share` (gunbc/machine_intake/mtcollins1_boot_run.dag) renders `srv2_lan_ipv4` (gunbc/fleet/fleet_reach.dag), and that row is `192.168.1.201`. The `.188` spelling survives only as a captured receipt: `share_as_of_2026_09_27` (test/claim/machine_intake/megarac_media_convergence_witness.dag), a `MegaRacShareBinding` recording what the controller said on 2026-09-27. |
| "no image available to attach" is the firmware's refusal | **false — it is our own string** | Appears nowhere in the firmware bundle and nowhere in the BMC's served assets. It is our diagnostic, produced by `media_attach_outcome_reason` (gunbc/machine_intake/megarac_boot_diagnostic_bundle.dag) — "the image is not presented AND the controller offers no image available to attach". So the refusal at that boot was produced by OUR gate; "the share points at a dead IP" is a hypothesis, not a reading of that string. |
| The boot converges `media/general` from `mtcollins1_media_share` | **false — it only READS it** | `megarac_op_media_config` (PUT /api/settings/media/general, extdeps/bmc/megarac.dag) has ZERO users outside its own module; `megarac_media_general_put_body` and `megarac_media_general_document` have zero producers. The share row is consumed only as the EXPECTED value of the read-compare in `share_binding_standing` (gunbc/machine_intake/megarac_media_attach.dag), whose disagreement yields `ShareMismatch`, consumed by `presentation_still_served` (same module) to refuse before attach. Editing the row changes what we REQUIRE the controller to already say — it writes nothing. |
| The LAN name resolves | **true, read at the authority** | Direct UDP query to 192.168.1.1:53, not a cache: `srv2.mynetworksettings.com` → rcode 0, one A record `192.168.1.201`. |
| MegaRAC accepts a hostname in `cd_remote_server_address` | **true, FQDN only — and this is the load-bearing finding** | See below. |

## The firmware's own field validator, executed

`extdeps.bmc.megarac` documents the endpoint shape and states the boundary rule: the member set is
the firmware's, ours is which facts get fixed. It does not settle hostname-vs-IP, so the question
was answered from the firmware itself rather than inferred.

The controller at 192.168.1.228 serves its UI bundle unauthenticated (`GET https://192.168.1.228/`
→ 200; `/source.min.js` → 200, 6.6 MB). Inside it, the `cd_remote_server_address` field carries a
client-side validator. It was extracted and **executed** (node, predicate verbatim), so what follows
is observed behaviour, not reading:

| Input | Verdict |
|---|---|
| `192.168.1.201` | ACCEPTED |
| `192.168.1.188` | ACCEPTED |
| `srv2` | **REFUSED** |
| `srv2.mynetworksettings.com` | ACCEPTED |
| `SRV2.MYNETWORKSETTINGS.COM` | ACCEPTED |
| `srv2.mynetworksettings.com.` (trailing dot) | **REFUSED** |
| `srv_2.mynetworksettings.com` | **REFUSED** |
| `localhost` | **REFUSED** |
| `::1` | **REFUSED** |
| `255.255.255.0` | **REFUSED** |

These verdicts are carried in the corpus as data, keyed to the release they were read from:
`extdeps.bmc.megarac.megarac_media_address_field_probe_set`, keyed by
`extdeps.bmc.megarac.megarac_committed_hpm_read_release` -- the release identity DERIVED from the committed
`ipmitool hpm-check` capture (boot run 40082618814), the build the device has run since the 2026-10-02
reflash. The key is that derived identity and not the vendor page label "0.45.3", because production keys
nothing on the label: the redirection table, the media-attach stop and the release capability catalog all key
on the derived grain, and nothing asserts the label names it. The generalization this repo draws from them — a label floor —
is a separate row, `gunbc.host.host_name_resolution.megarac_media_address_rule`, and the witness claims
`the_floor_refuses_nothing_the_firmware_accepted`, `the_floor_is_above_at_least_one_refused_name` and
`the_floor_is_necessary_and_not_sufficient` hold that inference against these rows rather than against the
paragraph below.

The predicate is an IPv4/IPv6 literal whitelist that then exempts anything matching an RFC 1123
fully-qualified name. Three consequences, all modelled in gunbc/host/host_name_resolution:

1. A hostname **is** accepted, so the operator ruling (no IP reliance) is achievable without a DHCP
   reservation — the fallback in the brief is not needed.
2. A **single label is refused**. `srv2` is not a usable value for this field, only the FQDN. Any
   migration that "simplifies" the share to the short name breaks the boot. Carried in the corpus as
   `megarac_media_address_rule` and its `minimum_labels`, which the share gate consults — as a NECESSARY
   condition and not the device's rule: `a.b.c.d` has four labels and was refused too, so the floor may
   refuse cheaply and must never be read as "the field would accept this". Only the firmware's own answer
   says that, at write time.
3. The firmware accepts IPs too, so firmware behaviour cannot enforce the ruling. Enforcement belongs
   on the authorable side — which is what `ResolvableName` is.
4. **The trailing-dot form is refused, so nothing may demand it.** This is worth its own number because the
   module got it wrong first: an early draft of `host_name_resolution_admit` refused any name whose
   qualification was not `FullyQualified`, i.e. any name *without* a trailing dot. Read against this table,
   that rule would have refused `srv2.mynetworksettings.com` — the only hostname spelling the controller
   accepts — and demanded `srv2.mynetworksettings.com.`, which the controller rejects. The arm
   (`NameNotFullyQualified`) has been deleted rather than left unreachable, and qualification stays available
   as data (`resolvable_name_qualification`) for any consumer whose own question needs the no-search-list
   meaning. `parse_host_name` sets `FullyQualified` only for the trailing-dot spelling, so this is a trap the
   type system will not catch on its own: the wrong rule type-checks.

## The DNS question, and why the switch is still blocked

The controller's network settings do have a DNS surface (bundle identifiers: `dns_manual`,
`dns_options`, `dns_priority`, `dns_registration`, `nsupdate`, `mdns`, `dhcp_client_fqdn`, and a
`settings/network/dns` DnsSettings surface), so DNS is configurable — consistent with the brief's
premise that a resolver must be named.

**What it is currently set to is unverified, and that blocks the cutover.** `extdeps.bmc.megarac`
models only `/api/session`, `/api/settings/media/*`, `/api/fan_control/*` — there is no modelled
network/DNS read. And this container holds no BMC credentials (no BMC/mtcollins/srv/nfs/password
environment variables, no credential artifact under artifacts/bmc/, and the credential store is itself
refused-as-of-today — see `credential_acquisition_slots_path`, gunbc/dispatch_credential_acquisition.dag).
So the DNS server list could not be
read, and per DESIGN §4d it is not assumed to be 192.168.1.1. **If it is empty, an FQDN in the share
field cannot resolve and attach fails again** — a name the controller cannot resolve is the
controller's dependency, not ours.

## The other reads

* `srv3.mynetworksettings.com` → **NXDOMAIN**, while srv3 has a LAN address row, `srv3_lan_ipv4`
  (gunbc/fleet/fleet_reach.dag). An address exists and a name does not — the state that made this
  failure possible, in a second host. No LAN name row for srv3 is filed, because there is no
  observation to cite. (Its BMC at `192.168.1.192` is a different interface — see
  `srv3_baseboard.bmc` in gunbc/fleet/fleet_intent.dag — and is not this host's LAN address.)
* `srv2.tailecbe08.ts.net` → **NXDOMAIN at the router**. The tailnet name is not LAN-resolvable, which
  is why "Tailscale names are acceptable for now" cannot be applied to the BMC leg: the mtcollins1
  controller is not a tailnet member — carried as `megarac_bmc_tailnet_membership`
  (`NotOnTailnet`, gunbc/host/host_name_resolution.dag), so the claim has a carrier and a date rather
  than living only in this sentence. Fleet SSH and ops may use the tailnet name; the BMC may not.
* BMC 192.168.1.228 is up and answering (200 on `/`). srv2 at `srv2_lan_ipv4` answers nothing on
  80/tcp from here (connection refused/timeout) — expected, and not a name question.

## What was built as a result, and what deliberately was not

Built (gunbc/host/host_name_resolution.dag, extdeps/dns/resolver.dag, and their witnesses): the
name-vs-address boundary (`ResolvableName` refuses an IP literal), the two typed name spaces
(`NameSpace.Lan` / `NameSpace.Tailnet`, each carrying a parsed `Zone` rather than a domain string),
the cited observation rows (`ResolutionObservation`, `TailnetMembership`), and the pre-attach share
gate `media_share_address_gate` that refuses an unresolvable name — the check whose absence is why
this was discovered at attach instead of at authoring. Why each space exists is recorded as comments
on its arms, not as a field: review 87164 (DESIGN §4c) ruled out the earlier `why: String`.

Deliberately **not** done, per manager ruling msg_10f4c58a:
* The share row is **not** changed to the FQDN, and the gate is **not** wired into
  `megarac_observe_boot_presentation`. Wiring it while the controller still holds the address would
  turn a passing share check into a refusal mid-boot-series. The row change and the one-time
  `media/general` PUT (`megarac_op_media_config`, extdeps/bmc/megarac.dag) land together, after the
  BMC's DNS server setting is read. Declared as `media_share_name_cutover` with a bound
  `DissolutionCondition` trigger rather than as prose.
* No BMC or host write of any kind.

## The IP-literal audit (work item step 2)

Every IPv4 literal in a fleet-facing row, sorted by what it may legitimately be. The operator ruling
removes IPs from *identity*, not from *protocol*, and the distinction is what makes the list decidable:
a literal that names a host is debt, a literal that is a wire value (a bind address, a CIDR, a prefix,
a loopback) is not. Each row states the name space its consumer resolves in, per DESIGN §3 — one host
identity, several typed resolutions, no nicknames.

### Legitimate: not a host identity

| Row | Value | Why an address is the right type here |
|---|---|---|
| `operator_lan_segment.cidr` (gunbc/fleet/fleet_intent_network.dag) | `192.168.1.0/24` | A prefix is not a host. `NetworkSegment` (gunbc/product/network_topology.dag) models `cidr` as `NonEmptyStr`; the replacement cut (gunbc/endpoint_authority_replacement_cut.dag) models it as `CidrAddress`, correctly. |
| `operator_lan_segment.gateway` (gunbc/fleet/fleet_intent_network.dag) | `192.168.1.1` | A next-hop, consumed by netplan. Identity would be wrong: the gateway is whatever the router is, and the route needs the number. |
| `operator_docker_bridge_default_segment` (gunbc/fleet/fleet_intent_network.dag) | `172.17.0.0/16`, `172.18.0.0/16` | Prefixes on an ephemeral bridge. |
| `router_address` (gunbc/host/host_name_resolution.dag) | `192.168.1.1` | Declared here because it is a wire value — the resolver a cited probe queried and the server every `ResolutionCause` names — and nothing else in the corpus carries it. |

### Host identity by IP: the debt, by consumer leg

Each entry states the NAME SPACE its consumer must resolve in, and why that space and not another.
Two rules decide it: an operator or fleet tool that reaches a host over the VPN should resolve the
TAILNET space; a BMC-side leg resolves the LAN space, because the BMC is not a tailnet member.

| Row | Value today | Consumer's leg | Should be |
|---|---|---|---|
| `srv2_host_lan_endpoint.address`, `srv2_bmc_endpoint.address`, `srv2_tailscale_endpoint.address` (gunbc/fleet/fleet_intent_network.dag) | `192.168.1.201`, `192.168.1.195`, `100.73.169.40` | one host, three typed resolutions | The pattern DESIGN §3 wants is already here: ONE `subject` (an `EndpointHost` over a `HostIdentity`) and three typed resolutions over three paths, with no nickname. Same shape is wanted for the LAN address — a LAN resolution row, not an address in a field called address. Note `srv2_tailscale_endpoint.address` carries a CGNAT address while its label says tailscale: the label is a nickname, which §3 rules out. |
| `srv3_baseboard.bmc`, and the four sibling baseboards (gunbc/fleet/fleet_intent.dag) | `BmcEndpoint.host`: `192.168.1.183`, `.184`, `.192`, `.195`, `.247` | a BMC reachable from the LAN | A LAN FQDN, paired with a DNS server on each BMC. This is the fleet-wide version of the mtcollins1 defect: the BMC is already told to reach hosts by number. `BmcEndpoint` (gunbc/fleet/fleet_intent.dag) carries `host: NonEmptyStr` — a string, with no name-vs-address boundary. |
| `srv2_lan_reach_host` (gunbc/fleet/fleet_reach.dag) | via `srv2_lan_ipv4` = `192.168.1.201` | SSH jump through srv2 to srv3 | A LAN name: SSH re-resolves, so the row survives a DHCP move. `Reach` today can name only an address, which is why "the jump works" and "the jump will work tomorrow" are different facts here. Fleet ops should prefer the tailnet path, which `srv3_reach_fabric_intended` models as a tunnel (no address at all). |
| `supermicro_h11ssl_unit_readings[].bmc_address` (gunbc/machine_intake/supermicro_h11ssl_unit_observation.dag), `supermicro_h11ssl_first_contact_receipt.bmc_address` (gunbc/machine_intake/supermicro_h11ssl_first_contact.dag) | `192.168.1.254`, `192.168.1.252` | Redfish read from the LAN | A LAN FQDN, with the BMC's DNS server cited. Same consumer leg as mtcollins1's media share. The 254.254.254.0 entry in fleet_intent_network's docker-bridge rows is a MASK, not a host, and stays a number. |
| `mtjade1_firmware_family_standing()` capture path (gunbc/machine_intake/mtjade1_access_observation.dag) | `.246` inside a captured-JSON path | an observation's provenance | Leave it, and say why: a capture path is a receipt naming the address the observation was taken FROM. Rewriting history to a name would make the receipt unverifiable, which is the opposite of §4d. |
| `bmc-mtcollins1` endpoint row (gunbc/machine_intake/mtcollins1_access_observation.dag) | `192.168.1.228` | the BMC this item exists for | A LAN FQDN. The row is also the only place the BMC's own name is spelled, so its name belongs in a `HostNameResolution` row and the observation should reference the host, not the number. |
| `mtcollins1_media_share.server_address` (gunbc/machine_intake/mtcollins1_boot_run.dag) | `render_ipv4_address(addr: srv2_lan_ipv4)` | the BMC's NFS mount | `srv2.mynetworksettings.com`, gated by `media_share_address_gate`, with readback. The ONLY row on this list that blocks a boot today. Lands paired with the one-time `media/general` PUT (`megarac_op_media_config`, extdeps/bmc/megarac.dag) — declared in `media_share_name_cutover`, not done here. |

Not debt, and deliberately left: `apps/dashboard/.env` (127.0.0.1, 0.0.0.0 — bind addresses); the
docker bridge prefixes above; the 255.255.255.0/255.255.0.0 netmasks; the two capture paths; and the
`192.168.1.192` BMC row, which is a second interface on a host whose LAN name is the thing missing.

Already done elsewhere, and not restated here: the router-endpoint authority replacement is tracked by
`endpoint_authority_cut_identity` (gunbc/endpoint_authority_replacement_cut.dag), and the reach/naming
ruling by `ruling_names` (gunbc/plans/fleet_naming_reach_access.dag). This document adds only the BMC
and media-share rows those do not cover.
