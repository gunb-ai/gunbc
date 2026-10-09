git fetch -q --depth=1 origin $1 && git -c advice.detachedHead=false checkout -q -f $1 || { echo FETCHFAIL; exit 3; }
echo "rev=$(git rev-parse HEAD)"
cargo build --release -q -p v1-compiler --bin gunbc 2>&1 | tail -3
mkdir -p /sys/fs/cgroup/swf && echo 21474836480 > /sys/fs/cgroup/swf/memory.max && echo $BASHPID > /sys/fs/cgroup/swf/cgroup.procs
G="target/release/gunbc run --source-root dag --source-root src/v2"
git fetch -q --depth=1 origin main && git checkout -q FETCH_HEAD -- src/v2/std/algebra.dag
$G --entry dag/gunbc/instruments/source_reference_repoint.dag --function repoint > /tmp/rp.txt 2>&1; echo "REPOINT rc=$?"; tail -3 /tmp/rp.txt | cut -c1-300
$G --entry dag/gunbc/instruments/source_reference_repoint.dag --function repoint_pending > /tmp/rpp.txt 2>&1; echo "PENDING rc=$?"; tail -3 /tmp/rpp.txt | cut -c1-600
git checkout -q HEAD -- src/v2/std/algebra.dag
$G --entry dag/gunbc/instruments/source_reference_repoint.dag --function repoint_pending > /tmp/rpp2.txt 2>&1; echo "PENDING_AFTER_DELETE rc=$?"; tail -2 /tmp/rpp2.txt | cut -c1-300
echo "CHANGED $(git status --short | wc -l)"; git status --short | head -5
echo "PATCH $(git diff | gzip -9c | base64 -w0)"
cargo build --release -q -p v1-compiler --bin claim_batch 2>&1 | tail -3
p=dag/test/claim/source_reference_repoint_witness_test.dag; fns=$(grep -oP '^test fn \K\w+' $p | paste -sd,)
git stash -q; target/release/claim_batch --hermetic --source-root dag --source-root src/v2 --entry $p --functions "$fns" > /tmp/w.txt 2>&1; echo "WIT rc=$? pass=$(grep -c '^PASS' /tmp/w.txt)"; grep '^FAIL' /tmp/w.txt; git stash pop -q
