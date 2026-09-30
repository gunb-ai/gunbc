set -x
CG=/sys/fs/cgroup/gunbc-probe; echo "+memory" > /sys/fs/cgroup/cgroup.subtree_control || true; mkdir -p $CG; echo 12884901888 > $CG/memory.max; echo $$ > $CG/cgroup.procs; cat $CG/memory.max /proc/self/cgroup
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -3
B=./target/release/gunbc
rm -rf /tmp/pb && mkdir -p /tmp/pb && cp -r src /tmp/pb/ && cp -r dag /tmp/pb/ && cp probe_tmp/04_infer.base.dag /tmp/pb/src/v2/compiler/04_infer.dag
run() { local t0=$(date +%s.%N); ( cd $1 && $OLDPWD/$B run --source-root dag --source-root src/v2 --entry src/v2/test/manual/infer_gather_differential_probe.dag --function $2 ); local rc=$?; echo "ELAPSED $2 $(python3 -c "import time;print(round(time.time()-$t0,2))") rc=$rc" >&2; return $rc; }
for side in head base; do d=.; [ $side = base ] && d=/tmp/pb
  run $d infer_probe_differential > /tmp/diff_$side.txt 2> /tmp/diff_$side.err; echo "$side exit $?"; head -c 3000 /tmp/diff_$side.err
  for k in 8 16 32 64; do echo "$side k=$k"; run $d infer_probe_scale_${k}_nodes 2>&1 | tail -2; run $d infer_probe_scale_$k 2>&1 | tail -2; done
done
[ -s /tmp/diff_head.txt ] && cmp /tmp/diff_head.txt /tmp/diff_base.txt && echo DIFFERENTIAL_IDENTICAL
wc -c /tmp/diff_*.txt; head -c 1500 /tmp/diff_head.txt
diff <(tr "," "\n" < /tmp/diff_head.txt) <(tr "," "\n" < /tmp/diff_base.txt) | head -60; md5sum /tmp/diff_*.txt; cat /tmp/diff_head.err | head -20
