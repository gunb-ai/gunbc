#!/bin/sh
# Read-only capture of the AMD GPU on host-6ce85c (h11-a16): PCI identity and link, the amdgpu sysfs
# device attributes, and the hwmon power and temperature attributes. Nothing here writes to sysfs.
# lspci -vv needs root for the capability list, so it runs `sudo -n` (fails rather than prompts).
set -u
echo "## date_utc $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "## hostname $(hostname)"
echo "## uname_r $(uname -r)"
for d in $(lspci -D -d 1002: | awk '/VGA|Display|3D/{print $1}'); do
  echo "## lspci_nn $d"; lspci -nn -s "$d"
  echo "## lspci_vv $d"; sudo -n lspci -vv -s "$d" | grep -E "Subsystem|LnkCap:|LnkSta:|Kernel driver|Region 0"
  echo "## parent_chain $d"; readlink -f /sys/bus/pci/devices/$d
  root=$(readlink -f /sys/bus/pci/devices/$d | awk -F/ '{print $5}')
  echo "## root_port_lspci_vv $root"; sudo -n lspci -vv -s "$root" | grep -E "^[0-9a-f]|LnkCap:|LnkSta:"
  D=/sys/bus/pci/devices/$d
  for f in vendor device subsystem_vendor subsystem_device unique_id vbios_version mem_info_vram_total \
           current_link_speed current_link_width max_link_speed max_link_width power_dpm_force_performance_level \
           pp_dpm_sclk pp_dpm_mclk; do
    echo "## sysfs $f"; cat "$D/$f" 2>&1
  done
  H=$(ls -d $D/hwmon/hwmon* | head -1)
  for f in name power1_label power1_cap power1_cap_min power1_cap_max power1_cap_default power1_average power1_input \
           temp1_label temp1_crit temp1_emergency temp2_label temp2_crit temp2_emergency temp3_label temp3_crit temp3_emergency; do
    echo "## hwmon $f"; cat "$H/$f" 2>&1
  done
done
echo "## ppfeaturemask"; cat /sys/module/amdgpu/parameters/ppfeaturemask
