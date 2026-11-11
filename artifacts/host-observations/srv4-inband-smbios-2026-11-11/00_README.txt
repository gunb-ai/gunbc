srv4 IN-BAND SMBIOS OBSERVATION — 2026-11-11 (gunbc.plans.fleet_machine_authority P3)

WHAT THIS IS
  A read-only observation of what srv4's own firmware reports about its baseboard identity, taken IN BAND: from
  the running operating system, over SSH. It is the fleet lane's hardware observation producer
  (gunbc.fleet.fleet_host_smbios_observation); the out-of-band producer is the BMC FRU read in
  gunbc.machine_intake.arrival_converge's identity_bind_provisional, which consumes a BMC access receipt this read
  does not use. The reading feeds one identity-binding admission, gunbc.fleet.fleet_machine_unit_binding (A2), as
  a gunbc.machine_intake.subject BoardSerialObservation with source BoardSerialSource#HostSmbios — the same
  vocabulary the FRU producer uses, not a fleet-side copy of it.

WHAT WAS READ, AND WHAT IT SAID
  Every value below is what the host printed, not what the fleet expected. Three states are kept apart, because
  they are three different facts and two of them look alike in ordinary output: a field the kernel does not
  expose at all, a field the firmware left EMPTY, and a field that exists but is mode 0400 root so an
  unprivileged read cannot see it. Where a field was unreadable unprivileged, the privileged read is printed
  alongside it rather than replacing it.

    board_vendor   = ASRockRack
    board_name     = ALTRAD8UD-1L2T                     (the board model the standing cross-checks)
    board_serial   = NS1NG4000900022                    (THE UNIT KEY CANDIDATE. Printed identically by the
                                                          privileged sysfs attribute and by the dmidecode type-2
                                                          decode — two read paths over ONE source, not two
                                                          sources. Unprivileged, the same attribute prints
                                                          "cat: /sys/class/dmi/id/board_serial: Permission
                                                          denied", which is why this arm is privileged: the ABI
                                                          makes it 0400 root)
    board_version  = (empty)
    product_name   = ALTRAD8UD-1L2T        product_version = (empty)      product_sku = (empty)
    sys_vendor     = (empty)               product_family  = (empty)
    product_serial = exists, 0400 root, and EMPTY under privilege         (not the system serial)
    product_uuid   = 40c70e28-588a-48e7-ab02-48210b872cdf (0400 root; matches the type-1 UUID)
    chassis_vendor = (empty)               chassis_serial  = exists, 0400 root, EMPTY under privilege
    chassis_asset_tag = (empty)            chassis_type    = 17 (rack mount chassis)
    bios_vendor    = American Megatrends International, LLC.   bios_version = 3.10   bios_date = 02/26/2026
    kernel         = Linux 6.8.0-147-generic aarch64     hostname printed = srv4
    processor_count= 128                   MemTotal        = 526643092 kB  (~502 GiB usable)
    observed_at    = 2026-11-11T21:29:45Z, printed by the host itself.

  THE SERIAL THIS MACHINE CARRIES IS THE BASEBOARD'S. The type-1 (system) table's Serial Number is EMPTY, and so
  are product_serial, chassis_serial and chassis_asset_tag. A reader that greps a whole dmidecode dump for
  "Serial Number" would therefore take the empty type-1 line as the machine's identity — which is why this
  producer reads only the named table (type 2) or its sysfs attribute, and refuses a field present but empty
  (SmbiosFieldBlank -> BoardSerialBlank) instead of binding a fleet to "".

WHO TOOK IT, FROM WHERE, HOW
  gunbc session gentle-moth-338 (P3, hardware observation producers), from the srv1 session host, as
  briansrls@srv4 with the in-tree fleet automation key (~/.ssh/fleet-automation, ssh-ed25519; the private half
  lives on srv1 and is not in this repository). The command was:

    ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new -i ~/.ssh/fleet-automation \
        -o IdentitiesOnly=yes briansrls@srv4 'bash -s' < read_smbios.sh

  srv4 resolved through DNS to srv4.mynetworksettings.com (2600:4040:9d00:4b00:4a21:bff:fe87:2d64). The host key
  was accepted on FIRST USE: gunbc.fleet.fleet_known_hosts_anchor carries no anchor for srv4, so there was no
  pinned key to check against. That is stated rather than hidden — the transcript's identity does not rest on a
  pinned key, and the anchor lane that would pin it is a separate follow-up.

