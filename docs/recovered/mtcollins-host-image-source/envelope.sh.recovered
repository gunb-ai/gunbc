set -eu
B=/var/tmp/gunbc-hostimage
K=$B/linux-6.12.108
IMG=$K/arch/arm64/boot/Image

# THE ENVELOPE IS A TRANSPORT CONTAINER, NOT THE HOST IMAGE. The canonical artifact is the
# EFI-stub kernel with its initramfs built in; this ISO exists only because the MegaRAC controller
# presents block media. Its digest is worth recording because those are the bytes delivered, but it
# must not become a second authority for what the host contains.
rm -rf $B/iso $B/esp.img
mkdir -p $B/iso/EFI/BOOT $B/iso/boot

# ---- the EFI system partition, as an El Torito no-emulation boot image ----------------------
# UEFI specifies an EFI optical boot as a no-emulation El Torito image with platform id 0xEF,
# interpreted as a FAT EFI System Partition. Sized from the kernel rather than a guessed constant:
# an arm64 EFI-stub Image is UNCOMPRESSED (the stub lives in the Image, so Image.gz is not directly
# EFI-bootable) and this one is ~84 MiB, which overran the 64 MiB first attempt.
ESP_KB=$(( ( $(stat -c%s "$IMG") / 1024 ) + 32768 ))
mkfs.vfat -C -F 32 -n GUNBCESP $B/esp.img $ESP_KB >/dev/null
mmd   -i $B/esp.img ::/EFI
mmd   -i $B/esp.img ::/EFI/BOOT
mcopy -i $B/esp.img "$IMG" ::/EFI/BOOT/BOOTAA64.EFI

# ---- the same loader in the ISO9660 tree ----------------------------------------------------
# BELT AND BRACES, AND THE ALPINE NEGATIVE IS WHY. That image presented a block device this
# firmware mapped with no filesystem alias, so it never reached a loader. Firmwares differ in
# whether they boot an optical UEFI medium through the El Torito catalog or by reading the
# removable-media default path off the mounted filesystem. Carrying the loader BOTH ways costs a
# copy of the kernel and removes a whole class of first-boot failure we have already seen once.
cp "$IMG" $B/iso/EFI/BOOT/BOOTAA64.EFI
cp $B/esp.img $B/iso/boot/esp.img

# THE GUEST RIDES ON THE ISO FILESYSTEM, NOT INSIDE THE LOADER. Firmware loads only the EFI image,
# so keeping the guest here keeps the loaded artifact small; /init mounts the medium to reach these.
mkdir -p $B/iso/guest
cp $B/vmlinux-guest $B/iso/guest/vmlinux
cp $B/guest-rootfs-probe.squashfs $B/iso/guest/rootfs.squashfs
cp $B/vm.json $B/iso/guest/vm.json

# ── the verifier and the control plane's public key ride the ISO, not the kernel ───────────────
# THE FIRMWARE LOADS ONLY THE KERNEL, so anything the host needs but the firmware does not is
# cheaper here. openssl and its libraries are 6.8 MB; the image is 88 MiB and an untested load
# ceiling sits somewhere below 173 MiB, which is not margin to spend on a component that can travel
# in the filesystem beside the guest artifacts.
mkdir -p $B/iso/tools/lib
cp /usr/bin/openssl $B/iso/tools/openssl
ldd /usr/bin/openssl | awk '{for(i=1;i<=NF;i++) if($i ~ /^\//) print $i}' | sort -u | while read -r lib; do
  cp "$lib" "$B/iso/tools/lib/$(basename "$lib")"
done

# THE PUBLIC HALF ONLY. The signing key never leaves the control plane, and an image carrying it
# would hand every reader of the NFS export the ability to mint envelopes this host trusts.
cp $B/cp/cp-signing.pub $B/iso/tools/cp-signing.pub

xorriso -as mkisofs \
  -V GUNBCHOST \
  -e boot/esp.img -no-emul-boot \
  -o $B/gunbc-runner-host.iso \
  $B/iso >/dev/null 2>&1

ls -la $B/gunbc-runner-host.iso
echo "--- artifact digests ---"
echo "kernel  $(sha256sum "$IMG" | cut -d' ' -f1)  $(stat -c%s "$IMG") bytes"
echo "cppub   $(sha256sum $B/cp/cp-signing.pub | cut -d' ' -f1)  cp-signing.pub"
echo "guest   $(sha256sum $B/guest-rootfs-probe.squashfs | cut -d' ' -f1)  rootfs.squashfs"
echo "iso     $(sha256sum $B/gunbc-runner-host.iso | cut -d' ' -f1)  $(stat -c%s $B/gunbc-runner-host.iso) bytes"
