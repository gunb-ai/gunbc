ken (host-e2d4b7, Supermicro H11SSL unit3) -- GPU read, 2026-11-18

WHAT THIS DIRECTORY IS
  Read-only output taken from the machine the operator calls ken, the third H11SSL-i in the fleet
  (host-e2d4b7, BMC 192.168.1.249, install NIC ac:1f:6b:e6:75:eb). It is the receipt for the three
  NVIDIA Tesla T10 rows this branch adds: the board asset identities and serials in
  gunbc.fleet.fleet_physical_inventory, the device observations in gunbc.fleet.gpu_device_observation,
  and the hand-applied 100 W cap intents in gunbc.serving.gpu_power_state_policy.

HOW IT WAS TAKEN
  One admitted, read-only ssh session to the LAN address this unit answers at (192.168.1.236), as the
  unprivileged ubuntu user, with the session's host key checked against the pinned entry already in
  this repository's known_hosts handling. No sudo was used for any of the reads below. The commands
  were: `systemctl list-unit-files` filtered to the GPU/power units, `ls /etc/systemd/system`,
  `cat` of the t10-power-cap unit, the ken-strata unit and the cap script, `systemctl status` of the
  cap unit and of ken-strata, `cat` of each GPU function's sysfs vendor/device/class attributes
  (domain-prefixed paths), the same cat attempted with un-prefixed paths (it failed; the failure is
  in unit_read1.txt and was NOT cleaned out of the receipt), `nvidia-smi -L`, `nvidia-smi -q -d POWER`,
  and `nvidia-smi --query-gpu=pci.bus_id,uuid,serial`.

  ONE THING IS DISCLOSED, NOT HIDDEN: the admission for that session included the command
  `sudo -n true`, whose output ("yes") was printed and which the admission did NOT authorize. Nothing
  privileged was run through it. It is recorded here because a receipt that quietly omits an
  over-scoped command is the kind of record that makes the next reader trust more than was granted.

FILES
  unit_read1.txt          systemd unit-file listing filtered to GPU/power units, the contents of
                          /etc/systemd/system, whether nvidia-smi is present, and the `lspci` GPU
                          functions. Establishes that t10-power-cap.service and ken-strata.service are
                          enabled, and which PCI functions carry GPUs (01:00.0, 81:00.0, c1:00.0).
  unit_read2.txt          the t10-power-cap.service unit file, the ken-strata.service unit file, the
                          gpu-manager path check, `nvidia-smi -L`, and the sysfs vendor/device/class
                          read for each function: 0000:01:00.0, 0000:81:00.0 and 0000:c1:00.0 all read
                          vendor=0x10de device=0x1e37 class=0x030200. THIS IS THE PCI-ID EVIDENCE: 1e37
                          is extdeps.gpu.nvidia_t10's pci_device_id, so the catalog binding on the asset
                          rows stands on a read and not on the operator's report alone. It also carries
                          ken-strata.service's [Unit] section, whose Requires=t10-power-cap.service and
                          After=... t10-power-cap.service edges make the cap unit load-bearing for a
                          running service -- the fact the cap register row turns on.
  unit_read3.txt          the t10-power-cap.sh script (its header quotes the operator's 2026-11-15
                          instruction and asserts the unit's three-card population with slot-to-bus
                          pairing), then `nvidia-smi -q -d POWER`, then ken-strata's status and journal
                          tail (which is where the 192.168.1.236:18081 listening address appears). No
                          journal lines for the CAP unit were taken, and `systemctl status` warned that
                          some journal files were not opened for lack of permission. THE POWER NUMBERS ARE HERE: for each of the three GPUs
                          (00000000:01:00.0, 00000000:81:00.0, 00000000:C1:00.0) the read is
                          Min Power Limit 100.00 W, Max Power Limit 150.00 W, Default Power Limit
                          150.00 W, Current Power Limit 100.00 W. The `N/A` lines that follow each
                          block are the instantaneous-draw fields of the same `GPU Power Readings`
                          section (this card does not report them); they are NOT the limits, and no
                          row here reads them as such. The script's own EXPECT=3 loop and its
                          fail-unless-exactly-three-readback are what pins the cap as applied by a
                          unit rather than assumed.
  gpu_identity_query.txt  `nvidia-smi --query-gpu=pci.bus_id,uuid,serial` -- the three UUIDs and board
                          serials the asset and observation rows carry.

WHAT THIS DIRECTORY DOES NOT ESTABLISH
  - It does not confirm which machine answers at 192.168.1.236. The lease table read naming which MAC
    holds that address, an in-band address read on the installed OS, and a host key confirmed from this
    unit's own console are all still owed, and are carried as the read_obligation on
    gunbc.machine_intake.supermicro_h11ssl_unit3_intake_observation::
    supermicro_h11ssl_unit3_install_nic_address_standing.
  - The SSH host key that answered at that address is deliberately NOT committed here (it would read as
    a pinned key for a machine whose identity is exactly what is in question). Its line digest is
    de480d24b235bd853ee69937eaf6ca75de86cbe641fa485d10538a8021d719b1 so the sighting can be re-matched
    without the fleet holding a key it has not confirmed at a console.
  - Whether the operator's 100 W bound is a SUSTAINED-draw bound or a peak bound is UNSTATED; the cap
    intents record it as CappedQuantityUnstated, and an operator answer is what changes that.
  - Who wrote t10-power-cap.sh, or when, is not established: the file's header quotes an operator
    instruction, which is not authorship. Likewise the script's slot-to-bus pairing (SLOT4 01:00.0,
    SLOT6 81:00.0, SLOT2 c1:00.0) is a statement ON THE MACHINE, corroborated by the operator's own
    2026-11-17 session naming the third card in SLOT2 (c1:00.0), but it is not a read through any
    modeled route and no modeled route pairs a bus id to a chassis slot.
  - No board power envelope for the Tesla T10 is claimed: extdeps.gpu.nvidia_t10 carries
    board_power = none because no authority was found. The 100-150 W range above is the DEVICE range
    read from these three cards, which is what the cap is admitted against.
