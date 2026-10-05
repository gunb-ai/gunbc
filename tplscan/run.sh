set -e
CG=/sys/fs/cgroup/tplscan; mkdir -p $CG; echo $((48*1024*1024*1024)) > $CG/memory.max; echo $$ > $CG/cgroup.procs
cargo build --release -q --bin gunbc 2>&1 | tail -3
B=$PWD/target/release/gunbc
rm -rf tplscan/v2main tplscan/m && mkdir -p tplscan/m && git fetch -q --depth=1 origin 1147628e2276c642246791905c2876a7b4d51a86 && git archive 1147628e2276c642246791905c2876a7b4d51a86 src/v2 | tar -x -C tplscan/m && mv tplscan/m/src/v2 tplscan/v2main
for side in after:src/v2 before:tplscan/v2main; do
  t=${side#*:}; n=${side%%:*}
  echo "=== $n ROSTER"; $B run --source-root dag --source-root "$t" --source-root tplscan/drv --entry tplscan/drv/drive.dag --function roster 2>&1 | tail -40 || true
  echo "=== $n CONTROLS"; $B run --source-root dag --source-root "$t" --source-root tplscan/drv --entry tplscan/drv/drive.dag --function controls 2>&1 | tail -20 || true
  echo "=== $n FILES"; $B run --source-root dag --source-root "$t" --source-root tplscan/drv --entry tplscan/drv/drive.dag --function run 2>&1 | tail -120 || true
done
