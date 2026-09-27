# usage (srv1, repo root at this ref): bash src/v2/test/manual/llvprobe/probe.sh <path-to-gunbc>
B=${1:?gunbc binary}
for f in ${PROBES:-parse_noop parse_len40 parse_len80}; do
  s=$(date +%s%N)
  systemd-run --user --scope -q -p MemoryMax=24G timeout 2400 $B run --source-root dag --source-root src/v2 --entry src/v2/test/manual/llvprobe/llv_probe.dag --function probe_$f 2>&1 | grep -v floor-drain | tail -1 | cut -c1-160
  e=$(date +%s%N); echo "PROBE $f wall_ms=$(( (e - s)/1000000 ))"
done
# recompute trace (claim_batch built at the same head): TRACE=<claim_batch path> bash probe.sh <gunbc>
if [ -n "$TRACE" ]; then
  for t in trace_len20 trace_len40; do
    GUNBC_RECOMPUTE_TRACE=1 systemd-run --user --scope -q -p MemoryMax=24G timeout 2400 $TRACE --claim-run --source-root dag --source-root src/v2 --entry src/v2/test/manual/llvprobe/llv_probe.dag --function $t 2>&1 | grep -E "recompute-trace\] (keyed|gap|dup)|PASS|FAIL|refus" | head -40 | sed "s/^/TRACE $t /"
  done
fi
