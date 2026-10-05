set -e
SIDE=$1; shift
CG=/sys/fs/cgroup/tplscan$$; mkdir -p $CG; echo $((48*1024*1024*1024)) > $CG/memory.max; echo $$ > $CG/cgroup.procs
cargo build --release -q --bin gunbc 2>&1 | tail -3
B=$PWD/target/release/gunbc
t=src/v2
if [ "$SIDE" = before ]; then
  MAIN=$(cat tplscan/main_sha)
  rm -rf tplscan/v2main tplscan/m && mkdir -p tplscan/m && git fetch -q --depth=1 origin $MAIN && git archive $MAIN src/v2 | tar -x -C tplscan/m && mv tplscan/m/src/v2 tplscan/v2main
  t=tplscan/v2main
fi
for f in "$@"; do
  echo "=== $SIDE $f"; $B run --source-root dag --source-root "$t" --source-root tplscan/drv --entry tplscan/drv/drive.dag --function $f 2>&1 | grep -v "pre-entry\|typecheck" | tail -120 || true
done
