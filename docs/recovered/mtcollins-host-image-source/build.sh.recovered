set -eu
B=/var/tmp/gunbc-hostimage
K=$B/linux-6.12.108
BUILD_ID="${BUILD_ID:-dev}"

# ---- initramfs tree -------------------------------------------------------------------------
rm -rf $B/irfs
mkdir -p $B/irfs
cp /bin/busybox $B/irfs/busybox
sed "s/@@BUILD_ID@@/$BUILD_ID/" $B/initramfs/init > $B/irfs/init
chmod +x $B/irfs/init $B/irfs/busybox
FC=$B/release-v1.16.1-aarch64/firecracker-v1.16.1-aarch64
JL=$B/release-v1.16.1-aarch64/jailer-v1.16.1-aarch64
# ---- the policy installer and its library closure -------------------------------------------
# nft IS DYNAMICALLY LINKED AND THE HOST HAS NO PACKAGE MANAGER, so the closure is staged explicitly
# rather than assumed present. Every entry here is a file the runtime surface must account for --
# which is the argument for the terminal form, where the supervisor programs netfilter over netlink
# and none of these exist. Until then they are declared, not smuggled.
NFT=/usr/sbin/nft
: > $B/irfs/nft.spec
echo "dir /lib 0755 0 0" >> $B/irfs/nft.spec
echo "dir /lib/aarch64-linux-gnu 0755 0 0" >> $B/irfs/nft.spec
echo "file /bin/nft $NFT 0755 0 0" >> $B/irfs/nft.spec
ldd $NFT | awk '{for(i=1;i<=NF;i++) if($i ~ /^\//) print $i}' | sort -u | while read -r lib; do
  echo "file $lib $lib 0755 0 0" >> $B/irfs/nft.spec
done

sed -e "s|@BUSYBOX@|$B/irfs/busybox|" -e "s|@INIT@|$B/irfs/init|" -e "s|@UDHCPC@|$B/udhcpc.script|" \
    -e "s|@FIRECRACKER@|$FC|" -e "s|@JAILER@|$JL|" \
    $B/initramfs.spec.in > $B/irfs/spec
cat $B/irfs/nft.spec >> $B/irfs/spec

# ---- kernel config --------------------------------------------------------------------------
cd $K
make -s ARCH=arm64 defconfig

# The measured requirements from gunbc.runner.runner_host_hardware_observation, plus the host
# virtualization surface Firecracker needs. Built IN, not modular: the runner host should have no
# reason to load kernel code after boot.
./scripts/config \
  --enable  EFI --enable EFI_STUB --enable ACPI \
  --enable  IGB --disable IGB_HWMON \
  --enable  KVM --enable VIRTUALIZATION \
  --enable  TUN --enable BRIDGE --enable VETH \
  --enable  DEVTMPFS --enable DEVTMPFS_MOUNT \
  --enable  TMPFS --enable PROC_FS --enable SYSFS \
  --enable  BLK_DEV_INITRD \
  --enable  ISO9660_FS --enable BLK_DEV_SR --enable SCSI --enable BLK_DEV_SD \
  --enable  USB_XHCI_HCD --enable USB_STORAGE --enable SQUASHFS \
  --enable  SERIAL_AMBA_PL011 --enable SERIAL_AMBA_PL011_CONSOLE \
  --enable  SERIAL_8250 --enable SERIAL_8250_CONSOLE \
  --enable  HW_RANDOM --enable HW_RANDOM_ARM_SMCCC_TRNG \
  --enable  NAMESPACES --enable NET_NS --enable PID_NS --enable USER_NS \
  --enable  CGROUPS --enable MEMCG --enable CGROUP_SCHED \
  --enable  SECCOMP --enable SECCOMP_FILTER \
  --enable  TCG_TPM --enable TCG_CRB \
  --enable  CFS_BANDWIDTH \
  --enable  NETFILTER --enable NETFILTER_ADVANCED \
  --enable  NF_CONNTRACK --enable NF_NAT --enable NF_TABLES \
  --enable  NFT_CT --enable NFT_NAT --enable NFT_MASQ \
  --enable  NF_TABLES_IPV4 --enable NF_TABLES_INET \
  --enable  IP_ADVANCED_ROUTER \
  --set-str CMDLINE "console=ttyS0,115200 console=ttyAMA0,115200 earlycon" \
  --disable CMDLINE_FROM_BOOTLOADER --disable CMDLINE_EXTEND --enable CMDLINE_FORCE \
  --set-str INITRAMFS_SOURCE "$B/irfs/spec" \
  --disable MODULES

make -s ARCH=arm64 olddefconfig

# ---- build ----------------------------------------------------------------------------------
# THE PREVIOUS FORM ENDED WITH AN `ls` AND ASSERTED NOTHING. Two runs of that script over the same
# failing link reported DIFFERENT exit codes -- one aborted at 2, one reported 0 while its log
# carried make's Error 2 and an ls of the previous run's Image. Why they differ is UNRESOLVED, and
# an explanation invented to fit it would be worse than the gap.
#
# WHAT IS CERTAIN IS ENOUGH TO ACT ON: a failed link leaves the previous Image in place, so an `ls`
# cannot distinguish this run's output from the last one's. The removal and the two checks below
# make the artifact's provenance a property of the run rather than of the directory, which closes
# the class whatever the exit-code discrepancy turns out to be.
#
# KALLSYMS_EXTRA_PASS IS NOT A WORKAROUND HERE, IT IS THE DOCUMENTED FIX. A built-in initramfs
# changes vmlinux's size between kallsyms passes, so the symbol table's own size shifts the symbols
# it is describing; the kernel's build detects the inconsistency and refuses rather than shipping a
# wrong table. The extra pass converges it.
rm -f $K/arch/arm64/boot/Image
if ! make -s ARCH=arm64 -j"$(nproc)" KALLSYMS_EXTRA_PASS=1 Image; then
  echo "BUILD FAILED: kernel did not link" >&2
  exit 1
fi
if [ ! -f $K/arch/arm64/boot/Image ]; then
  echo "BUILD FAILED: make reported success but produced no Image" >&2
  exit 1
fi

ls -la $K/arch/arm64/boot/Image
