john (host-ba61f9, Supermicro H11SSL unit 4) -- GPU identity and power-limit read, 2026-11-28

WHAT THIS DIRECTORY IS
  Read-only output taken from the machine the operator calls john (gunbc.fleet.fleet_host_identity
  operator_host_ba61f9; its install is the receipt in artifacts/host-observations/john-hand-install-2026-11-26/).
  It is the receipt for the three GPU rows that gunb-ai/gunbc#17203 adds: the board asset identities
  gpu_board_14, gpu_board_15 and gpu_board_16 in gunbc.fleet.fleet_asset_identity, their containment in
  gunbc.fleet.fleet_physical_inventory installed_gpu_board_containments, and their device observations in
  gunbc.fleet.gpu_device_observation current_gpu_device_observations.

HOW IT WAS TAKEN
  Three ssh sessions on 2026-11-28 between 00:44:49Z and 00:44:58Z, to the LAN address john answers at
  (192.168.1.240), as the unprivileged ubuntu user with the fleet automation key. No sudo was used and
  nothing on the host was changed. Each file's first line records the read time (date -u on the host),
  the hostname the host reported ("john") and the exact command; everything after that line is the
  command's unedited stdout.

FILES
  gpu_identity_power_query.csv
      nvidia-smi --query-gpu=index,name,uuid,serial,pci.bus_id,power.min_limit,power.max_limit,
      power.default_limit,power.limit,vbios_version,driver_version --format=csv
      The names, UUIDs, board serials, PCI functions and the four power limits of each GPU:
        00000000:01:00.0  Tesla V100-SXM2-16GB  min 150 / max 300 / default 300 / limit 150 W
        00000000:81:00.0  Tesla P100-PCIE-16GB  min 125 / max 250 / default 250 / limit 125 W
        00000000:C1:00.0  Tesla PG500-216       min 100 / max 250 / default 250 / limit 100 W
      Driver 580.178.04 on all three.
  nvidia_smi_q_power.txt
      nvidia-smi -q -d POWER: the same limits in the driver's long form (Current, Requested, Default,
      Min and Max Power Limit under "GPU Power Readings"), plus an instantaneous draw and a 2-second
      sample window per GPU. The draw figures are one idle-ish instant and are NOT read as ceilings or
      as a host's draw by any row.
  gpu_pci_device_id_query.csv
      nvidia-smi --query-gpu=index,pci.bus_id,pci.device_id,pci.sub_device_id,memory.total --format=csv
      The PCI device ids that bind each card to a catalog row: 0x1DB1 (extdeps.gpu.nvidia_v100
      nvidia_tesla_v100_sxm2_16gb_board_facts pci_device_id "1db1"), 0x15F8 (extdeps.gpu.nvidia_p100
      nvidia_tesla_p100_pcie_16gb_board_facts pci_device_id "15f8"), and 0x1DF0, which NVIDIA's driver
      appendix (extdeps.nvidia.linux_driver_supported_chips, release 580.95.05) names "Tesla PG500-216"
      and for which this repository has NO board catalog row.

WHAT THIS DIRECTORY DOES NOT ESTABLISH
  - The current limits (150 / 125 / 100 W, each card's own minimum) are what the driver reported at the
    instant of the read. They are NOT evidence of which mechanism applied them; the operator's session
    records a hand-written gpu-power-cap.service on john that sets every GPU to its own minimum, but its
    bytes and journal are not in this directory.
  - The V100-SXM2-16GB is an SXM2 module; how it is carried in a PCIe slot (the operator calls it a
    blower card) is not read here.
  - Which physical slot each PCI function sits in is not read.
  - The PG500-216's memory size (32768 MiB in gpu_pci_device_id_query.csv) is this card's report, not a
    catalog fact; no upstream document this repository holds states it.
  - Nothing about john's power supply, its non-GPU draw, or the circuit that feeds it is read here; those
    are the operator's statements of 2026-11-28, recorded as gunbc.design_ruling rows.
