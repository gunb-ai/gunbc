john (host-ba61f9, Supermicro H11SSL unit 4) -- hand install from srv4's PXE edge, 2026-11-26

WHAT THIS DIRECTORY IS
  The record of a hand install the operator chose on 2026-11-26 ("hand install, recorded") because the
  modeled route (fleet-converge mode managed_host_network_install -> gunbc.host.network_install_ci) refuses
  every new unit today; gunbc.plans.new_server_bringup section 2 names the two blockers, roadmap nodes
  install-subject-owns-secured-controller and install-chain-selected-by-content. The steps were performed
  by one Claude Code session (128748d2) on the operator's instruction. The typed record of what these files
  say is gunbc.machine_intake.supermicro_h11ssl_unit4_hand_install_receipt; this directory is its evidence.

FILES
  john-hand-steps.log
      The step log, one line per step, written by the performing session AS IT WENT, each line stamped
      with the UTC instant of the step. It is the session's own transcription, not tool output: it names
      the commands and the outcomes, and it is the only record of the installer-shell udev read and of
      subiquity's refusal text quoted in it. It carries no credential (the BMC was driven with a credential
      the operator holds; none is written here or anywhere in the repository).

  seed-user-data-as-staged-20261126T224812Z.yaml
      The key-only autoinstall user-data as FIRST staged on srv4 at 22:48:12Z, mirroring
      gunbc.fleet.site_pxe_edge_converge site_pxe_edge_key_only_autoinstall as rendered by
      gunbc.os_install_emit. Its storage.layout.match carries the BARE serial 20152C807439 -- the value the
      model's renderer (gunbc.os_install_emit autoinstall_match_entries) would render for this disk. It is
      NOT byte-identical to what any install boot fetched: per the log, only the serial line changed before
      each boot -- to "*20152C807439" at 22:48:28Z (install attempt 1, refused by subiquity) and to
      "*20152C807439*" at 22:53:58Z (install attempt 2, installed). The served files themselves were moved to
      /srv/gunbc-site-pxe-edge/staging/withdrawn-john/ on srv4 at 23:03:55Z and are not copied here.
      The three ssh keys in it are the public keys of gunbc.fleet.fleet_ssh_access's principals.

WHAT IS NOT HERE, and is owed
  No in-band capture of the installed host (dmidecode, lsblk, udevadm, nvidia-smi, ip -o link) was
  committed; the receipt module names each such read beside the figure it would ground.
