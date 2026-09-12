#!/bin/bash
# PID 1 IN THE GUEST. There is no init in this image's boot path -- this script IS it -- so nothing
# is mounted, nothing is configured, and every dependency the runner has must be met here explicitly.
# That is the point: the runtime surface of this guest is exactly what these lines create.
# /dev IS MOUNTED BEFORE ANYTHING IS WRITTEN TO IT, and the ordering is the whole point. This image's
# /dev is EMPTY on disk -- unsquashfs cannot create device nodes without root, so the extracted tree
# that this rootfs was rebuilt from carries none. Whether /dev/console exists at all therefore
# depends on the guest kernel auto-mounting devtmpfs, which is a property of a kernel this repository
# did not build.
#
# A FAILED REDIRECT ON `exec` KILLS A NON-INTERACTIVE SHELL WITH STATUS 1 AND PRINTS NOTHING, which
# MATCHES the observed failure -- the kernel logged "Run /gunbc-runner-init.sh as init process" and
# panicked 0.19s later on exitcode 0x100 -- but does not explain it. THE PROBE GUEST RAN THE SAME
# REDIRECT ON THE SAME IMAGE AND PRINTED FINE, which is direct counter-evidence, so /dev/console was
# reachable at least once without this mount.
#
# SO THIS IS A NARROWING, NOT A DIAGNOSIS. Mounting devtmpfs first removes one dependency the script
# had been betting on; if the guest still exits 1 the cause is elsewhere and this line will have
# cost nothing. The instrument change that accompanies it -- the host now dumps the guest log whole
# instead of through a fixed grep -- is the part actually expected to produce the answer.
set +e
mount -t devtmpfs dev /dev 2>/dev/null
exec >/dev/console 2>&1

echo "=== GUNBC-RUNNER-GUEST ==="
echo "guest-uname=$(uname -r)"
echo "guest-dev-entries=$(ls /dev 2>/dev/null | wc -l)"

mount -t proc proc /proc 2>/dev/null
mount -t sysfs sys /sys 2>/dev/null
mount -t tmpfs tmpfs /tmp 2>/dev/null
mount -t tmpfs -o size=64m tmpfs /run 2>/dev/null
hostname gunbc-mtcollins-1 2>/dev/null
ip link set lo up 2>/dev/null

echo "guest-addr=$(ip -4 addr show eth0 2>/dev/null | grep -o 'inet [0-9./]*')"
echo "guest-route=$(ip route show 2>/dev/null | tr '\n' ';')"

# THE RESOLVER IS BAKED INTO THE IMAGE AND IS ONLY READ BACK HERE. An earlier revision wrote it at
# boot and could not: the root is a read-only squashfs, so both the write and its fallback failed and
# the guest went on to look up names with no resolver at all. It is a property of the artifact now,
# which is also where it belongs -- the host's forward chain grants UDP 53 to exactly one address, so
# the image and the filter must agree, and an agreement between two artifacts is easier to keep than
# one between an artifact and a script.
echo "guest-resolver=$(cat /etc/resolv.conf 2>/dev/null | tr '\n' ' ')"

# ── the registration, read off a raw block device ──────────────────────────────────────────────
# THE JIT CONFIG ARRIVES AS THE ENTIRE CONTENTS OF /dev/vdb, WITH NO FILESYSTEM. A filesystem here
# would be a second thing to get right -- a driver in the guest kernel, a mount, a path -- to carry
# one opaque string. The host writes the string and pads it with NULs; the guest strips them.
#
# IT IS SINGLE-USE AND SHORT-LIVED BY CONSTRUCTION. A JIT config registers exactly one ephemeral
# runner, is spent the moment it is redeemed, and expires unredeemed. That is what makes it safe to
# hand to a guest at all, and it is the reason this path uses a JIT config rather than a
# registration token, which would let the holder create runners repeatedly.
JITCONFIG=$(tr -d '\000' < /dev/vdb 2>/dev/null)
if [ -z "$JITCONFIG" ]; then
  echo "RUNNER REFUSED: /dev/vdb carried no JIT configuration"
  echo "=== GUNBC-RUNNER-GUEST END ==="
  sleep 30
  exit 1
fi
echo "jitconfig-bytes=${#JITCONFIG}"

