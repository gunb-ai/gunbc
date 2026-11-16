H11SSL-i unit 3 -- LAN-side read of the hand-installed unit, 2026-11-15, read-only
==================================================================================

WHAT THIS IS
  The evidence behind gunbc.machine_intake.supermicro_h11ssl_unit3_intake_observation: the network
  sightings for the third Supermicro H11SSL-i in the corpus, the one whose OS the operator installed BY
  HAND (BMC at 192.168.1.249, install NIC ac:1f:6b:e6:75:eb). Nothing in this directory was captured by
  the converge, and nothing here was written to the unit or its BMC: every line is a LAN-side sighting
  made from another machine.

WHO TOOK IT, AND HOW
  Read on srv1, read-only, by smart-crane-730 (this lane's manager) at ~2026-11-15T23:25Z, and reported
  in the dashboard (msg_01a71ff1). This session transcribed that report into this file rather than
  capturing it itself, and says so because the difference matters for what the file can support: it is a
  faithful copy of another session's read, not a transcript this repository produced. The commands named
  in the report were `ip neigh` (the neighbour table) and an ssh probe with a throwaway known_hosts
  file; srv1's own known_hosts was left untouched.

WHAT WAS SEEN
  * 192.168.1.249 -> ac:1f:6b:e6:72:16, state REACHABLE. That MAC matches neither unit1
    (ac:1f:6b:e3:f5:7b, the serving host h11-a16) nor unit2 (ac:1f:6b:e3:f6:22, first-contact rows
    only), so the board is a THIRD unit and not a known one reappearing.
  * ac:1f:6b:e6:75:eb -> 192.168.1.236, state STALE. 192.168.1.236 is already modelled as
    srv9's host LAN endpoint (gunbc.fleet.fleet_intent_network.srv9_host_lan_endpoint), so this pairing
    is either lease reuse of an address srv9 had vacated or a genuine collision, and this read cannot
    tell them apart. No NetworkEndpoint row is written for the unit at that address.
  * The ssh host key met at 192.168.1.236: ED25519 SHA256:rS86bHoHXcFgifCUhrKDi6+vmjgL8GVz+MhHkd/Pb0o.
    srv1's known_hosts held an OLDER key for that same IP, which is what an address reuse looks like
    from the client side. The fingerprint is recorded here as UNCONFIRMED and is deliberately kept out
    of every trust roster; the operator confirming it from the unit's console is what would change that.
  * Inbound ssh to that address refused for every fleet identity tried (briansrls, ubuntu, gunbc,
    admin): the hand install did not seed the fleet's authorized_keys roster. That is a finding against
    gunbc.design_ruling::fleet_authorized_keys_on_every_install_route, and it is carried in the
    integration register (gunbc.integration_concern
    h11ssl_unit3_hand_install_route_seeded_no_fleet_authorized_keys_roster), not argued here.

WHAT THIS DIRECTORY IS NOT EVIDENCE OF
  Not a read of the board. The BMC never reported its own address here -- a neighbour table says what
  the wire carried, not what a controller claims -- so `ipmitool lan print 1` (or Redfish
  EthernetInterfaces) is still owed, and gunbc.machine_intake.bmc_recovery_route
  (the_bind_refuses_an_endpoint_the_bmc_never_reported) is the standing that refuses to bind an endpoint
  a controller never reported. Not a disk reading: the install-disk receipt is a separate post-install
  capture and is NOT in this directory. Not a boot-path capture: the forced-PXE-lands-on-LAN1 account is
  an operator statement, and the SOL transcript that would ground it is owed. Not a BMC credential read:
  the factory-password state is an operator statement, and nothing here attempted a login.
