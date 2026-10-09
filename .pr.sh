git fetch -q --depth=1 origin $1 && git -c advice.detachedHead=false checkout -q -f $1 || exit 3
cargo build --release -q -p v1-compiler --bin gunbc 2>&1 | tail -3
mkdir -p /sys/fs/cgroup/swp && echo 21474836480 > /sys/fs/cgroup/swp/memory.max && echo $BASHPID > /sys/fs/cgroup/swp/cgroup.procs
GITHUB_SHA=$1 target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/host_text_callback_witness_test.dag --function htc_debug_blocking_rows > /tmp/p.txt 2>&1; echo "PROBE rc=$?"; tail -15 /tmp/p.txt | cut -c1-600
cargo build --release -q -p v1-compiler --bin claim_batch 2>&1 | tail -3
p=dag/test/claim/host_text_callback_witness_test.dag; fns=$(grep -oP '^test fn \K\w+' $p | paste -sd,)
target/release/claim_batch --hermetic --source-root dag --source-root src/v2 --entry $p --functions "$fns" > /tmp/w.txt 2>&1; echo "WIT rc=$? pass=$(grep -c '^PASS' /tmp/w.txt)"; grep -E '^FAIL|resolve failed|cause' /tmp/w.txt | cut -c1-300
