set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/v12-root
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin origin/main || git fetch -q --depth 1 origin main || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL; exit 1; }
echo "MAIN_SHA $(git rev-parse HEAD)"
cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch 2>&1 | tail -2; echo "BUILD_RC=$? t=$(date +%H:%M:%S)"
CB=$CARGO_TARGET_DIR/release/claim_batch
E=dag/test/claim/claim_coverage_census_witness.dag
echo "== main dry, all 7 t=$(date +%H:%M:%S)"
awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' $E > /tmp/v12.list
timeout 2400 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$(paste -sd, /tmp/v12.list)" > /tmp/v12.log 2>&1
echo "maindry RC=$? PASS=$(grep -acE '^PASS ' /tmp/v12.log) FAIL=$(grep -acE '^FAIL ' /tmp/v12.log) t=$(date +%H:%M:%S)"
grep -aE '^(PASS|FAIL) ' /tmp/v12.log | cut -c1-160
echo "ALL_DONE_V12 $(date +%H:%M:%S)"
