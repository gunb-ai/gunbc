srv2 in-band SMBIOS observation -- read-only, 2026-11-12
========================================================

WHAT THIS IS
  The evidence behind srv2's row in gunbc.fleet.fleet_host_smbios_observation and, through it, in A2
  (gunbc.fleet.fleet_machine_unit_binding). srv2 is an ASRock Rack ALTRAD8UD-1L2T, the second of the three
  in-band-read machines and the FIRST whose out-of-band arm is discharged rather than owed: this directory holds
  both the in-band capture and the BMC FRU capture. Nothing here was written to srv2: the session read files, ran
  decoders, and asked the BMC management controller for its inventory.

WHO TOOK IT, AND HOW
  Read over one ssh hop FROM vivid-koi-387's session on srv1 (the lane's manager):
  `ssh -o BatchMode=yes briansrls@srv2 'bash -s' < read_smbios.sh`, with the stdout redirect held locally so the
  bytes that landed in the file are the bytes the host printed, rc=0, at 2026-11-12T11:16:56Z. This session holds
  no principal on srv2 -- every credential here is refused by it (publickey,password) -- so the record names the
  manager as the principal, the same way srv1's names them for srv1.
  The script is byte-identical to srv4's and srv1's (sha256
  12a573676ef2112aa2bc85aefe18be95cf06459cf73c98db48e5ba2c548a2d44; cmp says all three captures ran the one
  procedure), and srv2's transcript is sha256
  48d1816430d19b7e904a74b83c8317ba586e562b367078da15f3a2cd28bffe9e. SHA256SUMS pins all four files in this
  directory and verify_host_smbios_reading_transcript re-checks the transcript's digest against the record before
  any field is believed.

  NOTE ON HOW THIS PAYLOAD TRAVELLED, because it matters more than provenance decoration. The first copy reached
  this session base64-encoded inside a message and was CORRUPTED IN TRANSIT: gzip failed its CRC and length
  checks, the transcript's digest did not match, and the damage was the convincing kind -- the first twenty-five
  lines (hostname, os, board vendor and name, even the base-board serial) read perfectly, and the file then
  dissolved partway through the type-2 section, with the type-1 section gone entirely. A reader eyeballing it
  would have signed it off. What saved the record was that the two-read-path agreement is COMPUTED from the
  transcript rather than asserted next to it: the deriver found zero 'Serial Number' rows in the type-2 section
  and refused. The bytes here are the ones that came through git instead, where the transfer is content-addressed,
  verified against the digests above before a single field was read.

WHAT IT READS, AND WHY PRIVILEGE IS IN THE RECORD
  The base-board serial is mode 0400 root under /sys/class/dmi/id, so an unprivileged read is not an observation
  of it -- it is `Permission denied`, printed verbatim in dmi.board_serial_unprivileged beside the privileged
  value. The script was run as the unprivileged LOGIN USER and calls sudo -n internally at exactly the four places
  the ABI needs it; it was not wrapped in sudo, because a wrapper would make the unprivileged probe print a value
  instead of the permission error and collapse two distinct facts into one. srv2 agrees with srv1 and srv4 on
  every interesting blank: product_serial and chassis_serial exist under 0400 and are EMPTY (priv.dmi.product_serial
  is present-and-blank, which is a different state from the <unreadable-without-privilege> marker the unprivileged
  line carries), product_version/sku/family are blank, and the type-1 system table prints its own Serial Number
  empty -- so the base-board serial is the only serial this board carries, and a reader that grepped the whole
  dmidecode dump for "Serial Number" would have minted a unit key from an empty field.

  srv2's readings: base-board serial NS1NF9000200179, IDENTICAL across the privileged sysfs attribute read and the
  type-2 decode (type2_base_board_serial reads the second out of this file rather than beside it, and its field
  rule is the arrival lane's -- see the register row it retired into); board ASRockRack ALTRAD8UD-1L2T, matching
  srv2_baseboard.catalog.model_label; kernel hostname srv2; Linux 6.8.0-147-generic aarch64; 128 processors;
  MemTotal 131166616 kB; product_uuid (privileged) cf02cb0d-c9f2-43f8-9992-48210b81c4c4.

DO NOT CONFUSE SRV2 AND SRV1 BY THEIR UUID
  srv1's product_uuid is cf02cb0d-c9f2-43f8-9992-48210b81c503 and srv2's is
  cf02cb0d-c9f2-43f8-9992-48210b81c4c4: these sibling ASRock boards share the first 33 characters and differ only
  in the tail, which is easy to miss and expensive to conflate. They are different machines with different
  base-board serials -- NS1NF9000200201 for srv1, NS1NF9000200179 for srv2 -- and the unit key comes from the
  board serial, not the UUID. srv2 also differs from srv1 and srv4 in memory (131166616 kB against their
  526630644 kB) while printing the same kernel string as srv4, so neither the os line nor the core count is
  what ties a capture to a machine; the printed hostname, the board row and the serial are.

WHAT IS READ OUT OF BAND, AND WHY srv2 IS NOT LIKE THE OTHER TWO
  The BMC FRU was read in-band over KCS from srv2 itself: `sudo -n ipmitool fru print 0` (sha256
  befa386348857b2416680142de9bdf1622cd996f7918027c709827fe188f3fe5) and `sudo -n ipmitool mc info` (sha256
  e33df908fb12e92ac224431fa66cedee8b40041378777bdd16d4780277af6d41), both rc=0. The FRU reports Board Mfg
  ASRockRack, Board Product ALTRAD8UD-1L2T, and Board Serial NS1NF9000200179 -- the same serial the two in-band
  paths printed. The management controller reports Firmware Revision 2.07, IPMI 2.0, Manufacturer ASRock Rack
  Incorporation, at srv2_baseboard.bmc (192.168.1.184, Redfish).
  So srv2_bmc_fru_second_source carries a real gunbc.machine_intake.subject BoardSerialObservation on BmcFru and
  the shared bmc_fru_second_source_status reads DissolutionFired off that list -- whereas the same call on
  srv1_bmc_fru_second_source and srv4_bmc_fru_second_source, whose lists are empty because no FRU read of those
  boards was ever taken, still reads Pending. One rule, three machines, opposite answers, and the difference is
  purely data: srv2 is the first machine in this lane whose unit identity stands on two independent channels
  agreeing rather than on one reading and an honest gap.
