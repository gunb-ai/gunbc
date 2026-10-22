set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/v14-root
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
SHA=8f7af1271081476ec2c2019db4d2d448c481ea52
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin $SHA || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL; exit 1; }
echo "TREE_OK $(git rev-parse HEAD)"
cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch 2>&1 | tail -2; echo "BUILD_RC=$? t=$(date +%H:%M:%S)"
CB=$CARGO_TARGET_DIR/release/claim_batch
E=dag/test/claim/claim_coverage_census_witness.dag
C=the_live_entry_exits_one_while_the_wet_lane_has_no_executor
echo "== claim1 dry t=$(date +%H:%M:%S)"
timeout 1500 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$C" > /tmp/v14.log 2>&1
echo "claim1dry RC=$? t=$(date +%H:%M:%S)"
echo "---- FULL LOG (verbatim) ----"
cat -v /tmp/v14.log | sed 's/\x1b\[[0-9;]*m//g'
echo "---- END LOG ----"
echo "ALL_DONE_V14"
