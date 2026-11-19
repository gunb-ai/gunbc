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
  `cat` of the t10-power-cap unit and script, `nvidia-smi -q -d POWER`, `nvidia-smi --query-gpu
  =pci.bus_id,uuid,serial`, `systemctl status` of the cap unit, and `lspci` for the GPU functions.

  ONE THING IS DISCLOSED, NOT HIDDEN: the admission for that session included the command
  `sudo -n true`, whose output ("yes") was printed and which the admission did NOT authorize. Nothing
  privileged was run through it. It is recorded here because a receipt that quietly omits an
  over-scoped command is the kind of record that makes the next reader trust more than was granted.

FILES
  unit_read1.txt          systemd unit-file listing filtered to GPU/power units, the contents of
                          /etc/systemd/system, whether nvidia-smi is present, and the `lspci` GPU
                          functions. Establishes that t10-power-cap.service and ken-strata.service are
                          enabled, and which PCI functions carry GPUs (01:00.0, 81:00.0, c1:00.0).
  unit_read2.txt          the t10-power-cap.service unit file itself, plus the cap unit's `systemctl
                          status` output. Establishes Type=oneshot, WantedBy=multi-user.target, the
                          ExecStart path, and the unit's last-run state.
  unit_read3.txt          the t10-power-cap.sh script, then `nvidia-smi -q -d POWER`, then the cap
                          unit's status. THE POWER NUMBERS ARE HERE: for each of the three GPUs
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
  - No board power envelope for the Tesla T10 is claimed: extdeps.gpu.nvidia_t10 carries
    board_power = none because no authority was found. The 100-150 W range above is the DEVICE range
    read from these three cards, which is what the cap is admitted against.
