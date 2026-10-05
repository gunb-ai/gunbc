set -e
CG=/sys/fs/cgroup/tplscan; mkdir -p $CG; echo $((48*1024*1024*1024)) > $CG/memory.max; echo $$ > $CG/cgroup.procs
cargo build --release -q --bin gunbc 2>&1 | tail -3
B=$PWD/target/release/gunbc
rm -rf tplscan/v2main && cp -r src/v2 tplscan/v2main && cp tplscan/dag_main.dag.txt tplscan/v2main/extdeps/languages/dag.dag
for side in after:src/v2 before:tplscan/v2main; do
  t=${side#*:}; n=${side%%:*}
  echo "=== $n ROSTER"; $B run --source-root dag --source-root "$t" --source-root tplscan/drv --entry tplscan/drv/drive.dag --function roster 2>&1 | tail -40 || true
  echo "=== $n CONTROLS"; $B run --source-root dag --source-root "$t" --source-root tplscan/drv --entry tplscan/drv/drive.dag --function controls 2>&1 | tail -20 || true
  echo "=== $n FILES"; $B run --source-root dag --source-root "$t" --source-root tplscan/drv --entry tplscan/drv/drive.dag --function run 2>&1 | tail -120 || true
done
