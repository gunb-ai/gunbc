srv1 in-band SMBIOS observation -- read-only, 2026-11-11
=======================================================

WHAT THIS IS
  The evidence behind srv1's row in gunbc.fleet.fleet_host_smbios_observation and, through it, in A2
  (gunbc.fleet.fleet_machine_unit_binding). srv1 is an ASRock Rack ALTRAD8UD-1L2T, one of six owned machines
  carrying that fleet board row, and it has a BMC -- but the in-band read is what A2 admits on, and it is the
  read this directory holds. Nothing here was written to srv1: the session read files and ran decoders.

WHO TOOK IT, AND HOW
  Read on srv1 by vivid-koi-387 (the lane's manager), over `ssh briansrls@srv1 'bash -s' < read_smbios.sh` with
  passwordless sudo, rc=0, at 2026-11-11T22:28:21Z. The script is byte-identical to srv4's
  (sha256 12a573676ef2112aa2bc85aefe18be95cf06459cf73c98db48e5ba2c548a2d44 -- the same procedure on both
  machines, which is the point of pinning it), and srv1's transcript is sha256
  fcb92cf4e5d480a84111113a9d9b0dc6973633749b0ad17934fdb2097ac51d09; SHA256SUMS holds both and
  verify_host_smbios_reading_transcript re-checks the transcript's against the record before any field is
  believed. srv1's container cannot reach this path, which is why the capture came through the manager's
  session on the host rather than from this session, and why the principal names them: the record says who
  actually held the keyboard.

WHAT IT READS, AND WHY PRIVILEGE IS IN THE RECORD
  The base-board serial is mode 0400 root under /sys/class/dmi/id, so an unprivileged read is not an observation
  of it -- it is a Permission denied, which is what the transcript prints in the unprivileged line, next to the
  privileged one. srv1 agrees with srv4 on every interesting blank: product_serial and chassis_serial exist
  under 0400 and are EMPTY, product_version/sku/family are blank, and the system (type-1) table prints its own
  Serial Number empty -- so the only serial this board carries is the base-board one, and a reader that grepped
  the whole dmidecode dump for "Serial Number" would have minted a unit key from an empty field.

  srv1's readings: base-board serial NS1NF9000200201 (privileged sysfs attribute and the type-2 decode agree,
  and type2_base_board_serial reads the second out of this file rather than beside it); board ASRockRack
  ALTRAD8UD-1L2T, matching srv1_baseboard.catalog.model_label; kernel hostname srv1; Linux 7.0.0-42-generic
  aarch64; 128 processors; product_uuid (privileged) cf02cb0d-c9f2-43f8-9992-48210b81c503.

WHAT IS OWED, AND WHAT IS NOT CLAIMED
  No BMC FRU read of srv1 was taken, so srv1_bmc_fru_board_serial_observations is empty and
  srv1_bmc_fru_second_source_status reads Pending off that list alone. The out-of-band arm is owed, not agreed,
  and A2 admits on the one admitted reading -- the same shape srv4 landed with, and deliberately so: a second
  BMC row is corroboration of a channel this session has no credential for (bmc-srv1-admin was not reachable
  here), not a box to tick.
