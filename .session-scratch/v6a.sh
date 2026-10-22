set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/v6a-root
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
SHA=a80a63e85a55354ad07f34e4ad47c70f91f0adb7
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin $SHA || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL; exit 1; }
echo "TREE_OK $(git rev-parse HEAD)"
cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch 2>&1 | tail -2; echo "BUILD_RC=$?"
CB=$CARGO_TARGET_DIR/release/claim_batch
ENTRY=dag/test/claim/claim_coverage_census_witness.dag
awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' "$ENTRY" > /tmp/v6a.list
N=$(wc -l < /tmp/v6a.list)
echo "== wetwit n=$N t=$(date +%H:%M:%S)"
timeout 3300 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --entry "$ENTRY" --functions "$(paste -sd, /tmp/v6a.list)" > /tmp/v6a.log 2>&1
echo "wetwit RC=$? PASS=$(grep -acE '^PASS ' /tmp/v6a.log) FAIL=$(grep -acE '^FAIL ' /tmp/v6a.log)"
grep -aE '^FAIL ' /tmp/v6a.log | head -10 | cut -c1-120
echo "---- wetwit LOG ----"; head -30 /tmp/v6a.log; echo "---- END ----"
echo "ALL_DONE_V6A $(date +%H:%M:%S)"
