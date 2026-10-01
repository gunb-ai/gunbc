set -u
SHA=$1; SIDE=$2; HALF=$3
A=/root/workspace/artifacts/command-0; mkdir -p $A
R=$(pwd); cp $R/n3s_$HALF.txt /tmp/roster.txt
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null; mkdir -p /sys/fs/cgroup/cl && echo 22G > /sys/fs/cgroup/cl/memory.max && echo $$ > /sys/fs/cgroup/cl/cgroup.procs && echo "cgroup memory.max=$(cat /sys/fs/cgroup/cl/memory.max)"
git fetch -q origin $SHA; git checkout -q --detach -f $SHA; echo "$SIDE rev=$(git rev-parse HEAD) dirty=$(git status --porcelain --untracked-files=no | wc -l)"
cargo build --release -p v1-compiler --bin claim_batch > /tmp/b.log 2>&1 || { echo BUILD_FAILED; tail -5 /tmp/b.log; exit 1; }
echo "$SIDE claim_batch sha256=$(sha256sum target/release/claim_batch | cut -c1-64)"
: > /tmp/v.txt
while read f; do
  if [ ! -f "$f" ]; then echo "ABSENT $f" >> /tmp/v.txt; continue; fi
  fns=$(grep -oP '(?<=^test fn )\w+' "$f" | paste -sd,)
  timeout 1200 target/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > /tmp/o.log 2>&1; rc=$?
  for fn in ${fns//,/ }; do v=$(grep -oE "^(PASS|FAIL) $fn\b" /tmp/o.log | head -1 | cut -d' ' -f1); echo "${v:-NOVERDICT(rc=$rc)} $f::$fn" >> /tmp/v.txt; done
done < /tmp/roster.txt
sort /tmp/v.txt | gzip -9 > $A/v_${SIDE}_$HALF.gz
echo "$SIDE $HALF tally: $(cut -d' ' -f1 /tmp/v.txt | sort | uniq -c | tr '\n' ' ')"
echo "DIGEST v_${SIDE}_$HALF.gz $(sha256sum $A/v_${SIDE}_$HALF.gz | cut -c1-64)/$(stat -c%s $A/v_${SIDE}_$HALF.gz)"
