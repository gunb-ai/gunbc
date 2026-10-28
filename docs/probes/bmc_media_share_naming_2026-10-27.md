# BMC media share naming: what was read, and what it rules out (2026-10-27)

Trigger: boot run 40140037654 refused at media attach. Work item `adhoc-37e2c372-ad5`.
Everything below was READ; nothing was written to the BMC, to srv2, or to any host.

## What was claimed, and what the reading found

The brief's premises were checked rather than assumed, and two of them did not survive.

| Claim | Verdict | Evidence |
|---|---|---|
| The BMC's NFS server is `192.168.1.188` | **already false in the corpus** | `mtcollins1_media_share.server_address` renders `srv2_lan_ipv4`, and that row is `192.168.1.201` (gunbc/fleet/fleet_reach.dag:20-42). `.188` survives only as a historical note (fleet_intent_network.dag:250). |
| "no image available to attach" is the firmware's refusal | **false — it is our own string** | Appears nowhere in the firmware bundle and nowhere in the BMC's served assets. It is our diagnostic at gunbc/machine_intake/megarac_media_attach.dag:887-891 ("the image is not presented AND the controller offers no image available to attach"). So the refusal at that boot was produced by OUR gate; "the share points at a dead IP" is a hypothesis, not a reading of that string. |
| The boot converges `media/general` from `mtcollins1_media_share` | **false — it only READS it** | `megarac_op_media_config` (PUT /api/settings/media/general, extdeps/bmc/megarac.dag:93-97) has ZERO users outside its own module; `megarac_media_general_put_body` and `megarac_media_general_document` have zero producers. The share row is consumed only as the EXPECTED value of a read-compare (megarac_media_attach.dag:398-410) whose disagreement yields `ShareMismatch` and a refusal before attach (:2255, :2291). Editing the row changes what we REQUIRE the controller to already say — it writes nothing. |
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

The predicate is an IPv4/IPv6 literal whitelist that then exempts anything matching an RFC 1123
fully-qualified name. Three consequences, all modelled in gunbc/host/host_name_resolution:

1. A hostname **is** accepted, so the operator ruling (no IP reliance) is achievable without a DHCP
   reservation — the fallback in the brief is not needed.
2. A **single label is refused**. `srv2` is not a usable value for this field, only the FQDN. Any
   migration that "simplifies" the share to the short name breaks the boot.
3. The firmware accepts IPs too, so firmware behaviour cannot enforce the ruling. Enforcement belongs
   on the authorable side — which is what `ResolvableName` is.

## The DNS question, and why the switch is still blocked

The controller's network settings do have a DNS surface (bundle identifiers: `dns_manual`,
`dns_options`, `dns_priority`, `dns_registration`, `nsupdate`, `mdns`, `dhcp_client_fqdn`, and a
`settings/network/dns` DnsSettings surface), so DNS is configurable — consistent with the brief's
premise that a resolver must be named.

**What it is currently set to is unverified, and that blocks the cutover.** `extdeps/bmc/megarac.dag`
models only `/api/session`, `/api/settings/media/*`, `/api/fan_control/*` — there is no modelled
network/DNS read. And this container holds no BMC credentials (no BMC/mtcollins/srv/nfs/password
environment variables, no credential artifact under artifacts/bmc/, and the credential store is itself
refused-as-of-today at dispatch_credential_acquisition.dag:98). So the DNS server list could not be
read, and per DESIGN §4d it is not assumed to be 192.168.1.1. **If it is empty, an FQDN in the share
field cannot resolve and attach fails again** — a name the controller cannot resolve is the
controller's dependency, not ours.

## The other reads

* `srv3.mynetworksettings.com` → **NXDOMAIN**, while srv3 holds a standing lease at 192.168.1.192
  (fleet_reach). An address exists and a name does not — the state that made this failure possible.
  No LAN name row for srv3 is filed, because there is no observation to cite.
* `srv2.tailecbe08.ts.net` → **NXDOMAIN at the router**. The tailnet name is not LAN-resolvable, which
  is why "Tailscale names are acceptable for now" cannot be applied to the BMC leg: the mtcollins1
  controller is not a tailnet member. Fleet SSH and ops may use it; the BMC may not.
* BMC 192.168.1.228 is up and answering (200 on `/`). srv2 at 192.168.1.201 answers nothing on 80/tcp
  from here (connection refused/timeout) — expected, and not a name question.

## What was built as a result, and what deliberately was not

