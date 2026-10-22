set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/v16-root
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
timeout 3400 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$C" > /tmp/v16.log 2>&1
echo "claim1 RC=$? t=$(date +%H:%M:%S)"
echo "---- verdict lines (UNCUT, full) ----"
grep -aE '^(PASS|FAIL) ' /tmp/v16.log || echo NO_VERDICT_LINES
echo "---- FAIL context ----"
grep -a -B2 -A6 'type error' /tmp/v16.log | grep -avE 'floor-drain|typed_cache' | head -30
echo "ALL_DONE_V16 $(date +%H:%M:%S)"