# ── a writable copy, because the root is a read-only squashfs ──────────────────────────────────
# THE RUNNER WRITES: .runner, .credentials, _diag, _work. A read-only root is the right shape for an
# image whose contents are verified by digest, so the writable area is created here as tmpfs rather
# than by making the image mutable. It dies with the guest, which is what "ephemeral per boot" means
# concretely rather than as a slogan.
# /mnt EXISTS IN THE IMAGE NOW. It did not before, and the failure chained the way read-only failures
# do: the mount failed, the mkdir that would have created the mountpoint failed for the same reason,
# the copy silently wrote nothing, and the first line to actually stop was a `cd` three steps later
# -- which named /mnt/runner and looked like a missing runner rather than a missing mountpoint.
mount -t tmpfs -o size=3g tmpfs /mnt || echo "MOUNT FAILED: no writable area for the runner"
mkdir -p /mnt/runner
cp -a /runner/. /mnt/runner/ 2>/dev/null
echo "runner-copy=$(du -sm /mnt/runner 2>/dev/null | cut -f1)MB"
echo "runner-present=$(test -x /mnt/runner/run.sh && echo yes || echo NO)"

# ── the attempt workspace, on a real block device ──────────────────────────────────────────────
# THE FIRST CANARY RAN WITH NO WRITABLE DISK AT ALL. GitHub warned "only 0 MB of available disk
# space left", and the job passed anyway because two echo steps need no workspace -- which is
# precisely the shape of a green result that establishes less than it appears to.
#
# THE RUNNER'S _work TREE IS WHERE EVERY JOB ACTUALLY LIVES: the checkout, the temp directory the
# runner exports as RUNNER_TEMP, downloaded actions, and any cache. Mounting the device there gives
# all of them one bounded, attempt-owned home rather than letting them compete with the guest's RAM
# through the tmpfs the runner tree sits on.
mkdir -p /mnt/runner/_work
if mount -t ext2 /dev/vdc /mnt/runner/_work 2>/dev/null; then
  echo "workspace-mounted=yes"
else
  echo "WORKSPACE REFUSED: /dev/vdc did not mount at /mnt/runner/_work"
fi
echo "workspace-free=$(df -Pm /mnt/runner/_work 2>/dev/null | awk 'NR==2{print $4}')MB"
echo "workspace-fs=$(df -PT /mnt/runner/_work 2>/dev/null | awk 'NR==2{print $2}')"

# PID 1 STARTED BY THE KERNEL INHERITS NO PATH, and every process the runner spawns inherits that
# emptiness. The first job dispatched to this host reached the machine, was accepted, and its step
# executed -- and then failed on `sh: command not found`, which names neither the missing binary nor
# the missing variable. A login shell would have set this; init is not one, and nothing else in this
# boot path was going to.
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
export HOME=/mnt
export RUNNER_ALLOW_RUNASROOT=1
export DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1
echo "guest-path=$PATH"
echo "guest-bash=$(command -v bash 2>/dev/null || echo MISSING)"

# EXITING AS PID 1 PANICS THE GUEST, and a panic backtrace is a much worse receipt than a sentence.
# The previous run ended on "Attempted to kill init! exitcode=0x00000100", which says the script
# stopped and nothing about why -- the actual reason was three lines above it and would have been
# lost entirely if the host had still been dumping a filtered excerpt of this log.
if ! cd /mnt/runner; then
  echo "RUNNER REFUSED: no prepared runner directory at /mnt/runner"
  echo "=== GUNBC-RUNNER-GUEST END ==="
  sleep 30
  exit 0
fi

# RUNNING AS ROOT INSIDE THE GUEST IS DELIBERATE AND IS NOT THE ISOLATION STORY. The boundary for
# this workload is the microVM -- a separate kernel, its own memory, a filtered forward path -- not
# a uid inside it. A non-root uid in a single-purpose VM that is destroyed after one job protects
# nothing the VM does not already protect, and pretending otherwise would be the cosmetic-safety
# move the ladder warns about. RUNNER_ALLOW_RUNASROOT exists precisely because the runner cannot
# tell an isolated VM from a shared host; here we can.
echo "--- runner starting ---"
timeout 420 ./run.sh --jitconfig "$JITCONFIG"
RC=$?
echo "runner-exit=$RC"

echo "=== GUNBC-RUNNER-GUEST END ==="
sleep 20
