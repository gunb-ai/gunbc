#!/usr/bin/env bash
# lively-tern-72 srv1 probe. Usage: lt72_srv1_probe.sh <mode> [envelope=2G] [seconds=150]
# modes: anonfull (REPRO of drift-peak: anon to within ~1 MiB of max, swap 0, then CPU-bound loop executing interpreter text; discriminator pgmajfault)
#        cpusteady (anon ~60% of max, 100% user CPU, never hits max)
#        burst     (alloc to ~95% of max for 8 s, free, then CPU work; max events should be ~0)
# Safe by construction: transient USER unit, MemoryMax=$ENV, MemorySwapMax=0, MemoryHigh=infinity,
# hard stop at <seconds>. Touches nothing else. Writes /var/tmp/lt72-probe-<mode>-<ts>/samples.tsv
set -u
MODE=${1:?mode}; ENV=${2:-2G}; SECS=${3:-150}
OUT=/var/tmp/lt72-probe-$MODE-$(date -u +%Y%m%dT%H%M%SZ); mkdir -p "$OUT"
UNIT=lt72-$MODE-$$
case $MODE in
anonfull) PY='
import os
cap=int(os.environ["CAP"])
cg="/sys/fs/cgroup"+open("/proc/self/cgroup").read().split("::")[1].strip()
cur=lambda: int(open(cg+"/memory.current").read())
chunks=[]; sz=1<<20
# fill anon until memory.current is within ~1 MiB of max (less than the interpreter text working set)
while cur() < cap-(1<<20) and len(chunks) < (cap>>20)+64:
    chunks.append(bytearray(b"\x01")*sz)
print("held",len(chunks),cur(),flush=True)
x=0
while True:   # CPU-bound: keeps executing interpreter text, never allocates
    x=(x*1103515245+12345)&0xffffffff';;
cpusteady) PY='
import os
cap=int(os.environ["CAP"]); h=bytearray(b"\x01")*int(cap*0.6); x=0
while True:
    x=(x*1103515245+12345)&0xffffffff';;
burst) PY='
import os,time
cap=int(os.environ["CAP"]); h=bytearray(b"\x01")*int(cap*0.95); time.sleep(8); del h
x=0
while True:
    x=(x*1103515245+12345)&0xffffffff';;
*) echo bad mode; exit 2;; esac
CAPB=$(numfmt --from=iec "$ENV")
systemd-run --user --unit="$UNIT" --collect -p MemoryMax="$ENV" -p MemorySwapMax=0 -p MemoryHigh=infinity \
  -E CAP="$CAPB" python3 -c "$PY" >"$OUT/launch.txt" 2>&1
CG=/sys/fs/cgroup$(systemctl --user show "$UNIT" -p ControlGroup --value)
echo "cgroup $CG" | tee "$OUT/cgroup.txt"; cat "$CG/memory.max" "$CG/memory.high" "$CG/memory.swap.max" >>"$OUT/cgroup.txt"
printf 't\tactive\tcurrent\tanon\tfile\tpgmajfault\tev_high\tev_max\tev_oom\tev_oom_kill\tuser_usec\n' >"$OUT/samples.tsv"
T0=$(date +%s)
while :; do
  t=$(( $(date +%s)-T0 ))
  act=$(systemctl --user is-active "$UNIT" 2>/dev/null)
  if [ -r "$CG/memory.events" ]; then
    ev=$(awk '{a[$1]=$2} END{print a["high"]"\t"a["max"]"\t"a["oom"]"\t"a["oom_kill"]}' "$CG/memory.events")
    st=$(awk '{a[$1]=$2} END{print a["anon"]"\t"a["file"]"\t"a["pgmajfault"]}' "$CG/memory.stat")
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$t" "$act" "$(cat $CG/memory.current)" "$st" "$ev" "$(awk '/user_usec/{print $2}' $CG/cpu.stat)" >>"$OUT/samples.tsv"
  else printf '%s\t%s\tcgroup-gone\n' "$t" "$act" >>"$OUT/samples.tsv"; fi
  [ "$act" != active ] && break
  [ "$t" -ge "$SECS" ] && { echo "STOPPING at ${t}s (still active)" >>"$OUT/samples.tsv"; systemctl --user stop "$UNIT"; break; }
  sleep 5
done
echo "$OUT"; tail -n 6 "$OUT/samples.tsv"
