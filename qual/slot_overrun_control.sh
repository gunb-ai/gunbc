#!/usr/bin/env bash
# Slot-overrun control for gunbc work item adhoc-42ce3750-45f. RUN ON srv1 AS ROOT (sudo).
# Question: which unit shape turns an over-envelope job into a bounded-time KILL instead of a throttle stall?
# Scale-free: envelope E=2 GiB (the mechanism is ratio-based; the live slot's 22/21 or 25/x GiB is the same shape x ~11).
# Touches nothing persistent: transient units only (slotctl-*), removed on exit; one 1 GiB scratch file in /var/tmp, removed on exit.
# Bounded: every trial has RuntimeMaxSec=75 (a stall is reported as result=timeout); oomd shapes use a per-unit 15s pressure duration so a kill fits in 75s -> worst case 3 fixtures x 4 shapes x ~80s = 16 min.
# Host protection: CPUQuota=100%, IOWeight=1, TasksMax=64, MemoryMax=2G so the fixture can never exceed 2 GiB of srv1's RAM.
set -u
[ "$(id -u)" = 0 ] || { echo "REFUSE: run as root"; exit 2; }
command -v python3 >/dev/null && command -v systemd-run >/dev/null || { echo "REFUSE: need python3+systemd-run"; exit 2; }
[ -e /sys/fs/cgroup/cgroup.controllers ] || { echo "REFUSE: cgroup v2 required"; exit 2; }
systemctl is-active --quiet systemd-oomd || { echo "NOTE: systemd-oomd not active -> oomd shapes will show stall, that itself is the finding"; }
echo "host=$(hostname) systemd=$(systemctl --version | head -1) kernel=$(uname -r) swap_total_kB=$(awk '/SwapTotal/{print $2}' /proc/meminfo)"
W=$(mktemp -d /var/tmp/slotctl.XXXXXX)
trap 'rm -rf "$W"; systemctl list-units "slotctl-*" --no-legend --plain | awk "{print \$1}" | xargs -r systemctl stop 2>/dev/null' EXIT
dd if=/dev/urandom of="$W/hot.bin" bs=1M count=1024 status=none
cat > "$W/fixture.py" <<'PY'
import mmap, os, random, sys, time
kind, E = sys.argv[1], 2 * 1024**3
chunk = 16 * 1024**2
keep = []
def alloc(n):
    while n > 0:
        b = bytearray(chunk); b[::4096] = b"\x01" * len(b[::4096]); keep.append(b); n -= chunk
f = open(os.path.join(os.path.dirname(__file__), "hot.bin"), "rb")
m = mmap.mmap(f.fileno(), 0, access=mmap.ACCESS_READ)
def touch(secs):
    end = time.time() + secs; n = len(m) // 4096
    while time.time() < end:
        for _ in range(2000): m[random.randrange(n) * 4096]
if kind == "under":      # working set 0.5E anon + 0.2E hot file: must PASS
    alloc(int(0.5 * E)); touch(15); print("under: done", flush=True); sys.exit(0)
if kind == "thrash":     # anon 0.9E + hot file 0.5E > E, anon alone below max: reclaim-only stall, no allocation fails
    alloc(int(0.9 * E)); print("thrash: allocated", flush=True); touch(600)
if kind == "grow":       # anon grows to 1.6E: plain allocation overrun
    alloc(int(1.6 * E)); print("grow: SURVIVED (bad)", flush=True); touch(600)
PY
run_case() { # name highcfg oomd fixture
  local shape=$1 high=$2 oomd=$3 fx=$4 unit="slotctl-$1-$4"
  local props=(-p MemoryMax=2G -p MemorySwapMax=0 -p MemoryHigh="$high" -p CPUQuota=100% -p IOWeight=1 -p TasksMax=64 -p RuntimeMaxSec=75)
  [ "$oomd" = 1 ] && props+=(-p ManagedOOMMemoryPressure=kill -p ManagedOOMMemoryPressureLimit=40% -p ManagedOOMMemoryPressureDurationSec=15s)
  local cg=/sys/fs/cgroup/system.slice/$unit.service t0=$(date +%s.%N)
  ( while sleep 2; do [ -d "$cg" ] && { cat "$cg/memory.events" > "$W/ev.$unit" 2>/dev/null; cat "$cg/memory.pressure" > "$W/psi.$unit" 2>/dev/null; }; done ) & local samp=$!
  local out; out=$(systemd-run --unit="$unit" --wait "${props[@]}" python3 "$W/fixture.py" "$fx" 2>&1); local rc=$?
  kill $samp 2>/dev/null; wait $samp 2>/dev/null
  local dt; dt=$(awk -v a="$t0" -v b="$(date +%s.%N)" 'BEGIN{printf "%.0f", b-a}')
  local res; res=$(echo "$out" | sed -n 's/.*Finished with result: //p;s/.*Result: //p' | head -1)
  local oomk; oomk=$(awk '$1=="oom_kill"{print $2}' "$W/ev.$unit" 2>/dev/null)
  local hi; hi=$(awk '$1=="high"{print $2}' "$W/ev.$unit" 2>/dev/null)
  local oomd_k; oomd_k=$(journalctl -u systemd-oomd --since "-$((dt+5))s" --no-pager -q 2>/dev/null | grep -c "$unit" || true)
  printf 'RESULT shape=%-12s fixture=%-7s rc=%s result=%-10s secs=%-3s kernel_oom_kill=%s high_events=%s oomd_log_lines=%s\n' "$shape" "$fx" "$rc" "${res:-?}" "$dt" "${oomk:-?}" "${hi:-?}" "$oomd_k"
  [ "$oomd" = 1 ] && journalctl -u systemd-oomd --since "-$((dt+5))s" --no-pager -q 2>/dev/null | grep "$unit" | sed 's/^/OOMD_JOURNAL /'
  systemctl reset-failed "$unit.service" 2>/dev/null
}
for fx in under grow thrash; do
  run_case A_current   1950M 0 $fx   # today's shape scaled: high just under max
  run_case B_nohigh    infinity 0 $fx   # MemoryHigh removed: kernel OOM only
  run_case C_cur_oomd  1950M 1 $fx   # today's shape + oomd pressure kill on the unit
  run_case D_nohi_oomd infinity 1 $fx   # no high + oomd pressure kill
done
echo "PASS CRITERIA: fixture=under must be result=success for every shape. A shape is ADMISSIBLE iff grow AND thrash end with result in {oom-kill,signal,exit-code} and secs well under 75 (result=timeout == stalled == INADMISSIBLE)."
