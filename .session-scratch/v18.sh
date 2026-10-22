set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
CB=$CARGO_TARGET_DIR/release/claim_batch
E=dag/test/claim/claim_coverage_census_witness.dag
C=the_live_entry_exits_one_while_the_wet_lane_has_no_executor
runat() { # $1 sha  $2 tag
  R=/root/workspace/v18-$2; rm -rf $R; mkdir -p $R; cd $R
  git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
  git fetch -q --depth 1 origin $1 || { echo "CLONE_FAIL $2"; return; }
  git checkout -q FETCH_HEAD || { echo "CHECKOUT_FAIL $2"; return; }
  echo "TREE[$2] $(git rev-parse HEAD)"
  timeout 3300 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$C" > /tmp/v18-$2.log 2>&1
  echo "run[$2] RC=$? t=$(date +%H:%M:%S)"
  grep -aE '^(PASS|FAIL|PANICKED) ' /tmp/v18-$2.log || echo "[$2] NO_VERDICT"
}
R=/root/workspace/v18-head; rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin 8f7af1271081476ec2c2019db4d2d448c481ea52 || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD
cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch 2>&1 | tail -2; echo "BUILD_RC=$? t=$(date +%H:%M:%S)"
runat 8f7af1271081476ec2c2019db4d2d448c481ea52 head
runat 201fe0bd10da30ff0eeabaf2eabfc1b42f5e02f1 base
echo "ALL_DONE_V18 $(date +%H:%M:%S)"