READ-ONLY
  Every command in read_smbios.sh PRINTS: `cat` of files under /sys/class/dmi/id, `dmidecode --type 1` and
  `--type 2` (a decode of tables the firmware already publishes), `hostname`, `uname`, `date`, `nproc`, and
  `head -1 /proc/meminfo`. Nothing is written, enrolled, reflashed, reset or rebooted; no file on srv4 is opened
  for writing, and nothing outside /tmp on the observation side is touched. srv4's sudoers is NOPASSWD: ALL, so
  read-only-ness is a property of WHICH COMMANDS RAN, not of the privilege used — `sudo -n` (which fails rather
  than prompting) appears only where the kernel ABI makes it necessary: several identity attributes are mode 0400
  root (Documentation/ABI/testing/sysfs-devices-dmi), and dmidecode needs privilege to read /dev/mem. An
  interactive escalation therefore could not have happened, and the unprivileged attempts are printed so that the
  privilege actually required is visible in the record rather than asserted by this README.

WHY THE HOST THAT ANSWERED IS srv4 — AND WHAT THIS DOES NOT PROVE
  What corroborates:
    - the kernel printed hostname "srv4";
    - MemTotal 526643092 kB (~502 GiB usable) is consistent with srv4's committed population of eight
      M393A8G40D40-CRB0Q (512 GiB), which was read OUT OF BAND from srv4's own BMC — Redfish MemorySummary at
      192.168.1.195 on 2026-08-22 (gunbc.fleet.fleet_intent::srv4_memory_population_observation_note). The two
      channels agree without either reading the other;
    - the board model, vendor and 128 processors match the rows the fleet holds (extdeps.boards.asrock_rack::
      asrock_altrad8ud_1l2t_catalog, "ASRock Rack ALTRAD8UD-1L2T", and gunbc.fleet.fleet_intent::fleet_intent_cpu,
      Ampere Altra Max M128-30) — and chassis_type 17 says it is in a rack.
  What that corroboration is NOT: the ALTRAD8UD-1L2T row is carried by SIX owned hosts (srv1, srv2, srv3, srv4,
  srv14, srv15), and the arrival lane has captured populated baseboard serials on two of them (srv14
  BR80S6102300077, srv15 BR80S6102300049). Board model, CPU count and memory size therefore cannot single srv4
  out of this fleet — they say the host is one of ours and of this model. The identity claim rests on the printed
  hostname, on the out-of-band memory agreement above, and on NS1NG4000900022 being unknown to the corpus before
  this read (no other row claimed it).
  What it does NOT prove: that this board is not ALSO in another machine we own, or that the name srv4 points at
  the machine the fleet thinks it does. That is W3's failure (two hosts, one board, one of them wrong) and no
  single mechanism rules it out — an in-band read catches the wrong-HOST half (a host that answers as another
  name refuses: HostAnsweredAsAnotherHost) and an out-of-band read catches the wrong-BOARD half. The BMC FRU read
  of the same board is the second source, and it is OWED, not agreed: no bmc-srv4-admin credential was reachable
  from this session (no gcloud / Secret Manager client here). A2 admits on one admitted reading, so the owed read
  does not gate srv4's flip; it is recorded as a typed obligation
  (gunbc.fleet.fleet_host_smbios_observation::srv4_bmc_fru_second_source_frontier), and if it ever contradicts
  this serial, subject.bind_unit_key refuses with BoardSerialContradictory and srv4's cutover stops.

FILES
  read_smbios.sh                        the script piped to the host, verbatim
  srv4_inband_smbios_observation.txt    the transcript the host printed, verbatim
  SHA256SUMS                            digests of both; the transcript's is the evidence the reading cites, and
                                        gunbc.fleet.fleet_host_smbios_observation's
                                        verify_host_smbios_reading_transcript recomputes it and re-parses the
                                        serial and the hostname out of these bytes
