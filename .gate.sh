SHA=$1
git fetch -q --depth=1 origin $SHA && git -c advice.detachedHead=false checkout -q -f $SHA || { echo FETCHFAIL; exit 3; }
echo "rev=$(git rev-parse HEAD)"
cargo build --release -q -p v1-compiler --bin claim_batch 2>&1 | tail -5
mkdir -p /sys/fs/cgroup/swg && echo 21474836480 > /sys/fs/cgroup/swg/memory.max && echo $BASHPID > /sys/fs/cgroup/swg/cgroup.procs
for f in $MODS; do
  p=src/v2/test/claim/${f}_test.dag; fns=$(grep -oP '^test fn \K\w+' $p | paste -sd,)
  target/release/claim_batch --hermetic --source-root dag --source-root src/v2 --entry $p --functions "$fns" > /tmp/o.txt 2>&1; rc=$?
  echo "MOD $f rc=$rc pass=$(grep -c '^PASS ' /tmp/o.txt) fail=$(grep -c '^FAIL ' /tmp/o.txt)"
  grep -E '^FAIL |^claim_batch: .*(refus|error|no --)|^error|refused' /tmp/o.txt | cut -c1-220 | head -8
done
