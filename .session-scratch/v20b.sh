set -u
R=/root/workspace/v20-head; rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin 8f7af1271081476ec2c2019db4d2d448c481ea52 || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL; exit 1; }
echo "TREE[head] $(git rev-parse HEAD)"
CB=/root/workspace/v20-claim_batch-diag
echo "BINARY_SHA256 $(sha256sum $CB | cut -d' ' -f1)"
E=dag/test/claim/claim_coverage_census_witness.dag
C=the_live_entry_exits_one_while_the_wet_lane_has_no_executor
echo "== claim1 head t=$(date +%H:%M:%S)"
timeout 2800 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$C" > /tmp/v20-head.log 2>&1
echo "run[head] RC=$? t=$(date +%H:%M:%S)"
grep -aE '^(PASS|FAIL|PANICKED) ' /tmp/v20-head.log || echo "[head] NO_VERDICT"
cp /tmp/v20-head.log /root/workspace/v20-head.log
echo "ALL_DONE_V20B $(date +%H:%M:%S)"