Built (gunbc/host/host_name_resolution.dag, extdeps/dns/resolver.dag, and their witnesses): the
name-vs-address boundary (`ResolvableName` refuses an IP literal), the two typed name spaces with
cited observations, and the pre-attach share gate that refuses an unresolvable name — the check whose
absence is why this was discovered at attach instead of at authoring.

Deliberately **not** done, per manager ruling msg_10f4c58a:
* The share row is **not** changed to the FQDN, and the gate is **not** wired into
  `megarac_observe_boot_presentation`. Wiring it while the controller still holds the address would
  turn a passing share check into a refusal mid-boot-series. The row change and the one-time
  `media/general` PUT land together, after the BMC's DNS server setting is read.
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
| fleet_intent_network.dag:57 | cidr `192.168.1.0/24` | A prefix is not a host. `NetworkSegment` in the replacement cut (gunbc/endpoint_authority_replacement_cut.dag:96) models it as `CidrAddress`, correctly. |
| fleet_intent_network.dag:58 | gateway `192.168.1.1` | A next-hop, consumed by netplan. Identity would be wrong: the gateway is whatever the router is, and the route needs the number. |
| roadmap_dashboard_instance.dag | listen_host `127.0.0.1`, `0.0.0.0` | A bind address. `0.0.0.0` means "every interface"; no name means that. |
| fleet_intent_network.dag:64-70 | docker bridge `172.17.0.0/16`, `172.18.0.0/16` | Prefixes on an ephemeral bridge. |

### Debt: a literal naming a host, by the consumer's leg

| Row | Value | Consumer leg | Name space it needs |
|---|---|---|---|
| **mtcollins1_boot_run.dag:216** `mtcollins1_media_share.server_address` | renders `srv2_lan_ipv4` | **BMC dials NFS** | **LAN** — the controller is not a tailnet member; needs `srv2.mynetworksettings.com` and the controller's DNS server read. This is the row the work item turns on. |
| mtcollins1_access_observation.dag:50 `mtcollins1_endpoint.host` | `192.168.1.228` | operator/fleet dials the BMC | LAN (the BMC *is* the LAN device; its own name would come from the router, and today the router has no name for it — unverified, so unfiled) |
| mtjade1_access_observation.dag:611; supermicro_h11ssl_unit_observation.dag:129,179; supermicro_h11ssl_first_contact.dag:137 | `192.168.1.246`, `.254`, `.252` | fleet dials BMCs | LAN, same shape as above |
| fleet_intent.dag:275-295 `BmcEndpoint.host` | `.183 .184 .192 .195 .247` | fleet dials BMCs via Redfish | LAN |
| fleet_intent_network.dag:77-261 `NetworkEndpoint.address` | 14 LAN literals + 2 `100.x` | topology/inventory | LAN for the `192.168.x` rows; the `100.69.18.126`/`100.73.169.40` rows are tailnet addresses and belong to the tailnet space |
| boot_medium.dag:30 `bmc_boot_image_export.target` | **already a name** (`host_by_name`, srv2) | fleet SSHes to srv2 | The name space is **unverified here** — the spelling is the bare host, resolved by whatever stub the calling host runs (tailnet in practice, but not measured from this container, so this row does not claim which). What it establishes is the shape: a name where the share row has a number, which is the precedent the share should follow. It is `PartiallyQualified`, which the resolution admission refuses for a *resolution row* — correct there (an SSH target may ride the local search list; a resolution row must state the answer it expects). |

`fleet_reach` already holds srv2's three resolutions side by side (`srv2_lan_ipv4`, `srv2_tailscale_ipv4`,
`srv2_tailscale_fqdn`), which is the shape DESIGN §3 wants; what it lacks is a **LAN name** beside the LAN
address, and that absence is exactly why the share could only be authored as a number.

### What this audit is NOT

No row above is changed by this PR. Re-typing `NetworkEndpoint.address` to `IpAddress` and
`NetworkSegment` to `CidrAddress` is already planned and owned elsewhere
(gunbc/plans/fleet_naming_reach_access.dag:85-97, whose `replacement_cut` names the root
`gunbc.endpoint_authority_replacement_cut`); the two `fleet_reach_endpoint.dag` literals are an
external provider's public addresses, not host identities, and the tailscale/acl and rfc_5280 literals are
protocol fixtures. Restating those as debt here would be a second authority over a debt already tracked
(DESIGN §2), so they are named once and left alone.
