T=$(awk '/MemTotal/{print int($2*1024*0.85)}' /proc/meminfo); grep MemTotal /proc/meminfo
CG=/sys/fs/cgroup/gunbc-probe; echo "+memory" > /sys/fs/cgroup/cgroup.subtree_control || true; mkdir -p $CG; echo $T > $CG/memory.max; echo $$ > $CG/cgroup.procs
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
B=$PWD/target/release/gunbc
rm -rf /tmp/pb && mkdir -p /tmp/pb && cp -r src dag /tmp/pb/ && cp probe_tmp/04_infer.base.dag /tmp/pb/src/v2/compiler/04_infer.dag
for d in $PWD /tmp/pb; do
  s=$(date +%s.%N); ( cd $d && timeout 1500 $B run --source-root dag --source-root src/v2 --entry src/v2/test/manual/infer_gather_cost_probe.dag --claim-run --function igc_probe_32 --function igc_probe_64 --function igc_probe_128 --function igc_probe_256 2>&1 | grep -v "^\[workspace" | tail -8 ); echo "$d total_secs=$(python3 -c "import time;print(round(time.time()-$s,1))")"
done
