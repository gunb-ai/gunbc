git fetch -q --depth=1 origin $1 && git -c advice.detachedHead=false checkout -q -f $1 || exit 3
cargo build --release -q -p v1-compiler --bin claim_batch 2>&1 | tail -3
mkdir -p /sys/fs/cgroup/swo && echo 21474836480 > /sys/fs/cgroup/swo/memory.max && echo $BASHPID > /sys/fs/cgroup/swo/cgroup.procs
p=$2; fns=$(grep -oP '^test fn \K\w+' $p | paste -sd,)
target/release/claim_batch --hermetic --source-root dag --source-root src/v2 --entry $p --functions "$fns" > /tmp/o.txt 2>&1; echo "rc=$?"
grep -vE '^\[(floor-phase|interp-stats|data-profile|pre-entry|process-partition|resolve|assembly|cost)' /tmp/o.txt | grep -vE '^◷|^✓' | cut -c1-300 | tail -40
