set -e
CG=/sys/fs/cgroup/rg$$; mkdir -p $CG; echo $((40*1024*1024*1024)) > $CG/memory.max; echo $$ > $CG/cgroup.procs
cargo build --release -q --bin gunbc 2>&1 | tail -3
for f in probeI; do echo == $f; ./target/release/gunbc run --source-root dag --source-root src/v2 --source-root tplscan/drv --entry tplscan/drv/probe.dag --function $f 2>&1 | grep -v "pre-entry\|typecheck" | cut -c1-1500 | tail -12; done
