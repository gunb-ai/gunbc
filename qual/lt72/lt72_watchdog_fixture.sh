#!/usr/bin/env bash
# lively-tern-72 slot-watchdog fixture. Usage: lt72_watchdog_fixture.sh <mode> <gunbc-bin> <checkout> [envelope=2G] [seconds=420] [memory.high=envelope; "infinity" for the drift-peak shape]
# modes: livelockedge = livelock but fills until current is within 256 KiB of max (64 KiB steps)
# modes: livelock  (run-4 shape: anon to the cap, then CPU-bound text refaulting)      EXPECT: verb kills job tree, exit!=0 receipt; Worker gone, Listener alive, memory.current ~ listener baseline
#        cpusteady (anon 60% of cap, 100% user CPU)                                   EXPECT: LIVES to the deadline
#        burst     (alloc 95% of cap 8 s, free, then CPU work)                         EXPECT: LIVES
#        cachechurn(reads a file larger than the cap in a loop at 100% CPU; file pages at cap => high events + high CPU) EXPECT: LIVES
# Slot shape: transient USER unit (flat leaf), MemoryMax=MemoryHigh=$ENV, swap 0, holding a fake Runner.Listener (sleep copy named so) and a fake Runner.Worker tree.
# Env: verb runs UNPRIVILEGED; /run/gunbc-slot-watchdog must be pre-created and writable. The job runs a private fadvised copy of python (py-job) in ALL modes. The verb gets the ABSOLUTE slot cgroup dir (user units sit >3 levels down).
set -u
MODE=${1:?mode}; BIN=${2:?gunbc}; CHK=${3:?checkout}; ENV=${4:-2G}; SECS=${5:-420}; HIGH=${6:-$ENV}
OUT=/var/tmp/lt72-wd-$MODE-$(date -u +%Y%m%dT%H%M%SZ); mkdir -p "$OUT/bin"; echo "envelope=$ENV high=$HIGH" >"$OUT/shape.txt"
cp "$(command -v sleep)" "$OUT/bin/Runner.Listener"
cp "$(command -v python3)" "$OUT/bin/Runner.Worker" 2>/dev/null || ln -s "$(command -v python3)" "$OUT/bin/Runner.Worker"
# private interpreter copy for the JOB (new inodes, page cache evicted) so its text is charged to the slot, as in probe run 4
REAL=$(readlink -f "$(command -v python3)"); mkdir -p "$OUT/lib"; cp --reflink=never "$REAL" "$OUT/bin/py-job"
ldd "$REAL" | awk '/libpython/{print $3}' | while read -r L; do [ -n "$L" ] && cp --reflink=never "$(readlink -f "$L")" "$OUT/lib/$(basename "$L")"; done
sync; for F in "$OUT"/bin/py-job "$OUT"/lib/*; do [ -f "$F" ] && python3 -c 'import os,sys; fd=os.open(sys.argv[1],os.O_RDONLY); os.posix_fadvise(fd,0,0,os.POSIX_FADV_DONTNEED)' "$F"; done
ls -l "$OUT/bin" "$OUT/lib" >"$OUT/copies.txt"
CAPB=$(numfmt --from=iec "$ENV"); UNIT=lt72-slot-$MODE-$$
case $MODE in
livelock|livelockedge) PY='
import os,json,re,ast
cap=int(os.environ["CAP"]); EDGE=os.environ["EDGE"]=="1"
cg="/sys/fs/cgroup"+open("/proc/self/cgroup").read().split("::")[1].strip()
cur=lambda: int(open(cg+"/memory.current").read())
chunks=[]
def mx():
    for l in open(cg+"/memory.events"):
        k,v=l.split()
        if k=="max": return int(v)
if EDGE:
    sz=64<<10
    while mx()==0 and cur()<cap-(256<<10): chunks.append(bytearray(b"\x01")*sz)
else:
    sz=256<<10
    while mx()==0 and len(chunks) < (cap>>18)+256: chunks.append(bytearray(b"\x01")*sz)
print("held",len(chunks),cur(),flush=True)
src="def f(a,b):\n    return [a*i+b for i in range(100)]\n"*40
x=0; i=0
while True:
    buf=bytearray(2<<20); buf[::4096]=b"\x01"*len(buf[::4096]); del buf
    json.dumps({"k":list(range(200))}); re.compile("(a|b)+c%d"%(i%97)); ast.parse(src); i+=1
    x=(x*1103515245+12345)&0xffffffff';;
cpusteady) PY='
import os
h=bytearray(b"\x01")*int(int(os.environ["CAP"])*0.6); x=0
while True: x=(x*1103515245+12345)&0xffffffff';;
burst) PY='
import os,time
h=bytearray(b"\x01")*int(int(os.environ["CAP"])*0.95); time.sleep(8); del h; x=0
while True: x=(x*1103515245+12345)&0xffffffff';;
cachechurn) PY='
import os
n=int(int(os.environ["CAP"])*1.5); p=os.environ["OUT"]+"/churn.dat"
with open(p,"wb") as f:
    for _ in range(n>>20): f.write(os.urandom(1<<20))
x=0
while True:
    with open(p,"rb") as f:
        while (b:=f.read(1<<20)):
            for _ in range(2000): x=(x*1103515245+12345)&0xffffffff';;
*) echo bad mode; exit 2;; esac
# slot: Listener + Worker tree (Worker spawns a child shell that spawns the workload, so the tree is >1 deep)
systemd-run --user --unit="$UNIT" --collect -p MemoryMax="$ENV" -p MemoryHigh="$HIGH" -p MemorySwapMax=0 -p KillMode=control-group \
  -E EDGE="$([ "$MODE" = livelockedge ] && echo 1 || echo 0)" -E CAP="$CAPB" -E OUT="$OUT" -E PY="$PY" -E LD_LIBRARY_PATH="$OUT/lib" bash -c "$OUT/bin/Runner.Listener 100000 & echo \$! >$OUT/listener.pid; exec $OUT/bin/Runner.Worker -c 'import os,subprocess; subprocess.run([\"bash\",\"-c\",\"$OUT/bin/py-job -c \\\"\$PY\\\"\"])'" >"$OUT/launch.txt" 2>&1
sleep 2
CG=/sys/fs/cgroup$(systemctl --user show "$UNIT" -p ControlGroup --value); echo "$CG" >"$OUT/cgroup.txt"
LPID=$(cat "$OUT/listener.pid"); echo "listener pid $LPID" >>"$OUT/cgroup.txt"
sleep 3; BASE=$(cat "$CG/memory.current"); echo "baseline_with_worker_at_3s $BASE" >>"$OUT/cgroup.txt"
# sampler (independent of the verb): t, members, current, high+max, user_usec
( T0=$(date +%s); printf 't\tprocs\tcurrent\tev_high\tev_max\tev_oom_kill\tuser_usec\tlistener_alive\n' >"$OUT/samples.tsv"
  while [ $(( $(date +%s)-T0 )) -lt "$SECS" ] && [ -d "$CG" ]; do
    ev=$(awk '{a[$1]=$2} END{print a["high"]"\t"a["max"]"\t"a["oom_kill"]}' "$CG/memory.events")
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$(( $(date +%s)-T0 ))" "$(wc -l <"$CG/cgroup.procs")" "$(cat $CG/memory.current)" "$ev" "$(awk '/user_usec/{print $2}' $CG/cpu.stat)" "$(kill -0 "$LPID" 2>/dev/null && echo 1 || echo 0)" >>"$OUT/samples.tsv"
    sleep 5
  done ) & SAMP=$!
# PRECONDITION (livelock only): the stall must be established before the verb starts, else a non-livelock reads as "watchdog did nothing"
if [ "$MODE" = livelock ] || [ "$MODE" = livelockedge ]; then
  evsum() { awk '$1=="high"||$1=="max"{s+=$2} END{print s}' "$CG/memory.events"; }
  E0=$(evsum); U0=$(awk '/user_usec/{print $2}' "$CG/cpu.stat"); sleep 20
  E1=$(evsum); U1=$(awk '/user_usec/{print $2}' "$CG/cpu.stat")
  echo "precondition ev_delta=$((E1-E0)) user_usec_delta=$((U1-U0)) over 20s" | tee "$OUT/precondition.txt"
  if [ $((E1-E0)) -le 1000 ] || [ $((U1-U0)) -ge 4000000 ]; then
    echo "livelock not established" | tee -a "$OUT/precondition.txt"; kill $SAMP 2>/dev/null; systemctl --user stop "$UNIT"; exit 3
  fi
fi
# the verb under test, with a hard stop at $SECS (fixture, not production)
/usr/bin/time -v -o "$OUT/verb.time" timeout "$SECS" ${SUDO:-} "$BIN" slot-watchdog --slot-cgroup "$CG" --source-root "$CHK/dag" --source-root "$CHK/src/v2" >"$OUT/verb.out" 2>&1
echo "verb_rc=$?" >"$OUT/verdict.txt"
sleep 3
{ echo "listener_alive_after=$(kill -0 "$LPID" 2>/dev/null && echo 1 || echo 0)"; echo "procs_after=$(wc -l <"$CG/cgroup.procs" 2>/dev/null)"; echo "current_after=$(cat $CG/memory.current 2>/dev/null)"; } >>"$OUT/verdict.txt"
grep -E 'Maximum resident' "$OUT/verb.time" >>"$OUT/verdict.txt"
wait $SAMP 2>/dev/null; systemctl --user stop "$UNIT" 2>/dev/null; rm -f "$OUT/churn.dat"
echo "$OUT"; cat "$OUT/verdict.txt"; tail -n 4 "$OUT/samples.tsv"; head -c 600 "$OUT/verb.out"
