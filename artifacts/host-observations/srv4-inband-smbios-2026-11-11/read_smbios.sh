#!/bin/bash
# Read-only in-band SMBIOS observation of this machine's firmware identity.
# Every command below PRINTS something; none writes, enrolls, reflashes, resets or reboots, and no file on this
# host is opened for writing. `sudo -n` (never prompts; fails instead) appears only where the kernel ABI makes it
# necessary: several attributes under /sys/class/dmi/id are mode 0400 root, and dmidecode needs privilege to
# decode the tables from /dev/mem.
#
# THREE STATES ARE KEPT APART, because they are three different facts and two of them look alike in output:
#   <absent-from-the-dmi-abi>          the kernel exposes no such attribute at all (a probe of product_serial is
#                                      included for exactly this reason -- it is not an ABI attribute, and a
#                                      reader that cannot tell it from a blank field will mint a unit key from an
#                                      empty string)
#   (nothing after '=')                the attribute exists and the firmware left it empty -- a blank, refused
#   <unreadable-without-privilege>     the attribute exists, is mode 0400 root, and this read was unprivileged;
#                                      the matching priv.dmi.* line prints what privilege sees
set -u
echo "observed_at_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "kernel_hostname=$(hostname)"
echo "os=$(uname -srm)"
echo "processor_count=$(nproc)"
echo "memory_total_kib=$(head -1 /proc/meminfo | tr -d ' ')"
for f in bios_vendor bios_version bios_date board_vendor board_name board_version \
         chassis_asset_tag chassis_vendor chassis_serial chassis_type \
         product_family product_name product_sku product_uuid product_version sys_vendor \
         product_serial; do
  p="/sys/class/dmi/id/$f"
  if [ ! -e "$p" ]; then
    echo "dmi.$f=<absent-from-the-dmi-abi>"
  elif [ -r "$p" ]; then
    echo "dmi.$f=$(cat "$p")"
  else
    echo "dmi.$f=<unreadable-without-privilege>"
    echo "priv.dmi.$f=$(sudo -n cat "$p" 2>&1)"
  fi
done
echo "dmi.board_serial_unprivileged=$(cat /sys/class/dmi/id/board_serial 2>&1)"
echo "dmi.board_serial=$(sudo -n cat /sys/class/dmi/id/board_serial 2>&1)"
echo "smbios.type2=$(sudo -n dmidecode --type 2 2>&1 | tr '\n' '|')"
echo "smbios.type1=$(sudo -n dmidecode --type 1 2>&1 | tr '\n' '|')"
