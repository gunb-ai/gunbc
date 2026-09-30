grep MemTotal /proc/meminfo; nproc
T=$(awk '/MemTotal/{print int($2*1024*0.85)}' /proc/meminfo)
CG=/sys/fs/cgroup/gunbc-probe; echo "+memory" > /sys/fs/cgroup/cgroup.subtree_control || true; mkdir -p $CG; echo $T > $CG/memory.max; echo $$ > $CG/cgroup.procs; cat $CG/memory.max
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
B=$PWD/target/release/gunbc
rm -rf /tmp/pb && mkdir -p /tmp/pb && cp -r src dag /tmp/pb/ && cp probe_tmp/04_infer.base.dag /tmp/pb/src/v2/compiler/04_infer.dag
for d in $PWD /tmp/pb; do
  s=$(date +%s); ( cd $d && timeout 900 $B run --source-root dag --source-root src/v2 --entry src/v2/test/manual/infer_gather_differential_probe.dag --function infer_probe_differential > /tmp/out_$(basename $d).txt 2>/tmp/err_$(basename $d).txt ); echo "$d rc=$? secs=$(( $(date +%s)-s ))"; tail -c 600 /tmp/err_$(basename $d).txt
done
ls -la /tmp/out_*.txt; md5sum /tmp/out_*.txt
[ -s /tmp/out_pb.txt ] && cmp /tmp/out_repo-root.txt /tmp/out_pb.txt && echo DIFFERENTIAL_IDENTICAL
head -c 800 /tmp/out_repo-root.txt
